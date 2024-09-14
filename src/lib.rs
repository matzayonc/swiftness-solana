use solana_program::entrypoint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};
use swiftness_air::layout::recursive::Layout;
use swiftness_stark::types::StarkProof;

// declare and export the program's entrypoint
entrypoint!(process_instruction);

// program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let stark_proof: StarkProof =
        bincode::deserialize(instruction_data).map_err(|_| ProgramError::InvalidArgument)?;
    let security_bits = stark_proof.config.security_bits();
    let result = stark_proof
        .verify::<Layout>(security_bits)
        .map_err(|_| ProgramError::InvalidArgument)?;

    msg!("Proof verified {:?}", result);

    Ok(())
}
