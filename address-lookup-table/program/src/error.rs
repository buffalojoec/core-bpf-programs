//! Program error types

use {
    solana_program::{instruction::InstructionError, program_error::ProgramError},
    spl_program_error::*,
};

/// Errors that may be returned by the program.
#[spl_program_error]
pub enum AddressLookupError {
    /// Attempted to lookup addresses from a table that does not exist
    #[error("Attempted to lookup addresses from a table that does not exist")]
    LookupTableAccountNotFound,
    /// Attempted to lookup addresses from an account owned by the wrong program
    #[error("Attempted to lookup addresses from an account owned by the wrong program")]
    InvalidAccountOwner,
    /// Attempted to lookup addresses from an invalid account
    #[error("Attempted to lookup addresses from an invalid account")]
    InvalidAccountData,
    /// Address lookup contains an invalid index
    #[error("Address lookup contains an invalid index")]
    InvalidLookupIndex,
    /// Address lookup is immutable
    #[error("Address lookup is immutable")]
    LookupTableImmutable,
    /// Incorrect address lookup authority provided
    #[error("Incorrect address lookup authority provided")]
    IncorrectAuthority,
    // Failed to serialize address lookup table
    #[error("Failed to serialize address lookup table")]
    FailedToSerialize,
    // Failed to deserialize address lookup table
    #[error("Failed to deserialize address lookup table")]
    FailedToDeserialize,
}

/// A trait for converting from an instruction error to a program error
/// for preserving ABI compatibility.
pub(crate) trait ToProgramError<T> {
    /// Convert from an instruction error to a program error.
    fn map_to_program_error(self) -> Result<T, ProgramError>;
}

impl<T> ToProgramError<T> for Result<T, InstructionError> {
    fn map_to_program_error(self) -> Result<T, ProgramError> {
        self.map_err(|err| match err {
            InstructionError::GenericError => AddressLookupError::FailedToSerialize.into(),
            InstructionError::InvalidAccountData => ProgramError::InvalidAccountData,
            InstructionError::InvalidInstructionData => ProgramError::InvalidInstructionData,
            InstructionError::InvalidAccountOwner => ProgramError::InvalidAccountOwner,
            _ => ProgramError::InvalidArgument,
        })
    }
}
