use pinocchio::{AccountView,Address,ProgramResult,error::ProgramError};
use pinocchio_system::instructions::Transfer;

pub struct DepositAccounts<'a> {
    pub owner :&'a AccountView,
    pub vault :&'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountData);
        }
        /*
        pinocchio_system::ID,  Solana 系统程序的 ID,是 Solana 系统程序 的公钥标识符，它是 Solana 区块链的核心组件之一用于验证账户的所有者
        crate::ID, 当前开发的 vault 程序
        系统程序的功能
            主要职责
            1、账户创建：创建新的账户并分配存储空间
            2、余额转账：在账户之间转移 lamports（Solana 的原生代币单位）
            3、账户所有权管理：设置账户的程序所有者
            4、程序部署：部署新的智能合约程序
        系统程序 ID
            1、固定不变的公钥：Sysvar1nstructions1111111111111111111111111
            2、所有 Solana 网络都使用相同的 ID
        为什么验证 vault 由系统程序拥有？
            设计原因
            1、安全初始化：确保 vault 账户是全新的、未被其他程序使用的系统账户
            2、权限控制：防止用户传入已被其他程序占用的账户
            3、状态验证：配合 lamports().ne(&0) 确保账户是干净的
        
        系统程序与 PDA 的关系
            PDA 的双重特性
            1、地址生成：由 crate::ID（当前程序）通过 PDA 机制计算
            2、账户拥有：物理上由 pinocchio_system::ID（系统程序）拥有    
         */

        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }
        /*
        生成程序派生地址（PDA）
        Address::find_program_address 方法创建一个程序派生地址（Program Derived Address）
        种子参数包括：
            b"vault" - 固定种子字符串
            owner.address().as_ref() - 所有者账户地址作为种子
            &crate::ID - 当前程序的 ID 作为查找范围
        
        这种验证机制确保了    
            防篡改：只有使用正确的种子组合才能生成有效的 vault 地址
            关联性：每个 vault 都与特定的 owner 账户绑定
            确定性：相同的输入总是产生相同的 PDA
        参数解析
            1. seeds: &[&[u8]] - 种子数组- 这些种子必须是静态可验证的，确保每次计算结果一致
            b"vault" - 作为 vault 类型的标识符
            owner.address().as_ref() - 与特定所有者关联，确保每个用户有独立的 vault    
            2. program_id: &Pubkey - 程序 ID - 指定哪个程序拥有这个 PDA
            PDA 必须由特定程序"拥有"
            不同程序使用相同种子会生成不同的地址
            确保地址空间隔离

            crate::ID 是当前程序的公钥标识符，通常在程序入口文件中定义：
            // 一般在 lib.rs 或程序主文件中
            solana_program::declare_id!("YourProgramPublicKeyHere");
            // 或者现代版本中
            pub static ID: Pubkey = solana_program::pubkey!("YourProgramPublicKeyHere");
         */
        let (vault_key, _) = Address::find_program_address(
            &[b"vault", owner.address().as_ref()],
            &crate::ID,
        );

        // 验证 vault 账户的地址是否正确
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self{owner, vault})
    }
}

pub struct DepositInstructionData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        if amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { amount })
    }
}

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'a [u8], &'a [AccountView])
    ) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructionData::try_from(data)?;

        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> Deposit<'a> {

    pub const DISCRIMINATOR: &'a u8 = &0;
    pub fn process(&self) -> ProgramResult {
        Transfer{
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_data.amount,
        }.invoke()?;

        Ok(())
    }
}