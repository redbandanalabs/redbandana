use crate::{
    error::HandCrankedError,
    instruction::{HandCrankedInstruction, ProofSystem},
    state::{GlobalState, NoteState},
    utils::assert_rent_exempt,
    zk::{DefaultVerifier, ProofVerifier},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = HandCrankedInstruction::unpack(instruction_data)
            .map_err(|_| HandCrankedError::InvalidInstruction)?;

        match instruction {
            HandCrankedInstruction::Initialize => {
                msg!("Instruction: Initialize");
                Self::process_initialize(program_id, accounts)
            }
            HandCrankedInstruction::Deposit { commitment } => {
                msg!("Instruction: Deposit");
                Self::process_deposit(program_id, accounts, commitment)
            }
            HandCrankedInstruction::PrivateTransfer {
                proof_system,
                proof,
                public_inputs_commitment,
                nullifier,
                new_commitment_1,
                new_commitment_2,
            } => {
                msg!("Instruction: PrivateTransfer");
                Self::process_private_transfer(
                    program_id,
                    accounts,
                    proof_system,
                    &proof,
                    &public_inputs_commitment,
                    &nullifier,
                    &new_commitment_1,
                    &new_commitment_2,
                )
            }
            HandCrankedInstruction::Withdraw { nullifier } => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(program_id, accounts, &nullifier)
            }
        }
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let global_state_ai = next_account_info(account_info_iter)?;
        let authority_ai = next_account_info(account_info_iter)?;
        let system_program_ai = next_account_info(account_info_iter)?;

        if !authority_ai.is_signer {
            return Err(HandCrankedError::Unauthorized.into());
        }

        if global_state_ai.owner != program_id && global_state_ai.lamports() == 0 {

            let (expected_pda, bump) = Pubkey::find_program_address(&[b"global-state"], program_id);
            if expected_pda != *global_state_ai.key {
                return Err(HandCrankedError::InvalidAccountData.into());
            }

            let space = std::mem::size_of::<GlobalState>();
            let rent = solana_program::rent::Rent::get()?;
            let lamports = rent.minimum_balance(space);

            let create_ix = system_instruction::create_account(
                authority_ai.key,
                global_state_ai.key,
                lamports,
                space as u64,
                program_id,
            );

            invoke_signed(
                &create_ix,
                &[authority_ai.clone(), global_state_ai.clone(), system_program_ai.clone()],
                &[&[b"global-state", &[bump]]],
            )?;
        }

  
        let mut state: GlobalState = if global_state_ai.data_is_empty() {
            GlobalState::default()
        } else {
            GlobalState::try_from_slice(&global_state_ai.data.borrow())?
        };

        if state.is_initialized {
            return Err(HandCrankedError::AlreadyInitialized.into());
        }

        state.is_initialized = true;
        state.version = GlobalState::VERSION;
        state.authority = *authority_ai.key;

        assert_rent_exempt(global_state_ai, std::mem::size_of::<GlobalState>())?;

        state.serialize(&mut &mut global_state_ai.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_deposit(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        commitment: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let global_state_ai = next_account_info(account_info_iter)?;
        let user_ai = next_account_info(account_info_iter)?;
        let note_ai = next_account_info(account_info_iter)?;
        let system_program_ai = next_account_info(account_info_iter)?;

        if !user_ai.is_signer {
            return Err(HandCrankedError::Unauthorized.into());
        }

        if global_state_ai.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let _global_state: GlobalState = GlobalState::try_from_slice(&global_state_ai.data.borrow())
            .map_err(|_| HandCrankedError::InvalidAccountData)?;

        let (expected_pda, bump) = Pubkey::find_program_address(&[b"note", &commitment], program_id);
        if expected_pda != *note_ai.key {
            return Err(HandCrankedError::InvalidAccountData.into());
        }

        if note_ai.lamports() == 0 {
            let space = std::mem::size_of::<NoteState>();
            let rent = solana_program::rent::Rent::get()?;
            let lamports = rent.minimum_balance(space);

            let create_ix = system_instruction::create_account(
                user_ai.key,
                note_ai.key,
                lamports,
                space as u64,
                program_id,
            );

            invoke_signed(
                &create_ix,
                &[user_ai.clone(), note_ai.clone(), system_program_ai.clone()],
                &[&[b"note", &commitment, &[bump]]],
            )?;
        }

        let mut note_state: NoteState = if note_ai.data_is_empty() {
            NoteState::default()
        } else {
            NoteState::try_from_slice(&note_ai.data.borrow())
                .map_err(|_| HandCrankedError::InvalidAccountData)?
        };

        if note_state.is_initialized {
            return Err(HandCrankedError::DuplicateCommitment.into());
        }


        note_state.is_initialized = true;
        note_state.commitment = commitment;
        note_state.nullifier = [0u8; 32];
        note_state.spent = false;

        assert_rent_exempt(note_ai, std::mem::size_of::<NoteState>())?;
        note_state.serialize(&mut &mut note_ai.data.borrow_mut()[..])?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn process_private_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        proof_system: ProofSystem,
        proof: &[u8],
        public_inputs_commitment: &[u8; 32],
        nullifier: &[u8; 32],
        new_commitment_1: &[u8; 32],
        new_commitment_2: &[u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let global_state_ai = next_account_info(account_info_iter)?;
        let spent_note_ai = next_account_info(account_info_iter)?;
        let new_note1_ai = next_account_info(account_info_iter)?;
        let new_note2_ai = next_account_info(account_info_iter)?;
        let prover_ai = next_account_info(account_info_iter)?;
        let system_program_ai = next_account_info(account_info_iter)?;

        if !prover_ai.is_signer {
            return Err(HandCrankedError::Unauthorized.into());
        }

        if global_state_ai.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }


        let _global_state: GlobalState =
            GlobalState::try_from_slice(&global_state_ai.data.borrow())
                .map_err(|_| HandCrankedError::InvalidAccountData)?;

        let mut spent_note: NoteState = NoteState::try_from_slice(&spent_note_ai.data.borrow())
            .map_err(|_| HandCrankedError::InvalidAccountData)?;
        if !spent_note.is_initialized || spent_note.spent {
            return Err(HandCrankedError::NullifierAlreadySpent.into());
        }

        // Verify zk proof off-chain or in a dedicated verifier program.
        DefaultVerifier::verify(
            proof_system,
            proof,
            public_inputs_commitment,
            nullifier,
        )?;


        spent_note.spent = true;
        spent_note.nullifier = *nullifier;
        spent_note.serialize(&mut &mut spent_note_ai.data.borrow_mut()[..])?;


        Self::create_or_init_note(
            program_id,
            new_note1_ai,
            new_commitment_1,
            system_program_ai,
            prover_ai,
        )?;


        if new_commitment_2 != &[0u8; 32] {
            Self::create_or_init_note(
                program_id,
                new_note2_ai,
                new_commitment_2,
                system_program_ai,
                prover_ai,
            )?;
        }

        Ok(())
    }

    fn create_or_init_note(
        program_id: &Pubkey,
        note_ai: &AccountInfo,
        commitment: &[u8; 32],
        system_program_ai: &AccountInfo,
        payer_ai: &AccountInfo,
    ) -> ProgramResult {
        let (expected_pda, bump) =
            Pubkey::find_program_address(&[b"note", commitment], program_id);
        if expected_pda != *note_ai.key {
            return Err(HandCrankedError::InvalidAccountData.into());
        }

        if note_ai.lamports() == 0 {
            let space = std::mem::size_of::<NoteState>();
            let rent = solana_program::rent::Rent::get()?;
            let lamports = rent.minimum_balance(space);

            let create_ix = system_instruction::create_account(
                payer_ai.key,
                note_ai.key,
                lamports,
                space as u64,
                program_id,
            );

            invoke_signed(
                &create_ix,
                &[payer_ai.clone(), note_ai.clone(), system_program_ai.clone()],
                &[&[b"note", commitment, &[bump]]],
            )?;
        }

        let mut note_state: NoteState = if note_ai.data_is_empty() {
            NoteState::default()
        } else {
            NoteState::try_from_slice(&note_ai.data.borrow())
                .map_err(|_| HandCrankedError::InvalidAccountData)?
        };

        if note_state.is_initialized {
            return Err(HandCrankedError::DuplicateCommitment.into());
        }

        note_state.is_initialized = true;
        note_state.commitment = *commitment;
        note_state.nullifier = [0u8; 32];
        note_state.spent = false;

        assert_rent_exempt(note_ai, std::mem::size_of::<NoteState>())?;
        note_state.serialize(&mut &mut note_ai.data.borrow_mut()[..])?;
        Ok(())
    }

    fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        nullifier: &[u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let global_state_ai = next_account_info(account_info_iter)?;
        let note_ai = next_account_info(account_info_iter)?;
        let recipient_ai = next_account_info(account_info_iter)?;
        let system_program_ai = next_account_info(account_info_iter)?;

        if !recipient_ai.is_signer {
            return Err(HandCrankedError::Unauthorized.into());
        }

        if global_state_ai.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }


        let mut note_state: NoteState = NoteState::try_from_slice(&note_ai.data.borrow())
            .map_err(|_| HandCrankedError::InvalidAccountData)?;

        if note_state.spent || &note_state.nullifier != nullifier {
            return Err(HandCrankedError::NullifierAlreadySpent.into());
        }

        note_state.spent = true;
        note_state.serialize(&mut &mut note_ai.data.borrow_mut()[..])?;


        msg!("Withdraw executed (dummy; hook SPL transfer here)");

        let _ = system_program_ai;
        Ok(())
    }
}
