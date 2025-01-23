use swiftness_proof_parser::parse;
use tokio::fs;

#[tokio::main]
async fn main() {
    let small_json = include_str!("../resources/small.json");
    let stark_proof = parse(small_json).unwrap();
    let proof_bytes = bincode::serialize(&stark_proof).unwrap();
    fs::write("resources/proof.bin", &proof_bytes)
        .await
        .unwrap();
}
