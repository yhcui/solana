// 导入 Pinocchio 核心组件：
// - AccountView: 用于读取和操作账户数据的视图
// - Address: 表示 Solana 地址（公钥/程序 ID）
// - entrypoint: 宏，用于定义程序的入口点
// - ProgramResult: 程序执行结果的类型别名（Result<(), ProgramError>）
use pinocchio::{AccountView,Address, entrypoint, ProgramResult};
use pinocchio::error::ProgramError;
// 声明程序的入口点函数
// Solana 运行时会调用这个函数来执行程序逻辑
entrypoint!(process_instruction);

/*
pub mod instructions;
声明模块：告诉编译器存在一个名为 instructions 的模块
创建命名空间：建立模块层级结构
可见性控制：pub 使模块对外部可见

模块层级定义
定义项目的模块层次结构
让编译器知道存在哪些子模块
通常与对应的 instructions.rs 文件或 instructions/mod.rs 目录关联
*/
pub mod instructions;
/*
pub use instructions::*;
重新导出：将 instructions 模块中的所有公共项导入当前作用域
方便使用：让外部使用者可以直接访问模块内的项
*/
pub use instructions::*;
pub mod errors;
pub use errors::*;
pub mod state;
pub use state::*;
/*
ID 是这个 Solana 程序的 程序地址/程序 ID，具体说明如下：

作用：
程序唯一标识符 - 这是该 Escrow 程序在 Solana 区块链上的唯一地址
程序身份 - 其他程序或客户端通过这个 ID 来识别和调用该程序


技术细节：
类型：Address（32 字节数组）
值：固定的 32 字节公钥（对应上面的十六进制数组）
创建方式：使用 Address::new_from_array 从字节数组创建

在 Solana 生态中的重要性：
程序调用 - 客户端需要知道这个 ID 来调用程序
账户关联 - 程序创建的 PDA（程序派生地址）会与此 ID 相关
权限验证 - 验证某些操作是否由正确的程序执行


绝对不能定义了已经在solana中存在的地址，否则部署失败。
可以使用 solana program deploy 自动生成新地址
如果你想更新程序逻辑，使用 solana program upgrade
*/
pub const ID: Address = Address::new_from_array(
    [
        0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07,
        0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
        0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07,
        0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7,
    ]
);

// 参数说明：
// - _program_id: 当前程序的 ID（此处未使用，因为有常量 ID）
// - accounts: 交易中传入的所有账户列表（可变和只读账户）
// - instruction_data: 指令数据，包含操作类型和参数

// 指令路由机制：
// 程序使用"判别器（Discriminator）"模式来路由不同的指令
// 每个指令都有一个唯一的字节（DISCRIMINATOR）作为标识
// Solana 运行时会将 instruction_data 的第一个字节与判别器匹配
// 来决定调用哪个指令处理器
fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    

    match instruction_data.split_first() {
        Some((Make::DISCRIMINATOR, data)) => Make::try_from((data,accounts))?.process(),
        Some((Take::DISCRIMINATOR,_)) => Take::try_from(accounts)?.process(),
        Some((Refund::DISCRIMINATOR,_)) => Refund::try_from(accounts)?.process(),
        _=> Err(ProgramError::InvalidInstructionData),
    }
}