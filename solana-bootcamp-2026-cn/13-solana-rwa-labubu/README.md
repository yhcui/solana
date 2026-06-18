# Solana RWA 演示 - Labubu

一个基于 Solana 的神秘盲盒 NFT 抽签系统，收录 11 款限量版 Labubu 角色，采用 Token-2022 标准。

## 功能特性

- **11 种独特 Labubu 类型**：10 种普通类型（每种 120 份）+ 1 种稀有类型（6 份）= 共 1,206 个 NFT
- **神秘盲盒机制**：根据剩余供应量进行加权随机铸造
- **Token-2022 NFT**：使用 Solana 的 token-2022 标准铸造代币
- **Anchor 程序**：链上藏品状态管理与铸造逻辑
- **Next.js 前端**：集成钱包的交互式盲盒 UI

## 前置条件

- Node.js 18+ 及 pnpm
- Rust 和 Anchor CLI（0.31.1+）
- 配置为 devnet 的 Solana CLI
- Devnet SOL（[水龙头](https://faucet.solana.com/)）

## 快速开始

```bash
# 1. 安装依赖
cd anchor && pnpm install
cd ../app && pnpm install

# 2. 构建并部署程序
cd anchor
anchor build
solana config set --url https://api.devnet.solana.com
anchor deploy --provider.cluster devnet

# 3. 生成 TypeScript 客户端
pnpm codegen

# 4. 初始化藏品
anchor run initialize

# 5. 启动前端
cd ../app && pnpm dev
```

访问 [http://localhost:3000](http://localhost:3000)

## 程序架构

三条主要指令：
1. `initialize_collection` - 创建带有供应计数器的藏品账户
2. `create_labubu_mint` - 为每种 Labubu 类型创建 Token-2022 铸币账户
3. `mint_random` - 向用户的关联代币账户随机铸造一个 NFT


## 相关资源

- [Solana 文档](https://solana.com/docs)
- [Anchor 框架](https://www.anchor-lang.com/)
- [Token-2022 指南](https://spl.solana.com/token-2022)
- [@solana/react-hooks](https://github.com/solana-foundation/framework-kit/tree/main/packages/react-hooks)

## 许可证

MIT
