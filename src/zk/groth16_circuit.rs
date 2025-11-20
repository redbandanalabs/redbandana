#![cfg(feature = "zk-groth16")]

use ark_bn254::Bn254;
use ark_groth16::{verify_proof, PreparedVerifyingKey, Proof};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use ark_std::vec::Vec;
use solana_program::program_error::ProgramError;

use crate::error::HandCrankedError;

pub struct PrivateTransferCircuit<F: ark_ff::Field> {
    pub commitment: Option<F>,
    pub nullifier: Option<F>,
}

impl<F: ark_ff::Field> ConstraintSynthesizer<F> for PrivateTransferCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate public inputs
        let commitment_var =
            FpVar::<F>::new_input(cs.clone(), || self.commitment.ok_or(SynthesisError::AssignmentMissing))?;
        let nullifier_var =
            FpVar::<F>::new_input(cs.clone(), || self.nullifier.ok_or(SynthesisError::AssignmentMissing))?;

        commitment_var.enforce_equal(&nullifier_var)?;

        Ok(())
    }
}

static mut PREPARED_VK: Option<PreparedVerifyingKey<Bn254>> = None;

pub fn set_prepared_vk(pvk: PreparedVerifyingKey<Bn254>) {
    unsafe {
        PREPARED_VK = Some(pvk);
    }
}

pub fn verify_groth16(
    proof_bytes: &[u8],
    public_commitment: &[u8; 32],
    nullifier_bytes: &[u8; 32],
) -> Result<(), ProgramError> {
    use ark_ff::PrimeField;

    let proof: Proof<Bn254> = bincode::deserialize(proof_bytes)
        .map_err(|_| HandCrankedError::InvalidProof)?;

    let pvk = unsafe {
        PREPARED_VK
            .as_ref()
            .ok_or(HandCrankedError::InvalidProof)?
    };

    let c = <Bn254 as ark_ec::pairing::Pairing>::ScalarField::from_le_bytes_mod_order(public_commitment);
    let n = <Bn254 as ark_ec::pairing::Pairing>::ScalarField::from_le_bytes_mod_order(nullifier_bytes);

    let public_inputs = vec![c, n];

    verify_proof(pvk, &proof, &public_inputs)
        .map_err(|_| HandCrankedError::InvalidProof.into())
        .and_then(|ok| {
            if ok {
                Ok(())
            } else {
                Err(HandCrankedError::InvalidProof.into())
            }
        })
}
