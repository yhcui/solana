use pinocchio::Address;
use pinocchio::error::ProgramError;
use core::mem::size_of; // 引入 Rust 核心库的内存大小计算函数
/*
#[repr(C)] 是一个 Rust 属性（attribute），用于控制结构体的内存布局：

主要作用：
1、指定内存表示：告诉编译器按照 C 语言的内存布局规则来排列结构体字段
2、保证字段顺序：确保结构体字段在内存中的排列顺序与源代码中定义的顺序一致
3、确定对齐方式：使用 C 语言的对齐规则而非 Rust 的默认优化对齐

在 Solana 程序中的重要性：
1、跨语言兼容性：确保 Rust 结构体与 C/C++ 或其他语言的结构体具有相同的内存布局
2、ABI 兼容性：保证与其他组件（如客户端或系统调用）交互时的数据格式正确
3、序列化一致性：在将结构体转换为字节数组时，保证字节顺序和布局的可预测性

*/
#[repr(C)]
pub struct Escrow{
    // 种子：用于派生 PDA 的随机数
    // 确保每个托管账户都有唯一的地址
    // 客户端和程序使用相同的种子 + maker + mint_a 可以派生出相同的 PDA
    pub seed: u64,

    // 创建者：发起托管交易的用户地址
    // 用于验证只有创建者才能执行退款操作
    pub maker: Address,
    /*
    Mint 地址：Solana 上代币类型的唯一标识符，类似于 ERC-20 合约地址
    作用：
    1、标识代币类型：记录被托管的代币是什么（如 USDC、SOL 或其他 SPL 代币）
    2、验证安全性：确保存入和提取的是正确的代币类型
    3、PDA 派生依据：作为派生程序地址的重要参数之一
    */
    // 代币 A 的 mint 地址：被存入金库的代币类型
    // 例如：如果是 SOL，则是 SOL 的 mint 地址
    pub mint_a: Address,

    pub mint_b: Address,
    
    // 期望数量：创建者希望获得的代币 B 的数量
    // 接受者必须发送至少这个数量的代币 B 才能接受交易
    
    pub receive: u64,
    /*
    虽然 [u8; 1] 和 u8 在逻辑上都表示单个字节，但在 内存布局和 ABI 兼容性 方面存在重要差异：

    关键区别：
    1. 类型表示
    u8 - 单个字节值
    [u8; 1] - 包含一个 u8 元素的数组
    2. 内存布局差异
    u8 - 可能因对齐要求而占用更多空间（取决于编译器优化）
    [u8; 1] - 明确表示一个固定大小的单字节数组，有更严格的内存布局
    3. #[repr(C)] 的影响
    从上下文可以看到，Escrow 结构体使用了 #[repr(C)] 属性，这要求：

    严格按照字段定义顺序排列
    保证跨平台的内存布局一致性
    [u8; 1] 提供了更明确的内存布局保证
    在 Solana 程序中的实际意义：
    序列化/反序列化 - 确保字节数据的准确转换
    PDA 验证 - PDA 派生时需要精确的内存布局
    跨语言互操作 - 与其他组件通信时保持数据格式一致性
    正如代码注释所说："使用 [u8; 1] 而不是 u8 是为了确保内存布局"，这是为了保证结构体在序列化为字节流时，每个字段都占据预期内的字节位置，避免因编译器对齐优化导致的意外偏移。
    */
    // Bump 种子：PDA 派生时找到的有效 bump 值
    // Solana 使用 "find_program_address" 查找 PDA，会返回一个 bump 值
    // 验证签名时需要提供这个 bump 值（通常追加在 seeds 后面）
    // 使用 [u8; 1] 而不是 u8 是为了确保内存布局
    // [u8; 1] 是 Rust 中的数组类型语法，表示一个包含 1 个 u8 类型元素的固定大小数组。
    // [T; N]：Rust 数组类型的通用语法
    pub bump:[u8;1]
}

// Escrow 结构体的方法实现
impl Escrow {
    // ------------------------------------------------------------------------
    // 常量：账户数据长度
    // ------------------------------------------------------------------------
    // 这是 Escrow 结构体在链上账户中占用的总字节数
    // 计算方式：每个字段的大小之和
    // - u64: 8 字节
    // - Address: 32 字节
    // - [u8; 1]: 1 字节
    // 总计：8 + 32 + 32 + 32 + 8 + 1 = 113 字节
    //
    // 用途：创建账户时需要指定空间大小，客户端和程序都需要知道这个值
    // <Address>() 泛型参数的显式指定语法。<Address> - 指定泛型类型参数为 Address 类型。() - 函数调用的参数列表（无参数）
    pub const LEN: usize = size_of::<u64>()                     // seed: 8 字节
        + size_of::<Address>()                                  // maker: 32 字节
        + size_of::<Address>()                                  // mint_a: 32 字节
        + size_of::<Address>()                                  // mint_b: 32 字节
        + size_of::<u64>()                                      // receive: 8 字节
        + size_of::<[u8;1]>();                                  // bump: 1 字节

    // 从字节数组（账户数据）中加载 Escrow 结构体的可变引用
    // 参数：
    //   bytes: 账户数据的可变字节数组切片
    // 返回：
    //   成功：返回 Escrow 的可变引用
    //   失败：返回 InvalidAccountData 错误
    //
    // 安全性：
    //   使用 unsafe 代码块和 transmute 将字节指针转换为结构体指针
    //   这是因为我们需要直接操作原始内存，避免复制开销
    //   前提条件：字节数组必须足够大且内存布局正确
    //
    // #[inline(always)]:
    //   强制编译器内联此函数，消除函数调用开销
    //   对于这种小型辅助函数，内联能提高性能
    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Escrow::LEN{
            return Err(ProgramError::InvalidAccountData);
        }

        /*
        什么是 unsafe？
        unsafe 块：允许 Rust 开发者绕过某些内存安全检查的特殊代码块
        原因：当编译器无法验证某些操作的安全性，但开发者可以保证其安全性时使用
        为什么要在这里使用 unsafe？
        原始内存操作：直接将字节数组转换为结构体指针
        内存布局假设：假设输入的字节数组具有正确的 Escrow 结构体布局
        性能优化：避免数据复制，直接操作内存
        as_mut_ptr() 解析：将 &mut [u8] 切片转换为原始可变指针 *mut u8。返回指向切片首字节的原始指针
        *mut u8 - 可变原始指针
            特点：
            原始指针：不涉及所有权概念
            不安全：绕过 Rust 的借用检查器
            无生命周期：不受生命周期约束
            无边界检查：访问时不进行安全检查

        &mut [u8] - 可变借用切片
        特点：
        借用引用：不拥有数据的所有权，只是借用了数据
        类型安全：受到 Rust 借用检查器的严格管控
        生命周期：有明确的生命周期限制
    
        ransmute 函数：
            功能：将一种类型强制转换为另一种类型，不改变底层数据
            参数：<原类型, 目标类型>
            危险性：完全绕过类型系统检查，必须由开发者保证安全
         转换过程：
            &mut [u8] → *mut u8 (通过 as_mut_ptr())
            *mut u8 → *mut Self (通过 transmute)   
        */
        Ok(unsafe {&mut *core::mem::transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

     // 从字节数组（账户数据）中加载 Escrow 结构体的只读引用
    #[inline(always)]
    pub fn load(bytes:&[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Escrow::LEN{
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe {&*core::mem::transmute::<*const u8, *const Self>(bytes.as_ptr())})
    }
     // 为什么需要这些 setter 方法？
    // - Pinocchio 不像 Anchor 那样自动实现序列化
    // - 需要手动提供方法来修改结构体字段
    // - 提供一致的 API 接口
    #[inline(always)]
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    #[inline(always)]
    pub fn set_maker(&mut self, maker: Address) {
        self.maker = maker;
    }

    #[inline(always)]
    pub fn set_mint_a(&mut self, mint_a: Address) {
        self.mint_a = mint_a;
    }

    #[inline(always)]
    pub fn set_mint_b(&mut self, mint_b: Address) {
        self.mint_b = mint_b;
    }

    #[inline(always)]
    pub fn set_receive(&mut self, receive: u64) {
        self.receive = receive;
    }

    #[inline(always)]
    pub fn set_bump(&mut self, bump: [u8;1]) {
        self.bump = bump;
    }

    // 一次性设置所有字段，避免多次函数调用
     #[inline(always)]
    pub fn set_inner(&mut self, seed: u64, maker: Address, mint_a: Address, mint_b: Address, receive: u64, bump: [u8;1]) {
        self.seed = seed;
        self.maker = maker;
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self.receive = receive;
        self.bump = bump;
    }

}