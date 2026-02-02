use pinocchio::error::ProgramError;
use core::fmt;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowError{
    
    NotRentExempt=0,
    
    NotSigner=1,

    InvalidOwner=2,

    InvalidAccountData=3,

    InvalidAddress = 4,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }

}

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