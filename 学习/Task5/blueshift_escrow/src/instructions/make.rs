use pinocchio::{Address, AccountView, ProgramResult};
use pinocchio::cpi::Seed;
use pinocchio::error::ProgramError;
use pinocchio_token::instructions::Transfer;
use crate::{AccountCheck, SignerAccount, MintInterface, AssociatedTokenAccount, AssociatedTokenAccountCheck, ProgramAccount, Escrow, ProgramAccountInit, AssociatedTokenAccountInit};

pub struct MakeAccounts<'info> {
    pub maker: &'info AccountView,

    pub escrow: &'info AccountView,

    pub mint_a: &'info AccountView,

    pub mint_b: &'info AccountView,

    pub maker_ata_a:&'info AccountView,

    pub vault: &'info AccountView,

    pub system_program: &'info AccountView,

    pub token_program: &'info AccountView,

}

impl<'info> TryFrom<&'info [AccountView]> for MakeAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(maker)?;

        MintInterface::check(mint_a)?;

        MintInterface::check(mint_b)?;

        AssociatedTokenAccount::check(maker_ata_a,maker,mint_a, token_program)?;
        Ok(Self{
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
        })
    }
}

pub struct MakeInstructionData {
    pub seed: u64,
    pub receive: u64,
    pub amount: u64,
}

impl<'info> TryFrom<&'info [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
       if data.len() != size_of::<u64>() * 3 {
           return Err(ProgramError::InvalidInstructionData);
       }
       let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
       let receive = u64::from_le_bytes(data[8..16].try_into().unwrap());
       let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());

       if amount == 0 {
           return Err(ProgramError::InvalidInstructionData);
       }

       Ok(Self{
           seed,
           receive,
           amount,
       })
    }
}

pub struct Make<'info> {
    pub accounts: MakeAccounts<'info>,
    pub data: MakeInstructionData,
    pub bump: u8,
}

impl<'info> TryFrom<&'info [u8],&'info [AccountView]> for Make<'info> { 
    type Error = ProgramError;
    
    fn try_from((data, accounts):(&'info [u8], &'info [AccountView])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?; 

        let instruction_data = MakeInstructionData::try_from(data)?;

        let make_accounts = MakeAccounts::try_from(&accounts[0..accounts.len()-1])?;

        let (_, bump) = Address::find_program_address(
            &[
               b"escrow",
               accounts.maker.key.as_ref(),
                &instruction_data.seed.to_le_bytes(),
            ],
            &crate::ID
        );

        let seed_binding = instruction_data.seed.to_le_bytes();
        let bump_binding = [bump];
        let escrow_seeds=[
            Seed::from(b"escrow"),
            Seed::from(accounts.maker.address().as_ref().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        ProgramAccount::init::<Escrow>(
            accounts.maker,
            accounts.escrow,
            &escrow_seeds,
            Escrow::LEN,
        )?;

        AssociatedTokenAccount::init(
            accounts.vault,
            accounts.mint_a,
            accounts.escrow,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self{
            accounts,
            instruction_data,
            bump,
        })
    }
}

impl<'info> Make<'info> {
    pub const DISCRIMINATOR: &'info u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
       
    }
}