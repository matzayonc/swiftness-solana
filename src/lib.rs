use solana_program::entrypoint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use swiftness_air::layout::recursive::Layout;
use swiftness_stark::types::StarkProof as StarkProofVerifier;

// declare and export the program's entrypoint
entrypoint!(process_instruction);

// program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let stark_proof: StarkProofVerifier =
        bincode::deserialize(instruction_data).map_err(|_| ProgramError::InvalidArgument)?;
    let security_bits = stark_proof.config.security_bits();
    let result = stark_proof
        .verify::<Layout>(security_bits)
        .map_err(|_| ProgramError::InvalidArgument)?;

    msg!("Proof verified {:?}", result);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use swiftness::{types::StarkProof as StarkProofVerifier, TransformTo};
    use swiftness_proof_parser::parse;

    #[test]
    fn test_deserialize_proof() {
        let instruction_data = include_bytes!("../serializer/proof.bin");

        let stark_proof: StarkProofVerifier = bincode::deserialize(instruction_data)
            .map_err(|_| ProgramError::InvalidArgument)
            .unwrap();
        let security_bits = stark_proof.config.security_bits();

        let _result = stark_proof
            .verify::<Layout>(security_bits)
            .map_err(|_| ProgramError::InvalidArgument)
            .unwrap();
    }

    #[test]
    fn test_verify_proof() {
        let small_json = include_str!("../serializer/small.json");
        let stark_proof: StarkProofVerifier = parse(small_json.to_string()).unwrap().transform_to();

        // let proof_bytes = bincode::serialize(&stark_proof).unwrap();
        // let stark_proof: StarkProofVerifier = bincode::deserialize(&proof_bytes).unwrap();

        let security_bits = stark_proof.config.security_bits();

        let (program_hash, _output) = stark_proof.verify::<Layout>(security_bits).unwrap();
        assert_eq!(
            program_hash.to_hex_string(),
            "0x2820cfb261b9ffa9f5fe7af15ff3d4df545154f26bfd4f234d1f6ba18171157"
        );
    }
}
