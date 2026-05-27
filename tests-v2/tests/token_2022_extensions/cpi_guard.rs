use {
    super::common::*,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    spl_token_2022_interface::{
        extension::{cpi_guard::CpiGuard, ExtensionType, StateWithExtensionsMut},
        state::Account as Token2022Account,
    },
};

#[test]
fn cpi_guard_toggles_fail_under_cpi_without_mutating_state() {
    let (mut svm, payer, id) = setup(
        "cpi-guard",
        "token_2022_ext_cpi_guard.so",
        "7aCUXoc5WNTQUVTeT7mJ6hXGAdR6fXumTZVFF3zoV1cV",
    );
    let account = Pubkey::new_unique();
    let mint = Pubkey::new_unique();
    let owner = tests_v2::keypair_for("token-2022-ext-cpi-guard-owner");
    let mut data =
        initialized_token_account_data(mint, owner.pubkey(), 0, &[ExtensionType::CpiGuard]);
    {
        let mut state =
            StateWithExtensionsMut::<Token2022Account>::unpack(&mut data).expect("unpack token");
        state
            .init_extension::<CpiGuard>(false)
            .expect("initialize cpi-guard extension");
    }
    seed_token_2022_account(&mut svm, account, data);
    svm.airdrop(&owner.pubkey(), 1_000_000_000).unwrap();
    assert_cpi_guard(&svm, account, false);

    for (discrim, label) in [(0, "enable"), (1, "disable")] {
        let metas = vec![
            Meta::new(account, false),
            Meta::new_readonly(owner.pubkey(), true),
            Meta::new_readonly(token_2022_program_id(), false),
        ];
        expect_program_failure(
            send(&mut svm, id, vec![discrim], metas, &payer, &[&owner]),
            &format!("cpi-guard {label} should be rejected when invoked through CPI"),
        );
        assert_cpi_guard(&svm, account, false);
    }
}

fn assert_cpi_guard(svm: &litesvm::LiteSVM, account: Pubkey, expected_enabled: bool) {
    let mut data = svm.get_account(&account).expect("token exists").data;
    let state =
        StateWithExtensionsMut::<Token2022Account>::unpack(&mut data).expect("unpack token");
    let extension = state
        .get_extension::<CpiGuard>()
        .expect("cpi guard extension exists");
    assert_eq!(bool::from(extension.lock_cpi), expected_enabled);
}
