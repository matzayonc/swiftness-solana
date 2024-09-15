use serde::Deserialize;
use solana_program::entrypoint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

// declare and export the program's entrypoint
entrypoint!(process_instruction);

#[derive(Debug, Deserialize)]
struct AllocateRequest {
    idx_start: usize,
    data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
enum AllocatorRequest {
    AllocateRequest(AllocateRequest)
}

// program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info = &accounts[0];

    // Check that this account is owned by your program
    if account_info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let instruction: AllocatorRequest = bincode::deserialize(instruction_data).unwrap();

    match instruction {
        AllocatorRequest::AllocateRequest(req) => {
            // Write the initial data (for example, a counter)
            let mut account_data = account_info.try_borrow_mut_data()?;
            account_data[req.idx_start..req.idx_start + req.data.len()].copy_from_slice(&req.data);

            msg!("Data allocated {:?}", req);
        }
    }

    Ok(())
}
