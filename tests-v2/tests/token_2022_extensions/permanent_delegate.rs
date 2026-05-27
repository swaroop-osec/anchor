use {
    super::common::*,
    solana_pubkey::Pubkey,
    spl_token_2022_interface::extension::{permanent_delegate::PermanentDelegate, ExtensionType},
};

#[test]
fn initializes_delegate_and_rejects_wrong_program() {
    let (mut svm, payer, id) = setup(
        "permanent-delegate",
        "token_2022_ext_permanent_delegate.so",
        "7Xp89PEgC8vJzUMCRZ8itHmfurazLM7SSNJ3R8hyaG6t",
    );
    let mint = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    seed_mint_with_extensions(&mut svm, mint, &[ExtensionType::PermanentDelegate]);
    let mut data = vec![0];
    data.extend_from_slice(&address_bytes(delegate));

    let ok_metas = vec![
        Meta::new(mint, false),
        Meta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send(&mut svm, id, data.clone(), ok_metas, &payer, &[])
        .expect("permanent-delegate helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "permanent-delegate initialize");

    let mut account_data = svm.get_account(&mint).expect("mint exists").data;
    let state = mint_state(&mut account_data);
    let extension = state
        .get_extension::<PermanentDelegate>()
        .expect("permanent delegate extension exists");
    assert_eq!(Option::<Pubkey>::from(extension.delegate), Some(delegate));

    let bad_metas = vec![
        Meta::new(mint, false),
        Meta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send(&mut svm, id, data, bad_metas, &payer, &[]),
        "permanent-delegate helper should reject non-Token-2022 program",
    );
}
