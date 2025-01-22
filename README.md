# Swiftness Solana Program

Program Id: x7dqKWiBUcWXzRVjdK3UeeFMPiRDivhrnmt2hoCZbwA

## Local Development

Start Local Solana Validator and set it as default endpoint

```bash
solana-test-validator
solana config set -u localhost
```

Create a new account and request funds from the local validator

```bash
solana-keygen new
solana airdrop 100
```

Build Program

```bash
cargo build-bpf
```

Build and deploy Program

```bash
cargo build-bpf && solana program deploy target/deploy/swiftness.so
```

Notice:

- Program id has to be hardcoded in `src/lib.rs`
- Program id will change after each build on a new machine

## Usage

Run client to send and verify an example proof

```bash
cargo run --example client
```
