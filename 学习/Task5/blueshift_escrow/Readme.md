# Blueshift Escrow - Solana 托管系统

基于 Solana 区块链的去中心化代币托管系统，使用 Pinocchio 框架实现，同时提供 Anchor 版本对比。

## 🌟 核心特性

- **无需信任的代币交换**：安全的原子交换，无需中介机构
- **三大核心操作**：
  - **Make（创建）**：创建托管并存入 Token A
  - **Take（接受）**：接受托管并用 Token B 交换
  - **Refund（退款）**：取消托管并取回 Token A
- **双框架实现**：Pinocchio（高性能）和 Anchor（开发友好）
- **完善的文档**：详细的中文注释和完整业务流程文档
- **安全优先**：五层安全机制，包括 PDA 权限、原子性保证和重放保护

## 🔑 核心概念

### PDA（程序派生地址）
- 由程序 ID + 种子派生的确定性地址
- 没有私钥，仅由程序控制
- 通过程序化权限确保资金安全

### ATA（关联代币账户）
- Solana 标准的代币存储账户
- 从 `[owner, token_program, mint]` 自动计算地址
- 用于金库和用户代币账户

## 📁 项目结构

```
blueshift_escrow/
├── src/
│   ├── lib.rs              # Pinocchio 入口文件
│   ├── state.rs            # 托管账户状态定义
│   ├── errors.rs           # 自定义错误类型
│   ├── instructions/
│   │   ├── mod.rs          # 指令分发器
│   │   ├── helpers.rs      # 账户验证 Trait
│   │   ├── make.rs         # Pinocchio Make 指令
│   │   ├── take.rs         # Pinocchio Take 指令
│   │   ├── refund.rs       # Pinocchio Refund 指令
│   │   ├── make_anchor.rs  # Anchor Make 指令
│   │   ├── take_anchor.rs  # Anchor Take 指令
│   │   ├── refund_anchor.rs# Anchor Refund 指令
│   │   └── take_copy.rs    # Take 指令副本
│   └── tests/
│       ├── mod.rs          # 测试模块入口
│       └── test.rs         # 集成测试
├── doc/
│   ├── WORKFLOW.md         # 完整业务流程文档
│   ├── FUND_FLOW.md        # 资金流向说明
│   ├── PPT_OUTLINE.md      # 演示文稿大纲
│   ├── PPT_STYLE.md        # 演示文稿样式
│   ├── SPEECH.md           # 演讲稿
│   └── Blueshift 托管系统.pptx  # 演示文稿
├── CLAUDE.md               # 项目说明文档
├── Cargo.toml              # Rust 项目配置
└── README.md               # 项目说明
```

## 🚀 快速开始

### 前置要求

- Rust 工具链（已安装 `rustup`）
- Solana CLI（`solana-install`）
- Anchor 框架（用于 Anchor 版本）

### 安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/blueshift_escrow.git
cd blueshift_escrow

# 构建 Pinocchio 版本
cargo build-sbf
```

## 📖 使用方法

### Make（创建托管）

创建托管账户并存入 Token A，指定希望获得的 Token B 数量。

```rust
make(
    ctx.accounts maker,      # 创建者
    ctx.accounts escrow,     # 托管账户
    ctx.accounts vault,      # 金库账户
    ctx.accounts token_a,    # Token A Mint
    ctx.accounts maker_ata_a,# 创建者 Token A ATA
    seed,                    # 随机种子
    receive,                 # 期望的 Token B 数量
    amount                   # 存入的 Token A 数量
)
```

### Take（接受托管）

接受托管，向创建者发送 Token B，从金库中获得 Token A。

```rust
take(ctx.accounts)  # 包含 taker、maker、escrow、vault 等账户
```

### Refund（取消托管）

取消托管，取回存入的 Token A（仅创建者可调用）。

```rust
refund(ctx.accounts)  # 包含 maker、escrow、vault 等账户
```

## 🛡️ 安全机制

1. **PDA 权限控制**：金库由托管 PDA 拥有，只有程序能签名
2. **has_one 约束验证**：确保账户关系完整性
3. **原子性保证**：所有操作要么全部成功，要么全部失败
4. **重放保护**：Take/Refund 后账户关闭，防止重复使用
5. **租金豁免**：存入足够的 lamports 防止垃圾回收

## 📚 文档

- **[WORKFLOW.md](doc/WORKFLOW.md)** - 完整业务生命周期，包含状态转换和资金流向
- **[CLAUDE.md](CLAUDE.md)** - 项目架构和开发说明
- **[FUND_FLOW.md](doc/FUND_FLOW.md)** - 资金流向详细说明
- **[PPT_OUTLINE.md](doc/PPT_OUTLINE.md)** - 演示文稿大纲
- **[Blueshift 托管系统.pptx](doc/Blueshift%20托管系统.pptx)** - 演示文稿

## 🎯 Pinocchio vs Anchor 对比

| 特性 | Anchor | Pinocchio |
|-----|--------|-----------|
| 抽象级别 | 高级宏 | 手动实现 |
| 代码量 | 少 | 多 |
| 性能 | 良好 | 优秀 |
| 学习曲线 | 平缓 | 陡峭 |
| 灵活性 | 受限 | 完全控制 |
| 判别器大小 | 8 字节 | 1 字节 |

**建议**：快速原型开发使用 Anchor，生产环境优化使用 Pinocchio。

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

## 📝 开源协议

本项目采用 MIT 协议开源。

## 🙏 致谢

- [Pinocchio 框架](https://github.com/cap_prints/pinocchio) - 高性能 Solana 开发框架
- [Anchor 框架](https://www.anchor-lang.com/) - Solana 开发框架
- [Solana](https://solana.com/) - 高性能区块链

## 💬 技术支持

如有问题和讨论：
- 在 GitHub 上提 Issue
- 加入 Solana Discord 社区
- 查看详细的代码注释（中文）

---

**用 ❤️ 为 Solana 生态系统构建**