use std::{collections::HashMap, str::FromStr, time::Duration};

use candid::{CandidType, Nat, Principal};
use ethabi::{
    ethereum_types::{H160, H256},
    Error as EthabiError, Event, EventParam, ParamType, RawLog,
};
use ic_cdk::api::instruction_counter;
use ic_cdk_timers::{clear_timer, TimerId, set_timer};
use ic_web3_rs::{
    transports::ICHttp,
    types::{BlockNumber, FilterBuilder},
    Error as Web3Error, Web3,
};
use lazy_static::lazy_static;
use scopeguard::defer;
use serde::{Deserialize, Serialize};

use crate::{
    log, storage_get,
    types::chains::{ChainType, ChainsStorage},
    utils::transform_processors::call_options,
    STORAGE,
};

use super::{
    balances::BalancesStorage, evm_chains::EvmChainsStorage, messages::Message,
    HTTP_OUTCALL_CYCLES_COST, MINIMUM_CYCLES,
};

const DAEMON_HTTP_OUTCALLS_COUNT: u64 = 2;
const DAEMON_JOB_CYCLES_COST: u64 = 2_000_000;

lazy_static! {
    pub static ref MESSAGE_EVENT: Event = Event {
        name: "CcmpMessage".into(),
        inputs: vec![
            EventParam {
                name: "index".into(),
                kind: ParamType::Uint(256),
                indexed: true,
            },
            EventParam {
                name: "ccmp_chain_id".into(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "sender".into(),
                kind: ParamType::Address,
                indexed: false,
            },
            EventParam {
                name: "message".into(),
                kind: ParamType::Bytes,
                indexed: false,
            },
            EventParam {
                name: "receiver".into(),
                kind: ParamType::Bytes,
                indexed: false,
            },
        ],
        anonymous: false,
    };
    pub static ref MESSAGE_EVENT_SIGNATURE: H256 = MESSAGE_EVENT.signature();
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonsError {
    #[error("web3 error: {0}")]
    Web3(#[from] Web3Error),
    #[error("ethabi error: {0}")]
    Ethabi(#[from] EthabiError),
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct Daemon {
    pub id: u64,
    pub creator: Principal,
    pub listen_chain_id: u64,
    pub ccmp_contract: String,
    pub interval: Duration,
    pub is_active: bool,
    pub timer_id: String,
}

impl Default for Daemon {
    fn default() -> Self {
        Self {
            id: 0,
            creator: Principal::anonymous(),
            listen_chain_id: 0,
            ccmp_contract: "".to_string(),
            interval: Duration::from_secs(0),
            is_active: false,
            timer_id: "".to_string(),
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default)]
pub struct DaemonsStorage {
    pub daemon_count: u64,
    pub daemons: HashMap<u64, Daemon>,
}

impl DaemonsStorage {
    pub fn add_daemon(
        listen_chain_id: u64,
        ccmp_contract: String,
        interval: Duration,
        creator: Principal,
    ) -> u64 {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();

            let id = storage.daemon_storage.daemon_count;

            let daemon = Daemon {
                id,
                creator,
                listen_chain_id,
                ccmp_contract,
                interval,
                is_active: true,
                ..Default::default()
            };

            storage.daemon_storage.daemons.insert(id, daemon);
            storage.daemon_storage.daemon_count += 1;

            id
        })
    }

    pub fn get_daemon(id: u64) -> Option<Daemon> {
        STORAGE.with(|storage| {
            let storage = storage.borrow();

            storage.daemon_storage.daemons.get(&id).cloned()
        })
    }

    pub fn start_active_daemons() {
        for (id, daemon) in storage_get!(daemon_storage).daemons.iter() {
            if daemon.is_active {
                Daemon::start(*id);
            }
        }
    }
}

impl Daemon {
    pub async fn listen(id: u64) -> Result<(), DaemonsError> {
        let daemon = DaemonsStorage::get_daemon(id).expect("Daemon not found");
        defer! {
            Self::start(id);
            Self::collect_listening_cycles(id, daemon.creator);
        };

        let chain_metadata = ChainsStorage::get_chain_metadata(daemon.listen_chain_id)
            .expect("Chain metadata not found");

        let mut messages = match chain_metadata.chain_type {
            ChainType::Evm => Self::listen_evm_chain(&daemon).await?,
            _ => panic!("Unsupported chain type"),
        };

        if messages.is_empty() {
            return Ok(());
        }

        log!(
            "[DAEMONS] listening chain finished, id: {}, produced messages number: {}",
            id,
            messages.len()
        );

        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.listened_messages.append(&mut messages)
        });

        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.signer_job.start();
        });

        Ok(())
    }

    pub async fn listen_evm_chain(daemon: &Daemon) -> Result<Vec<Message>, DaemonsError> {
        let evm_chain =
            EvmChainsStorage::get_chain(daemon.listen_chain_id).expect("EVM chain not found");
        let balance = BalancesStorage::get_balance(&daemon.creator).expect("Balance not found");
        let chain_data = balance
            .chains_data
            .get(&daemon.listen_chain_id)
            .expect("Chain data not found");

        let w3 = Web3::new(ICHttp::new(&evm_chain.rpc, None).unwrap());

        let from_block = chain_data.last_block + 1;
        let to_block = w3
            .eth()
            .block_number(call_options("transform".to_string()))
            .await?
            .as_u64();

        let filter = FilterBuilder::default()
            .from_block(BlockNumber::Number(from_block.into()))
            .to_block(BlockNumber::Number(to_block.into()))
            .address(vec![H160::from_str(&daemon.ccmp_contract).unwrap()])
            .build();

        log!("[DAEMINS] listerning on height: {}-{}, daemon id: {}", from_block, to_block, daemon.id);

        let logs = w3
            .eth()
            .logs(filter, call_options("transform".to_string()))
            .await?;

        if logs.is_empty() {
            log!(
                "[DAEMONS] daemon listening finished, daemon id: {}, no messages",
                daemon.id
            );
            BalancesStorage::update_last_block(
                &daemon.creator,
                daemon.listen_chain_id,
                to_block,
            );
            return Ok(vec![]);
        }

        let parsed_logs = logs
            .into_iter()
            .filter(|log| log.topics[0] == *MESSAGE_EVENT_SIGNATURE)
            .map(|log| {
                MESSAGE_EVENT.parse_log(RawLog {
                    topics: log.topics,
                    data: log.data.0,
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let mut messages = vec![];
        for log in parsed_logs {
            if let Some(message) = Message::new(log, daemon.listen_chain_id, daemon.id) {
                messages.push(message);
                continue;
            }
        }

        BalancesStorage::update_last_block(&daemon.creator, daemon.listen_chain_id, to_block);

        Ok(messages)
    }

    pub fn collect_listening_cycles(id: u64, principal: Principal) {
        let mut used_cycles = (instruction_counter() / 10) * 4;
        used_cycles += HTTP_OUTCALL_CYCLES_COST * DAEMON_HTTP_OUTCALLS_COUNT;
        used_cycles += DAEMON_JOB_CYCLES_COST;

        BalancesStorage::reduce_cycles(&principal, Nat::from(used_cycles));

        let balance = BalancesStorage::get_balance(&principal).expect("Balance not found");
        if balance.cycles < MINIMUM_CYCLES {
            log!("[DAEMONS] insufficient cycles, principal: {}", principal);
            Self::stop(id);
        }
    }

    pub fn start(id: u64) {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            let daemon = storage.daemon_storage.daemons.get_mut(&id).unwrap();

            daemon.is_active = true;

            let timer_id = set_timer(daemon.interval, move || {
                log!("[DAEMONS] starting]");

                ic_cdk::spawn(async move {
                    if let Err(err) = Self::listen(id).await {
                        log!("[DAEMONS] error: {}", err);
                    };
                });
            });

            let serialized_timer_id = serde_json::to_string(&timer_id).unwrap();

            daemon.timer_id = serialized_timer_id;
        });
    }

    pub fn stop(id: u64) {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            let daemon = storage.daemon_storage.daemons.get_mut(&id).unwrap();

            daemon.is_active = false;

            let timer_id: TimerId = serde_json::from_str(&daemon.timer_id).unwrap();

            clear_timer(timer_id);
        });
    }
}
