use serde::{Deserialize, Serialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use swiftness_air::layout::recursive::Layout;
pub use swiftness_stark::types::{Cache, Felt, StarkProof};

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

#[derive(Clone, Copy, Default, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct ProofAccount {
    pub proof: StarkProof,
    pub cache: Cache,
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
            let (program_hash, output) = verify_recursive_bytes(&mut account_data).unwrap();
            msg!("VerifyProof");
            msg!("Program hash: {}", program_hash);
            msg!(
                "Output: [{}]",
                output
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            return Err(ProgramError::Custom(42));
        }
    }

    Ok(())
}

pub fn verify_recursive_bytes(
    proof_account: &mut [u8],
) -> Result<(Felt, Vec<Felt>), ProgramResult> {
    let ProofAccount { proof, mut cache } = bytemuck::from_bytes::<ProofAccount>(&proof_account);

    let security_bits = proof.config.security_bits();
    let res = proof.verify::<Layout>(&mut cache, security_bits).unwrap();

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use swiftness::{parse, TransformTo};

    pub fn read_proof() -> ProofAccount {
        let small_json = include_str!("../resources/small.json");
        let stark_proof = parse(small_json).unwrap();
        let proof = stark_proof.transform_to();

        ProofAccount {
            proof,
            ..Default::default()
        }
    }

    #[test]
    fn test_deserialize_proof() {
        let mut proof_account: ProofAccount = read_proof();
        let proof_account_memory = bytemuck::bytes_of_mut(&mut proof_account);

        let (program_hash, output) = verify_recursive_bytes(proof_account_memory).unwrap();

        let output = output
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            program_hash.to_string(),
            "1134405407503728996667931466883426118808998438966777289406309056327695405399"
        );
        assert_eq!(output, vec!["0", "1", "5"]);
    }
}
