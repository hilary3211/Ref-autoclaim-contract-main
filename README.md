#  Ref Finance auto-claim  smart contract

This is a NEAR Protocol smart contract that enables users stake, auto claim and compound rewards. The contract is written in Rust and deployed on the NEAR blockchain.


## Features
- Create and manage Subaccounts 

- Stake Tokens: Users can stake tokens to earn rewards.

- Claim Rewards: Users can claim their staking rewards.

- View Data: Users can query their staked balance and rewards.

- Withdraw rewards



## Technologies Used
- Rust: The programming language used to write the smart contract.

- NEAR SDK: For interacting with the NEAR blockchain.

- NEAR Mainnet: The contract is deployed on the NEAR Mainnet for testing.


## Contract ID
```
auto-claim-main2.near
```



# Contract ID

### lib.rs
This is the main contract that serves as the central hub for the platform. It is responsible for:

- Tracking users: Maintains a record of all users and their activities on the platform.

- Compounding rewards: Automatically compounds rewards earned by users.

- Reinvesting rewards: Reinvests the claimed rewards into the appropriate pools or strategies.








## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```


## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy build-reproducible-wasm <account-id>
```

## Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
