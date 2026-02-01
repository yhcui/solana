use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};

use pinocchio_system::instructions::Transfer;
/*
bump 值的重要性
唯一性保证：确保生成的 PDA 地址是唯一的
安全计算：PDA 计算过程中需要正确的 bump 值
签名验证：使用相同的 bump 值重建 PDA 地址

*/
pub struct WithdrawAccounts<'a> {
    pub owner: &'a AccountView,
    pub vault: &'a AccountView,
    pub bumps:[u8;1],
}

impl<'a> TryFrom<&'a [AccountView]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountData);
        }
        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        // 在验证阶段获取 bump 值
        // Bump 值的确定性分析  每次传入相同数据时，bump 值一定相同
        /*
        PDA 的 find_program_address 函数具有完全的确定性：

            输入相同 → 输出相同
            种子 + 程序 ID → 固定的 PDA 地址 + 固定的 bump 值
            每次调用都会返回相同的 (address, bump) 对
        */
        let (vault_key, bump) =
            Address::find_program_address(&[b"vault", owner.address().as_ref()], &crate::ID);

        // bump 值被保存下来
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self{owner, vault, bumps:[bump]})
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountView]> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

/*
完整工作流程
1、验证 PDA 地址：确认 vault 是用正确种子生成的
2、构建签名种子：使用相同种子让 PDA "签名"
3、执行转账：从 PDA 账户转出资金到用户账户
4、安全验证：确保只有合法的程序能执行此操作
这是 Solana 程序中 PDA 资金管理的标准模式，确保了程序对 PDA 账户的控制权。

与存款操作的区别
1、存款 (deposit.rs)
    普通转账：owner → vault (使用 Transfer::invoke())
    不需要 PDA 签名：因为是从用户账户转出
2、取款 (withdraw.rs)
    PDA 转账：vault → owner (使用 Transfer::invoke_signed())
    需要 PDA 签名：因为是从 PDA 转出资金
安全保障
防止未经授权的访问
    1、正确的种子验证：确保是正确的 vault 地址
    2、PDA 签名要求：只有程序能代表 PDA 签名
    3、余额提取限制：只能提取 vault 的全部余额
*/

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) ->ProgramResult {
        /*
        核心概念：PDA 签名
            为什么需要 PDA 签名？
            1、PDA 本身没有私钥：无法像普通账户那样签名
            2、程序代理签名：程序可以代表 PDA 签名
            3、安全控制：只有拥有正确种子的程序才能代表 PDA 签名
        为什么需要 bump 值？
            有些地址可能是椭圆曲线上的有效公钥（有私钥）
            PDA 必须是没有私钥的地址

         */
        // 种子数组构建
        let seeds = [
            Seed::from(b"vault"), // 固定种子
            Seed::from(self.accounts.owner.address().as_ref()), // 所有者地址作为种子
            Seed::from(&self.accounts.bumps), // bump 值种子 --  PDA 计算时的 bump 值，确保地址唯一性
        ];
        // 签名者构建
        let signers = [Signer::from(&seeds)]; // 将种子数组包装成签名者对象

        Transfer {
            from: self.accounts.vault, // 从 vault PDA 转出
            to: self.accounts.owner, // 转给所有者
            lamports: self.accounts.vault.lamports(), // 转出全部余额
        }.invoke_signed(&signers)?; // 使用 PDA 签名执行转账
 
        Ok(())
    }
}