use std::path::PathBuf;

use clap::Parser;
use swiftness::{
    config, types::StarkProof as StarkProofVerifier, PublicInput, StarkWitness, TransformTo,
};
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

    let serialized = bincode::serialize(&stark_proof).unwrap();
    let stark_proof: StarkProofVerifier = bincode::deserialize(&serialized).unwrap();
    // std::fs::write(&cli.output, &serialized).unwrap();

    // let instruction_data_from_file = std::fs::read(cli.output)?;

    Ok(())
}

#[test]
fn test_stark_proof() {
    let small_json = include_str!("../small.json");
    let stark_proof: StarkProofVerifier = parse(small_json.to_string()).unwrap().transform_to();

    let serialized = bincode::serialize(&stark_proof).unwrap();
    println!("proof size: {:?}", serialized.len());
    let _stark_proof: StarkProofVerifier = bincode::deserialize(&serialized).unwrap();
}

#[test]
fn test_stark_config() {
    let small_json = include_str!("../small.json");
    let stark_proof: StarkProofVerifier = parse(small_json.to_string()).unwrap().transform_to();

    let serialized = bincode::serialize(&stark_proof.config).unwrap();
    println!("config size: {:?}", serialized.len());
    let _stark_proof: config::StarkConfig = bincode::deserialize(&serialized).unwrap();
}

#[test]
fn test_stark_public_input() {
    let small_json = include_str!("../small.json");
    let stark_proof: StarkProofVerifier = parse(small_json.to_string()).unwrap().transform_to();

    let serialized = bincode::serialize(&stark_proof.public_input).unwrap();
    println!("public input size: {:?}", serialized.len());
    let _stark_proof: swiftness_air::public_memory::PublicInput =
        bincode::deserialize(&serialized).unwrap();
}

#[test]
fn test_stark_unsent_commitment() {
    let small_json = include_str!("../small.json");
    let stark_proof: StarkProofVerifier = parse(small_json.to_string()).unwrap().transform_to();

    let serialized = bincode::serialize(&stark_proof.unsent_commitment).unwrap();
    println!("unsent commitment size: {:?}", serialized.len());
    let _stark_proof: swiftness::types::StarkUnsentCommitment =
        bincode::deserialize(&serialized).unwrap();
}

#[test]
fn test_stark_witness() {
    let small_json = include_str!("../small.json");
    let stark_proof: StarkProofVerifier = parse(small_json.to_string()).unwrap().transform_to();

    let serialized = bincode::serialize(&stark_proof.witness).unwrap();
    println!("witness size: {:?}", serialized.len());
    let _stark_proof: swiftness::types::StarkWitness = bincode::deserialize(&serialized).unwrap();
}
