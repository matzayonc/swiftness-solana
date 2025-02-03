use serde::{Deserialize, Serialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use swiftness::types::StarkProof;
use swiftness_air::layout::recursive::Layout;

#[cfg(feature = "custom-heap")]
mod allocator;

// declare and export the program's entrypoint
entrypoint!(process_instruction);

pub const PROGRAM_ID: &str = "GiakVnic8keq93sh2TBx8X51Snqqi1FwcPhFyze7YnkU";

#[repr(u8)]
#[derive(Serialize, Deserialize)]
pub enum Entrypoint<'a> {
    PublishFragment { offset: usize, data: &'a [u8] },
    VerifyProof {},
}

// program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey,
    account_info: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // if program_id != &Pubkey::from_str(PROGRAM_ID).unwrap() {
    //     return Err(ProgramError::Custom(1));
    // }
    let instruction: Entrypoint = bincode::deserialize(instruction_data).unwrap();
    msg!("instruction");
    let accounts_iter = &mut account_info.iter();

    let account = next_account_info(accounts_iter).unwrap();
    assert!(account.is_writable);
    let mut account_data = account.try_borrow_mut_data()?;
    msg!("account_data");

    match instruction {
        Entrypoint::PublishFragment { offset, data } => {
            account_data[offset..offset + data.len()].copy_from_slice(data);
            msg!("PublishFragment");
        }

        Entrypoint::VerifyProof {} => {
            let stark_proof = bytemuck::from_bytes::<StarkProof>(&account_data);

            let security_bits = stark_proof.config.security_bits();
            stark_proof.verify::<Layout>(security_bits).unwrap();

            msg!("VerifyProof with {} security_bits", security_bits);

            return Err(ProgramError::Custom(42));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use swiftness::{parse, types::StarkProof, TransformTo};

    pub fn read_proof() -> StarkProof {
        let small_json = include_str!("../resources/small.json");
        let stark_proof = parse(small_json).unwrap();
        stark_proof.transform_to()
    }

    #[test]
    fn test_deserialize_proof() {
        let stark_proof: StarkProof = read_proof();
        let stark_proof_memory = bytemuck::bytes_of(&stark_proof);

        let stark_proof = bytemuck::from_bytes::<StarkProof>(stark_proof_memory);

        let security_bits = stark_proof.config.security_bits();
        let _result = stark_proof.verify::<Layout>(security_bits).unwrap();
    }
}
