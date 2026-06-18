import { randomBytes } from "node:crypto";
import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  type TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert } from "chai";
import type { Escrow } from "../target/types/escrow";
import { Program } from "@coral-xyz/anchor";
import { BN } from "bn.js";

import {
  confirmTransaction,
  createAccountsMintsAndTokenAccounts,
} from "@solana-developers/helpers";

/**
 * 兼容两种代币程序：
 * - TOKEN_PROGRAM_ID：传统的 SPL Token 程序
 * - TOKEN_2022_PROGRAM_ID：新的 Token Extensions 程序（支持更多功能）
 *
 * 本测试使用 TOKEN_2022_PROGRAM_ID，因为 Anchor 1.0 默认支持 Token-2022。
 * 程序的 Rust 代码使用 `TokenInterface`，所以能同时兼容两种程序。
 */
const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

const SECONDS = 1000;

// 测试超过这个时间会被标记为 "slow"
// Anchor 测试涉及网络 I/O，通常需要约 15 秒，所以设为 40 秒
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;

/**
 * describe 是 Mocha 测试框架的分组函数
 * "swap" 表示这是一组关于代币交换（托管）的测试
 */
describe("swap", () => {
  // ===== 初始化 Anchor 环境 =====
  // 从 Anchor.toml 读取配置（比如连接到本地验证器 localhost:8899）
  // 使用本地钱包作为交易费用支付者
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 获取钱包的 payer（用于创建代币、铸造等操作的费用支付者）
  // See https://github.com/coral-xyz/anchor/issues/3122
  const user = (provider.wallet as anchor.Wallet).payer;
  const payer = user;

  // Solana 的连接对象，用于查询链上状态
  const connection = provider.connection;

  // 加载编译后的链上程序，TypeScript 类型来自 `target/types/escrow`
  // 这个文件是 Anchor 编译 Rust 程序后自动生成的 IDL 类型
  const program = anchor.workspace.Escrow as Program<Escrow>;

  // ===== 测试用变量 =====
  let alice: anchor.web3.Keypair;  // Alice —— Maker（挂单人），提供代币 A 想要代币 B
  let bob: anchor.web3.Keypair;    // Bob —— Taker（接单人），提供代币 B 想要代币 A
  let tokenMintA: anchor.web3.Keypair;  // 代币 A 的 Mint 账户（铸造账户）
  let tokenMintB: anchor.web3.Keypair;  // 代币 B 的 Mint 账户
  let makerTokenAccountA: anchor.web3.PublicKey;   // Alice 的代币 A 账户（ATA）
  let makerTokenAccountB: anchor.web3.PublicKey;   // Alice 的代币 B 账户（ATA）
  let takerTokenAccountA: anchor.web3.PublicKey;   // Bob 的代币 A 账户（ATA）
  let takerTokenAccountB: anchor.web3.PublicKey;   // Bob 的代币 B 账户（ATA）
  let offer: anchor.web3.PublicKey;         // Offer PDA 账户地址（存储报价条款）
  let offerTokenAccount: anchor.web3.PublicKey;  // 托管账户地址（存放 Alice 的代币 A）

  // 报价的具体数量
  const tokenAOfferedAmount = new BN(1_000_000);  // Alice 提供 100 万枚代币 A
  const tokenBWantedAmount = new BN(1_000_000);   // Alice 想要 100 万枚代币 B

  /**
   * before 是所有测试开始前的准备工作，只执行一次
   * 这里负责创建测试账户、代币 Mint、以及代币账户并预分配余额
   */
  before(
    "Creates Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users",
    async () => {
      /**
       * createAccountsMintsAndTokenAccounts 是 @solana-developers/helpers 提供的工具函数，
       * 它会一次性完成以下操作：
       * 1. 创建用户密钥对（Alice、Bob）
       * 2. 创建两种代币的 Mint 账户（tokenMintA、tokenMintB）
       * 3. 为每个用户创建对应的代币账户（ATA）
       * 4. 向账户中铸造指定数量的代币
       *
       * 参数说明：
       * - 第一个数组：每个用户对应两种代币的初始余额
       *   Alice: [10 亿枚 A, 0 枚 B]
       *   Bob:   [0 枚 A, 10 亿枚 B]
       * - 第二个参数：给每个用户账户的 SOL 余额（用于支付交易费）
       * - connection: Solana 连接
       * - payer: 支付创建费用的账户
       */
      const usersMintsAndTokenAccounts =
        await createAccountsMintsAndTokenAccounts(
          [
            // Alice 的代币余额
            [
              1_000_000_000,  // 10 亿枚代币 A（Alice 有很多 A，可以用来交换）
              0,              // 0 枚代币 B（Alice 没有 B，所以她想要 B）
            ],
            // Bob 的代币余额
            [
              0,              // 0 枚代币 A（Bob 没有 A，所以他想要 A）
              1_000_000_000,  // 10 亿枚代币 B（Bob 有很多 B，可以用来交换）
            ],
          ],
          1_000_000_000,  // 每个用户获得 10 亿 lamports 的 SOL（约 1 SOL）
          connection,
          payer
        );

      // 提取用户密钥对
      const users = usersMintsAndTokenAccounts.users;
      alice = users[0];
      bob = users[1];

      // 提取代币 Mint 密钥对
      const mints = usersMintsAndTokenAccounts.mints;
      tokenMintA = mints[0];
      tokenMintB = mints[1];

      // 提取代币账户（ATA）
      const tokenAccounts = usersMintsAndTokenAccounts.tokenAccounts;

      const aliceTokenAccountA = tokenAccounts[0][0];  // Alice 的 A 代币账户
      const aliceTokenAccountB = tokenAccounts[0][1];  // Alice 的 B 代币账户

      const bobTokenAccountA = tokenAccounts[1][0];    // Bob 的 A 代币账户
      const bobTokenAccountB = tokenAccounts[1][1];    // Bob 的 B 代币账户

      // 保存账户地址供测试用例使用
      // Maker (= Alice) 的账户
      makerTokenAccountA = aliceTokenAccountA;
      makerTokenAccountB = aliceTokenAccountB;
      // Taker (= Bob) 的账户
      takerTokenAccountA = bobTokenAccountA;
      takerTokenAccountB = bobTokenAccountB;
    }
  );

  /**
   * 测试用例 1：测试 make_offer 指令
   *
   * 这个测试验证：
   * 1. Alice 能成功创建报价
   * 2. 代币 A 确实从 Alice 的账户转入了托管账户
   * 3. Offer PDA 账户中存储的数据正确
   */
  it("Puts the tokens Alice offers into the vault when Alice makes an offer", async () => {
    // 生成一个随机的 8 字节 offer ID
    // 使用随机 ID 确保每次测试的 PDA 地址都不同，避免冲突
    const offerId = new BN(randomBytes(8));

    // ===== 计算 PDA 地址 =====
    // 在客户端也需要用相同的种子计算 PDA 地址
    // findProgramAddressSync 返回 [地址, bump] 的元组，我们只需要地址（[0]）
    offer = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),                          // 种子1：固定前缀（与 Rust 代码中的 b"offer" 对应）
        alice.publicKey.toBuffer(),                    // 种子2：maker 的公钥
        offerId.toArrayLike(Buffer, "le", 8),          // 种子3：报价 ID（小端编码为 8 字节）
      ],
      program.programId                                // 程序的 Program ID
    )[0];

    // ===== 计算托管账户地址 =====
    // 托管账户是一个 ATA（关联代币账户），authority 是上面计算的 Offer PDA
    // 参数：mint 地址, authority 地址, 是否允许离线计算（true）, 代币程序
    offerTokenAccount = getAssociatedTokenAddressSync(
      tokenMintA.publicKey,  // 托管的是代币 A
      offer,                 // authority 是 Offer PDA
      true,                  // 允许离线计算（PDA 不是有效的 ed25519 公钥）
      TOKEN_PROGRAM          // 使用 Token-2022 程序
    );

    // ===== 调用链上程序的 make_offer 指令 =====
    // 这是测试的核心——向链上程序发送交易
    const transactionSignature = await program.methods
      .makeOffer(offerId, tokenAOfferedAmount, tokenBWantedAmount)  // 传入三个参数
      .accounts({
        // 指定指令所需的各个账户地址
        // Anchor 会根据 Rust 中的 MakeOffer 结构体自动校验账户
        maker: alice.publicKey,          // maker 是 Alice
        mintA: tokenMintA.publicKey,     // 代币 A 的 Mint
        mintB: tokenMintB.publicKey,     // 代币 B 的 Mint
        makerTokenAccountA,              // Alice 的代币 A 账户
        offer,                           // Offer PDA 账户
        offerTokenAccount,               // 托管账户
        tokenProgram: TOKEN_PROGRAM,     // 代币程序
      })
      .signers([alice])    // Alice 签名（因为 maker 是 Signer 类型）
      .rpc();              // 发送交易到链上

    // 等待交易确认（确保交易已被处理）
    await confirmTransaction(connection, transactionSignature);

    // ===== 验证结果 =====

    // 检查 1：托管账户中的代币 A 余额 = Alice 提供的数量
    const vaultBalanceResponse = await connection.getTokenAccountBalance(offerTokenAccount);
    const vaultBalance = new BN(vaultBalanceResponse.value.amount);
    assert(vaultBalance.eq(tokenAOfferedAmount));

    // 检查 2：Offer 账户中存储的数据正确
    const offerAccount = await program.account.offer.fetch(offer);

    assert(offerAccount.maker.equals(alice.publicKey));           // maker 是 Alice
    assert(offerAccount.mintA.equals(tokenMintA.publicKey));      // mint_a 正确
    assert(offerAccount.mintB.equals(tokenMintB.publicKey));      // mint_b 正确
    assert(offerAccount.tokenBWantedAmount.eq(tokenBWantedAmount)); // 想要的数量正确
  }).slow(ANCHOR_SLOW_TEST_THRESHOLD);  // 标记这个测试可能比较慢

  /**
   * 测试用例 2：测试 take_offer 指令
   *
   * 这个测试验证：
   * 1. Bob 能成功接受 Alice 的报价
   * 2. Bob 收到了 Alice 提供的代币 A（从托管账户取出）
   * 3. Alice 收到了 Bob 支付的代币 B
   * 4. 整个交换是原子性的——要么都成功，要么都失败
   *
   * 执行前的状态：
   *   Alice: 有很多 A，0 个 B
   *   Bob:   0 个 A，有很多 B
   *   托管:  存有 Alice 的 100 万 A
   *
   * 执行后的状态：
   *   Alice: 收到了 100 万 B
   *   Bob:   收到了 100 万 A
   *   托管:  空了（账户已关闭）
   */
  it("Puts the tokens from the vault into Bob's account, and gives Alice Bob's tokens, when Bob takes an offer", async () => {
    // ===== 调用链上程序的 take_offer 指令 =====
    const transactionSignature = await program.methods
      .takeOffer()  // 不需要参数，所有信息从 Offer 账户中读取
      .accounts({
        // 指定指令所需的各个账户地址
        taker: bob.publicKey,            // taker 是 Bob
        maker: alice.publicKey,          // maker 是 Alice（注意：Alice 不需要签名！）
        mintA: tokenMintA.publicKey,     // 代币 A 的 Mint
        mintB: tokenMintB.publicKey,     // 代币 B 的 Mint
        takerTokenAccountA,              // Bob 的代币 A 账户（接收 Alice 的 A）
        takerTokenAccountB,              // Bob 的代币 B 账户（支付 B 给 Alice）
        makerTokenAccountB,              // Alice 的代币 B 账户（接收 Bob 的 B）
        offer,                           // Offer PDA 账户（读取报价条款）
        offerTokenAccount,               // 托管账户（取出代币 A）
        tokenProgram: TOKEN_PROGRAM,     // 代币程序
      })
      .signers([bob])    // Bob 签名（只有 taker 需要签名）
      .rpc();            // 发送交易到链上

    // 等待交易确认
    await confirmTransaction(connection, transactionSignature);

    // ===== 验证结果 =====

    // 检查 1：Bob 收到了代币 A（从托管账户中取出的）
    // 注意：Bob 之前没有代币 A，所以不需要比较 before 余额
    const bobTokenAccountBalanceAfterResponse =
      await connection.getTokenAccountBalance(takerTokenAccountA);
    const bobTokenAccountBalanceAfter = new BN(
      bobTokenAccountBalanceAfterResponse.value.amount
    );
    assert(bobTokenAccountBalanceAfter.eq(tokenAOfferedAmount));

    // 检查 2：Alice 收到了代币 B（Bob 支付的）
    // 注意：Alice 之前没有代币 B，所以不需要比较 before 余额
    const aliceTokenAccountBalanceAfterResponse =
      await connection.getTokenAccountBalance(makerTokenAccountB);
    const aliceTokenAccountBalanceAfter = new BN(
      aliceTokenAccountBalanceAfterResponse.value.amount
    );
    assert(aliceTokenAccountBalanceAfter.eq(tokenBWantedAmount));
  }).slow(ANCHOR_SLOW_TEST_THRESHOLD);  // 标记这个测试可能比较慢
});
