use std::path::PathBuf;

use clap::Parser;
use swiftness::{types::StarkProof as StarkProofVerifier, TransformTo};
use swiftness_proof_parser::parse;

#[derive(Parser)]
#[command(author, version, about)]
struct SolanaClient {
    /// Path to proof JSON file
    #[clap(short, long)]
    proof: PathBuf,

    /// Path to save serialized proof
    #[clap(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = SolanaClient::parse();
    let stark_proof: StarkProofVerifier =
        parse(std::fs::read_to_string(cli.proof)?)?.transform_to();
    std::fs::write(cli.output, &bincode::serialize(&stark_proof).unwrap()).unwrap();

    Ok(())
}
