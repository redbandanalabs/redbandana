use solana_program::{program_error::ProgramError, msg};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum HandCrankedError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Invalid account data")]
    InvalidAccountData,

    #[error("Account is not rent exempt")]
    NotRentExempt,

    #[error("State already initialized")]
    AlreadyInitialized,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Note commitment already exists")]
    DuplicateCommitment,

    #[error("Nullifier already spent")]
    NullifierAlreadySpent,

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Unsupported proof system")]
    UnsupportedProofSystem,
}

impl From<HandCrankedError> for ProgramError {
    fn from(e: HandCrankedError) -> Self {
        msg!(&e.to_string());
        ProgramError::Custom(e as u32)
    }
}
