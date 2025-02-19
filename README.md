# Swiftness Solana Program

Solana program, using Swiftness to verify Cairo proofs on the Solana blockchain.

## Local Node Setup

Use the Solana CLI to create a new account.

```bash
solana-keygen new
```

Start Local Validator and set it as default endpoint

```bash
solana-test-validator
solana config set -u localhost
```

## Program Setup

Build the program, this will generate a new program id.

```bash
cargo build-sbf
```

Update the program id in `src/lib.rs`, this has to be done only once.

```bash
solana address -k target/deploy/swiftness_solana-keypair.json
```

Proceed to deploy the program.

## Deployment

After setting up, new changes can be made to the program by rebuilding and redeploying.

```bash
cargo build-sbf && solana program deploy target/deploy/swiftness_solana.so
```

## Usage

Run client to send and verify an example proof

```bash
cargo run --example client
```

To only verify already uploaded proofs, run the validate example, but update the address of the proof data account.

```bash
cargo run --example validate
```

## Tests

Run the tests, requires more stack space than default.

```bash
RUST_MIN_STACK=4096000 cargo test
```
