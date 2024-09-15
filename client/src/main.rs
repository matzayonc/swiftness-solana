use clap::Parser;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::Serialize;
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

#[derive(Parser)]
#[command(author, version, about)]
struct SolanaClient {
    /// Path to proof JSON file
    #[clap(short, long)]
    serialized_proof: PathBuf,
}

#[derive(Debug, Serialize)]
struct AllocateRequest {
    idx_start: usize,
    data: Vec<u8>,
}

#[derive(Debug, Serialize)]
enum AllocatorRequest {
    AllocateRequest(AllocateRequest),
}

struct TransactionManager<'a> {
    client: &'a RpcClient,
}

impl<'a> TransactionManager<'a> {
    fn new(client: &'a RpcClient) -> Self {
        Self { client }
    }

    // Function to return a Vec of futures for tracking transactions
    async fn send_all_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Vec<Result<Signature, solana_rpc_client_api::client_error::Error>> {
        let mut futures = FuturesUnordered::new();

        for (idx, tx) in transactions.iter().enumerate() {
            // Wrap each transaction in a future and track the result
            sleep(Duration::from_millis(10));
            let future = async move { (idx, self.client.send_and_confirm_transaction(tx).await) };
            println!("Transaction {:?} requested", idx);
            futures.push(future);
        }


        let mut results = Vec::new();

        while let Some(res) = futures.next().await {
            println!("{:?}", res);
            results.push(res.1)
        }

        results
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = SolanaClient::parse();
    let client = RpcClient::new("https://api.devnet.solana.com".to_string());
    let stark_proof = &std::fs::read(cli.serialized_proof)?;
    let allocator_id = Pubkey::from_str("6w9j57UyR8TT9PhtaiRBV4GV99DUjXcDyqVmEde8FHxa").unwrap();
    let payer = Keypair::read_from_file("/home/bartosz/.config/solana/id.json").unwrap();
    let proof_data_account = Keypair::new();
    println!("account pubkey: {:?}", proof_data_account.pubkey());

    let rent_exemption_amount = client
        .get_minimum_balance_for_rent_exemption(stark_proof.len())
        .await?;
    let ix = system_instruction::create_account(
        &payer.pubkey(),
        &proof_data_account.pubkey(),
        rent_exemption_amount,
        stark_proof.len() as u64,
        &allocator_id,
    );
    let create_account_tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &proof_data_account],
        client.get_latest_blockhash().await?,
    );
    client
        .send_and_confirm_transaction(&create_account_tx)
        .await?;

    // Allocate data instructions
    let instructions: Vec<Instruction> = stark_proof.chunks(100).enumerate().map(|(idx, chunk)| Instruction {
        program_id: allocator_id,
        accounts: vec![AccountMeta::new(proof_data_account.pubkey(), false)],
        data: bincode::serialize(&AllocatorRequest::AllocateRequest(AllocateRequest {
            idx_start: idx*100,
            data: chunk.to_vec(),
        }))
        .unwrap(),
    }).collect();

    // Create corresponding transactions
    let block_hash = client.get_latest_blockhash().await?;
    let mut txs = vec![];
    for ix in instructions {
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            block_hash
        );
        txs.push(tx);
    }

    // Create a transaction manager
    let manager = TransactionManager::new(&client);

    // Send all transactions and track progress
    let _results = manager.send_all_transactions(txs).await;

    println!("{:?}", client.get_account(&proof_data_account.pubkey()).await?);
    
    Ok(())
}
