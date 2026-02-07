use pinocchio::{AccountView, Address, ProgramResult, entrypoint, error::ProgramError};
use solana_address::declare_id;

use crate::instructions::Deposit;
use crate::instructions::Withdraw;

declare_id!("22222222222222222222222222222222222222222222");

entrypoint!(process_instruction);

mod instructions;

fn process_instruction(
    _program_id: &Address,
    account: &[AccountView],
    instruction_data: &[u8]
) -> ProgramResult {
    let (desciminator, instruction_data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match *desciminator {
        0 => Deposit::try_from((account, instruction_data))?.process(),
        1 => Withdraw::try_from((account, instruction_data))?.process(),
        _ => Err(ProgramError::InvalidInstructionData)
    }
}
