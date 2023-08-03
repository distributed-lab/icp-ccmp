// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import "./interfaces/ICcmpContract.sol";

import "hardhat/console.sol";

contract ReceiverExampleContract {
    ICcmpContract public ccmp_contract;
    bytes message;

    constructor(address _ccmp_contract) {
        ccmp_contract = ICcmpContract(_ccmp_contract);
    }

    function receiveMessage(
        uint256 _index,
        uint256 _from_chain_id,
        uint256 _to_chain_id,
        bytes memory _sender,
        bytes memory _message,
        address _receiver,
        bytes memory _signature
    ) public {
        console.logBytes(abi.encodePacked(_index, _from_chain_id, _to_chain_id, _sender, _message, _receiver));

        bytes32 message_hash = keccak256(abi.encodePacked(_index, _from_chain_id, _to_chain_id, _sender, _message, _receiver));

        console.logBytes32(message_hash);

        require(ccmp_contract.isValidMessage(message_hash, _signature), "invalid signature");

        message = _message;
    }
}