use serde::Deserialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{path::PathBuf, str::FromStr};

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
    println!("meta: {:?}", meta);

    let data = client.get_account_data(&proof_data_account).await?;
    println!("data: {:?}", data);

    Ok(())
}
