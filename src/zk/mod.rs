use crate::instruction::ProofSystem;
use crate::error::HandCrankedError;
use solana_program::program_error::ProgramError;

/// Generic verifier trait – implemented by Groth16/STARK modules off-chain.
pub trait ProofVerifier {
    fn verify(
        system: ProofSystem,
        proof: &[u8],
        public_commitment: &[u8; 32],
        nullifier: &[u8; 32],
    ) -> Result<(), ProgramError>;
}

pub struct DefaultVerifier;

impl ProofVerifier for DefaultVerifier {
    fn verify(
        system: ProofSystem,
        proof: &[u8],
        public_commitment: &[u8; 32],
        nullifier: &[u8; 32],
    ) -> Result<(), ProgramError> {
        #[cfg(target_arch = "bpf")]
        {
   
            let _ = (system, proof, public_commitment, nullifier);
            return Err(HandCrankedError::UnsupportedProofSystem.into());
        }

        #[cfg(not(target_arch = "bpf"))]
        {
            match system {
                ProofSystem::Groth16 => {
                    #[cfg(feature = "zk-groth16")]
                    {
                        crate::zk::groth16_circuit::verify_groth16(proof, public_commitment, nullifier)
                    }
                    #[cfg(not(feature = "zk-groth16"))]
                    {
                        let _ = (proof, public_commitment, nullifier);
                        Err(HandCrankedError::UnsupportedProofSystem.into())
                    }
                }
                ProofSystem::Stark => {
                    #[cfg(feature = "zk-stark")]
                    {
                        crate::zk::stark::verify_stark(proof, public_commitment, nullifier)
                    }
                    #[cfg(not(feature = "zk-stark"))]
                    {
                        let _ = (proof, public_commitment, nullifier);
                        Err(HandCrankedError::UnsupportedProofSystem.into())
                    }
                }
            }
        }
    }
}

#[cfg(feature = "zk-groth16")]
pub mod groth16_circuit;

#[cfg(feature = "zk-stark")]
pub mod stark;
