use anchor_lang::prelude::*; 
use anchor_spl::{ // 引入 Anchor SPL 辅助与 Token 接口。
    associated_token::AssociatedToken, // 关联代币程序类型。
    token_interface::{ // Token 接口模块，兼容 SPL Token / Token-2022。
        transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked, // CPI 方法与账户类型。
    }, // token_interface 导入结束。
}; // anchor_spl 导入结束。

use crate::{ // 引入当前 crate 的内容。
    errors::EscrowError, // 自定义错误定义。
    state::{Escrow}, // Escrow 状态与 PDA 种子常量。
}; // crate 导入结束。
/*
1、自动生成账户验证逻辑
    为 Make 结构体自动生成账户验证代码
    确保传入的账户满足指定的约束条件
2、简化账户验证
    无需手动编写复杂的账户验证逻辑
    自动处理账户所有权、可变性、PDA 等验证

Escrow::INIT_SPACE 是一个常量，定义在 Escrow 结构体中，表示创建 Escrow 账户时所需的初始存储空间大小（以字节为单位）
space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len()
实际的账户存储结构是： [8字节鉴别器][实际Escrow数据]
Seeds（种子）：
1、PDA 地址生成的基础
决定性：相同的 seeds 总是生成相同的 PDA
唯一性：不同的 seeds 产生不同的地址
安全性：确保程序能准确找到对应的 PDA
2. 种子组成解析
b"escrow" - 固定标识符，表明这是托管账户
maker.key().as_ref() - 创建者的公钥，确保每个创建者有独立的托管
seed.to_le_bytes().as_ref() - 用户提供的种子，允许同一创建者创建多个托管

Bump（偏移值）
冲突处理：如果基于 seeds 生成的地址恰好是真实公钥，系统会尝试不同的 "bump" 值直到找到一个真正的 PDA

如果后续为了验证pda是不是我的账户生成的，则需要保留种子和bump

*/
// make 指令的账户上下文。 
#[derive(Accounts)] // 派生账户校验逻辑。
#[instruction(seed: u64)] // make 使用 seed 作为 PDA 种子参数。
pub struct Make<'info> { // Make 账户结构体开始。
    #[account(mut)] // maker 需要可变，用于支付租金与签名转账。
    pub maker: Signer<'info>, // 创建托管的 maker 签名者。
    #[account( // escrow PDA 账户约束。
        init, // 初始化 escrow 账户。
        payer = maker, // 由 maker 支付租金。
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(), // 分配所需空间与鉴别器。
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()], // escrow PDA 种子。
        bump, // 由 Anchor 计算并记录 bump。
    )] // escrow 账户约束结束。
    pub escrow: Account<'info, Escrow>, // 保存交易条款的 escrow PDA 账户。
    // Token 账户与 mint。 
    #[account( // mint A 约束。
        mint::token_program = token_program // mint A 必须属于当前 token_program。
    )] // mint A 约束结束。
    pub mint_a: InterfaceAccount<'info, Mint>, // Token A 的 mint。
    #[account( // mint B 约束。
        mint::token_program = token_program // mint B 必须属于当前 token_program。
    )] // mint B 约束结束。
    pub mint_b: InterfaceAccount<'info, Mint>, // Token B 的 mint。
    #[account( // maker 的 ATA（mint A）。
        mut, // 该账户将被扣款。
        associated_token::mint = mint_a, // ATA 必须是 mint A。
        associated_token::authority = maker, // ATA 权限为 maker。
        associated_token::token_program = token_program // ATA 使用指定 token_program。
    )] // maker ATA 约束结束。
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>, // maker 的 Token A 账户。
    /*
    1. 为什么 vault 的权限是 escrow？
        安全控制：associated_token::authority = escrow 表示金库的权限设置为 escrow PDA，而不是 maker
        托管机制：当 token 存入金库后，不再由原始所有者 maker 控制，而是由托管协议 escrow 控制
        防止撤回：这样设计确保了 maker 无法直接取回已存入的 token，必须通过协议规定的流程（如兑换或取消）才能取出
    2. 为什么会有 mint 账户？
        类型标识：associated_token::mint = mint_a 指定了金库只能存放特定类型的代币
        关联关系：这个约束确保 vault 是 mint_a 这种代币类型的关联代币账户（ATA）
        验证机制：Anchor 会自动验证 vault 账户是否确实是对应 mint_a 的正确 ATA    
    3. vault 属于什么账户？
        vault 是一个 关联代币账户（Associated Token Account, ATA）：
        初始化：通过 init 指令动态创建
        付费方：由 maker 支付创建费用（payer = maker）
        功能：专门用于存放 mint_a 类型的代币
        所有权：归 escrow PDA 所有，用于托管功能   

    这种设计实现了资金托管的核心概念：代币被锁定在由协议控制的账户中，确保交易的安全性和可信度。     

    这里的 token_program 是一个SPL Token 程序接口：
        1. 类型定义
            在结构体中：pub token_program: Interface<'info, TokenInterface>
            这是一个通用的 Token 程序接口，兼容传统的 SPL Token 和新的 Token-2022
        2. 作用和功能
            程序标识：指定要使用的 Token 程序的实际地址
            CPI 调用：在 deposit_tokens 函数中作为 CPI 调用的目标程序
            兼容性：支持多种 Token 标准（SPL Token 和 Token-2022
    */
  
    #[account( // escrow 的金库 ATA。
        init, // 创建金库 ATA。
        payer = maker, // maker 支付金库租金。
        associated_token::mint = mint_a, // 金库 ATA 对应 mint A。
        associated_token::authority = escrow, // 金库权限为 escrow PDA。
        associated_token::token_program = token_program // 金库使用指定 token_program。
    )] // 金库 ATA 约束结束。
    pub vault: InterfaceAccount<'info, TokenAccount>, // 存放 Token A 的金库账户。
    // 程序账户。
    /*
    token_program 负责实际的代币操作逻辑
    associated_token_program 负责自动创建和管理与钱包地址关联的标准代币账户
    在代码中，associated_token_program 通常依赖 token_program 来执行底层的代币操作。

    为什么 associated_token_program 是必需的
    1. 隐式依赖
    在以下账户约束中，虽然没有直接调用，但需要这个程序：
    #[account(
        init, // 创建金库 ATA
        payer = maker,
        associated_token::mint = mint_a,      // 关联代币程序参与验证
        associated_token::authority = escrow, // 关联代币程序参与验证  
        associated_token::token_program = token_program // 关联代币程序参与验证
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    2. 在哪些地方使用
    Make 指令 (src/instructions/make.rs)
        vault 账户创建：需要 associated_token_program 来创建关联代币账户
        maker_ata_a 验证：验证是否为正确的关联代币账户
    
    Refund 指令 (src/instructions/refund.rs)
        maker_ata_a 创建：使用 init_if_needed 可能需要创建 ATA
        vault 账户验证：确认是正确的关联代币账户
    Take 指令 (src/instructions/take.rs)
        taker_ata_a、taker_ata_b、maker_ata_b 创建：都需要关联代币程序
        所有关联代币账户的验证都需要这个程序
    3. 具体工作原理
    当 Anchor 验证带有 associated_token:: 约束的账户时：

        创建账户：如果使用 init 或 init_if_needed，需要调用 associated_token_program 来创建标准的 ATA
        验证账户：验证现有账户是否符合 ATA 的标准格式，这依赖 associated_token_program 的地址推导逻辑
    
    4. 如果没有这个程序声明
    如果从账户结构中移除 associated_token_program，编译会失败，因为 Anchor 无法验证关联代币账户约束。
    
    
     */
    // associated_token_program 传入的是官方的 SPL Associated Token Account 程序地址：
    pub associated_token_program: Program<'info, AssociatedToken>, // 关联代币程序。
    pub token_program: Interface<'info, TokenInterface>, // Token 程序接口。
    pub system_program: Program<'info, System>, // 系统程序。
} 
/*
2. 账户角色对比
账户	角色	资金流向
escrow	状态账户	仅存储交易信息，不持有代币
vault	代币账户	实际持有托管的代币

3. 为什么这样设计
escrow 账户的作用
状态记录：存储交易参数（期望数量、代币类型、maker 等）
身份验证：通过 PDA 机制验证交易合法性
权限控制：vault 的权限设置为 escrow PDA

vault 账户的作用
代币存储：实际持有被托管的代币
权限隔离：由 escrow PDA 控制，maker 无法直接访问

*/
// Make 辅助方法实现。
impl<'info> Make<'info> { // Make 的 impl 开始。
    pub fn populate_escrow(&mut self, seed: u64, receive: u64, bump: u8) -> Result<()> { // 填充 escrow 字段。
        self.escrow.seed = seed; // 保存 seed 用于后续 PDA 推导。
        self.escrow.maker = self.maker.key(); // 保存 maker 公钥。
        self.escrow.mint_a = self.mint_a.key(); // 保存 mint A 公钥。
        self.escrow.mint_b = self.mint_b.key(); // 保存 mint B 公钥。
        self.escrow.receive = receive; // 保存期望接收的 Token B 数量。
        self.escrow.bump = bump; // 保存 PDA bump。
        Ok(()) 
    } // populate_escrow 结束。
    // 将 maker 的 Token A 存入金库。
    pub fn deposit_tokens(&mut self, amount: u64) -> Result<()> { // 转账 Token A 到金库。
        transfer_checked( // CPI 调用 Token 程序（带 decimals 校验）。
            CpiContext::new( // 构造 CPI 上下文。
                self.token_program.to_account_info(), // Token 程序账户。
                TransferChecked { // TransferChecked CPI 账户集合。
                    from: self.maker_ata_a.to_account_info(), // 转出：maker ATA。
                    mint: self.mint_a.to_account_info(), // mint A 账户。
                    to: self.vault.to_account_info(), // 转入：金库 ATA。
                    authority: self.maker.to_account_info(), // 授权者：maker 签名者。
                }, 
            ), 
            amount, 
            self.mint_a.decimals, 
        )?; 
        Ok(()) 
    } 
} 
// make 指令处理器。 
pub fn handler(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> { // make 入口逻辑。
    // 校验数量参数。 // 校验说明。
    require_gt!(receive, 0, EscrowError::InvalidAmount); // receive 必须大于 0。
    require_gt!(amount, 0, EscrowError::InvalidAmount); // deposit 必须大于 0。
    // 写入 Escrow 数据。 // 状态初始化说明。
    ctx.accounts.populate_escrow(seed, receive, ctx.bumps.escrow)?; // 持久化 escrow 字段。
    // 存入 Token。 // 转账说明。
    ctx.accounts.deposit_tokens(amount)?; // 将 maker 的 Token A 存入金库。
    Ok(()) // 返回成功。
} 
