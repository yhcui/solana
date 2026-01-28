use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        transfer_checked, Mint, TokenInterface, TransferChecked,
    },
};

use crate::{
    errors::EscrowError,
    state::{Escrow},
};

#[derive(Account)]
#[instruction(seed: u64)]
pub struct Make<'info>{
    #[account(mut)]
    pub maker: Singer<'info>,
    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE+ Escrow.DISCRIMINATOR.len(),
        seeds = [b"escrow",maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info,Escrow>,

    #[account(
        mint:token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info,Mint>,
    #[accout(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info,Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info,TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info,TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info,System>,
}