use hand_cranked_privacy::{
    instruction::HandCrankedInstruction,
    processor::Processor,
};
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_initialize_and_deposit() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "hand_cranked_privacy",
        program_id,
        processor!(Processor::process),
    );

    let mut context = program_test.start_with_context().await;

    let payer = &context.payer;
    let global_state = Pubkey::find_program_address(&[b"global-state"], &program_id).0;
    let system_program = solana_sdk::system_program::id();

    // Initialize
    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(global_state, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program, false),
        ],
        data: HandCrankedInstruction::Initialize.try_to_vec().unwrap(),
    };

    let mut tx = Transaction::new_with_payer(&[init_ix], Some(&payer.pubkey()));
    tx.sign(&[payer], context.last_blockhash);
    context.banks_client.process_transaction(tx).await.unwrap();

    // Deposit
    let commitment = [1u8; 32];
    let note_pda = Pubkey::find_program_address(&[b"note", &commitment], &program_id).0;

    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(global_state, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(note_pda, false),
            AccountMeta::new_readonly(system_program, false),
        ],
        data: HandCrankedInstruction::Deposit { commitment }
            .try_to_vec()
            .unwrap(),
    };

    let mut tx2 = Transaction::new_with_payer(&[deposit_ix], Some(&payer.pubkey()));
    tx2.sign(&[payer], context.last_blockhash);
    context.banks_client.process_transaction(tx2).await.unwrap();
}
