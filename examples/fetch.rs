use futures::{stream::FuturesUnordered, StreamExt};
use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::{EncodableKey, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::{path::PathBuf, str::FromStr, thread::sleep, time::Duration};
use swiftness_solana::{Entrypoint, PROGRAM_ID};
use tokio::fs;

const CHUNK_SIZE: usize = 500;

async fn send_transactions(
    client: &RpcClient,
    transactions: &[Transaction],
) -> Vec<Result<Signature, solana_rpc_client_api::client_error::Error>> {
    let mut futures = FuturesUnordered::new();

    for (idx, tx) in transactions.iter().enumerate() {
        sleep(Duration::from_millis(100));
        // Wrap each transaction in a future and track the result
        let future = async move { (idx, client.send_transaction(tx).await) };
        futures.push(future);
    }

    let mut results = Vec::new();

    while let Some(res) = futures.next().await {
        println!("{:?}", res);
        results.push(res.1)
    }

    results
}

/// Creates a `Transaction` to create an account with rent exemption
async fn create_proof_data_account(
    client: &RpcClient,
    payer: &Keypair,
    proof_data_account: &Keypair,
    proof_size: usize,
    owner: &Pubkey,
) -> Result<Transaction, Box<dyn std::error::Error>> {
    let rent_exemption_amount = client
        .get_minimum_balance_for_rent_exemption(proof_size)
        .await?;

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &proof_data_account.pubkey(),
        rent_exemption_amount,
        proof_size as u64,
        owner,
    );

    let blockhash = client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix],
        Some(&payer.pubkey()),
        &[payer, proof_data_account],
        blockhash,
    );

    Ok(tx)
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
struct SolanaConfig {
    json_rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let config =
        PathBuf::from(std::env::var("HOME").unwrap()).join(".config/solana/cli/config.yml");

    let config: SolanaConfig = serde_yaml::from_reader(std::fs::File::open(config)?)?;
    let client = RpcClient::new(config.json_rpc_url);

    let proof_data_account = Pubkey::from_str("28Juca3HcQpPATAGcH2Uf96C2XrUCvG3Vb8d6Ny73dzf")?;

    let meta = client.get_account(&proof_data_account).await?;
    dbg!(&meta);

    let data = client.get_account_data(&proof_data_account).await?;

    dbg!(&data);

    // sleep(Duration::from_millis(200));
    // }

    // loop {
    //     let data = client
    //         .get_account_data(&proof_data_account.pubkey())
    //         .await?;

    //     if data.eq(stark_proof) {
    //         println!("proof_data_account correct!");
    //         break;
    //     } else {
    //         println!("proof_data_account data not maching!");
    //         sleep(Duration::from_secs(5));
    //     }
    // }

    Ok(())
}
