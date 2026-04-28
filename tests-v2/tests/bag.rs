//! Integration test for the `bag` program — combined repro for v2
//! bugs **#1** (`require_*!` chain leaks `solana_program_log`) and
//! **#19** (`CapacityError` not in prelude + no `From` impl for
//! `ProgramError`).
//!
//! Setup mirrors `accounts.rs`: `tests_v2::build_program` invokes
//! `cargo build-sbf` on `programs/bag`. Today that compile fails with
//! both E0433 (bug #1) and E0277 (bug #19), so `build_program` panics
//! and every test below panics during `setup()`. That panic IS the
//! combined bug repro — the day v2 fixes either or both bugs, the
//! program builds, the tests run, and any remaining failure becomes a
//! behavioural assertion to write.

use {
    anchor_lang_v2::solana_program::instruction::{AccountMeta, Instruction},
    litesvm::{types::TransactionResult, LiteSVM},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    tests_v2::{build_program, keypair_for},
};

fn program_id() -> Pubkey {
    "BagBugBag11111111111111111111111111111111111".parse().unwrap()
}

fn bag_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"bag"], &program_id()).0
}

fn setup() -> (LiteSVM, Keypair) {
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deploy_dir = test_dir.join("target/deploy");

    // Today: cargo build-sbf fails with E0433 (bug #1) and E0277 (bug
    // #19). `build_program` panics — that panic is the combined repro.
    build_program(
        test_dir.join("programs/bag").to_str().unwrap(),
        deploy_dir.to_str().unwrap(),
    );

    let mut svm = LiteSVM::new();
    svm.add_program_from_file(program_id(), deploy_dir.join("bag.so"))
        .expect("load bag program");

    let payer = keypair_for("bag-payer");
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn call(
    svm: &mut LiteSVM,
    payer: &Keypair,
    data: Vec<u8>,
    accounts: Vec<AccountMeta>,
) -> TransactionResult {
    let ix = Instruction::new_with_bytes(program_id(), &data, accounts);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let signers: Vec<&dyn solana_signer::Signer> = vec![payer];
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers).unwrap();
    let r = svm.send_transaction(tx);
    svm.expire_blockhash();
    r
}

fn init(svm: &mut LiteSVM, payer: &Keypair) {
    call(
        svm,
        payer,
        vec![0],
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(bag_pda(), false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    )
    .unwrap_or_else(|f| panic!("init: {:?}\n{}", f.err, f.meta.pretty_logs()));
}

fn read_len(svm: &LiteSVM, key: &Pubkey) -> u16 {
    let acc = svm.get_account(key).expect("bag account");
    // Layout: 8-byte disc + PodVec { len: u16 LE, data: [u64; MAX_ITEMS] }
    u16::from_le_bytes([acc.data[8], acc.data[9]])
}

#[test]
fn init_creates_empty_bag() {
    let (mut svm, payer) = setup();
    init(&mut svm, &payer);
    assert_eq!(read_len(&svm, &bag_pda()), 0);
}

#[test]
fn try_add_appends_when_expected_len_matches() {
    let (mut svm, payer) = setup();
    init(&mut svm, &payer);

    // try_add(value = 42, expected_len = 0) — should succeed.
    let mut data = vec![1u8];
    data.extend_from_slice(&42u64.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    call(
        &mut svm,
        &payer,
        data,
        vec![AccountMeta::new(bag_pda(), false)],
    )
    .unwrap_or_else(|f| panic!("try_add: {:?}\n{}", f.err, f.meta.pretty_logs()));

    assert_eq!(read_len(&svm, &bag_pda()), 1);
}

#[test]
fn try_add_rejects_when_expected_len_mismatches() {
    let (mut svm, payer) = setup();
    init(&mut svm, &payer);

    // try_add(value = 7, expected_len = 5) — bag is empty (len = 0), so
    // require_eq! must fire BagError::WrongLength = Custom(6000).
    let mut data = vec![1u8];
    data.extend_from_slice(&7u64.to_le_bytes());
    data.extend_from_slice(&5u64.to_le_bytes());
    let err = call(
        &mut svm,
        &payer,
        data,
        vec![AccountMeta::new(bag_pda(), false)],
    )
    .expect_err("require_eq! should reject expected_len mismatch");

    let txt = format!("{:?}", err.err);
    assert!(
        txt.contains("Custom(6000)"),
        "expected Custom(6000) (BagError::WrongLength), got {txt}",
    );
}
