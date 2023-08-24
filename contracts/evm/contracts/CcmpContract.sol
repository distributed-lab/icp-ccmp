// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract CcmpContract {
    using ECDSA for bytes32;

    uint256 public index;
    address public canister_address;    

    constructor(address _canister_address) {
        canister_address = _canister_address;
    }

    function sendMessage(uint256 _ccmp_chain_id, bytes memory _message, bytes memory _receiver) external {
        emit CcmpMessage(index, _ccmp_chain_id, msg.sender, _message, _receiver);
        index += 1;
    }

    function isValidMessage(
        bytes32  _message_hash,
        bytes memory _signature
    ) external view returns (bool) {
        return canister_address == ECDSA.recover(_message_hash, _signature);
    }

    event CcmpMessage(uint256 indexed index, uint256 ccmp_chain_id, address sender, bytes message, bytes receiver);
}
