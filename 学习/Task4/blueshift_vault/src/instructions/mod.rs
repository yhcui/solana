pub mod deposit;
pub mod withdraw;

pub use deposit::*;
#[cfg(feature="idl-build")]
use pinocchio::account;
pub use withdraw::*;

#[cfg(feature="idl-build")]
use {
    borsh::{BorshDeserialize, BorshSerialize},
    shank::ShankInstruction,    
};

#[cfg(feature="idl-build")]
#[derive(Debug, Clone,ShankInstruction, BorshSerialize, BorshDeserialize )]
#[rustfmt::skip]
pub enum VaultInstructions {

    #[account(0, signers, writable, name="owner", desc="存款人和支付者")]
    #[account(1, writable, name="vault", desc="派生的PDA托管账户")]
    #[account(2, name="system_program", desc="系统程序")]
    Deposit(DepositArgs),
    #[account(0, signers, writable, name="owner", desc="提款人和接收者")]
    #[account(1, writable, name="vault", desc="派生的PDA托管账户")]
    #[account(2, name="system_program", desc="系统程序")]
    Withdraw,
}

#[cfg(feature="idl-build")]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize )]
pub struct DepositArgs {
    pub amount: u64,
}