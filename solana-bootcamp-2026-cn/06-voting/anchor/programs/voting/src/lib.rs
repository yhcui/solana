//! # Solana 链上投票程序
//!
//! 一个基于 Anchor 框架的 Solana 智能合约，实现链上投票/投票功能。
//!
//! ## 核心概念
//!
//! - **PDA（程序派生地址）**: 所有状态账户都使用 PDA，意味着地址由程序 ID 和种子确定性派生，
//!   而非由用户密钥对控制。这是 Solana 上程序管理状态的标准模式。
//! - **时间门控**: 投票指令会检查当前链上时间是否在投票窗口内，不在窗口内的投票会被拒绝。
//! - **零权限模型**: 投票创建后，任何符合条件的钱包都可以投票，无需特殊权限。
//!
//! ## 账户结构
//!
//! ```text
//! PollAccount PDA: seeds = ["poll", poll_id(小端序u64)]
//!   ├── poll_name: String (≤32字符)
//!   ├── poll_description: String (≤280字符)
//!   ├── poll_voting_start: u64 (Unix时间戳)
//!   ├── poll_voting_end: u64 (Unix时间戳)
//!   └── poll_option_index: u64 (已添加候选者计数)
//!
//! CandidateAccount PDA: seeds = [poll_id(小端序u64), candidate_name]
//!   ├── candidate_name: String (≤32字符)
//!   └── candidate_votes: u64 (得票数)
//! ```
//!
//! ## 业务流程
//!
//! 1. 调用 `initialize_poll` 创建投票 → 生成 PollAccount PDA
//! 2. 调用 `initialize_candidate` 添加候选者 → 生成 CandidateAccount PDA，poll_option_index +1
//! 3. 在投票窗口期内调用 `vote` 投票 → candidate_votes +1

use anchor_lang::prelude::*;

// 程序 ID，用于在链上唯一标识此程序
// 此地址由 `anchor build` 和 `anchor deploy` 自动生成或指定
declare_id!("65KHV8cXwJ8apTKMqnpSdhdHkHhRySatgKMwnxm6C3gG");

/// 指令处理模块
///
/// `#[program]` 宏将模块内的函数暴露为 Solana 程序的入口点（instruction handler）。
/// 每个 `pub fn` 对应一个可被客户端调用的指令。
/// 包含多个公共函数（pub fn）。每一个公共函数都会成为该智能合约的一个独立指令入口点（Instruction Entry Point）
#[program]
pub mod voting {
    use super::*;

    /// 创建一个新的投票账户
    ///
    /// 使用 PDA 派生地址，seed 为 `["poll", poll_id的小端序字节]`。
    /// 如果该 PDA 已存在则复用（`init_if_needed`），因此同一个 poll_id 可以安全地重复调用。
    ///
    /// # 参数
    ///
    /// * `_poll_id` - 投票的唯一标识符，用作 PDA 种子（前缀下划线表示在函数体内未直接使用，
    ///   但 Anchor 通过 `#[instruction]` 属性在账户验证阶段使用它来计算 PDA）
    /// * `start_time` - 投票开始的 Unix 时间戳（秒）
    /// * `end_time` - 投票结束的 Unix 时间戳（秒）
    /// * `name` - 投票名称
    /// * `description` - 投票描述
    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        _poll_id: u64,
        start_time: u64,
        end_time: u64,
        name: String,
        description: String,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll_account;
        poll.poll_name = name;
        poll.poll_description = description;
        poll.poll_voting_start = start_time;
        poll.poll_voting_end = end_time;
        // poll_option_index 默认初始化为 0
        Ok(())
    }

    /// 向现有投票添加一个候选者
    ///
    /// 为每个候选者创建独立的 PDA 账户，seed 为 `[poll_id的小端序字节, candidate_name]`。
    /// 同时将 PollAccount 中的 `poll_option_index` 加 1，用于记录候选者总数。
    ///
    /// # 参数
    ///
    /// * `_poll_id` - 目标投票的 ID（用于 PDA 种子计算）
    /// * `candidate` - 候选者名称，同时也是 PDA 种子之一
    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        _poll_id: u64,
        candidate: String,
    ) -> Result<()> {
        ctx.accounts.candidate_account.candidate_name = candidate;
        ctx.accounts.poll_account.poll_option_index += 1;
        Ok(())
    }

    /// 为指定候选者投一票
    ///
    /// 此指令会检查当前链上时间是否在投票窗口 [start_time, end_time) 内：
    /// - 如果当前时间 <= start_time，返回 VotingNotStarted 错误
    /// - 如果当前时间 >= end_time，返回 VotingEnded 错误
    ///
    /// 注意：投票者身份没有限制，任何签名者都可以在窗口期内投票。
    ///
    /// # 参数
    ///
    /// * `_poll_id` - 目标投票的 ID
    /// * `_candidate` - 要投票的候选者名称
    pub fn vote(ctx: Context<Vote>, _poll_id: u64, _candidate: String) -> Result<()> {
        let candidate_account = &mut ctx.accounts.candidate_account;
        // 获取当前链上时间（Unix 时间戳，秒）
        let current_time = Clock::get()?.unix_timestamp;

        // 检查投票是否已结束（>= end_time 表示已过期）
        if current_time >= ctx.accounts.poll_account.poll_voting_end as i64 {
            return Err(ErrorCode::VotingEnded.into());
        }

        // 检查投票是否尚未开始（< start_time 表示未开始）
        if current_time < ctx.accounts.poll_account.poll_voting_start as i64 {
            return Err(ErrorCode::VotingNotStarted.into());
        }

        candidate_account.candidate_votes += 1;

        Ok(())
    }
}

/// `InitializePoll` 指令的账户验证结构体
///
/// `#[derive(Accounts)]` 宏生成账户验证逻辑，确保传入的账户满足约束条件。
/// `#[instruction(poll_id: u64)]` 将指令参数引入账户验证上下文，
/// 使种子计算可以使用 `poll_id` 参数。
#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    /// 交易签名者（创建账户的付款人）
    /// `#[account(mut)]` 标记此账户需要可变访问（用于扣减 SOL 余额支付租金）
    #[account(mut)]
    pub signer: Signer<'info>,

    /// 投票账户（PDA）
    /// - `init_if_needed`: 如果账户不存在则创建，存在则直接访问（幂等操作）
    /// - `payer = signer`: 由签名者支付创建账户所需的 SOL（租金豁免押金）
    /// - `space = 8 + PollAccount::INIT_SPACE`: 分配空间 = 8字节锚点鉴别器 + 结构体实际大小
    /// - `seeds = [...]`: PDA 种子，派生规则为 ["poll", poll_id的小端序字节]
    /// - `bump`: 自动查找并使用有效的 bump seed
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    /// 系统程序（System Program），创建账户时必须传入
    pub system_program: Program<'info, System>,
}

/// `InitializeCandidate` 指令的账户验证结构体
#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct InitializeCandidate<'info> {
    /// 交易签名者（支付候选者账户创建费用）
    #[account(mut)]
    pub signer: Signer<'info>,

    /// 已有的投票账户（PDA），需要可变访问以更新 poll_option_index
    /// 注意：这里没有 `init`，只读取已有的 PollAccount
    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>,

    /// 候选者账户（PDA），新创建
    /// - `init`: 必须创建新账户（重复候选者名称会导致交易失败）
    /// - `seeds`: 派生规则为 [poll_id的小端序字节, candidate_name]
    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,

    /// 系统程序，创建账户时必须传入
    pub system_program: Program<'info, System>,
}

/*
#[derive(Accounts)]
作用：自动生成账户验证逻辑（Account Validation）。
1、生成验证代码：它会将结构体中的每个字段（如 signer, poll_account, candidate_account）转换为 Solana 交易所需的账户检查逻辑。
2、强制执行约束：它会读取字段上的属性（如 #[account(mut)], seeds = [...], init 等），并在运行时强制检查：
    账户是否已签名（Signer）。
    账户地址是否与指定的种子（Seeds）和程序 ID 匹配（PDA 验证）。
    账户所有者是否正确。
    账户是否有足够的空间或是否已初始化。
3、简化开发：如果没有这个宏，开发者需要手动编写大量的样板代码来检查传入的账户是否合法，容易出错且不安全。

#[instruction(poll_id: u64, candidate: String)]
作用将指令函数的参数引入到账户验证上下文中。
参数共享：在 Solana 中，指令的参数（如 poll_id 和 candidate）是在函数签名中定义的，但账户验证逻辑（在 #[derive(Accounts)] 生成的代码中）也需要知道这些参数，以便计算 PDA 地址或进行其他校验。
*/
/// `Vote` 指令的账户验证结构体
#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct Vote<'info> {
    /// 投票者（交易签名者），需要可变标记因为交易需要支付手续费
    #[account(mut)]
    pub signer: Signer<'info>,

    /// 投票账户（PDA），需要可变访问以读取投票时间窗口
    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>,

    /// 候选者账户（PDA），需要可变访问以增加得票数
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,
}

/// 候选者账户的数据结构
///
/// `#[account]` 宏将此结构体注册为 Anchor 账户类型，
/// `#[derive(InitSpace)]` 自动计算序列化后的空间大小。
#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    /// 候选者名称，最大 32 字节
    #[max_len(32)]
    pub candidate_name: String,
    /// 得票总数
    pub candidate_votes: u64,
}

/// 投票账户的数据结构
///
/// 存储投票的元数据和时间窗口信息。
/* 

#[account]
作用：将 Rust 结构体注册为 Solana 账户类型。
1、序列化/反序列化：它告诉 Anchor 如何将这个结构体的数据序列化（写入链上）和反序列化（从链上读取）。
2、鉴别器（Discriminator）：Anchor 会自动为该账户类型生成一个唯一的 8 字节鉴别器。这用于在运行时验证传入的账户确实是 PollAccount 类型，防止类型混淆攻击。
3、所有权检查：在账户验证阶段，Anchor 会检查该账户的所有者是否为当前程序的 ID。
4、实现 trait：它为结构体实现了 anchor_lang::AccountSerialize 和 anchor_lang::AccountDeserialize 等必要特质

#[derive(InitSpace)]
作用：自动计算并生成该结构体在链上所需的字节大小（空间）。

1、自动计算：它会分析结构体中的每个字段，根据它们的类型和属性（如 #[max_len(...)]）计算出序列化后占用的总字节数。
    例如：u64 固定占 8 字节。
    例如：String 带有 #[max_len(32)]，Anchor 会按照最大长度 32 字节加上长度前缀（通常 4 字节）来计算空间。
2、生成常量：它会为结构体生成一个关联常量 INIT_SPACE。
3、用途：在初始化账户时（如 InitializePoll 中的 #[account(init, space = 8 + PollAccount::INIT_SPACE, ...)]），你需要指定分配多少空间给这个账户。使用 INIT_SPACE 可以确保分配的空间刚好够用，既不会浪费 SOL（租金），也不会因为空间不足导致写入失败。
简单来说：它帮你算好了“这个账户需要多大地方”，你直接在 init 时使用 PollAccount::INIT_SPACE 即可，无需手动计算字节数，避免出错。
#[account(
    init_if_needed,
    payer = signer,
    // 这里使用了 InitSpace 生成的常量来指定空间大小
    // 8 字节是 Anchor 的鉴别器大小，INIT_SPACE 是结构体数据本身的大小
    space = 8 + PollAccount::INIT_SPACE, 
    seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
    bump
)]
pub poll_account: Account<'info, PollAccount>,

*/
#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    /// 投票名称，最大 32 字节
    #[max_len(32)]
    pub poll_name: String,
    /// 投票描述，最大 280 字节（类似推特的长度限制）
    #[max_len(280)]
    pub poll_description: String,
    /// 投票开始的 Unix 时间戳（秒）
    pub poll_voting_start: u64,
    /// 投票结束的 Unix 时间戳（秒）
    pub poll_voting_end: u64,
    /// 已添加的候选者数量计数器，每调用一次 initialize_candidate 加 1
    pub poll_option_index: u64,
}

/// 自定义错误码
///
/// `#[error_code]` 宏将枚举值注册为程序可返回的错误类型。
/// `#[msg(...)]` 提供人类可读的错误描述。
#[error_code]
pub enum ErrorCode {
    /// 当前时间早于或等于投票开始时间
    #[msg("Voting has not started yet")]
    VotingNotStarted,
    /// 当前时间大于或等于投票结束时间
    #[msg("Voting has ended")]
    VotingEnded,
}
