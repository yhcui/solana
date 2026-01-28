/*
层级关系：anchor_lang 是包（Crate），prelude 是模块（Module）。
通配符 (*)：被称为 "Glob import"。它会将 prelude 模块下所有常用的宏（如 #[program], #[account]）和类型（如 Pubkey, Context）一次性导入。
https://docs.rs/solarti-anchor-lang/latest/anchor_lang/
*/
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
/*
    #[derive(Accounts)]（注意通常是复数 Accounts）是 Anchor 框架内置的一个“超级宏”.
    作用
    1、反序列化 (Deserialization)：自动将账户的原始二进制数据转换成 Rust 的结构体对象。
    2、安全性检查 (Security Checks)：通过配合 #[account(..)] 属性标签，自动验证账户所有者、地址约束、签名状态等。
    3、账户加载：自动从 Solana 运行时环境中提取并关联这些账户。
    
    它在代码中的位置
    它通常定义在一个结构体之上，这个结构体定义了某个指令执行时必须提供哪些账户。

*/
#[derive(Accounts)] // 告诉 Anchor：请为这个结构体生成账户校验逻辑
pub struct VaultAction<'info> {
    #[account(mut)] // 约束：这个账户必须是可变的
    pub singer: Signer<'info>,
    
    /*
    #[account(...)] 括号内的内容被称为 "Account Constraints"（账户约束）
    只有满足这些条件的账户，才允许进入我的业务逻辑
    */
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