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
use swiftness::{parse, types::StarkProof, TransformTo};
use swiftness_solana::{Entrypoint, PROGRAM_ID};

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

pub fn read_proof() -> StarkProof {
    let small_json = include_str!("../resources/small.json");
    let stark_proof = parse(small_json).unwrap();
    stark_proof.transform_to()
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
    keypair_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let config =
        PathBuf::from(std::env::var("HOME").unwrap()).join(".config/solana/cli/config.yml");

    let config: SolanaConfig = serde_yaml::from_reader(std::fs::File::open(config)?)?;
    let client = RpcClient::new(config.json_rpc_url);
    let payer = Keypair::read_from_file(config.keypair_path)?;

    println!("Using keypair {}, at {}", payer.pubkey(), client.url());

    // let stark_proof = include_bytes!("../resources/proof.bin");
    let stark_proof_value = read_proof();
    let stark_proof = bytemuck::bytes_of(&stark_proof_value);

    let proof_data_account = Keypair::new();
    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    println!("account pubkey: {:?}", proof_data_account.pubkey());
    client
        .send_and_confirm_transaction(
            &create_proof_data_account(
                &client,
                &payer,
                &proof_data_account,
                stark_proof.len(),
                &program_id,
            )
            .await?,
        )
        .await?;

    // for (section, section_data) in stark_proof.chunks(10000).enumerate() {
    // Allocate data instructions

    for (big_chunk, big_data) in stark_proof.chunks(CHUNK_SIZE * 20).enumerate() {
        let blockhash = client
            .get_latest_blockhash()
            .await
            .expect("failed to connect to rpc");

        loop {
            let instructions: Vec<Instruction> = big_data
                .chunks(CHUNK_SIZE)
                .enumerate()
                .map(|(chunk, data)| Instruction {
                    program_id,
                    accounts: vec![AccountMeta::new(proof_data_account.pubkey(), false)],
                    data: bincode::serialize(&Entrypoint::PublishFragment {
                        offset: chunk * CHUNK_SIZE + big_chunk * CHUNK_SIZE * 20,
                        data,
                    })
                    .unwrap(),
                })
                .collect();

            // Create corresponding transactions
            let transactions = instructions
                .into_iter()
                .map(|instruction| {
                    Transaction::new_signed_with_payer(
                        &[instruction],
                        Some(&payer.pubkey()),
                        &[&payer],
                        blockhash,
                    )
                })
                .collect::<Vec<_>>();

            let results = send_transactions(&client, &transactions).await;
            if results.iter().all(|r| r.is_ok()) {
                break;
            }

            println!("Failed to send transactions, repeating batch.");
        }
    }

    loop {
        let data = client
            .get_account_data(&proof_data_account.pubkey())
            .await?;

        if data.eq(stark_proof) {
            println!("proof_data_account correct!");
            break;
        } else {
            println!("proof_data_account data not matching!");
            sleep(Duration::from_secs(1));
        }
    }

    let ix = Instruction {
        program_id,
        accounts: vec![AccountMeta::new(proof_data_account.pubkey(), false)],
        data: bincode::serialize(&Entrypoint::VerifyProof {}).unwrap(),
    };

    let blockhash = client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);

    client.send_and_confirm_transaction(&tx).await.unwrap();

    Ok(())
}
