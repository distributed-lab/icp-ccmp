# bridge_core_icp

Welcome to your new bridge_core_icp project and to the internet computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with bridge_core_icp, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.ic0.app)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd bridge_core_icp/
dfx help
dfx canister --help
```

## Running the project locally

```bash
# set up 
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
# you MUST set some password
dfx identity new dfx_test_key
rustup target add wasm32-unknown-unknown


# deploy
dfx stop
dfx start --clean --background
dfx deploy
```

# Getting start with the IC
[The Internet Computer: Redefining the Possibilities of Decentralized Computing](https://telegra.ph/The-Internet-Computer-Redefining-the-Possibilities-of-Decentralized-Computing-04-19)
