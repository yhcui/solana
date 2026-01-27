use anchor_lang::prelude::*;

use anchor_lang::system_program::{transfer, Transfer};

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;
    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        require_eq!(ctx.accounts.vault.lamports(), 0, VaultError::VaultAlreadyExists);
        require_gt!(amount,0, VaultError::InvalidAmount);
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer{
                    from: ctx.accounts.singer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                }
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>)  -> Result<()>{
        require_neq!(ctx.accounts.vault.lamports(), 0 , VaultError::InvalidAmount); 

        let singer_key = ctx.accounts.singer.key();
        let singer_seeds = &[b"vault", singer_key.as_ref(), &[ctx.bumps.vault]];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer{
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.singer.to_account_info(),
                },
                &[&singer_seeds[..]],
            ),
            ctx.accounts.vault.lamports(),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct VaultAction<'info> {
    #[account(mut)]
    pub singer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", singer.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum VaultError {
    #[msg("Vault already initialized")]
    VaultAlreadyExists,
    #[msg("Invalid amount")]
    InvalidAmount,
}