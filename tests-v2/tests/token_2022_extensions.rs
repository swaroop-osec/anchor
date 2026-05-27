use {
    anchor_lang_v2::solana_program::instruction::AccountMeta,
    litesvm::{types::TransactionMetadata, LiteSVM},
    sha2::{Digest, Sha256},
    solana_account::Account,
    solana_keypair::Keypair,
    solana_loader_v3_interface::get_program_data_address,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_rpc_client::rpc_client::RpcClient,
    solana_signer::Signer,
    spl_token_2022_interface::{
        extension::{
            default_account_state::DefaultAccountState, memo_transfer::MemoTransfer,
            BaseStateWithExtensions, BaseStateWithExtensionsMut, ExtensionType,
            StateWithExtensionsMut,
        },
        state::{Account as Token2022Account, AccountState, Mint},
    },
    std::path::PathBuf,
    tests_v2::{build_program, keypair_for, send_instruction},
};

const TOKEN_2022_MAINNET_SO: &str = "fixtures/programs/token_2022_mainnet.so";
const TOKEN_2022_MAINNET_SHA256: &str =
    "b2a7ce1ea6dfbcbc5ccb0e7f48f7c61dced1a86582d1c7d2e059ac54ed612da4";
const PROGRAMDATA_METADATA_LEN: usize = 45;

fn token_2022_program_id() -> Pubkey {
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        .parse()
        .unwrap()
}

fn wrong_token_program_id() -> Pubkey {
    "7TdHZyhueZP4B8fvbgvbGPTH4bijkBPtpWc3wBfTmWQv"
        .parse()
        .unwrap()
}

fn program_id(address: &str) -> Pubkey {
    address.parse().unwrap()
}

fn token_2022_mainnet_so_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TOKEN_2022_MAINNET_SO)
}

fn setup(program_dir: &str, program_so: &str, address: &str) -> (LiteSVM, Keypair, Pubkey) {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deploy_dir = test_dir.join("target/deploy");
    build_program(
        test_dir
            .join("programs/token-2022-extensions")
            .join(program_dir)
            .to_str()
            .unwrap(),
        deploy_dir.to_str().unwrap(),
    );

    let id = program_id(address);
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(id, deploy_dir.join(program_so))
        .expect("load extension fixture program");
    svm.add_program_from_file(token_2022_program_id(), token_2022_mainnet_so_path())
        .expect("load vendored mainnet Token-2022 program");

    let payer = keypair_for(&format!("token-2022-ext-{program_dir}-payer"));
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer, id)
}

fn seed_account(svm: &mut LiteSVM, address: Pubkey) {
    svm.set_account(
        address,
        Account {
            lamports: 10_000_000,
            data: vec![0; 256],
            owner: token_2022_program_id(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .expect("seed token-owned account");
}

fn seed_mint_with_extensions(svm: &mut LiteSVM, address: Pubkey, extensions: &[ExtensionType]) {
    let len = ExtensionType::try_calculate_account_len::<Mint>(extensions)
        .expect("calculate token-2022 mint account length");
    svm.set_account(
        address,
        Account {
            lamports: 10_000_000,
            data: vec![0; len],
            owner: token_2022_program_id(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .expect("seed uninitialized token-2022 mint account");
}

fn seed_token_account_with_extensions(
    svm: &mut LiteSVM,
    address: Pubkey,
    extensions: &[ExtensionType],
) {
    let len = ExtensionType::try_calculate_account_len::<Token2022Account>(extensions)
        .expect("calculate token-2022 token account length");
    svm.set_account(
        address,
        Account {
            lamports: 10_000_000,
            data: vec![0; len],
            owner: token_2022_program_id(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .expect("seed uninitialized token-2022 token account");
}

fn seed_initialized_memo_transfer_account(
    svm: &mut LiteSVM,
    address: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
) {
    let len = ExtensionType::try_calculate_account_len::<Token2022Account>(&[
        ExtensionType::MemoTransfer,
    ])
    .expect("calculate token-2022 memo-transfer account length");
    let mut data = vec![0; len];
    Token2022Account {
        mint,
        owner,
        amount: 0,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut data[..Token2022Account::LEN]);
    spl_token_2022_interface::extension::set_account_type::<Token2022Account>(&mut data)
        .expect("set token account type");
    {
        let mut state =
            StateWithExtensionsMut::<Token2022Account>::unpack(&mut data).expect("unpack token");
        state
            .init_extension::<MemoTransfer>(false)
            .expect("initialize memo-transfer extension slot");
    }
    svm.set_account(
        address,
        Account {
            lamports: 10_000_000,
            data,
            owner: token_2022_program_id(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .expect("seed initialized memo-transfer token account");
}

fn address_bytes(address: Pubkey) -> Vec<u8> {
    address.to_bytes().to_vec()
}

fn assert_token_2022_cpi_succeeded(metadata: &TransactionMetadata, context: &str) {
    let token_id = token_2022_program_id().to_string();
    assert!(
        metadata
            .logs
            .iter()
            .any(|log| log.contains(&format!("Program {token_id} invoke"))),
        "{context}: Token-2022 program was not invoked\n{}",
        metadata.pretty_logs()
    );
    assert!(
        metadata
            .logs
            .iter()
            .any(|log| log.contains(&format!("Program {token_id} success"))),
        "{context}: Token-2022 program did not report success\n{}",
        metadata.pretty_logs()
    );
}

fn expect_incorrect_program<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    let Err(error) = result else {
        panic!("{context}");
    };
    let error = error.to_string();
    assert!(
        error.contains("IncorrectProgramId") || error.contains("incorrect program id"),
        "{context}: expected IncorrectProgramId, got:\n{error}"
    );
}

fn expect_program_failure<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    let Err(error) = result else {
        panic!("{context}");
    };
    let error = error.to_string();
    assert!(
        error.contains("ProgramFailedToComplete")
            || error.contains("program failed")
            || error.contains("panicked"),
        "{context}: expected program failure, got:\n{error}"
    );
}

#[test]
fn immutable_owner_program_invokes_helper_and_rejects_wrong_program() {
    let (mut svm, payer, id) = setup(
        "immutable-owner",
        "token_2022_ext_immutable_owner.so",
        "DEYmLQZNGBBhzQM8vqfhKerrcybk3PxMXYj62NY8gwZR",
    );
    let token = Pubkey::new_unique();
    seed_token_account_with_extensions(&mut svm, token, &[ExtensionType::ImmutableOwner]);

    let ok_metas = vec![
        AccountMeta::new(token, false),
        AccountMeta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send_instruction(&mut svm, id, vec![0], ok_metas, &payer, &[])
        .expect("immutable-owner helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "immutable-owner initialize");

    let bad_metas = vec![
        AccountMeta::new(token, false),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(&mut svm, id, vec![0], bad_metas, &payer, &[]),
        "immutable-owner helper should reject non-Token-2022 program",
    );
}

#[test]
fn non_transferable_program_invokes_helper_and_rejects_wrong_program() {
    let (mut svm, payer, id) = setup(
        "non-transferable",
        "token_2022_ext_non_transferable.so",
        "3m3rPAmtMvkNvZcW1H773G1yLtgr7HLRCErSLTdz3NgZ",
    );
    let mint = Pubkey::new_unique();
    seed_mint_with_extensions(&mut svm, mint, &[ExtensionType::NonTransferable]);

    let ok_metas = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send_instruction(&mut svm, id, vec![0], ok_metas, &payer, &[])
        .expect("non-transferable helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "non-transferable initialize");

    let bad_metas = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(&mut svm, id, vec![0], bad_metas, &payer, &[]),
        "non-transferable helper should reject non-Token-2022 program",
    );
}

#[test]
fn mint_close_authority_program_invokes_helper_and_rejects_wrong_program() {
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
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send_instruction(&mut svm, id, data.clone(), ok_metas, &payer, &[])
        .expect("mint-close-authority helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "mint-close-authority initialize");

    let bad_metas = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(&mut svm, id, data, bad_metas, &payer, &[]),
        "mint-close-authority helper should reject non-Token-2022 program",
    );
}

#[test]
fn permanent_delegate_program_invokes_helper_and_rejects_wrong_program() {
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
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send_instruction(&mut svm, id, data.clone(), ok_metas, &payer, &[])
        .expect("permanent-delegate helper should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "permanent-delegate initialize");

    let bad_metas = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(&mut svm, id, data, bad_metas, &payer, &[]),
        "permanent-delegate helper should reject non-Token-2022 program",
    );
}

#[test]
fn cpi_guard_program_fails_fast_because_token_2022_rejects_cpi_toggles() {
    let (mut svm, payer, id) = setup(
        "cpi-guard",
        "token_2022_ext_cpi_guard.so",
        "7aCUXoc5WNTQUVTeT7mJ6hXGAdR6fXumTZVFF3zoV1cV",
    );
    let account = Pubkey::new_unique();
    let owner = keypair_for("token-2022-ext-cpi-guard-owner");
    seed_account(&mut svm, account);
    svm.airdrop(&owner.pubkey(), 1_000_000_000).unwrap();

    for discrim in [0, 1] {
        let metas = vec![
            AccountMeta::new(account, false),
            AccountMeta::new_readonly(owner.pubkey(), true),
            AccountMeta::new_readonly(token_2022_program_id(), false),
        ];
        expect_program_failure(
            send_instruction(&mut svm, id, vec![discrim], metas, &payer, &[&owner]),
            "cpi-guard helper should fail fast instead of emitting unsupported CPI",
        );
    }
}

#[test]
fn default_account_state_program_covers_initialize_and_update_guards() {
    let (mut svm, payer, id) = setup(
        "default-account-state",
        "token_2022_ext_default_account_state.so",
        "Fetkn8caf7wN24u751NWUYhtXXGuCPrqTLyDtqU25EY8",
    );
    let mint = Pubkey::new_unique();
    let freeze_authority = keypair_for("token-2022-ext-default-account-state-authority");
    seed_mint_with_extensions(&mut svm, mint, &[ExtensionType::DefaultAccountState]);
    svm.airdrop(&freeze_authority.pubkey(), 1_000_000_000)
        .unwrap();

    let init_metas = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send_instruction(&mut svm, id, vec![0], init_metas, &payer, &[])
        .expect("default-account-state initialize should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "default-account-state initialize");
    let mut mint_data = svm.get_account(&mint).expect("mint exists").data;
    let state = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut mint_data)
        .expect("unpack uninitialized mint with default-account-state extension");
    let extension = state
        .get_extension::<DefaultAccountState>()
        .expect("default-account-state extension exists");
    assert_eq!(extension.state, AccountState::Frozen as u8);

    let bad_update_metas = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new_readonly(freeze_authority.pubkey(), true),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(
            &mut svm,
            id,
            vec![1],
            bad_update_metas,
            &payer,
            &[&freeze_authority],
        ),
        "default-account-state update should reject non-Token-2022 program",
    );
}

#[test]
fn pointer_programs_cover_initialize_guards_and_real_token_2022_paths() {
    let cases = [
        (
            "group-pointer",
            "token_2022_ext_group_pointer.so",
            "3aVa6BL8bgD4My8vgUABgjGSBTYpHQ6Ft41wC3H5EQ5f",
            ExtensionType::GroupPointer,
        ),
        (
            "group-member-pointer",
            "token_2022_ext_group_member_pointer.so",
            "FCvBGDgHxLFU3rgL8jqp4aGJpjJzyyotDKQdrjLhUner",
            ExtensionType::GroupMemberPointer,
        ),
    ];

    for (dir, so, address, extension_type) in cases {
        let (mut svm, payer, id) = setup(dir, so, address);
        let mint = Pubkey::new_unique();
        let authority = keypair_for(&format!("token-2022-ext-{dir}-authority"));
        let pointer = Pubkey::new_unique();
        seed_mint_with_extensions(&mut svm, mint, &[extension_type]);
        svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();

        let mut init_data = vec![0];
        init_data.extend_from_slice(&address_bytes(authority.pubkey()));
        init_data.extend_from_slice(&address_bytes(pointer));
        let bad_init_metas = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(wrong_token_program_id(), false),
        ];
        expect_incorrect_program(
            send_instruction(&mut svm, id, init_data.clone(), bad_init_metas, &payer, &[]),
            "pointer initialize should reject non-Token-2022 program",
        );

        let ok_init_metas = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(token_2022_program_id(), false),
        ];
        let metadata = send_instruction(&mut svm, id, init_data, ok_init_metas, &payer, &[])
            .expect("pointer initialize should invoke real Token-2022");
        assert_token_2022_cpi_succeeded(&metadata, "pointer initialize");

        let mut update_data = vec![1];
        update_data.extend_from_slice(&address_bytes(pointer));
        let bad_update_metas = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(wrong_token_program_id(), false),
        ];
        expect_incorrect_program(
            send_instruction(
                &mut svm,
                id,
                update_data,
                bad_update_metas,
                &payer,
                &[&authority],
            ),
            "pointer update should reject non-Token-2022 program",
        );
    }
}

#[test]
fn initialize_style_extension_programs_invoke_real_token_2022_and_reject_wrong_programs() {
    let cases = [
        (
            "metadata-pointer",
            "token_2022_ext_metadata_pointer.so",
            "8PeNs8jhrvR4uDtSSyB2iYcyx5FUQBWhrnHJfwdHwXiS",
            vec![0],
            2,
            ExtensionType::MetadataPointer,
        ),
        (
            "transfer-hook",
            "token_2022_ext_transfer_hook.so",
            "Bs5CGVSvcNqrTyzZig9fVHcDZhCmvFeserfVFK7BiSjR",
            vec![0],
            2,
            ExtensionType::TransferHook,
        ),
        (
            "interest-bearing-mint",
            "token_2022_ext_interest_bearing_mint.so",
            "4Yv95TS6s4kME8qgqLpjkekknuXJnksuCnVbc37Pdp6j",
            vec![0],
            1,
            ExtensionType::InterestBearingConfig,
        ),
        (
            "pausable",
            "token_2022_ext_pausable.so",
            "EoHXMZePT9ShHp5tUxBhZQW4MRm4P4r2ejx7VXMpP2My",
            vec![0],
            1,
            ExtensionType::Pausable,
        ),
        (
            "transfer-fee",
            "token_2022_ext_transfer_fee.so",
            "CvCYVXhFDScZ8CNRtm6mSU8AkZrN5tk3NcFF8Q33M45z",
            vec![0],
            1,
            ExtensionType::TransferFeeConfig,
        ),
    ];

    for (dir, so, address, prefix, address_arg_count, extension_type) in cases {
        let (mut svm, payer, id) = setup(dir, so, address);
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let second_address = Pubkey::new_unique();
        seed_mint_with_extensions(&mut svm, mint, &[extension_type]);

        let mut data = prefix;
        data.extend_from_slice(&address_bytes(authority));
        if address_arg_count == 2 {
            data.extend_from_slice(&address_bytes(second_address));
        }

        let ok_metas = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(token_2022_program_id(), false),
        ];
        let metadata = send_instruction(&mut svm, id, data.clone(), ok_metas, &payer, &[])
            .expect("initialize-style extension helper should invoke real Token-2022");
        assert_token_2022_cpi_succeeded(&metadata, "initialize-style extension helper");

        let bad_metas = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(wrong_token_program_id(), false),
        ];
        expect_incorrect_program(
            send_instruction(&mut svm, id, data, bad_metas, &payer, &[]),
            "initialize-style helper should reject non-Token-2022 program",
        );
    }
}

#[test]
fn update_style_extension_programs_reject_wrong_token_programs() {
    let cases = [
        (
            "metadata-pointer",
            "token_2022_ext_metadata_pointer.so",
            "8PeNs8jhrvR4uDtSSyB2iYcyx5FUQBWhrnHJfwdHwXiS",
            vec![1],
            true,
        ),
        (
            "transfer-hook",
            "token_2022_ext_transfer_hook.so",
            "Bs5CGVSvcNqrTyzZig9fVHcDZhCmvFeserfVFK7BiSjR",
            vec![1],
            true,
        ),
        (
            "interest-bearing-mint",
            "token_2022_ext_interest_bearing_mint.so",
            "4Yv95TS6s4kME8qgqLpjkekknuXJnksuCnVbc37Pdp6j",
            vec![1],
            false,
        ),
        (
            "pausable",
            "token_2022_ext_pausable.so",
            "EoHXMZePT9ShHp5tUxBhZQW4MRm4P4r2ejx7VXMpP2My",
            vec![1],
            false,
        ),
        (
            "pausable",
            "token_2022_ext_pausable.so",
            "EoHXMZePT9ShHp5tUxBhZQW4MRm4P4r2ejx7VXMpP2My",
            vec![2],
            false,
        ),
    ];

    for (dir, so, address, mut data, has_address_arg) in cases {
        let (mut svm, payer, id) = setup(dir, so, address);
        let mint = Pubkey::new_unique();
        let authority = keypair_for(&format!("token-2022-ext-{dir}-toggle-authority"));
        seed_account(&mut svm, mint);
        svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();
        if has_address_arg {
            data.extend_from_slice(&address_bytes(Pubkey::new_unique()));
        }

        let metas = vec![
            AccountMeta::new(mint, false),
            AccountMeta::new_readonly(authority.pubkey(), true),
            AccountMeta::new_readonly(wrong_token_program_id(), false),
        ];
        expect_incorrect_program(
            send_instruction(&mut svm, id, data, metas, &payer, &[&authority]),
            "update-style extension helper should reject non-Token-2022 program",
        );
    }
}

#[test]
fn memo_transfer_program_invokes_enable_and_guards_disable() {
    let (mut svm, payer, id) = setup(
        "memo-transfer",
        "token_2022_ext_memo_transfer.so",
        "6wc58Q2xzU5Lw21XrsJXy31LJbcoTcGbEa3knKj7enwM",
    );
    let account = Pubkey::new_unique();
    let mint = Pubkey::new_unique();
    let owner = keypair_for("token-2022-ext-memo-owner");
    seed_initialized_memo_transfer_account(&mut svm, account, mint, owner.pubkey());
    svm.airdrop(&owner.pubkey(), 1_000_000_000).unwrap();

    let ok_metas = vec![
        AccountMeta::new(account, false),
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(token_2022_program_id(), false),
    ];
    let metadata = send_instruction(&mut svm, id, vec![0], ok_metas, &payer, &[&owner])
        .expect("memo transfer initialize should invoke real Token-2022");
    assert_token_2022_cpi_succeeded(&metadata, "memo transfer initialize");

    let bad_metas = vec![
        AccountMeta::new(account, false),
        AccountMeta::new_readonly(owner.pubkey(), true),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(&mut svm, id, vec![1], bad_metas, &payer, &[&owner]),
        "memo transfer disable should reject non-Token-2022 program",
    );
}

#[test]
fn transfer_fee_program_guards_non_initialize_helpers() {
    let (mut svm, payer, id) = setup(
        "transfer-fee",
        "token_2022_ext_transfer_fee.so",
        "CvCYVXhFDScZ8CNRtm6mSU8AkZrN5tk3NcFF8Q33M45z",
    );
    let mint = Pubkey::new_unique();
    let source = Pubkey::new_unique();
    let destination = Pubkey::new_unique();
    let authority = keypair_for("token-2022-ext-transfer-fee-authority");
    for account in [mint, source, destination] {
        seed_account(&mut svm, account);
    }
    svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();

    let cases = [
        (
            vec![1],
            vec![
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            true,
        ),
        (
            vec![2],
            vec![
                AccountMeta::new(source, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(destination, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            true,
        ),
        (
            vec![3],
            vec![
                AccountMeta::new(mint, false),
                AccountMeta::new(source, false),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            false,
        ),
        (
            vec![4],
            vec![
                AccountMeta::new(mint, false),
                AccountMeta::new(destination, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            true,
        ),
        (
            vec![5],
            vec![
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(destination, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new(source, false),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            true,
        ),
    ];

    for (data, metas, signs) in cases {
        let signers = if signs { vec![&authority] } else { vec![] };
        expect_incorrect_program(
            send_instruction(&mut svm, id, data, metas, &payer, &signers),
            "transfer fee helper should reject non-Token-2022 program",
        );
    }
}

#[test]
fn token_metadata_program_guards_all_helpers() {
    let (mut svm, payer, id) = setup(
        "token-metadata",
        "token_2022_ext_token_metadata.so",
        "7j7C1skvNNAm1GPr6icFPbdsare3eYpQGrtuzXJ6jzy6",
    );
    let metadata = Pubkey::new_unique();
    let mint = Pubkey::new_unique();
    let mint_authority = keypair_for("token-2022-ext-token-metadata-mint-authority");
    let update_authority = keypair_for("token-2022-ext-token-metadata-update-authority");
    let new_authority = Pubkey::new_unique();
    for account in [metadata, mint, new_authority] {
        seed_account(&mut svm, account);
    }
    svm.airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    svm.airdrop(&update_authority.pubkey(), 1_000_000_000)
        .unwrap();

    let cases = [
        (
            vec![0],
            vec![
                AccountMeta::new(metadata, false),
                AccountMeta::new_readonly(update_authority.pubkey(), false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(mint_authority.pubkey(), true),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            vec![&mint_authority],
        ),
        (
            vec![1],
            vec![
                AccountMeta::new(metadata, false),
                AccountMeta::new_readonly(update_authority.pubkey(), true),
                AccountMeta::new_readonly(new_authority, false),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            vec![&update_authority],
        ),
        (
            vec![2],
            vec![
                AccountMeta::new(metadata, false),
                AccountMeta::new_readonly(update_authority.pubkey(), true),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            vec![&update_authority],
        ),
        (
            vec![3],
            vec![
                AccountMeta::new(metadata, false),
                AccountMeta::new_readonly(update_authority.pubkey(), true),
                AccountMeta::new_readonly(wrong_token_program_id(), false),
            ],
            vec![&update_authority],
        ),
    ];

    for (data, metas, signers) in cases {
        expect_incorrect_program(
            send_instruction(&mut svm, id, data, metas, &payer, &signers),
            "token metadata helper should reject non-Token-2022 program",
        );
    }
}

#[test]
fn token_group_program_guards_group_and_member_helpers() {
    let (mut svm, payer, id) = setup(
        "token-group",
        "token_2022_ext_token_group.so",
        "BrUvjfkhwnq2oL4uvHCEZ3LDxaXsqbjUCvGUq4LtDHAb",
    );
    let group = Pubkey::new_unique();
    let mint = Pubkey::new_unique();
    let member = Pubkey::new_unique();
    let member_mint = Pubkey::new_unique();
    let mint_authority = keypair_for("token-2022-ext-token-group-mint-authority");
    let group_authority = keypair_for("token-2022-ext-token-group-authority");
    for account in [group, mint, member, member_mint] {
        seed_account(&mut svm, account);
    }
    svm.airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();
    svm.airdrop(&group_authority.pubkey(), 1_000_000_000)
        .unwrap();

    let group_metas = vec![
        AccountMeta::new(group, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new_readonly(mint_authority.pubkey(), true),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(
            &mut svm,
            id,
            vec![0],
            group_metas,
            &payer,
            &[&mint_authority],
        ),
        "token group initialize helper should reject non-Token-2022 program",
    );

    let member_metas = vec![
        AccountMeta::new(member, false),
        AccountMeta::new_readonly(member_mint, false),
        AccountMeta::new_readonly(mint_authority.pubkey(), true),
        AccountMeta::new(group, false),
        AccountMeta::new_readonly(group_authority.pubkey(), true),
        AccountMeta::new_readonly(wrong_token_program_id(), false),
    ];
    expect_incorrect_program(
        send_instruction(
            &mut svm,
            id,
            vec![1],
            member_metas,
            &payer,
            &[&mint_authority, &group_authority],
        ),
        "token member initialize helper should reject non-Token-2022 program",
    );
}

#[test]
#[ignore = "hits mainnet-beta RPC; run manually to refresh/verify the vendored Token-2022 fixture"]
fn vendored_token_2022_program_matches_mainnet_beta() {
    let local = std::fs::read(token_2022_mainnet_so_path()).expect("read vendored Token-2022 ELF");
    let local_hash = Sha256::digest(&local)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        local_hash, TOKEN_2022_MAINNET_SHA256,
        "vendored Token-2022 fixture hash changed unexpectedly"
    );

    let rpc = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let programdata = get_program_data_address(&token_2022_program_id());
    let account = rpc
        .get_account(&programdata)
        .expect("fetch Token-2022 ProgramData account from mainnet-beta");
    assert!(
        account.data.len() >= PROGRAMDATA_METADATA_LEN,
        "ProgramData account is shorter than upgradeable-loader metadata"
    );
    assert_eq!(
        &account.data[PROGRAMDATA_METADATA_LEN..],
        local.as_slice(),
        "vendored Token-2022 ELF differs from mainnet-beta ProgramData"
    );
}
