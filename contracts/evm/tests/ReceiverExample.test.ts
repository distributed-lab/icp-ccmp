import {ethers} from "hardhat";

import { ReceiverExampleContract, CcmpContract } from "../typechain-types";

describe("ReceiverExample", function () {
    let ccmp_contract: CcmpContract;
    let receiver_example_contract: ReceiverExampleContract;

    beforeEach(async function () {
        const CcmpContract = await ethers.getContractFactory("CcmpContract");
        ccmp_contract = await CcmpContract.deploy("0xf3197AbCab1712D6b24Ee09363901Bf7bfb1b1c0");
        
        await ccmp_contract.waitForDeployment();
        const ccmp_contract_address = await ccmp_contract.getAddress();

        const ReceiverExampleContract = await ethers.getContractFactory("ReceiverExampleContract");
        receiver_example_contract = await ReceiverExampleContract.deploy(ccmp_contract_address);
    });

    it("should succesfully check a signature", async () => {
        const _index = 7;
        const _from_chain_id = 2;
        const _to_chain_id = 2;
        const _sender = "0xe86c4a45c1da21f8838a1ea26fc852bd66489ce9";
        const _message = "0x68656c6c6f20776f726c64";
        const _receiver = "0x9A551f1a0e3416049CfC98cB694a7875757BaA3D";
        const _signature = "0x0735c2159bd1ddae1186ae09e85dc391a1236abd77bc113ee91d853627ff014b4714d76139b55aafcc04121970fb27210a9eb96fc9563ac22a52df923eaeccc11c";
        
        await receiver_example_contract.receiveMessage(_index, _from_chain_id, _to_chain_id, _sender, _message, _receiver, _signature);
    });
});
