use borsh::{BorshDeserialize, BorshSerialize};

/// Proof system enum – what kind of proof is attached.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ProofSystem {
    Groth16 = 0,
    Stark = 1,
}

/// Program instructions.
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum HandCrankedInstruction {
    /// Initialize global state (PDA)
    ///
    /// Accounts:
    /// 0. [writable] Global state account (PDA)
    /// 1. [signer]   Payer / authority
    /// 2. []         System program
    Initialize,

    /// Deposit (shield) into a note commitment.
    ///
    /// Accounts:
    /// 0. [writable] Global state
    /// 1. [signer]   User
    /// 2. [writable] Note account (PDA)
    /// 3. []         System program
    ///
    /// Data:
    /// - commitment: [u8; 32]
    Deposit {
        commitment: [u8; 32],
    },

    /// Private transfer using zkSNARKs / STARKs.
    ///
    /// Accounts:
    /// 0. [writable] Global state
    /// 1. [writable] Spent note account
    /// 2. [writable] New note account 1
    /// 3. [writable] New note account 2 (optional; can be zeroed)
    /// 4. [signer]   Prover / user
    /// 5. []         System program
    ///
    /// Data:
    /// - proof_system: u8
    /// - proof: Vec<u8> (serialized)
    /// - public_inputs_commitment: [u8; 32] (hash root / commitment)
    /// - nullifier: [u8; 32]
    PrivateTransfer {
        proof_system: ProofSystem,
        proof: Vec<u8>,
        public_inputs_commitment: [u8; 32],
        nullifier: [u8; 32],
        new_commitment_1: [u8; 32],
        new_commitment_2: [u8; 32],
    },

    /// Withdraw (unshield) a note.
    ///
    /// Accounts:
    /// 0. [writable] Global state
    /// 1. [writable] Note account
    /// 2. [signer]   Recipient
    /// 3. []         System program
    ///
    /// Data:
    /// - nullifier: [u8; 32]
    Withdraw {
        nullifier: [u8; 32],
    },
}

impl HandCrankedInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        Self::try_from_slice(input)
    }
}
