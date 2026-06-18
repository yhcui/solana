//! # 托管交换程序（Escrow）
//!
//! 这是一个 Solana 链上程序，用于实现「去中心化代币交换」功能。
//!
//! ## 什么是托管（Escrow）？
//! 在区块链世界中，托管就像是有一个「中间人」帮你保管资产：
//! - Maker（挂单人）把代币存入一个由程序控制的保险箱，并声明"我愿意用 X 个 A 代币换 Y 个 B 代币"
//! - Taker（接单人）看到这个报价后，可以把 Y 个 B 代币支付给 Maker，然后从保险箱中取出 X 个 A 代币
//! - 整个过程是**原子性**的——要么全部成功，要么全部失败，不存在中间状态
//!
//! ## 核心角色
//! - **Maker（挂单人）**：创建报价的人，先把代币存入托管账户
//! - **Taker（接单人）**：接受报价的人，支付代币后从托管账户取走代币
//!
//! ## 关键概念
//! - **PDA（程序派生地址）**：由程序控制的特殊账户，就像程序专属的「保险箱」
//! - **ATA（关联代币账户）**：Solana 中用于存储代币的账户，与所有者（Owner/Authority）和代币类型（Mint）进行关联。
//! ATA 的设计目的是为了解决“一个用户拥有多种代币”时的账户管理问题。它的地址不是随机生成的，而是通过确定性算法计算得出的。
//! -  关联规则：对于每一个 Owner 和每一个 Mint，在链上有且仅有一个对应的 ATA 地址。
//! -  计算公式：ATA_Address = DeriveAddress(Owner, Mint)
//! -  无需手动管理地址：前端或钱包只需知道用户的公钥和代币 Mint 地址，就能自动计算出 ATA 地址，无需用户每次创建新账户。
//! -  避免重复：确保同一个用户不会为同一种代币创建多个分散的账户，简化了余额查询和管理。
//! - **Mint（铸造账户）**：记录代币总供应量和精度的账户
//! - **CPI（跨程序调用）**：一个程序调用另一个程序的指令

// Anchor 框架预导入模块，包含所有常用的 Solana 类型和宏
use anchor_lang::prelude::*;
// 关联代币账户程序，用于创建和管理 ATA
use anchor_spl::associated_token::AssociatedToken;
// 代币接口模块，兼容 SPL Token 和 Token-2022 两种标准
// Rust语言的模块导入语句，具体使用了Anchor框架提供的库。
// anchor_spl：外部 crate（库名称），即 anchor-spl，它是Anchor框架中用于与 Solana Program Library (SPL) 交互的标准库。
// token_interface：该 crate 下的一个子模块。之所以叫 interface，是因为它旨在同时兼容旧版的 spl-token 和新版的 spl-token-2022（支持转账钩子等高级功能）。
// { ... }：大括号内列出了要从该模块中具体导入的内容。
// 你通过 Mint 确保大家交易的是正确的代币类型，且数量计算正确。
// 你通过操作 TokenAccount 来实际移动资产（从 Maker 的账户移到托管账户，再从托管账户移到 Taker 的账户）
use anchor_spl::token_interface::{
    close_account,          // CPI函数。关闭代币账户的 CPI 函数。用于在代码中调用 Token 程序的“关闭账户”指令。通常用于回收账户租金。
    transfer_checked,       // CPI函数。带精度校验的转账 CPI 函数（推荐使用的安全转账方式）
    CloseAccount,           // 结构体。关闭账户所需的账户结构体。定义了调用 close_account CPI 时需要传入的账户列表（如：要关闭的账户、接收租金的目标账户、授权人）。
    Mint,                   // 类型别名/结构体。铸造账户类型。Mint 回答的问题是：“这是什么币？总共发了多少？精度是多少？”。代表 Solana 上的“铸造账户”（Token Mint），包含代币名称、符号、供应量、小数位数等信息。
    TokenAccount,           // 类型别名/结构体。代币账户类型。TokenAccount 回答的问题是：“谁拥有这个币？他手里有多少？”。代表 Solana 上的“代币账户”（ATA 或普通 Token Account），包含所有者、余额、所属 Mint 等信息。
    TokenInterface,         // 类型/特征。代币程序接口（同时兼容旧版和新版 Token 程序）。这是一个特殊的类型，允许你的程序通过统一的接口与任意版本的 Token 程序（v3 或 2022）进行交互，提高了代码的兼容性。
};

// 声明程序的 Program ID，这是程序在链上的唯一标识
// 这个地址是通过 `solana address` 或 Anchor 编译时生成的
declare_id!("25Q841qjRsaGQzWSKh5kiEZ9qpXbWMzm3v4ytGXs6PzY");

/// 程序模块，包含所有可被外部调用的指令
#[program]
pub mod escrow {
    use super::*;

    /// ### make_offer — 创建报价（挂单）
    ///
    /// Maker（挂单人）调用此指令，将指定数量的 mint_a 代币存入托管账户，
    /// 并创建一份 Offer 记录，声明自己想要的 mint_b 代币数量。
    ///
    /// #### 参数说明
    /// - `offer_id`: 报价的唯一标识符（u64），允许同一个 maker 创建多个不同的报价
    /// - `token_a_offered_amount`: maker 愿意提供的 mint_a 代币数量
    /// - `token_b_wanted_amount`: maker 想要的 mint_b 代币数量
    ///
    /// #### 执行流程
    /// 1. 校验 mint_a 和 mint_b 不是同一个代币（防止自己换自己）
    /// 2. 将 maker 的 mint_a 代币转入托管账户（offer_token_account）
    /// 3. 将报价信息写入 Offer PDA 账户
    pub fn make_offer(
        ctx: Context<MakeOffer>,                          // 上下文，包含所有传入的账户
        offer_id: u64,                                    // 报价 ID
        token_a_offered_amount: u64,                      // 提供的 mint_a 数量
        token_b_wanted_amount: u64,                       // 想要的 mint_b 数量
    ) -> Result<()> {
        // require_keys_neq! 是 Anchor 的宏，要求两个公钥不相等
        // 如果相等则返回 SameMint 错误
        require_keys_neq!(
            ctx.accounts.mint_a.key(),
            ctx.accounts.mint_b.key(),
            EscrowError::SameMint
        );

        // ===== 步骤 1：将 maker 的代币转入托管账户 =====
        // 这一步使用的是 CPI（跨程序调用）——我们的程序调用 Token 程序的 transfer_checked 指令
        // transfer_checked 要求指定代币精度，比普通的 transfer 更安全
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.maker_token_account_a.to_account_info(),  // 从 maker 的 ATA 转出
            to: ctx.accounts.offer_token_account.to_account_info(),      // 转入托管 ATA
            mint: ctx.accounts.mint_a.to_account_info(),                 // 代币铸造账户（用于校验精度）
            authority: ctx.accounts.maker.to_account_info(),             // 转出账户的授权人（maker 本人）
        };

        // 创建 CPI 上下文，指定要调用哪个 Token 程序
        let cpi_context = CpiContext::new(*ctx.accounts.token_program.key, transfer_cpi_accounts);

        // 获取 mint_a 的精度（比如 9 表示 1 个代币 = 10^9 最小单位）
        let decimals = ctx.accounts.mint_a.decimals;
        // 执行转账 —— 注意：maker 已经在外层签名，所以不需要额外签名
        transfer_checked(cpi_context, token_a_offered_amount, decimals)?;

        // ===== 步骤 2：将报价信息写入链上存储 =====
        // set_inner 是 Anchor 提供的方法，用于将数据写入 Account 账户
        ctx.accounts.offer.set_inner(Offer {
            maker: ctx.accounts.maker.key(),              // 记录 maker 的公钥
            mint_a: ctx.accounts.mint_a.key(),            // 记录提供的代币类型
            mint_b: ctx.accounts.mint_b.key(),            // 记录想要的代币类型
            offer_id,                                      // 记录报价 ID
            token_a_offered_amount,                        // 记录提供的数量
            token_b_wanted_amount,                         // 记录想要的数量
            bump: ctx.bumps.offer,                         // 记录 PDA 的 bump 值（用于后续签名）
        });

        Ok(())
    }

    /// ### take_offer — 接受报价（接单）
    ///
    /// Taker（接单人）调用此指令，完成代币交换。这是一个**原子性操作**：
    /// - Taker 支付 mint_b 代币给 maker
    /// - Taker 从托管账户接收 mint_a 代币
    /// - 托管账户和 Offer 账户被自动关闭，租金（rent）退还给 maker
    ///
    /// #### 为什么是原子性的？
    /// 在 Solana 中，一个交易中的所有指令要么全部成功，要么全部失败。
    /// 所以 taker 不用担心「我付了钱但没收到代币」——如果任何一步失败，整个交易回滚。
    ///
    /// #### 执行流程
    /// 1. taker 将 mint_b 代币转给 maker（付款）
    /// 2. 程序用 PDA 签名，从托管账户将 mint_a 代币转给 taker（交货）
    /// 3. 关闭托管账户，租金退还给 maker
    /// 4. Offer PDA 账户自动关闭（由 Anchor 的 close = maker 约束处理）
    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        // ===== 步骤 1：taker 付款 —— taker 把 mint_b 代币转给 maker =====
        {
            let transfer_cpi_accounts = TransferChecked {
                from: ctx.accounts.taker_token_account_b.to_account_info(),   // 从 taker 的 mint_b 账户转出
                to: ctx.accounts.maker_token_account_b.to_account_info(),     // 转入 maker 的 mint_b 账户
                mint: ctx.accounts.mint_b.to_account_info(),                  // mint_b 的铸造账户
                authority: ctx.accounts.taker.to_account_info(),              // taker 本人授权
            };

            let cpi_context =
                CpiContext::new(*ctx.accounts.token_program.key, transfer_cpi_accounts);

            let decimals = ctx.accounts.mint_b.decimals;
            // 转账金额从 Offer 记录中读取（即 maker 想要的数量）
            transfer_checked(
                cpi_context,
                ctx.accounts.offer.token_b_wanted_amount,
                decimals,
            )?;
        }

        // ===== 步骤 2：程序交货 —— 从托管账户把 mint_a 代币转给 taker =====
        //
        // ⚠️ 关键点：为什么这里需要 PDA 签名？
        // 托管账户（offer_token_account）的 authority 是 Offer PDA，不是 maker 也不是 taker。
        // 只有我们的程序能通过 PDA 签名来操作这个账户——这就是「托管」的核心：
        // maker 无法私自取回代币，只有接受报价时程序才会释放。
        //
        // signer_seeds 就是重建 PDA 地址所需的「种子」，相当于程序的密码
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"offer",                                                       // 种子1：固定前缀
            ctx.accounts.offer.maker.as_ref(),                              // 种子2：maker 的公钥
            &ctx.accounts.offer.offer_id.to_le_bytes(),                     // 种子3：报价 ID（小端编码）
            &[ctx.accounts.offer.bump],                                     // 种子4：bump 值
        ]];

        {
            let transfer_cpi_accounts = TransferChecked {
                from: ctx.accounts.offer_token_account.to_account_info(),   // 从托管账户转出
                to: ctx.accounts.taker_token_account_a.to_account_info(),   // 转入 taker 的 mint_a 账户
                mint: ctx.accounts.mint_a.to_account_info(),                // mint_a 的铸造账户
                authority: ctx.accounts.offer.to_account_info(),            // 授权人是 Offer PDA（不是人！）
            };

            // 注意这里多了 .with_signer(signer_seeds) —— 这是 PDA 签名的关键！
            // 普通转账不需要这个，但从程序控制的账户转出必须用 PDA 签名
            let cpi_context =
                CpiContext::new(*ctx.accounts.token_program.key, transfer_cpi_accounts)
                    .with_signer(signer_seeds);

            let decimals = ctx.accounts.mint_a.decimals;
            // 转账金额从 Offer 记录中读取（即 maker 提供的数量）
            transfer_checked(
                cpi_context,
                ctx.accounts.offer.token_a_offered_amount,
                decimals,
            )?;
        }

        // ===== 步骤 3：关闭托管账户，释放租金 =====
        // Solana 中每个账户都需要存储租金（rent），关闭账户时租金会退还给指定地址
        {
            let close_cpi_accounts = CloseAccount {
                account: ctx.accounts.offer_token_account.to_account_info(), // 要关闭的托管账户
                destination: ctx.accounts.maker.to_account_info(),           // 租金退还给 maker
                authority: ctx.accounts.offer.to_account_info(),             // 授权人是 Offer PDA
            };

            // 同样需要 PDA 签名
            let cpi_context = CpiContext::new(*ctx.accounts.token_program.key, close_cpi_accounts)
                .with_signer(signer_seeds);

            close_account(cpi_context)?;
        }

        // ===== 步骤 4：Offer PDA 账户自动关闭 =====
        // 在 TakeOffer 的账户定义中，offer 字段设置了 `close = maker`，
        // Anchor 会自动在指令结束时关闭该账户并将租金退还给 maker。
        // 这里不需要写任何代码！
        Ok(())
    }
}

// ============================================================================
// 账户结构体定义
// ============================================================================
//
// #[derive(Accounts)] 是 Anchor 的宏，用于定义指令所需的账户列表。
// 每个字段上的 #[account(...)] 是约束条件，Anchor 会在指令执行前自动校验这些条件。
// 如果任何约束不满足，交易会在执行指令前被拒绝。

/// ### MakeOffer — make_offer 指令所需的账户
///
/// 这些账户在交易提交时由客户端（前端/测试）提供，Anchor 负责校验。
#[derive(Accounts)]
#[instruction(offer_id: u64)]  // 告诉 Anchor 指令参数中的 offer_id，用于 PDA 种子计算
pub struct MakeOffer<'info> {
    /// maker 是交易的签名者（必须是真人操作），且账户需要可变（因为要付租金和转账）
    #[account(mut)]
    pub maker: Signer<'info>,

    /// mint_a：maker 提供的代币类型的铸造账户
    /// mint::token_program = token_program 表示该 mint 由指定的 token_program 管理
    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    /// mint_b：maker 想要的代币类型的铸造账户
    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>,

    /// maker 的 mint_a 代币账户（关联代币账户，ATA）
    /// 这是 maker 存放 mint_a 代币的地方，指令会从这里转走代币
    /// mut：因为要转出代币，余额会减少
    #[account(
        mut,
        associated_token::mint = mint_a,        // 这个 ATA 对应的是 mint_a
        associated_token::authority = maker,    // ATA 的拥有者是 maker
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    /// Offer PDA 账户 —— 存储报价信息的链上记录
    ///
    /// PDA 地址由种子（seeds）计算得出，不是随机生成的：
    /// - b"offer"：固定前缀，用于区分不同类型的 PDA
    /// - maker.key()：maker 的公钥，确保每个 maker 的报价独立
    /// - offer_id：报价 ID，允许同一个 maker 创建多个报价
    ///
    /// init：表示首次创建该账户（如果已存在会报错）
    /// payer = maker：maker 支付账户创建所需的租金
    /// space = 8 + Offer::INIT_SPACE：8 字节是 Anchor 的类型标识符，INIT_SPACE 是 Offer 结构体大小
    #[account(
        init,
        payer = maker,
        space = 8 + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), &offer_id.to_le_bytes()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    /// 托管账户 —— 以 Offer PDA 为 authority 的 mint_a ATA
    ///
    /// 这是真正「存放代币」的地方。因为 authority 是 PDA（程序控制的地址），
    /// 所以 maker 无法私自取回代币——只有程序能在特定条件下释放。
    /// 这就是「托管」的核心机制！
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,        // 托管的是 mint_a 代币
        associated_token::authority = offer,    // authority 是 Offer PDA（程序控制）
        associated_token::token_program = token_program
    )]
    pub offer_token_account: InterfaceAccount<'info, TokenAccount>,

    /// 关联代币账户程序 —— 用于创建 ATA
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// 代币程序接口 —— 兼容 SPL Token 和 Token-2022
    pub token_program: Interface<'info, TokenInterface>,
    /// 系统程序 —— Solana 的基础程序，用于创建账户等
    pub system_program: Program<'info, System>,
}

/// ### TakeOffer — take_offer 指令所需的账户
///
/// 注意这里大量使用了 Box<> 包装 —— 这是因为 TakeOffer 的账户数量较多，
/// Anchor 对单个指令的账户大小有限制，Box 可以把大结构体放到堆上避免栈溢出。
#[derive(Accounts)]
pub struct TakeOffer<'info> {
    /// taker 是交易的签名者（接单人必须本人操作）
    #[account(mut)]
    pub taker: Signer<'info>,

    /// maker 是系统账户（不需要签名！）
    ///
    /// ⚠️ 为什么 maker 不需要签名？
    /// 因为 take_offer 对 maker 来说只有好处——收到 mint_b 代币。
    /// 即使 maker 离线了，taker 也可以完成交易。
    /// 这也是去中心化的优势：不需要 maker 在线！
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    /// mint_a：报价中涉及的代币 A 的铸造账户（加 Box 减少栈空间）
    #[account(mint::token_program = token_program)]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    /// mint_b：报价中涉及的代币 B 的铸造账户
    #[account(mint::token_program = token_program)]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    /// Offer PDA 账户 —— 读取报价条款
    ///
    /// 这里有多个约束条件，是安全性的关键：
    /// - mut：需要修改（后续要关闭）
    /// - close = maker：指令结束时自动关闭该账户，租金退还给 maker
    /// - has_one = maker：校验 offer.maker == 传入的 maker 账户，防止偷梁换柱
    /// - has_one = mint_a：校验 offer.mint_a == 传入的 mint_a
    /// - has_one = mint_b：校验 offer.mint_b == 传入的 mint_b
    /// - seeds + bump：重新校验 PDA 地址，确保传入的是正确的 Offer 账户
    #[account(
        mut,
        close = maker,
        has_one = maker @ EscrowError::MakerMismatch,
        has_one = mint_a @ EscrowError::MintMismatch,
        has_one = mint_b @ EscrowError::MintMismatch,
        seeds = [b"offer", offer.maker.as_ref(), &offer.offer_id.to_le_bytes()],
        bump = offer.bump
    )]
    pub offer: Box<Account<'info, Offer>>,

    /// taker 接收 mint_a 代币的 ATA
    ///
    /// init_if_needed：如果不存在则自动创建（taker 可能从未持有过 mint_a）
    /// payer = taker：taker 支付创建 ATA 的租金（因为这是 taker 自己的事）
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    /// taker 用于支付的 mint_b ATA
    /// 这个账户必须已存在（taker 必须有 mint_b 代币才能接单）
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// maker 接收 mint_b 代币的 ATA
    ///
    /// init_if_needed：如果 maker 还没有 mint_b 的 ATA 则自动创建
    /// payer = taker：taker 支付创建费用（因为 taker 是发起交易的人）
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// 托管账户 —— 存放 mint_a 代币的地方
    /// 注意这里没有 init，因为账户已经在 make_offer 时创建过了
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub offer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// 系统程序
    pub system_program: Program<'info, System>,
    /// 关联代币账户程序
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// 代币程序接口
    pub token_program: Interface<'info, TokenInterface>,
}

// ============================================================================
// 数据结构定义
// ============================================================================

/// ### Offer —— 存储报价信息的链上数据结构
///
/// 这个结构体的大小是固定的，每个字段都有明确的大小：
/// - Pubkey: 32 字节
/// - u64: 8 字节
/// - u8: 1 字节
/// 总计: 32*3 + 8*3 + 1 = 96 + 24 + 1 = 121 字节
///
/// #[account] 标记这是一个 Solana 链上账户（可持久化存储）
/// #[derive(InitSpace)] 让 Anchor 自动计算结构体的大小（INIT_SPACE 常量）
#[account]
#[derive(InitSpace)]
pub struct Offer {
    /// maker 的公钥 —— 谁创建了这个报价
    pub maker: Pubkey,
    /// mint_a 的公钥 —— maker 提供的代币类型
    pub mint_a: Pubkey,
    /// mint_b 的公钥 —— maker 想要的代币类型
    pub mint_b: Pubkey,
    /// 报价 ID —— 用于区分同一个 maker 的多个报价
    pub offer_id: u64,
    /// maker 提供的 mint_a 代币数量
    pub token_a_offered_amount: u64,
    /// maker 想要的 mint_b 代币数量
    pub token_b_wanted_amount: u64,
    /// PDA 的 bump 值 —— 用于重建 PDA 签名种子
    pub bump: u8,
}

// ============================================================================
// 自定义错误码
// ============================================================================

/// ### EscrowError —— 程序自定义的错误类型
///
/// 在 Solana 中，每个程序可以定义自己的错误码（从 6000 开始）。
/// 当交易失败时，客户端可以通过错误码知道具体哪里出了问题。
#[error_code]
pub enum EscrowError {
    /// 6000: maker 账户不匹配 —— 传入的 maker 不是创建该 Offer 的人
    #[msg("Maker account does not match Offer.maker")]
    MakerMismatch,
    /// 6001: mint 账户不匹配 —— 传入的 mint_a 或 mint_b 与 Offer 中记录的不一致
    #[msg("Mint account does not match Offer mints")]
    MintMismatch,
    /// 6002: 相同的代币 —— mint_a 和 mint_b 不能是同一种代币（自己换自己没有意义）
    #[msg("mint_a and mint_b must be different")]
    SameMint,
}
