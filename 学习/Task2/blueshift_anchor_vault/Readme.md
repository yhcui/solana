# Account Constraints（账户约束）
##  1. 基础约束 (Basic Constraints)  
这些是最常用的，用于定义账户的基本读写权限和所有权。
- mut: 声明账户是可变的。如果你的指令会修改账户数据或扣除 SOL，必须加这个。
- signer: 强制要求该账户必须在交易中签名。
- owner = <expr>: 验证账户的所有者是否为某个指定的程序（默认通常是当前程序）。
- executable: 验证该账户是否为一个可执行的程序账户。
## 2. PDA 与初始化约束 (PDA & Initialization)
这是你代码中用到的部分，用于创建和定位 PDA (Program Derived Address)。
- init: 告诉 Anchor 创建这个账户。必须配合 payer 和 space 使用。
- payer = <target>: 指定谁来支付创建账户所需的租金（Rent）。
- space = <size>: 指定账户分配的大小（以字节为单位）。通常使用 8 + size_of::<MyStruct>()。
- seeds = [...]: PDA 的种子。例如 [b"vault", user.key().as_ref()]。
- bump: PDA 的碰撞种子。如果不赋值，Anchor 会自动寻找并验证正确的 bump。
- init_if_needed: 如果账户不存在则初始化，存在则跳过（需谨慎使用，防止重新初始化攻击）。
## 3. 关联代币账户约束 (SPL Token Constraints)
当你处理代币（Token）时，这些约束非常强大：
- mint = <target>: 验证代币账户所属的币种（Mint）。
- authority = <target>: 验证代币账户的授权持有者。
- token::mint / token::authority: 专门用于简化的代币操作校验。
## 4. 安全验证约束 (Validation Constraints)用于执行更复杂的业务逻辑检查：
- constraint = <expression>: 万能约束。你可以写任何返回布尔值的 Rust 表达式。例如：#[account(constraint = user.age > 18)]。
- has_one = <target>: 验证账户中的某个字段是否匹配另一个账户。
  例如：#[account(has_one = admin)] 会检查 my_account.admin == admin.key()。
## 5. 常见组合示例
为了更直观，请看这张功能分类图：
|组合场景|常用属性清单|
|---|---|
|创建新账户| init, payer, space, seeds, bump|
|修改已有数据|mut, has_one|
|仅校验权限|signer|
|关闭账户|mut, close = <destination> (将余额退回并销毁账户)|

# 宏触发执行
#[derive(Accounts)]  这样定义了struct，什么时间触发执行这个宏？

这个宏触发执行分为两个阶段：编译时（Compile Time） 和 运行时（Runtime）

## 1. 编译阶段：代码生成 (Macro Expansion)
当你运行 anchor build 或 cargo build 时，Rust 编译器会扫描到 #[derive(Accounts)]。

- 触发动作：编译器会调用 Anchor 定义的“过程宏”逻辑。

- 生成内容：它会为你的 VaultAction 结构体自动生成一个名为 try_accounts 的函数（以及相关 Trait 的实现）。

- 生成的代码长什么样？ 它会把你在 #[account(...)] 里写的各种约束（如 mut, seeds, bump）转换成一行行硬核的 Rust 判断代码。

## 2. 运行阶段：逻辑触发 (Execution)
真正的“校验”发生在你调用合约指令的那一刻。

当一个交易（Transaction）到达你的程序时，流程如下：

1. Entrypoint 接收数据：Solana 运行时把一堆原始字节传给你的程序。

2. 触发反序列化：在进入你的业务函数（比如 pub fn deposit(...)）之前，Anchor 会首先调用生成的 try_accounts 方法。
3. 执行校验逻辑：
- 权限检查：检查 singer 账户是否真的在交易中签名了。
- PDA 派生检查：Anchor 会根据你提供的 seeds（b"vault" + singer 的地址）在后台重新计算一个地址，然后对比交易传入的 vault 账户地址是否一致。
- 状态检查：检查 vault 和 singer 是否被标记为 mut（可变）。

4. 注入业务逻辑：
- 成功：如果所有检查都通过，Anchor 会把这些验证过的账户填入 VaultAction 结构体，并传给你的函数上下文 ctx.accounts。

- 失败：只要有一个约束不满足（比如你传了一个错误的 PDA），try_accounts 就会立即返回错误，你的业务逻辑代码根本不会被执行。

5. 为什么这样设计？（解耦安全与逻辑）
这种模式被称为声明式校验。

# 'info 
'info 被称为 生命周期标注 (Lifetime Annotation)。
也可以用'a来替换或着'b?

简单来说，它的作用是告诉 Rust 编译器：“这些账户引用的有效时间，必须和这笔交易（Transaction）的生命周期一样长。”
