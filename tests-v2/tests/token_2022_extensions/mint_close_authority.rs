use {
    super::common::*,
    solana_pubkey::Pubkey,
    spl_token_2022_interface::extension::{
        mint_close_authority::MintCloseAuthority, ExtensionType,
    },
};

#[test]
fn initializes_close_authority_and_rejects_wrong_program() {
    let (mut svm, payer, id) = setup(
        "mint-close-authority",
        "token_2022_ext_mint_close_authority.so",
        "3riR5k4baKpAn75dhjKgQJtZHVRfsKy5211q2zxphgbC",
    );
    let mint = Pubkey::new_unique();
    let close_authority = Pubkey::new_unique();
    seed_mint_with_extensions(&mut svm, mint, &[ExtensionType::MintCloseAuthority]);
    let mut data = vec![0];
    data.extend_from_slice(&address_bytes(close_authority));

    let ok_metas = vec![
        Meta::new(mint, false),
        Meta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send(&mut svm, id, data.clone(), ok_metas, &payer, &[])
        .expect("mint-close-authority helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "mint-close-authority initialize");

    let mut account_data = svm.get_account(&mint).expect("mint exists").data;
    let state = mint_state(&mut account_data);
    let extension = state
        .get_extension::<MintCloseAuthority>()
        .expect("close authority extension exists");
    assert_eq!(
        Option::<Pubkey>::from(extension.close_authority),
        Some(close_authority)
    );

    let bad_metas = vec![
        Meta::new(mint, false),
        Meta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send(&mut svm, id, data, bad_metas, &payer, &[]),
        "mint-close-authority helper should reject non-Token-2022 program",
    );
}
