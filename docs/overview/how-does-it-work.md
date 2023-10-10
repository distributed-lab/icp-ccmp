---
description: >-
  CCMP is a comprehensive cross-chain communication solution that operates
  through distinct modules, each serving a crucial role in enabling seamless
  interaction between diverse blockchain networks.
---

# How does it work

#### Daemons <a href="#user-content-daemons" id="user-content-daemons"></a>

The Daemons Module is responsible for monitoring and capturing events from various smart contracts deployed across different blockchain networks.

Event Monitoring: The module continuously listens to predefined events from the smart contracts on the chains it supports. These events can include transaction requests, asset transfers, or any other form of inter-chain communication.

Request Aggregation: Upon detecting relevant events, the Listener Module aggregates and structures the cross-chain messages, preparing them for further processing.

#### Signer <a href="#user-content-signer" id="user-content-signer"></a>

The Signer Module ensures the security and authenticity of cross-chain messages by providing robust message signing capabilities.

Message Verification: When a message is aggregated by the Listener Module, the Signer Module performs necessary verifications to ensure that the request is valid, untampered, and authorized by the sender.

Cryptographic Signing: After verification, the Signer Module uses cryptographic techniques to generate a unique signature for each message. This signature acts as a proof of the message's origin and integrity

#### Writer <a href="#user-content-writer" id="user-content-writer"></a>

The Writer Module serves as the intelligent intermediary for cross-chain message transmission, seamlessly connecting disparate blockchain networks.

Chain Compatibility: The Writer Module identifies the target chain for each cross-chain message, taking into account the compatibility of the source and target chains, as well as any custom routing rules.

Message Routing: Once the target chain is determined, the Writer Module routes the signed cross-chain message to the corresponding chain, ensuring accurate and efficient delivery.

Atomic Execution: CCMP guarantees atomicity during message execution, meaning that a cross-chain transaction either succeeds entirely or fails without causing any inconsistencies.

#### Checker <a href="#user-content-checker" id="user-content-checker"></a>

The Checker Module is responsible for verifying the execution status of cross-chain messages and ensuring that the results are accurately reflected on the source chain.





<figure><img src="../.gitbook/assets/Screenshot 2023-10-10 at 14.26.54.png" alt=""><figcaption><p>Architecture</p></figcaption></figure>
