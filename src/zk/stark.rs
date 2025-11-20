#![cfg(feature = "zk-stark")]

use solana_program::program_error::ProgramError;
use crate::error::HandCrankedError;

pub fn verify_stark(
    proof_bytes: &[u8],
    public_commitment: &[u8; 32],
    nullifier_bytes: &[u8; 32],
) -> Result<(), ProgramError> {
    let _ = (proof_bytes, public_commitment, nullifier_bytes);

    // TODO: Implement real STARK verification with winterfell.
    Err(HandCrankedError::UnsupportedProofSystem.into())
}
