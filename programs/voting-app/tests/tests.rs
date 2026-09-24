use std::result;

use anchor_lang::solana_program::pubkey::Pubkey;
use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID,
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = voting_app::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!("../../../target/deploy/voting_app.so");
    svm.add_program(program_id, program_bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    (svm, payer)
}

#[test]
fn test_init_poll() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;

    let user = payer.pubkey();
    let (poll_pda, _) = Pubkey::find_program_address(
        &[voting_app::constants::POLL_SEED, &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let init_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializePoll {
            signer: user,
            poll_account: poll_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializePoll {
            poll_description: String::from("Test Poll init"),
            poll_id: 1,
            poll_name: String::from("POLL TEST"),
            poll_start: 0,
            poll_end: i64::MAX as u64,
        }
        .data(),
    };

    let message = Message::new(&[init_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // 2. Transaction should succeed
    let tx = svm
        .send_transaction(transaction)
        .expect("Initialize poll transaction failed");
    println!("Transaction succeeded: {}", tx.signature);
    // 3. PDA should now exist
    let account = svm
        .get_account(&poll_pda)
        .expect("Poll PDA was not created");
    println!("Poll PDA account: {account:#?}");
    // 4. Deserialize Anchor account data
    let mut data: &[u8] = &account.data;
    let poll = voting_app::state::PollAccaount::try_deserialize(&mut data)
        .expect("Failed to deserialize PollAccaount");
    // 5. Verify the stored values
    assert_eq!(poll.poll_name, "POLL TEST");
    assert_eq!(poll.poll_description, "Test Poll init");
    assert_eq!(poll.poll_voting_start, 0);
    assert_eq!(poll.poll_voting_end, i64::MAX as u64);
}

#[test]
fn test_init_candidate() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let candidate_name = "Noha".to_string();

    let user = payer.pubkey();
    let (poll_pda, _) = Pubkey::find_program_address(
        &[voting_app::constants::POLL_SEED, &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let (candidate_pda, _) = Pubkey::find_program_address(
        &[candidate_name.as_bytes(), &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let init_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializePoll {
            signer: user,
            poll_account: poll_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializePoll {
            poll_description: String::from("Test Poll init"),
            poll_id: 1,
            poll_name: String::from("POLL TEST"),
            poll_start: 0,
            poll_end: i64::MAX as u64,
        }
        .data(),
    };

    let message = Message::new(&[init_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    svm.send_transaction(transaction)
        .expect("Initialize poll transaction failed");

    let init_candidate_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializeCandidate {
            signer: user,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializeCandidate {
            candidate_name: candidate_name,
            poll_id: poll_id,
        }
        .data(),
    };

    let message = Message::new(&[init_candidate_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // 2. Transaction should succeed
    let tx = svm
        .send_transaction(transaction)
        .expect("Initialize candidate transaction failed");
    println!("Transaction succeeded: {}", tx.signature);

    let account = svm
        .get_account(&poll_pda)
        .expect("Poll PDA was not created");
    println!("Poll PDA account: {account:#?}");
    // 4. Deserialize Anchor account data
    let mut data: &[u8] = &account.data;

    let poll = voting_app::state::PollAccaount::try_deserialize(&mut data)
        .expect("Failed to deserialize PollAccaount");
    // 5. Verify the stored values
    assert_eq!(poll.poll_name, "POLL TEST");
    assert_eq!(poll.poll_description, "Test Poll init");
    assert_eq!(poll.poll_voting_start, 0);
    assert_eq!(poll.poll_voting_end, i64::MAX as u64);

    let account = svm
        .get_account(&candidate_pda)
        .expect("Candidate PDA was not created");
    println!("Candidate PDA account: {account:#?}");

    let mut data: &[u8] = &account.data;

    let candidate = voting_app::state::CandidateAccount::try_deserialize(&mut data)
        .expect("Failed to deserialize PollAccaount");

    assert_eq!(candidate.candidate_name, "Noha");
    assert_eq!(poll.poll_candidates_count, 1);
}

#[test]
fn test_vote() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let candidate_name = "Noha".to_string();

    let user = payer.pubkey();
    let (poll_pda, _) = Pubkey::find_program_address(
        &[voting_app::constants::POLL_SEED, &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let (candidate_pda, _) = Pubkey::find_program_address(
        &[candidate_name.as_bytes(), &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let init_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializePoll {
            signer: user,
            poll_account: poll_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializePoll {
            poll_description: String::from("Test Poll init"),
            poll_id: 1,
            poll_name: String::from("POLL TEST"),
            poll_start: 0,
            poll_end: i64::MAX as u64,
        }
        .data(),
    };

    let message = Message::new(&[init_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    svm.send_transaction(transaction)
        .expect("Initialize poll transaction failed");

    let init_candidate_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializeCandidate {
            signer: user,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializeCandidate {
            candidate_name: candidate_name.clone(),
            poll_id: poll_id,
        }
        .data(),
    };

    let message = Message::new(&[init_candidate_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    svm.send_transaction(transaction)
        .expect("Initialize candidate transaction failed");

    let vote_ix = Instruction {
        program_id: voting_app::id(),

        accounts: voting_app::accounts::Vote {
            signer: user,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
        }
        .to_account_metas(None),
        data: voting_app::instruction::Vote {
            candidate_name: candidate_name,
            poll_id: poll_id,
        }
        .data(),
    };

    let message = Message::new(&[vote_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    svm.send_transaction(transaction)
        .expect("vote transaction failed");

    let account = svm
        .get_account(&poll_pda)
        .expect("Poll PDA was not created");
    println!("Poll PDA account: {account:#?}");
    // 4. Deserialize Anchor account data
    let mut data: &[u8] = &account.data;

    let poll = voting_app::state::PollAccaount::try_deserialize(&mut data)
        .expect("Failed to deserialize PollAccaount");
    // 5. Verify the stored values
    assert_eq!(poll.poll_name, "POLL TEST");
    assert_eq!(poll.poll_description, "Test Poll init");
    assert_eq!(poll.poll_voting_start, 0);
    assert_eq!(poll.poll_voting_end, i64::MAX as u64);

    let account = svm
        .get_account(&candidate_pda)
        .expect("Candidate PDA was not created");
    println!("Candidate PDA account: {account:#?}");

    let mut data: &[u8] = &account.data;

    let candidate = voting_app::state::CandidateAccount::try_deserialize(&mut data)
        .expect("Failed to deserialize PollAccaount");

    assert_eq!(candidate.candidate_name, "Noha");
    assert_eq!(candidate.candidate_votes, 1);
}

#[test]
fn test_vote_end_fails() {
    let (mut svm, payer) = setup();

    let poll_id: u64 = 1;
    let candidate_name = "Noha".to_string();

    let user = payer.pubkey();
    let (poll_pda, _) = Pubkey::find_program_address(
        &[voting_app::constants::POLL_SEED, &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let (candidate_pda, _) = Pubkey::find_program_address(
        &[candidate_name.as_bytes(), &poll_id.to_le_bytes()],
        &voting_app::id(),
    );

    let init_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializePoll {
            signer: user,
            poll_account: poll_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializePoll {
            poll_description: String::from("Test Poll init"),
            poll_id: 1,
            poll_name: String::from("POLL TEST"),
            poll_start: 0,
            poll_end: 0,
        }
        .data(),
    };

    let message = Message::new(&[init_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    svm.send_transaction(transaction)
        .expect("Initialize poll transaction failed");

    let init_candidate_ix = Instruction {
        program_id: voting_app::id(),
        accounts: voting_app::accounts::InitializeCandidate {
            signer: user,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: voting_app::instruction::InitializeCandidate {
            candidate_name: candidate_name.clone(),
            poll_id: poll_id,
        }
        .data(),
    };

    let message = Message::new(&[init_candidate_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    svm.send_transaction(transaction)
        .expect("Initialize candidate transaction failed");

    let vote_ix = Instruction {
        program_id: voting_app::id(),

        accounts: voting_app::accounts::Vote {
            signer: user,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
        }
        .to_account_metas(None),
        data: voting_app::instruction::Vote {
            candidate_name: candidate_name,
            poll_id: poll_id,
        }
        .data(),
    };

    let message = Message::new(&[vote_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let result = svm.send_transaction(transaction);

    println!("Transaction result: {result:#?}");
    assert!(
        result.is_err(),
        "Transaction should fail because the poll has ended"
    );
}
