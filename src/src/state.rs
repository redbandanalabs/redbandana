use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Global state account (single instance, PDA).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GlobalState {
    pub is_initialized: bool,
    pub version: u8,
    pub authority: Pubkey,
}

/// Per-note account: a commitment + nullifier + spent flag.
///
/// This is intentionally generic: commitments are just 32-byte hashes
/// of whatever you decide off-chain (amount, owner, randomness, etc.).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct NoteState {
    pub is_initialized: bool,
    pub commitment: [u8; 32],
    pub nullifier: [u8; 32],
    pub spent: bool,
}

impl GlobalState {
    pub const VERSION: u8 = 1;
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            version: GlobalState::VERSION,
            authority: Pubkey::default(),
        }
    }
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            commitment: [0u8; 32],
            nullifier: [0u8; 32],
            spent: false,
        }
    }
}
