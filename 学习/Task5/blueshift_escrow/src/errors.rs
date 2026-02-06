use pinocchio::error::ProgramError;
use core::fmt;

// 每个错误都有一个明确的数值（从 0 开始递增），这个数值会被编码到 ProgramError::Custom(error as u32) 中返回给客户端
// 客户端可以通过这个错误码来识别具体的错误类型，并进行相应的处理
// derive 属性说明：
// - Clone: 允许错误被复制
// - Debug: 允许使用 {:?} 格式化输出（用于调试）
// - Eq: 允许错误之间进行相等比较
// - PartialEq: 允许错误进行部分相等比较
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowError{
    
    NotRentExempt=0,
    
    NotSigner=1,

    InvalidOwner=2,

    InvalidAccountData=3,

    InvalidAddress = 4,
}


// 这个实现允许将 EscrowError 自动转换为 ProgramError
// 在程序中使用 '?' 操作符时，如果返回的是 EscrowError，
// 会自动调用这个 from 方法转换为 ProgramError
impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }

}

// 这个实现允许将 EscrowError 格式化为可读的字符串
impl fmt::Display for EscrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EscrowError::NotRentExempt => write!(f, "Not rent exempt"),
            EscrowError::NotSigner => write!(f, "Not signer"),
            EscrowError::InvalidOwner => write!(f, "Invalid owner"),
            EscrowError::InvalidAccountData => write!(f, "Invalid account data"),
            EscrowError::InvalidAddress => write!(f, "Invalid address"),
        }
    }
}