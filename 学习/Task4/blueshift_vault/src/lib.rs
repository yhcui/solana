#![no_std]
use pinocchio::{
    AccountView, Address, ProgramResult,entrypoint,error::ProgramError,nostd_panic_handler,
};

use solana_address::declare_id;
use solana_program_log::log;

nostd_panic_handler!();

entrypoint!(process_instruction);

pub mod instructions;

pub use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");
fn process_instruction(
    _program_id: &Address,
    accounts:&[AccountView],
    instruction_data:&[u8],
) -> ProgramResult {
    log("Hello, Solana from Pinocchio!");
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::try_from((data, accounts))?.process(),
        Some((Withdraw::DISCRIMINATOR ,_)) => Withdraw::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }?;
    Ok(())
}