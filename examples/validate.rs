use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::{client_error::ErrorKind, config::RpcSendTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
    transaction::{Transaction, TransactionError},
};
use std::{path::PathBuf, str::FromStr, thread::sleep, time::Duration};
use swiftness_solana::{Entrypoint, PROGRAM_ID};

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
    let client = RpcClient::new_with_commitment(config.json_rpc_url, CommitmentConfig::processed());
    let payer = Keypair::read_from_file(config.keypair_path)?;

    println!("Using keypair {}, at {}", payer.pubkey(), client.url());

    let stark_proof = include_bytes!("../resources/proof.bin");

    let data_address = Pubkey::from_str("3ydMTAgs95qG2CJtbBRZoFLDpS3ZjdH1XDyaNC5N3PeH").unwrap();
    let data = client.get_account_data(&data_address).await?;

    if data != stark_proof {
        eprintln!("data in the account does not match the proof");
        return Ok(());
    }

    let ix = Instruction {
        program_id: Pubkey::from_str(PROGRAM_ID)?,
        accounts: vec![AccountMeta::new(data_address, false)],
        data: bincode::serialize(&Entrypoint::VerifyProof {}).unwrap(),
    };

    let blockhash = client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);

    let res = client
        .send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::processed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                max_retries: Some(10),
                ..Default::default()
            },
        )
        .await;

    if let Err(e) = res {
        if let ErrorKind::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(code),
        )) = e.kind
        {
            println!("Verification failed with code {}", code);
        } else {
            println!("Verification failed without custom code");
            eprintln!("{:?}", e);
        }
    } else {
        println!("Verification successful");
    }

    Ok(())
}
