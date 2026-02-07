use pinocchio::{Address, AccountView, ProgramResult};
use pinocchio::cpi::Seed;
use pinocchio::error::ProgramError;
use pinocchio_token::instructions::Transfer;
use crate::{AccountCheck, SignerAccount, MintInterface, AssociatedTokenAccount, AssociatedTokenAccountCheck, ProgramAccount, Escrow, ProgramAccountInit, AssociatedTokenAccountInit};

pub struct MakeAccounts<'info> {
    pub maker: &'info AccountView,

    // 托管账户（PDA）
    // *** 重要：这个账户是由客户端预先计算并传入的！***
    //
    // 客户端构建交易流程：
    // 1. 使用相同的 PDA 公式计算 escrow 地址：
    //    PDA = find_program_address(["escrow", maker, seed], program_id)
    // 2. 将计算出的地址放在账户列表的第 2 位（索引 1）
    // 3. 构建交易并发送到区块链
    //
    // 程序端接收：
    // - 从 accounts[1] 获取客户端传入的 escrow 账户
    // - 重新计算 PDA 验证客户端的计算是否正确
    // - 使用正确的 bump 创建账户
    //
    // 为什么客户端能计算？
    // - PDA 是确定性的：相同输入 → 相同输出
    // - 客户端和程序使用相同的种子和程序 ID
    // - Solana 要求所有非签名账户必须在交易中声明
    pub escrow: &'info AccountView,

    // 代币 A 的 Mint 账户
    pub mint_a: &'info AccountView,

    // 代币 B 的 Mint 账户
    pub mint_b: &'info AccountView,

    // 创建者的代币 A ATA
    // maker_ata_a 是“maker 的代币 A 的关联代币账户 (Associated Token Account, ATA)”。
    // 存放创建者（maker）持有的代币 A 的余额，程序从这个账户把代币转到 vault（托管账户）。
    pub maker_ata_a:&'info AccountView,

     // 金库账户（PDA）
    /*
    vault 是为代币 A 创建的关联代币账户（ATA），但它的所有者是 escrow 这个 PDA（程序派生地址），所以常说“vault 是 PDA 控制的 token 账户”
    程序需要一个由合约控制、可以安全存放托管代币的账户。
    PDA 没有私钥，只有程序能用相同的种子和 bump 在 CPI 时“代签”，这保证只有合约逻辑能从 vault 解锁/转出代币（提高安全性）。
    这里的 “程序” 指的是智能合约本身，也就是这个 on‑chain program（在代码里是 crate::ID，即 escrow 程序）
    PDA 是用 seeds + program id 派生出来的地址，PDA 本身没有私钥，属于该 program 的“名下地址”——只有程序运行时可以以该 PDA 的名义签名（通过运行时的 invoke_signed / CPI 机制），外部用户（maker）不能用私钥替它签名。
    */
    pub vault: &'info AccountView,

    /*
    System program: 11111111111111111111111111111111，Solana 的内置 System Program。
    负责创建/分配/转移 lamports、创建账户并给账户赋 owner 等低级账户操作。
    这里用于创建 PDA 对应的账户或 ATA（需要用 system_program 的 create_account/assign 功能）。
    两者都是“程序账户”（program id），必须作为交易的 account 参数传入——System 用来创建/分配账户，Token 用来执行 token CPI（比如转账、初始化 token account）。
    */
    pub system_program: &'info AccountView,
    /*
     Token program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA，SPL Token 的运行程序。
     负责 token 账户初始化、转账、铸币等与 SPL 代币直接相关的操作。
     代码里用于初始化/操作 vault（token ATA）并执行 Transfer。
     */
    pub token_program: &'info AccountView,

}

impl<'info> TryFrom<&'info [AccountView]> for MakeAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program,_] = accounts else {
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
    pub instruction_data: MakeInstructionData,
    pub bump: u8,
}
/*
(&'info [u8], &'info [AccountView]) 是一个二维元组类型（长度为2的 tuple）
*/
impl<'info> TryFrom<(&'info [u8],&'info [AccountView])> for Make<'info> { 
    type Error = ProgramError;
    /*
        左边 (data, accounts) 是模式（把传入的 tuple 解构为两个变量），右边 (&'info [u8], &'info [AccountView]) 是该参数的类型（一个包含两项的 tuple）。中间的 : 把模式和类型分开。
        等价写法（不解构）：
        let tuple: (&[u8], &[AccountView]) = ...;
        let data = tuple.0; let accounts = tuple.1;
        <'info> 是生命周期，表示两个借用都在同一生命周期内有效。
        用解构的好处：直接命名两个元素，代码更简洁，避免在函数体内再写 .0/.1。
    */
    fn try_from((data, accounts):(&'info [u8], &'info [AccountView])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?; 

        let instruction_data = MakeInstructionData::try_from(data)?;

        let (_, bump) = Address::find_program_address(
            &[
               b"escrow",
               accounts.maker.address().as_ref(),
                &instruction_data.seed.to_le_bytes(),
            ],
            &crate::ID,
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
            accounts.maker,
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
        let mut data = self.accounts.escrow.try_borrow_mut()?;
        let escrow = Escrow::load_mut(data.as_mut())?;

        escrow.set_inner(
            self.instruction_data.seed,
            self.accounts.maker.address().clone(),
            self.accounts.mint_a.address().clone(),
            self.accounts.mint_b.address().clone(),
            self.instruction_data.receive.clone(),
            [self.bump],
        );

        Transfer{
            from: self.accounts.maker_ata_a,
            to: self.accounts.vault,
            authority: self.accounts.maker,
            amount: self.instruction_data.amount
        }.invoke()?;

        Ok(())
    }
}