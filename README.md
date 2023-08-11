# Cross-Chain Messaging Protocol

The implementation of a cross-chain messaging protocol that uses ICP.

## Overview

CCMP facilitates cross-chain communication between various blockchain networks, which are typically built using different technologies and have unique consensus mechanisms. It enables different chains to exchange data, assets, and execute smart contracts, leading to a more interconnected and robust blockchain ecosystem.

## Supported Chains

Currently, CCMP supports communication with Ethereum Virtual Machine (EVM) chains, including the Ethereum mainnet and any EVM-compatible sidechains. However, the design of CCMP is highly flexible and extensible, making it easy to incorporate support for other chain types

## How it work

CCMP is a comprehensive cross-chain communication solution that operates through three distinct modules, each serving a crucial role in enabling seamless interaction between diverse blockchain networks.

### Listener

The Listener Module is responsible for monitoring and capturing events from various smart contracts deployed across different blockchain networks.

Event Monitoring: The module continuously listens to predefined events from the smart contracts on the chains it supports. These events can include transaction requests, asset transfers, or any other form of inter-chain communication.

Request Aggregation: Upon detecting relevant events, the Listener Module aggregates and structures the cross-chain messages, preparing them for further processing.

### Signer

The Signer Module ensures the security and authenticity of cross-chain messages by providing robust message signing capabilities.

Message Verification: When a message is aggregated by the Listener Module, the Signer Module performs necessary verifications to ensure that the request is valid, untampered, and authorized by the sender.

Cryptographic Signing: After verification, the Signer Module uses cryptographic techniques to generate a unique signature for each message. This signature acts as a proof of the message's origin and integrity

### Writer

The Writer Module serves as the intelligent intermediary for cross-chain message transmission, seamlessly connecting disparate blockchain networks.

Chain Compatibility: The Writer Module identifies the target chain for each cross-chain message, taking into account the compatibility of the source and target chains, as well as any custom routing rules.

Message Routing: Once the target chain is determined, the Writer Module routes the signed cross-chain message to the corresponding chain, ensuring accurate and efficient delivery.

Atomic Execution: CCMP guarantees atomicity during message execution, meaning that a cross-chain transaction either succeeds entirely or fails without causing any inconsistencies.

## Deploy

```sh
dfx start --clean --background
DEPLOY_ARGS=$(didc encode '(record {key="dfx_test_key"; signer_interval_secs=10:nat64; writer_interval_secs=10:nat64; checker_interval_secs=30:nat64})')
dfx deploy --argument-type raw --argument $DEPLOY_ARGS ccmp
```

## Usage

Firstly, you need to add an EVM chain to the CCMP canister. When adding EVM chains, you need to provide the address of a ccmp contract, which is deployed on the EVM chain. An example of a ccmp contract can be found in `contracts/evm/`. There you can also find an example of a receiver contract. After adding, you can send messages using the `sendMessage` method.

```sh
dfx canister call ccmp add_evm_chain '("Sepolia", "https://eth-sepolia.g.alchemy.com/v2/36Anwhc7TaSmzuxnIeQ49marn1w1iftu")'
dfx canister call ccmp get_public_key
WALLET=$(dfx identity get-wallet)
dfx canister call ccmp add_balance --wallet $WALLET
dfx canister call ccmp get_balance --wallet $WALLET
dfx canister call ccmp add_cycles --with-cycles 10000000000000 --wallet $WALLET
dfx canister call ccmp add_tokens_to_evm_chain '("0x179a8a65a251e7cfadd2681b790046edc56630aefee2b90c76ab0992ecb2c634", 0:nat64)' --wallet $WALLET
dfx canister call ccmp register_daemon '(record {listen_chain_id=0:nat64; interval_in_secs=60:nat64; ccmp_contract="0xe8fad0129a5aCfA30D211Bf2a2E9d8a5122B39bE"})' --wallet $WALLET
```
