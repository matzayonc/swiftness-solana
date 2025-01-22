use std::str::FromStr;

use serde::{Deserialize, Serialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use swiftness_air::layout::recursive::Layout;
use swiftness_stark::types::StarkProof as StarkProofVerifier;

// declare and export the program's entrypoint
entrypoint!(process_instruction);

pub const PROGRAM_ID: &str = "9yjazUFyg5nPA24oq1ErcuNhPEmNXqS6swBfu91RRqB7";

#[repr(u8)]
#[derive(Serialize, Deserialize)]
pub enum Entrypoint<'a> {
    PublishFragment { offset: usize, data: &'a [u8] },
    VerifyProof {},
}

// program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    account_info: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // if program_id != &Pubkey::from_str(PROGRAM_ID).unwrap() {
    //     return Err(ProgramError::Custom(1));
    // }
    msg!("start: {:?}", program_id);
    let instruction: Entrypoint = bincode::deserialize(instruction_data).unwrap();
    msg!("instruction");
    let accounts_iter = &mut account_info.iter();

    msg!("accounts_iter");

    let account = next_account_info(accounts_iter).unwrap();
    msg!("account");
    assert!(account.is_writable == true);
    let mut account_data = account.try_borrow_mut_data()?;
    msg!("account_data");
    // account_data[0] = 7;

    match instruction {
        Entrypoint::PublishFragment { offset, data } => {
            account_data[offset..offset + data.len()].copy_from_slice(&data);
            msg!("PublishFragment");
        }
        Entrypoint::VerifyProof {} => {
            let stark_proof: StarkProofVerifier = bincode::deserialize(&account_data).unwrap();
            msg!("VerifyProof");
        }
    }

    msg!("done");

    // let stark_proof: StarkProofVerifier =
    //     bincode::deserialize(instruction_data).map_err(|_| ProgramError::InvalidArgument)?;
    // let security_bits = stark_proof.config.security_bits();
    // let result = stark_proof
    //     .verify::<Layout>(security_bits)
    //     .map_err(|_| ProgramError::InvalidArgument)?;

    // msg!("Proof verified {:?}", result);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use swiftness::{stark_proof, types::StarkProof as StarkProofVerifier, TransformTo};
    use swiftness_proof_parser::parse;

    #[test]
    fn test_deserialize_proof() {
        let instruction_data = include_bytes!("../resources/proof.bin");

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
        let small_json = include_str!("../resources/small.json");
        let stark_proof = parse(small_json.to_string()).unwrap();
        let proof_bytes = bincode::serialize(&stark_proof).unwrap();
        dbg!(&proof_bytes.len());
        let stark_proof: stark_proof::StarkProof = bincode::deserialize(&proof_bytes).unwrap();
        let stark_proof: StarkProofVerifier = stark_proof.transform_to();

        let security_bits = stark_proof.config.security_bits();

        let (program_hash, _output) = stark_proof.verify::<Layout>(security_bits).unwrap();
        assert_eq!(
            program_hash.to_hex_string(),
            "0x2820cfb261b9ffa9f5fe7af15ff3d4df545154f26bfd4f234d1f6ba18171157"
        );
    }
}
