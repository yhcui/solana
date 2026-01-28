# 添加依赖
cargo add anchor-lang --features init-if-needed
cargo add anchor-spl


--features 是用来开启一个库的可选功能（Optional Features）。
具体到 anchor-lang 的 init-if-needed

默认情况下，如果你在 Anchor 中使用 init 约束来创建一个账户，而这个账户已经存在了，交易会报错并失败（报错信息通常是 AccountAlreadyInitialized）。
开启 init-if-needed 功能后，你可以在 #[account(...)] 约束中使用这个同名属性：

```
// 只有在 Cargo.toml 中开启了 init-if-needed，这段代码才能编译通过
#[account(
    init_if_needed,  // <--- 核心属性
    payer = signer,
    space = 8 + 32,
    seeds = [b"vault", signer.key().as_ref()],
    bump
)]
pub struct MyVault<'info> { ... }

```

它的逻辑是：  
如果账户不存在：执行 init 逻辑（分配空间、转移租金、设置 Owner）。  
如果账户已存在：跳过初始化步骤，直接加载该账户进行后续操作。

为什么它是一个“Feature”而不是默认开启？
Anchor 官方将其设为可选功能，主要是出于安全性考虑：

1、重置攻击风险 (Re-initialization Attacks)： 如果不加防护地使用 init_if_needed，攻击者可能会尝试通过某种方式触发初始化逻辑，从而覆盖或篡改现有账户的状态。

2、代码膨胀： 为了实现“如果不存在则初始化”的逻辑，Anchor 需要在生成的底层代码中加入更多的条件判断语句，这会稍微增加程序的编译大小（CU 消耗）。

# mod 的作用
代码的“文件夹”
mod（module）在 Rust 中用于组织代码结构。

mod state; / mod instructions;： 这告诉编译器：“去寻找同名的文件（如 state.rs）或者文件夹下的 mod.rs（如 instructions/mod.rs）。” 它相当于把那个文件的代码内容挂载到了当前位置。

如果不写 mod： 即便你的文件夹里有这些文件，编译器也不会去读取它们。在 Rust 中，文件必须被显式地声明为模块才能被编译。

# use super::*;
use super::*; 的作用：打破层级壁垒

这是一个作用域（Scope）的问题。

在 Rust 中，每一个 mod 块都会开启一个全新的、干净的命名空间。

你在文件最顶部（外部）写的 use anchor_lang::prelude::*; 只能在外部作用域生效。

当你进入 pub mod blueshift_anchor_escrow { ... } 内部时，里面的代码看不见外部已经导入的东西。

use super::*; 的含义是：

super：指代当前模块的父级（即这个文件最顶层的作用域）。

*：把父级作用域里所有的内容（包括你之前 use 的 prelude、instructions 等）全都“拉”进这个子模块里。

# #[instruction(...)]

在标准的 Anchor 开发中，你通常不需要手动指定 discriminator。Anchor 会根据函数名的前 8 字节哈希值自动生成。

手动指定通常用于：兼容旧的非 Anchor 程序 或者 特殊的协议需求。