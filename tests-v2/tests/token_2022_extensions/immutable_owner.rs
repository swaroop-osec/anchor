use {
    super::common::*,
    spl_token_2022_interface::{
        extension::{immutable_owner::ImmutableOwner, ExtensionType, StateWithExtensionsMut},
        state::Account as Token2022Account,
    },
};

#[test]
fn initializes_immutable_owner_extension_and_rejects_wrong_program() {
    let (mut svm, payer, id) = setup(
        "immutable-owner",
        "token_2022_ext_immutable_owner.so",
        "DEYmLQZNGBBhzQM8vqfhKerrcybk3PxMXYj62NY8gwZR",
    );
    let token = solana_pubkey::Pubkey::new_unique();
    seed_token_account_with_extensions(&mut svm, token, &[ExtensionType::ImmutableOwner]);

    let ok_metas = vec![
        Meta::new(token, false),
        Meta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send(&mut svm, id, vec![0], ok_metas, &payer, &[])
        .expect("immutable-owner helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "immutable-owner initialize");

    let mut data = svm.get_account(&token).expect("token exists").data;
    StateWithExtensionsMut::<Token2022Account>::unpack_uninitialized(&mut data)
        .expect("token account remains uninitialized")
        .get_extension::<ImmutableOwner>()
        .expect("immutable-owner extension should be initialized");

    let bad_metas = vec![
        Meta::new(token, false),
        Meta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send(&mut svm, id, vec![0], bad_metas, &payer, &[]),
        "immutable-owner helper should reject non-Token-2022 program",
    );
}
