use {
    super::common::*,
    spl_token_2022_interface::{
        extension::{non_transferable::NonTransferable, ExtensionType, StateWithExtensionsMut},
        state::Mint,
    },
};

#[test]
fn initializes_non_transferable_marker_and_rejects_wrong_program() {
    let (mut svm, payer, id) = setup(
        "non-transferable",
        "token_2022_ext_non_transferable.so",
        "3m3rPAmtMvkNvZcW1H773G1yLtgr7HLRCErSLTdz3NgZ",
    );
    let mint = solana_pubkey::Pubkey::new_unique();
    seed_mint_with_extensions(&mut svm, mint, &[ExtensionType::NonTransferable]);

    let ok_metas = vec![
        Meta::new(mint, false),
        Meta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send(&mut svm, id, vec![0], ok_metas, &payer, &[])
        .expect("non-transferable helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "non-transferable initialize");

    let mut data = svm.get_account(&mint).expect("mint exists").data;
    StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data)
        .expect("mint remains uninitialized")
        .get_extension::<NonTransferable>()
        .expect("non-transferable marker should be initialized");

    let bad_metas = vec![
        Meta::new(mint, false),
        Meta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send(&mut svm, id, vec![0], bad_metas, &payer, &[]),
        "non-transferable helper should reject non-Token-2022 program",
    );
}
