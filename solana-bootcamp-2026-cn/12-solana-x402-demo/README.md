# 基于 X402 支付协议的 AI 算命应用

一个 AI 驱动的算命应用，演示如何使用 X402 协议在 Solana 上进行加密货币支付。用户支付加密货币后即可解锁 AI 生成的命运解读和付费内容。

## 应用功能

- **AI 算命** - 由 OpenAI 驱动的个性化命运解读（$0.05）
- **分级内容** - 多个价格档次对应不同内容
- **直接加密支付** - 通过 Coinbase Pay 或加密钱包以 Solana 支付
- **无需注册** - 支付后立即访问内容

## 快速开始

```bash
pnpm install
cp .env.example .env.local
# 编辑 .env.local：填入你的 OpenAI API 密钥和 CDP 客户端密钥
pnpm dev
```

---

# X402 Next.js Solana 模板

**一个集成了 X402 支付协议的简洁 Next.js 入门模板，专为 Solana 设计。**

本模板演示了如何使用 `x402-next` 包实现 X402 支付协议，轻松为你的 Next.js 应用添加加密货币支付门控。

> ⚠️ **在主网使用？** 本模板默认配置为测试网（devnet）。若要在主网接收真实支付，需要设置 CDP API 密钥并配置手续费支付方。完整设置说明请参阅 [CDP X402 主网文档](https://docs.cdp.coinbase.com/x402/quickstart-for-sellers#running-on-mainnet)。

## 目录

- [什么是 X402？](#什么是-x402)
- [功能特性](#功能特性)
- [快速开始](#快速开始-1)
- [工作原理](#工作原理)
- [项目结构](#项目结构)
- [配置说明](#配置说明)
- [使用方法](#使用方法)

---

## 什么是 X402？

**X402** 是一个开放支付协议，利用 HTTP 状态码 **402 "Payment Required"** 为网页内容和 API 实现无缝加密货币支付。

### 核心优势

- **直接支付** - 无需第三方支付处理商，直接接收加密货币
- **无需账户** - 无需用户注册或身份验证
- **区块链验证** - 支付直接在 Solana 区块链上验证
- **简单集成** - 通过中间件为任意 Next.js 路由添加支付门控
- **灵活定价** - 为不同内容设置不同价格

### 工作流程

```
1. 用户请求受保护内容
2. 服务器返回 402 Payment Required
3. 用户通过 Coinbase Pay 或加密钱包完成支付
4. 用户提供交易签名作为支付凭证
5. 服务器在区块链上验证并授权访问
```

---

## 功能特性

- **X402 支付中间件** - 由 `x402-next` 包驱动
- **Solana 集成** - 使用 Solana 区块链进行支付验证
- **多价格档次** - 为不同路由配置不同价格
- **会话管理** - 支付后自动处理会话
- **类型安全** - 完整的 TypeScript 支持（Viem 类型）
- **Next.js 16** - 基于最新的 Next.js App Router 构建

---

## 快速开始

### 前置条件

- Node.js 18+ 或 Bun
- pnpm、npm 或 yarn
- 一个用于接收支付的 Solana 钱包地址

### 安装步骤

```bash
# 从模板创建项目
npx create-solana-dapp my-app --template x402-template

# 进入项目目录
cd my-app

# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev
```

访问 `http://localhost:3000` 查看运行中的应用。

### 测试支付流程

1. 访问 `http://localhost:3000`
2. 点击"访问廉价内容"或"访问昂贵内容"
3. 弹出 Coinbase Pay 支付对话框
4. 完成支付
5. 授权成功，即可查看受保护内容

---

## 工作原理

本模板使用 `x402-next` 包提供的中间件来处理完整的支付流程。

### 中间件配置

支付集成的核心在 `middleware.ts` 中：

```typescript
import { Address } from 'viem'
import { paymentMiddleware, Resource, Network } from 'x402-next'
import { NextRequest } from 'next/server'

// 接收支付的 Solana 钱包地址
const address = 'CmGgLQL36Y9ubtTsy2zmE46TAxwCBm66onZmPPhUWNqv' as Address
const network = 'solana-devnet' as Network
const facilitatorUrl = 'https://x402.org/facilitator' as Resource
const cdpClientKey = '3uyu43EHCwgVIQx6a8cIfSkxp6cXgU30'

const x402PaymentMiddleware = paymentMiddleware(
  address,
  {
    '/content/cheap': {
      price: '$0.01',
      config: {
        description: '访问廉价内容',
      },
      network,
    },
    '/content/expensive': {
      price: '$0.25',
      config: {
        description: '访问昂贵内容',
      },
      network,
    },
  },
  {
    url: facilitatorUrl,
  },
  {
    cdpClientKey,
    appLogo: '/logos/x402-examples.png',
    appName: 'x402 Demo',
    sessionTokenEndpoint: '/api/x402/session-token',
  },
)

export const middleware = (req: NextRequest) => {
  const delegate = x402PaymentMiddleware as unknown as (
    request: NextRequest,
  ) => ReturnType<typeof x402PaymentMiddleware>
  return delegate(req)
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico).*)', '/'],
}
```

### 底层实现逻辑

1. **请求拦截** - 中间件检查请求路由是否需要支付
2. **支付检查** - 若路由受保护，检查是否存在有效支付会话
3. **402 响应** - 若无有效支付，返回 402 及支付要求
4. **Coinbase Pay 弹窗** - 用户看到由 Coinbase 驱动的支付弹窗
5. **支付验证** - 支付完成后，通过协调者在 Solana 区块链上验证交易
6. **创建会话** - 有效支付后创建会话令牌
7. **授权访问** - 用户可访问受保护内容

---

## 项目结构

```
x402-template/
├── middleware.ts              # 🛡️  X402 支付中间件配置
├── app/
│   ├── page.tsx              # 🏠 包含受保护内容链接的首页
│   ├── layout.tsx            # 📐 根布局
│   ├── globals.css           # 🎨 全局样式
│   └── content/
│       └── [type]/
│           └── page.tsx      # 🔒 受保护内容页面
├── components/
│   └── cats-component.tsx    # 🐱 示例内容组件
├── lib/                      # 📚 工具函数（按需）
├── public/                   # 📁 静态资源
└── package.json              # 📦 依赖配置
```

---

## 配置说明

### 环境变量

模板提供了合理的默认值，你也可以创建 `.env.local` 文件进行自定义：

```bash
# 你的 Solana 钱包地址（接收支付）
NEXT_PUBLIC_WALLET_ADDRESS=your_solana_address_here

# 网络（solana-devnet 或 solana-mainnet-beta）
NEXT_PUBLIC_NETWORK=solana-devnet

# Coinbase Pay 客户端密钥（从 Coinbase 开发者门户获取）
NEXT_PUBLIC_CDP_CLIENT_KEY=your_client_key_here

# 协调者 URL（验证支付的服务）
NEXT_PUBLIC_FACILITATOR_URL=https://x402.org/facilitator
```

### 自定义路由和价格

编辑 `middleware.ts` 以添加或修改受保护路由：

```typescript
const x402PaymentMiddleware = paymentMiddleware(
  address,
  {
    '/premium': {
      price: '$1.00',
      config: {
        description: '付费内容访问',
      },
      network: 'solana-mainnet-beta',
    },
    '/api/data': {
      price: '$0.05',
      config: {
        description: 'API 数据访问',
      },
      network: 'solana-mainnet-beta',
    },
  },
  // ... 其余配置
)
```

### 网络选择

可使用以下网络：

- `solana-devnet` - 用于测试（使用测试代币）
- `solana-mainnet-beta` - 用于生产环境（真实资金！）
- `solana-testnet` - 备用测试网络

---

## 使用方法

### 创建受保护内容

只需在中间件定义的受保护路由下创建页面即可：

```tsx
// app/content/premium/page.tsx
export default async function PremiumPage() {
  return (
    <div>
      <h1>付费内容</h1>
      <p>此内容需要支付后才能访问。</p>
      {/* 在此处添加你的受保护内容 */}
    </div>
  )
}
```

### 添加新的价格档次

1. 在 `middleware.ts` 中添加路��配置
2. 创建对应的页面组件
3. 用户访问该路由时将自动收到支付提示

### 在开发网测试

使用 `solana-devnet` 时：

- 支付使用测试代币（无真实资金）
- 适合开发和测试
- 从 [Solana 水龙头](https://faucet.solana.com/) 获取测试代币

### 上线生产环境

接受真实支付前：

1. 在 `middleware.ts` 中将网络改为 `solana-mainnet-beta`
2. 将钱包地址更新为你的生产钱包
3. 部署前请充分测试！
4. 考虑实施额外的安全措施

---

## 依赖项

本模板使用最少的依赖：

```json
{
  "dependencies": {
    "next": "16.0.0",
    "react": "19.2.0",
    "react-dom": "19.2.0",
    "viem": "^2.38.5",
    "x402-next": "^0.7.1"
  }
}
```

- **next** - Next.js 框架
- **react** / **react-dom** - React 库
- **viem** - 类型安全的 Ethereum/Solana 类型
- **x402-next** - X402 支付中间件（处理所有支付逻辑）

---

## 了解更多

### X402 协议

- [X402 规范](https://github.com/coinbase/x402) - 官方协议文档
- [X402 Next 包](https://www.npmjs.com/package/x402-next) - 本模板使用的中间件

### Solana

- [Solana 文档](https://docs.solana.com/) - Solana 官方文档
- [Solana 浏览器](https://explorer.solana.com/) - 链上交易查询

### Coinbase 开发者

- [CDP 文档](https://docs.cdp.coinbase.com/) - Coinbase 开发者文档

---

## 常见问题

### 支付无法正常工作

1. 检查 `middleware.ts` 中的钱包地址是否正确
2. 确认使用的网络正确（devnet vs mainnet）
3. 查看浏览器控制台报错
4. 确保 Coinbase Pay 客户端密钥有效

### 402 错误未显示

1. 检查 `middleware.ts` 中的 matcher 配置
2. 确认路由路径与页面结构匹配
3. 清除 Next.js 缓存：`rm -rf .next && pnpm dev`

### 会话未保持

1. 确认浏览器已启用 Cookie
2. 验证会话令牌端点已配置
3. 检查使用自定义域名时是否存在 CORS 问题

---

## 支持

模板相关问题请在仓库中提交 Issue。

X402 协议相关问题请参阅[官方文档](https://github.com/coinbase/x402)。

---

## 许可证

MIT 许可证 - 欢迎在你的项目中使用本模板。

---

## 贡献

欢迎贡献！请随时提交 Pull Request。

---

**由 [Kronos](https://www.kronos.build/) 用 ❤️ 打造**
