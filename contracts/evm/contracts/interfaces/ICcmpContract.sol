// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

interface ICcmpContract {
    function sendMessage(uint256 _ccmp_chain_id, bytes memory _message, bytes memory _receiver) external;
    function isValidMessage(
        bytes32  _message_hash,
        bytes memory _signature
    ) external view returns (bool);
}