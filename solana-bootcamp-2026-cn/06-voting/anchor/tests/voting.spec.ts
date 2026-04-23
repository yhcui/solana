/**
 * Solana 链上投票程序 - 集成测试
 *
 * 测试流程：
 * 1. 创建一个投票（poll_id=1，投票窗口：0 ~ 2030-01-01）
 * 2. 添加两个候选者（Alice 和 Bob）
 * 3. 为 Alice 投票并验证票数
 *
 * 测试使用 Jest 运行，连接本地 Solana 验证器（由 Anchor 自动启动）。
 */
import * as anchor from "@anchor-lang/core";
import { BN, Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import { Voting } from "../target/types/voting";
// 为什么anchor还生成ts，这是rust？ -- 见说明
//  target/types/ 这个目录路径是 Anchor 框架的默认约定（Convention），并不是 TypeScript 语言本身的强制要求，而是 Anchor 构建工具链硬编码的行为。
//  Anchor 项目构建后的 target 
// target/
// ├── deploy/          # 存放编译好的 .so 程序文件
// ├── idl/             # 存放 .json 格式的 IDL 文件
// ├── types/           # 存放 .ts 格式的类型定义文件 (即 voting.ts)
// └── ..
// Voting 既不是普通的 JavaScript 对象，也不是传统的 TypeScript 类。
// 它是一个 TypeScript 接口（Interface） 或 类型别名（Type Alias），用于描述 Solana 智能合约的结构。

// 程序 ID，与 lib.rs 中的 declare_id! 和 Anchor.toml 中的配置保持一致
// 这个 PROGRAM_ID 不是完全随机自动生成的，而是由开发者生成并固定下来的。它必须保证在 Solana 网络上的全局唯一性。
// 它是如何生成的？
// 初始生成：当你使用 Anchor 框架初始化项目（anchor init）或生成新程序时，Anchor 会调用底层的 Solana 工具（如 solana-keygen）生成一个新的密钥对（Keypair）。
// 公钥即 ID：这个密钥对的公钥（Public Key）就被用作程序的 ID。
// 配置文件：这个 ID 会被写入两个关键位置：
// 1、Anchor.toml 文件中。
// 2、Rust 代码中的 declare_id!("...") 宏里。
// 为什么不能重复？
// 1、链上地址唯一性：在 Solana 区块链上，每个程序（Program）都部署在一个特定的账户地址上。这个地址就是它的 ID。如果两个不同的程序使用相同的 ID，会导致严重的冲突：
//    部署时会覆盖之前的程序（如果拥有该密钥对的私钥）。
//    客户端（如你的测试脚本）无法区分到底要调用哪个逻辑。
// 2、安全性：程序 ID 对应着一个私钥。只有拥有该私钥的人才能升级或重新部署该程序。如果 ID 重复或泄露，可能导致程序被恶意篡改。
const PROGRAM_ID = new PublicKey("65KHV8cXwJ8apTKMqnpSdhdHkHhRySatgKMwnxm6C3gG");

// 为什么在 TS 中可以直接使用？
// 全局声明: 当你安装了测试库（如 jest）及其类型定义（@types/jest）后，这些库会在全球作用域中声明 describe、it、expect 等函数。
// 类型支持: @types/jest 提供了这些函数的 TypeScript 类型定义，因此你在编写代码时可以获得智能提示和类型检查，即使它们不是 TS 语言的一部分。
// describe("voting", ...): 定义了一个名为 "voting" 的测试组。这有助于在测试报告中组织输出，并在逻辑上分组相关的测试。
// it(...): 定义具体的单个测试用例。

describe("voting", () => {
  // 配置 Anchor Provider，使用环境变量中的钱包和集群地址
  // 默认连接 http://127.0.0.1:8899（本地验证器）
  anchor.setProvider(anchor.AnchorProvider.env());

  // 1. anchor.workspace：工作空间注册表
// 含义：anchor.workspace 是一个对象，它包含了当前 Anchor 项目中所有已编译的智能程序（Programs）。
// 来源：当你运行 anchor build 时，Anchor 会读取 Anchor.toml 配置文件，找到其中定义的所有程序（例如 [programs.localnet] voting = "..."），并将它们加载到这个 workspace 中。
// 类比：想象 workspace 是一个“工具箱”，里面放着你项目里所有的工具（智能合约）。
// 2. .Voting：具体的程序名称
// 含义：这是你在 Anchor.toml 中定义的程序名称，通常对应于你的 Rust crate 名称或文件夹名称。
// 动态属性：anchor.workspace.Voting 是一个动态生成的对象。在运行时，它包含了指向该程序 ID 的引用以及基本的交互方法。
// 注意：如果你的程序叫 my_token，这里就是 anchor.workspace.MyToken。
// 3. as Program<Voting>：类型断言（Type Assertion）
// 这是 TypeScript 特有的语法，也是让开发体验变好的关键。
// Program：这是 Anchor 提供的一个通用类，代表一个可交互的 Solana 程序。它提供了 methods（调用指令）、account（读取账户数据）等通用 API。
// <Voting>：这是一个泛型参数。这里的 Voting 就是你之前问的那个从 target/types/voting.ts 导入的类型定义。
// as：因为 anchor.workspace.Voting 在底层可能只是一个普通的 JavaScript 对象，TypeScript 编译器最初不知道它具体有哪些方法。通过 as Program<Voting>，你告诉编译器：“相信我，这个对象符合 Program<Voting> 的结构”。
// 为什么要这么写？（好处）
// 一旦你完成了这个赋值，变量 program 就拥有了完整的类型智能提示：
// 方法补全： 当你输入 program.methods. 时，IDE 会自动列出 Rust 代码中定义的所有指令，如 .initializePoll(), .vote(), .initializeCandidate()。
// 参数检查： 如果你调用 program.methods.vote(...)，TS 会根据 Voting 类型定义，强制要求你传入正确数量和类型的参数（例如 pollId 必须是 BN 类型，candidateName 必须是 string）。如果传错，编辑器会直接报错。
// 账户类型安全： 当你使用 program.account.pollAccount.fetch() 时，返回的数据结构也会被自动推断为 PollAccount 类型，你可以直接访问 .pollName 等字段而不需要手动解析。
  // 获取类型化的 Voting 程序实例
  const program = anchor.workspace.Voting as Program<Voting>;



  // 测试用投票 ID，BN（BigNumber）因为 Solana 的 u64 超出 JavaScript 安全整数范围
  const POLL_ID = new BN(1);

  /**
   * 派生投票账户的 PDA 地址
   *
   * PDA 派生规则必须与链上程序中的 seeds 定义完全一致：
   * seeds = ["poll", poll_id的小端序8字节]
   *
   * findProgramAddressSync 是同步版本，返回 [地址, bump] 元组
   * 在 Solana 中，PDA 不是通过私钥生成的，而是通过“种子” deterministic（确定性）地计算出来的。
   */
  // 根据特定的规则（Seeds）和程序 ID，计算出对应的 PDA（程序派生地址）。
  const [pollAddress] = PublicKey.findProgramAddressSync(
    [Buffer.from("poll"), POLL_ID.toArrayLike(Buffer, "le", 8)],
    PROGRAM_ID
  );

  /**
   * 测试用例 1：初始化投票
   *
   * 验证：
   * - 投票账户被正确创建
   * - 名称、描述、时间窗口与传入参数一致
   * - pollOptionIndex 初始值为 0（尚未添加候选者）
   */
  it("initializes a poll", async () => {
    // 调用链上程序的 initialize_poll 指令
    // .rpc() 表示将交易发送到链上执行
    await program.methods
      .initializePoll(
        POLL_ID,
        new BN(0),           // 投票开始时间：Unix 时间戳 0（1970-01-01）
        new BN(1893456000),  // 投票结束时间：2030-01-01 00:00:00 UTC
        "Test Poll",
        "A poll to test the voting program"
      )
      .rpc();

    // 从链上获取投票账户数据并验证
    const pollAccount = await program.account.pollAccount.fetch(pollAddress);
    console.log("Poll account:", pollAccount);

    expect(pollAccount.pollName).toEqual("Test Poll");
    expect(pollAccount.pollDescription).toEqual("A poll to test the voting program");
    expect(pollAccount.pollVotingStart.toNumber()).toEqual(0);
    expect(pollAccount.pollVotingEnd.toNumber()).toEqual(1893456000);
    expect(pollAccount.pollOptionIndex.toNumber()).toEqual(0);
  });

  /**
   * 测试用例 2：初始化候选者
   *
   * 验证：
   * - Alice 和 Bob 的候选者账户被正确创建
   * - pollOptionIndex 从 0 增加到 2（每添加一个候选者 +1）
   */
  it("initializes candidates", async () => {
    // 添加候选者 Alice
    await program.methods
      .initializeCandidate(POLL_ID, "Alice")
      .rpc();

    // 添加候选者 Bob
    await program.methods
      .initializeCandidate(POLL_ID, "Bob")
      .rpc();

    // 验证投票账户中的候选者计数器
    const pollAccount = await program.account.pollAccount.fetch(pollAddress);
    expect(pollAccount.pollOptionIndex.toNumber()).toEqual(2);
  });

  /**
   * 测试用例 3：投票
   *
   * 验证：
   * - 可以成功为候选者投票
   * - 候选者得票数从 0 增加到 1
   *
   * 注意：由于投票窗口是 [0, 2030-01-01)，而当前时间在此范围内，
   * 所以投票应该成功。
   */
  it("casts a vote", async () => {
    // 派生 Alice 候选者账户的 PDA 地址
    // 派生规则：[poll_id的小端序8字节, "Alice"]
    const [aliceAddress] = PublicKey.findProgramAddressSync(
      [POLL_ID.toArrayLike(Buffer, "le", 8), Buffer.from("Alice")],
      PROGRAM_ID
    );

    // 为 Alice 投票
    await program.methods
      .vote(POLL_ID, "Alice")
      .rpc();

    // 获取 Alice 的候选者账户并验证得票数
    const aliceAccount = await program.account.candidateAccount.fetch(aliceAddress);
    console.log("Alice account:", aliceAccount);
    expect(aliceAccount.candidateVotes.toNumber()).toEqual(1);
  });
});
