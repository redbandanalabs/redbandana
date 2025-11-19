#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::{signature::Signer, transaction::Transaction};

    #[tokio::test]
    async fn test_private_transfer() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "hand_cranked_privacy",
            program_id,
            processor!(Processor::process),
        );

        // Setup accounts, mint tokens, etc.
        // Generate off-chain proof
        let circuit = circuits::PrivateTransferCircuit { /* params */ };
        let proof = circuit.generate_proof();

        // Build tx with proof data
        let mut transaction = Transaction::new_with_payer(/* instructions */, Some(&payer.pubkey()));

        // Execute and assert
        let mut context = program_test.start_with_context().await;
        context.process_transaction(&mut transaction).await.unwrap();
    }
}
