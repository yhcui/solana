/*
pub mod deposit; // 模块声明  这行代码告诉 Rust 编译器："存在一个名为 deposit 的模块"
创建命名空间：创建一个叫 deposit 的容器
文件关联：Rust 会在以下位置寻找模块内容：
    deposit.rs 文件
    deposit/mod.rs 文件
作用域限制：模块内的内容仍然在 deposit:: 命名空间下
*/
pub mod deposit;
pub mod withdraw; // 导入 withdraw 模块

/*
pub use deposit::*; - 重新导出
将 deposit 模块中的所有 public 项引入当前作用域
具体含义
    项提升：将模块内的公开项提升到当前模块级别
    命名空间扁平化：不再需要 deposit:: 前缀
    外部可见：使用当前模块的人也能访问这些项
*/
pub use deposit::*; // 将 deposit 模块的所有公开项引入当前作用域
pub use withdraw::*; // 将 withdraw 模块的所有公开项引入当前作用域

#[cfg(feature="idl-build")]
use pinocchio::account;
pub use withdraw::*;
/*
IDL 概念说明
IDL（Interface Definition Language）：定义程序接口的描述文件
用于前端应用与 Solana 程序交互

*/
#[cfg(feature="idl-build")] // 仅在启用 idl-build 功能时编译
use {
    borsh::{BorshDeserialize, BorshSerialize}, // 序列化/反序列化工具
    shank::ShankInstruction,    // 生成 IDL 的宏
};
/*

合约代码中没有使用
由于使用了 #[cfg(feature="idl-build")]，这个 VaultInstructions 枚举只在构建 IDL 时才会编译，这意味着：

1、生产环境：这个枚举不会被编译，减少程序大小
2、开发/部署环境：当需要生成 IDL 时才会编译这个部分
3、客户端开发：提供类型安全的接口定义

数据流向
1、客户端构建指令 → VaultInstructions::Deposit(DepositArgs { amount }) → 
2、序列化为字节数组 → 发送到 Solana 网络 → 程序入口点反序列化 → 
3、匹配到相应分支 → 调用 deposit 模块处理
*/
#[cfg(feature="idl-build")] // 仅在启用 idl-build 功能时编译
#[derive(Debug, Clone,ShankInstruction, BorshSerialize, BorshDeserialize )]
#[rustfmt::skip]
pub enum VaultInstructions {
    // Deposit 指令
    #[account(0, signers, writable, name="owner", desc="存款人和支付者")]
    #[account(1, writable, name="vault", desc="派生的PDA托管账户")]
    #[account(2, name="system_program", desc="系统程序")]
    Deposit(DepositArgs),
    // Withdraw 指令
    #[account(0, signers, writable, name="owner", desc="提款人和接收者")]
    #[account(1, writable, name="vault", desc="派生的PDA托管账户")]
    #[account(2, name="system_program", desc="系统程序")]
    Withdraw,
}

#[cfg(feature="idl-build")]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize )]
pub struct DepositArgs {
    pub amount: u64,
}