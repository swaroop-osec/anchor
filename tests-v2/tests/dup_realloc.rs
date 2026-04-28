//! Integration test for `dup-realloc` — combined repro for v2
//! **bug #3** (`unsafe(dup) + realloc` on aliased accounts).
//!
//! Setup mirrors `accounts.rs`: `tests_v2::build_program` invokes
//! `cargo build-sbf` on `programs/dup-realloc`. The program **compiles
//! fine** — bug #3 is a runtime defect. The bug fires when
//! `realloc_aliased` is invoked with the same PDA in both `sample1`
//! and `sample2` slots: v2 returns `AccountBorrowFailed`. The test
//! panics with that error visible in its output, which is the bug
//! repro. The day v2 fixes the borrow-on-aliased-realloc shape, the
//! test will pass and the asserted final length will line up with the
//! v1 semantics (`len + 10`).

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
    "DupRea11oc1111111111111111111111111111111111".parse().unwrap()
}

fn sample_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"sample"], &program_id()).0
}

fn setup() -> (LiteSVM, Keypair) {
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deploy_dir = test_dir.join("target/deploy");

    build_program(
        test_dir.join("programs/dup-realloc").to_str().unwrap(),
        deploy_dir.to_str().unwrap(),
    );

    let mut svm = LiteSVM::new();
    svm.add_program_from_file(program_id(), deploy_dir.join("dup_realloc.so"))
        .expect("load dup_realloc program");

    let authority = keypair_for("dup-realloc-authority");
    svm.airdrop(&authority.pubkey(), 100_000_000_000).unwrap();
    (svm, authority)
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

fn init(svm: &mut LiteSVM, authority: &Keypair) {
    call(
        svm,
        authority,
        vec![0],
        vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(sample_pda(), false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    )
    .unwrap_or_else(|f| panic!("init: {:?}\n{}", f.err, f.meta.pretty_logs()));
}

fn data_len(svm: &LiteSVM, key: &Pubkey) -> usize {
    svm.get_account(key).expect("sample account").data.len()
}

#[test]
fn init_creates_minimum_sized_account() {
    let (mut svm, authority) = setup();
    init(&mut svm, &authority);
    // disc(8) + Vec len-prefix(4) + 1 byte data + bump(1) = 14
    assert_eq!(data_len(&svm, &sample_pda()), 14);
}

/// **Bug #3 repro.** Calls `realloc_aliased(50)` with the same PDA in
/// both `sample1` and `sample2` slots. In v1 the resulting account
/// length would be `8 + 4 + 60 + 1 = 73` (the larger of the two
/// reallocs). In v2 today this panics with `AccountBorrowFailed`.
#[test]
fn aliased_dup_realloc_grows_to_larger_size() {
    let (mut svm, authority) = setup();
    init(&mut svm, &authority);

    let mut data = vec![1u8];
    data.extend_from_slice(&50u16.to_le_bytes());

    call(
        &mut svm,
        &authority,
        data,
        vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(sample_pda(), false), // sample1
            AccountMeta::new(sample_pda(), false), // sample2 (same PDA)
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ],
    )
    .unwrap_or_else(|f| panic!("realloc_aliased: {:?}\n{}", f.err, f.meta.pretty_logs()));

    // v1 semantics: account ends at 73 bytes (the larger of the two reallocs).
    assert_eq!(data_len(&svm, &sample_pda()), 73);
}
