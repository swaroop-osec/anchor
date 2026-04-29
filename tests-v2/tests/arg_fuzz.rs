use {
    anchor_lang_v2::solana_program::instruction::Instruction,
    litesvm::{types::TransactionResult, LiteSVM},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    tests_v2::{build_program, keypair_for},
};

fn program_id() -> Pubkey {
    "ArgFuzz1111111111111111111111111111111111111".parse().unwrap()
}

fn setup() -> (LiteSVM, Keypair) {
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deploy_dir = test_dir.join("target/deploy");
    build_program(
        test_dir.join("programs/arg-fuzz").to_str().unwrap(),
        deploy_dir.to_str().unwrap(),
    );

    let mut svm = LiteSVM::new();
    svm.add_program_from_file(program_id(), deploy_dir.join("arg_fuzz.so"))
        .expect("load arg_fuzz program");

    let payer = keypair_for("arg-fuzz-payer");
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn send(svm: &mut LiteSVM, payer: &Keypair, data: Vec<u8>) -> TransactionResult {
    let ix = Instruction::new_with_bytes(program_id(), &data, vec![]);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let signers: Vec<&dyn solana_signer::Signer> = vec![payer];
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &signers).unwrap();
    let r = svm.send_transaction(tx);
    svm.expire_blockhash();
    r
}

fn handler_disc(name: &str) -> [u8; 8] {
    use sha2::{Digest, Sha256};
    let h = Sha256::digest(format!("global:{name}").as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&h[..8]);
    out
}

fn is_pre_handler_rejection(err_text: &str) -> bool {
    err_text.contains("InvalidInstructionData") || err_text.contains("Custom(2503)")
}

#[test]
fn book_flag_check_canonical_true_logs() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("book_flag_check").to_vec();
    data.push(0x01);
    // PASS: canonical bool, dispatcher accepts, handler returns Ok.
    let logs = send(&mut svm, &payer, data).expect("ok").logs.join("\n");
    // PASS: handler ran with `flag=true`, log emitted.
    assert!(logs.contains("book_flag_check ran flag=true"), "got {logs}");
}

#[test]
fn book_flag_check_canonical_false_logs() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("book_flag_check").to_vec();
    data.push(0x00);
    // PASS: canonical bool, dispatcher accepts, handler returns Ok.
    let logs = send(&mut svm, &payer, data).expect("ok").logs.join("\n");
    // PASS: handler ran with `flag=false`, log emitted.
    assert!(logs.contains("book_flag_check ran flag=false"), "got {logs}");
}

#[test]
fn one_u64_canonical_logs_arg() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("one_u64").to_vec();
    data.extend_from_slice(&7u64.to_le_bytes());
    // PASS: 8 bytes of u64 = exact arg-block length, handler runs.
    let logs = send(&mut svm, &payer, data).expect("ok").logs.join("\n");
    // PASS: handler logged parsed `a=7`.
    assert!(logs.contains("one_u64 ran a=7"), "got {logs}");
}

#[test]
fn two_u64_canonical_logs_args() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("two_u64").to_vec();
    data.extend_from_slice(&3u64.to_le_bytes());
    data.extend_from_slice(&5u64.to_le_bytes());
    // PASS: 16 bytes = two u64s, handler runs.
    let logs = send(&mut svm, &payer, data).expect("ok").logs.join("\n");
    // PASS: handler logged parsed `a=3 b=5`.
    assert!(logs.contains("two_u64 ran a=3 b=5"), "got {logs}");
}

#[test]
fn no_args_canonical_logs() {
    let (mut svm, payer) = setup();
    let data = handler_disc("no_args").to_vec();
    // PASS: empty arg block, handler runs.
    let logs = send(&mut svm, &payer, data).expect("ok").logs.join("\n");
    // PASS: handler logged its sentinel.
    assert!(logs.contains("no_args ran"), "got {logs}");
}

#[test]
fn book_flag_check_non_canonical_byte_rejects() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("book_flag_check").to_vec();
    data.push(0x05);

    // PASS: wincode IS strict on per-field bool decoding (only 0/1 valid),
    // so 0x05 fails parsing and the dispatcher returns InstructionDidNotDeserialize.
    let err = send(&mut svm, &payer, data).expect_err("non-canonical bool must reject");
    let txt = format!("{:?}", err.err);

    // PASS: error matches the dispatcher-level rejection signature.
    assert!(
        is_pre_handler_rejection(&txt),
        "v2 silently decoded 0x05 as bool. got {txt}",
    );
}

#[test]
fn bug_one_u64_trailing_bytes() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("one_u64").to_vec();
    data.extend_from_slice(&7u64.to_le_bytes());
    data.extend_from_slice(&[0xAAu8; 16]);

    // FAIL: dispatcher reads 8 bytes for `a`, ignores 16 trailing,
    // handler runs Ok → tx succeeds → expect_err panics here.
    let err = send(&mut svm, &payer, data).expect_err("trailing bytes must reject");
    let txt = format!("{:?}", err.err);

    // Unreachable; would PASS after the consumed-len fix.
    assert!(
        is_pre_handler_rejection(&txt),
        "v2 silently accepted 16 trailing bytes after `u64`. got {txt}",
    );
}

#[test]
fn bug_book_flag_check_trailing_bytes() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("book_flag_check").to_vec();
    data.push(0x01);
    data.extend_from_slice(&[0xAAu8; 8]);

    // FAIL: dispatcher reads 1 byte for `flag`, ignores 8 trailing,
    // handler runs Ok → tx succeeds → expect_err panics here.
    let err = send(&mut svm, &payer, data)
        .expect_err("trailing bytes after bool must reject");
    let txt = format!("{:?}", err.err);

    // Unreachable; would PASS after the consumed-len fix.
    assert!(
        is_pre_handler_rejection(&txt),
        "v2 silently accepted 8 trailing bytes after `bool`. got {txt}",
    );
}

#[test]
fn bug_two_u64_trailing_bytes() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("two_u64").to_vec();
    data.extend_from_slice(&3u64.to_le_bytes());
    data.extend_from_slice(&5u64.to_le_bytes());
    data.extend_from_slice(&[0xCCu8; 24]);

    // FAIL: dispatcher reads 16 bytes for `a, b`, ignores 24 trailing,
    // handler runs Ok → tx succeeds → expect_err panics here.
    let err = send(&mut svm, &payer, data)
        .expect_err("trailing bytes after multi-arg must reject");
    let txt = format!("{:?}", err.err);

    // Unreachable; would PASS after the consumed-len fix.
    assert!(
        is_pre_handler_rejection(&txt),
        "v2 silently accepted 24 trailing bytes after `u64, u64`. got {txt}",
    );
}

#[test]
fn bug_no_args_trailing_bytes() {
    let (mut svm, payer) = setup();
    let mut data = handler_disc("no_args").to_vec();
    data.extend_from_slice(&[0xDDu8; 32]);

    // FAIL: handler takes no args so the dispatcher reads 0 bytes,
    // ignores 32 trailing, body runs Ok → tx succeeds → expect_err panics here.
    let err = send(&mut svm, &payer, data)
        .expect_err("trailing bytes after no-arg handler must reject");
    let txt = format!("{:?}", err.err);

    // Unreachable; would PASS after the consumed-len fix.
    assert!(
        is_pre_handler_rejection(&txt),
        "v2 silently ran a NO-ARG handler with 32 bytes of garbage in the data \
         field. got {txt}",
    );
}
