# 金库业务流程

1、 金库的创建和使用流程
// 1. 用户发起存款指令
User → Deposit Instruction → Owner Account + Vault PDA + Amount

// 2. 程序验证并创建/使用金库
- 验证 Owner 签名 ✅
- 验证 Vault 地址正确性（通过 PDA 计算）✅  
- 验证 Vault 由系统程序拥有 ✅
- 验证 Vault 为空（新创建）✅

// 3. 资金转移
Owner → Transfer → Vault (资金托管)

2. 取款流程

// 1. 用户发起取款指令
User → Withdraw Instruction → Owner Account + Vault PDA

// 2. 程序验证
- 验证 Owner 签名 ✅
- 验证 Vault 地址正确性 ✅
- 验证 Vault 属于该用户 ✅

// 3. 从金库转出资金
Vault → Transfer (signed by PDA) → Owner (取回资金)

3. 关键细节确认

存款人身份
    创建金库：是存款人/取款人（同一个用户）
    存入资金：存款人用自己的账户转账到 PDA 金库
    取出资金：同一存款人可以取回资金

4. 安全控制机制
|操作|	验证项	|作用|
|---|---|---|
|存款|	Owner 签名 + Vault 地址验证	|确保资金存入正确账户|
|取款|	Owner 签名 + Vault 验证 + PDA 签名	|防止他人取款    |


5. 完整的资金流向

User Wallet (Owner) → Deposit → Vault PDA (托管) → Withdraw → User Wallet (Owner)
     ↑                                      ↑
   签名验证                            签名验证