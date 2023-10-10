# Usage

To start using CCMP, you first need to select a supported chain family. Currently, only the EVM family is supported. The next step is to select the specific chain on which the listening daemon will be registered, such as Polygon. To register the daemon on an EVM family chain, you need to deploy a smart contract that can emit such an event:

```solidity
event CcmpMessage(
    uint256 indexed index, // an internal index of a message
    uint256 ccmp_chain_id, // a destination chain id of a supported chain in CCMP
    address sender, // an sender of a message
    bytes message, // a message that is going to be sent
    bytes receiver // a receiver of a message
);
```

For example, an EVM family chain that is going to receive the message should have the following method:

```solidity
function receiveMessage(
    uint256 _index,
    uint256 _from_chain_id,
    uint256 _to_chain_id,
    bytes memory _sender,
    bytes memory _message,
    address _receiver,
    bytes memory _signature
) public
```

Before registering a daemon, you should make sure that you have enough cycles in the CCMP container and tokens in the chains where you want to send messages. Below you can see the method calls that will help you with the funding:

```bash
WALLET=$(dfx identity get-wallet)
# Register new balance, returns your public key
dfx canister call ccmp add_balance --wallet $WALLET
# Add cycles for your code execution
dfx canister call ccmp add_cycles --with-cycles 10000000000000 --wallet $WALLET
# Add tokens to the local indexing. tx_hash - tx hash of the transaction in which you transferred tokens to your balance EVM address, chain_id - local identifier of the chain in which the transaction took place
dfx canister call ccmp add_tokens_to_evm_chain '("tx_hash", chain_id:nat64)' --wallet $WALLET
```

The final step will be calling a method that will register a daemon:

```bash
dfx canister call ccmp register_daemon '(record {listen_chain_id=chain_id:nat64; interval_in_secs=60:nat64; ccmp_contract="ccmp_contract_address"})' --wallet $WALLET
```

* listen\_chain\_id - a local chain id where the daemon will listen
* interval\_in\_secs - how often the daemon will check the contract for new events
* ccmp\_contract - a contract address for listening
