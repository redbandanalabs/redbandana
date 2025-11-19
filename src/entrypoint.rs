use solana_program::{
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    account_info::AccountInfo,
};

use crate::processor::Processor;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hand Cranked Privacy entrypoint");
    Processor::process(program_id, accounts, instruction_data)
}
