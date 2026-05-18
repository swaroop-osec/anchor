use {
    anchor_lang_v2::solana_program::instruction::{AccountMeta, Instruction},
    litesvm::LiteSVM,
    proptest::prelude::*,
    sha2::{Digest, Sha256},
    solana_account::Account,
    solana_message::{Message, VersionedMessage},
    solana_program_option::COption as Token2022COption,
    solana_program_pack::Pack as Token2022Pack,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    spl_token::{
        solana_program::{
            program_option::COption, program_pack::Pack, pubkey::Pubkey as SplPubkey,
        },
        state::{Account as SplTokenAccount, AccountState as SplAccountState, Mint as SplMint},
    },
    spl_token_2022_interface::{
        extension::{
            confidential_mint_burn::ConfidentialMintBurn,
            confidential_transfer::{ConfidentialTransferAccount, ConfidentialTransferMint},
            confidential_transfer_fee::{
                ConfidentialTransferFeeAmount, ConfidentialTransferFeeConfig,
            },
            cpi_guard::CpiGuard,
            default_account_state::DefaultAccountState,
            group_member_pointer::GroupMemberPointer,
            group_pointer::GroupPointer,
            immutable_owner::ImmutableOwner,
            interest_bearing_mint::InterestBearingConfig,
            memo_transfer::MemoTransfer,
            metadata_pointer::MetadataPointer,
            mint_close_authority::MintCloseAuthority,
            non_transferable::{NonTransferable, NonTransferableAccount},
            pausable::{PausableAccount, PausableConfig},
            permanent_delegate::PermanentDelegate,
            scaled_ui_amount::ScaledUiAmountConfig,
            transfer_fee::TransferFeeAmount,
            transfer_fee::TransferFeeConfig,
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseStateWithExtensionsMut, ExtensionType as Token2022ExtensionType,
            StateWithExtensionsMut,
        },
        state::{
            Account as Token2022Account, AccountState as Token2022AccountState,
            Mint as Token2022Mint,
        },
    },
    spl_token_group_interface::state::{TokenGroup, TokenGroupMember},
    std::sync::OnceLock,
    tests_v2::{build_program, keypair_for},
};

const OBSERVATION_LEN: usize = 256;
const MAX_SMALL_RANDOM_DATA_LEN: usize = 512;
const MAX_LARGE_RANDOM_DATA_LEN: usize = 20 * 1024;
const DEFAULT_ACCOUNT_LAMPORTS: u64 = 10_000_000;
const DEFAULT_PAYER_LAMPORTS: u64 = 10_000_000_000;
const V2_DISC_STRICT_MINT: u8 = 0;
const V2_DISC_STRICT_TOKEN_ACCOUNT: u8 = 1;
const V2_DISC_INTERFACE_MINT: u8 = 2;
const V2_DISC_INTERFACE_TOKEN_ACCOUNT: u8 = 3;

#[derive(Clone, Copy, Debug)]
enum OracleVersion {
    V1,
    V2,
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    StrictMint,
    StrictTokenAccount,
    InterfaceMint,
    InterfaceTokenAccount,
}

impl Operation {
    fn v1_name(self) -> &'static str {
        match self {
            Operation::StrictMint => "check_strict_mint",
            Operation::StrictTokenAccount => "check_strict_token_account",
            Operation::InterfaceMint => "check_interface_mint",
            Operation::InterfaceTokenAccount => "check_interface_token_account",
        }
    }

    fn v2_discriminator(self) -> u8 {
        match self {
            Operation::StrictMint => V2_DISC_STRICT_MINT,
            Operation::StrictTokenAccount => V2_DISC_STRICT_TOKEN_ACCOUNT,
            Operation::InterfaceMint => V2_DISC_INTERFACE_MINT,
            Operation::InterfaceTokenAccount => V2_DISC_INTERFACE_TOKEN_ACCOUNT,
        }
    }
}

#[derive(Clone, Debug)]
struct Case {
    operation: Operation,
    payer_lamports: u64,
    target: AccountCase,
    output: OutputAccountCase,
    target_signer: bool,
    target_writable: bool,
    output_signer: bool,
    output_writable: bool,
}

#[derive(Clone, Debug)]
struct AccountCase {
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
    executable: bool,
    rent_epoch: u64,
}

#[derive(Clone, Debug)]
struct OutputAccountCase {
    lamports: u64,
    data: Vec<u8>,
    owner: OutputOwner,
    executable: bool,
    rent_epoch: u64,
}

#[derive(Clone, Copy, Debug)]
enum OutputOwner {
    Oracle,
    Address(Pubkey),
}

impl OutputOwner {
    fn resolve(self, program_id: Pubkey) -> Pubkey {
        match self {
            OutputOwner::Oracle => program_id,
            OutputOwner::Address(owner) => owner,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NormalizedResult {
    Ok(Vec<NormalizedAccount>),
    Err,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedAccount {
    key: Pubkey,
    lamports: u64,
    owner: Pubkey,
    executable: bool,
    rent_epoch: u64,
    data: Vec<u8>,
}

fn v1_program_id() -> Pubkey {
    "HYLtHw8VKojJTZXzodkeaerYjK2um5bSrqyGwMYYTjNL"
        .parse()
        .unwrap()
}

fn v2_program_id() -> Pubkey {
    "5FGXfwXAgDDy76hUXWQEYdBF8ztPezhq7AwibdDtFWvs"
        .parse()
        .unwrap()
}

fn oracle_owner_sentinel() -> Pubkey {
    "EQSJ8wDNihTnzyBwu73sUNdo8fF5ZrsPkPHr2m9KpTMs"
        .parse()
        .unwrap()
}

fn token_program_id() -> Pubkey {
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        .parse()
        .unwrap()
}

fn token_2022_program_id() -> Pubkey {
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        .parse()
        .unwrap()
}

fn setup() -> std::path::PathBuf {
    static DEPLOY_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

    DEPLOY_DIR
        .get_or_init(|| {
            let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let deploy_dir = test_dir.join("target/deploy");
            build_program(
                test_dir
                    .join("programs/equivalence/spl/v1")
                    .to_str()
                    .unwrap(),
                deploy_dir.to_str().unwrap(),
            );
            build_program(
                test_dir
                    .join("programs/equivalence/spl/v2")
                    .to_str()
                    .unwrap(),
                deploy_dir.to_str().unwrap(),
            );
            deploy_dir
        })
        .clone()
}

fn run_tx(deploy_dir: &std::path::Path, version: OracleVersion, case: &Case) -> NormalizedResult {
    let mut svm = LiteSVM::new();
    let payer = keypair_for("spl-equivalence-payer");
    if svm.airdrop(&payer.pubkey(), case.payer_lamports).is_err() {
        return NormalizedResult::Err;
    }

    let program_id = match version {
        OracleVersion::V1 => {
            let id = v1_program_id();
            svm.add_program_from_file(id, deploy_dir.join("equivalence_spl_v1.so"))
                .expect("load v1 oracle");
            id
        }
        OracleVersion::V2 => {
            let id = v2_program_id();
            svm.add_program_from_file(id, deploy_dir.join("equivalence_spl_v2.so"))
                .expect("load v2 oracle");
            id
        }
    };

    let target = keypair_for("spl-equivalence-target").pubkey();
    let out = keypair_for("spl-equivalence-output").pubkey();

    if svm
        .set_account(
            target,
            Account {
                lamports: case.target.lamports,
                data: case.target.data.clone(),
                owner: case.target.owner,
                executable: case.target.executable,
                rent_epoch: case.target.rent_epoch,
            },
        )
        .is_err()
    {
        return NormalizedResult::Err;
    }
    if svm
        .set_account(
            out,
            Account {
                lamports: case.output.lamports,
                data: case.output.data.clone(),
                owner: case.output.owner.resolve(program_id),
                executable: case.output.executable,
                rent_epoch: case.output.rent_epoch,
            },
        )
        .is_err()
    {
        return NormalizedResult::Err;
    }

    let instruction_data = match version {
        OracleVersion::V1 => v1_instruction_data(case.operation),
        OracleVersion::V2 => vec![case.operation.v2_discriminator()],
    };
    let instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta {
                pubkey: target,
                is_signer: case.target_signer,
                is_writable: case.target_writable,
            },
            AccountMeta {
                pubkey: out,
                is_signer: case.output_signer,
                is_writable: case.output_writable,
            },
        ],
    );
    let blockhash = svm.latest_blockhash();
    let message = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let transaction =
        match VersionedTransaction::try_new(VersionedMessage::Legacy(message), &[&payer]) {
            Ok(transaction) => transaction,
            Err(_) => return NormalizedResult::Err,
        };

    match svm.send_transaction(transaction) {
        Ok(_) => NormalizedResult::Ok(snapshot_accounts(&svm, &[payer.pubkey(), target, out])),
        Err(_) => NormalizedResult::Err,
    }
}

fn snapshot_accounts(svm: &LiteSVM, keys: &[Pubkey]) -> Vec<NormalizedAccount> {
    keys.iter()
        .map(|key| {
            let account = svm.get_account(key).expect("snapshot account exists");
            NormalizedAccount {
                key: *key,
                lamports: account.lamports,
                owner: normalize_owner(account.owner),
                executable: account.executable,
                rent_epoch: account.rent_epoch,
                data: account.data,
            }
        })
        .collect()
}

fn normalize_owner(owner: Pubkey) -> Pubkey {
    if owner == v1_program_id() || owner == v2_program_id() {
        oracle_owner_sentinel()
    } else {
        owner
    }
}

fn v1_instruction_data(operation: Operation) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", operation.v1_name()).as_bytes());
    hasher.finalize()[..8].to_vec()
}

fn mint_case_strategy(
    operation: Operation,
    owner: impl Strategy<Value = Pubkey>,
) -> impl Strategy<Value = Case> {
    (
        Just(operation),
        payer_lamports_strategy(),
        account_case_strategy(owner, mint_data_strategy()),
        output_account_case_strategy(),
        signer_strategy(),
        writable_strategy(),
        signer_strategy(),
        writable_strategy(),
    )
        .prop_map(
            |(
                operation,
                payer_lamports,
                target,
                output,
                target_signer,
                target_writable,
                output_signer,
                output_writable,
            )| Case {
                operation,
                payer_lamports,
                target,
                output,
                target_signer,
                target_writable,
                output_signer,
                output_writable,
            },
        )
}

fn token_account_case_strategy(
    operation: Operation,
    owner: impl Strategy<Value = Pubkey>,
) -> impl Strategy<Value = Case> {
    (
        Just(operation),
        payer_lamports_strategy(),
        account_case_strategy(owner, token_account_data_strategy()),
        output_account_case_strategy(),
        signer_strategy(),
        writable_strategy(),
        signer_strategy(),
        writable_strategy(),
    )
        .prop_map(
            |(
                operation,
                payer_lamports,
                target,
                output,
                target_signer,
                target_writable,
                output_signer,
                output_writable,
            )| Case {
                operation,
                payer_lamports,
                target,
                output,
                target_signer,
                target_writable,
                output_signer,
                output_writable,
            },
        )
}

fn account_case_strategy(
    owner: impl Strategy<Value = Pubkey>,
    data: impl Strategy<Value = Vec<u8>>,
) -> impl Strategy<Value = AccountCase> {
    (
        account_lamports_strategy(),
        data,
        owner,
        executable_strategy(),
        rent_epoch_strategy(),
    )
        .prop_map(
            |(lamports, data, owner, executable, rent_epoch)| AccountCase {
                lamports,
                data,
                owner,
                executable,
                rent_epoch,
            },
        )
}

fn output_account_case_strategy() -> impl Strategy<Value = OutputAccountCase> {
    (
        account_lamports_strategy(),
        output_data_strategy(),
        output_owner_strategy(),
        executable_strategy(),
        rent_epoch_strategy(),
    )
        .prop_map(
            |(lamports, data, owner, executable, rent_epoch)| OutputAccountCase {
                lamports,
                data,
                owner,
                executable,
                rent_epoch,
            },
        )
}

fn payer_lamports_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        8 => Just(DEFAULT_PAYER_LAMPORTS),
        1 => 0u64..50_000,
        1 => 50_000u64..20_000_000_000,
    ]
}

fn account_lamports_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        6 => Just(DEFAULT_ACCOUNT_LAMPORTS),
        1 => Just(0),
        2 => 1u64..1_000_000_000_000,
    ]
}

fn output_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        6 => Just(vec![0; OBSERVATION_LEN]),
        4 => arbitrary_data_strategy(),
    ]
}

fn arbitrary_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        8 => prop::collection::vec(any::<u8>(), 0..MAX_SMALL_RANDOM_DATA_LEN),
        1 => prop::collection::vec(any::<u8>(), 0..MAX_LARGE_RANDOM_DATA_LEN),
    ]
}

fn output_owner_strategy() -> impl Strategy<Value = OutputOwner> {
    prop_oneof![
        6 => Just(OutputOwner::Oracle),
        1 => Just(OutputOwner::Address(token_program_id())),
        1 => Just(OutputOwner::Address(token_2022_program_id())),
        1 => any::<[u8; 32]>().prop_map(|owner| OutputOwner::Address(Pubkey::new_from_array(owner))),
    ]
}

fn executable_strategy() -> impl Strategy<Value = bool> {
    prop_oneof![
        8 => Just(false),
        1 => Just(true),
    ]
}

fn signer_strategy() -> impl Strategy<Value = bool> {
    prop_oneof![
        8 => Just(false),
        1 => Just(true),
    ]
}

fn writable_strategy() -> impl Strategy<Value = bool> {
    prop_oneof![
        8 => Just(true),
        1 => Just(false),
    ]
}

fn rent_epoch_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        6 => Just(0),
        1 => 1u64..1_000_000,
        1 => any::<u64>(),
    ]
}

fn strict_owner_strategy() -> impl Strategy<Value = Pubkey> {
    prop_oneof![
        6 => Just(token_program_id()),
        2 => Just(token_2022_program_id()),
        1 => any::<[u8; 32]>().prop_map(Pubkey::new_from_array),
    ]
}

fn interface_owner_strategy() -> impl Strategy<Value = Pubkey> {
    prop_oneof![
        4 => Just(token_program_id()),
        4 => Just(token_2022_program_id()),
        1 => any::<[u8; 32]>().prop_map(Pubkey::new_from_array),
    ]
}

fn mint_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        2 => arbitrary_data_strategy(),
        3 => prop::collection::vec(any::<u8>(), SplMint::LEN).prop_map(|mut data| {
            data[36 + 8 + 1] = 1;
            data
        }),
        2 => canonical_mint_data_strategy(),
        2 => token_2022_mint_data_strategy(),
        4 => malformed_mint_coption_data_strategy(),
    ]
}

fn token_account_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        2 => arbitrary_data_strategy(),
        3 => prop::collection::vec(any::<u8>(), SplTokenAccount::LEN).prop_map(|mut data| {
            data[32 + 32 + 8 + 4 + 32] = 1;
            data
        }),
        2 => canonical_token_account_data_strategy(),
        2 => token_2022_token_account_data_strategy(),
        4 => malformed_token_coption_data_strategy(),
    ]
}

fn canonical_mint_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    (
        any::<[u8; 32]>(),
        any::<u64>(),
        any::<u8>(),
        any::<bool>(),
        prop::option::of(any::<[u8; 32]>()),
    )
        .prop_map(
            |(authority, supply, decimals, has_freeze_authority, freeze_authority)| {
                let mut data = vec![0; SplMint::LEN];
                let mint = SplMint {
                    mint_authority: COption::Some(SplPubkey::new_from_array(authority)),
                    supply,
                    decimals,
                    is_initialized: true,
                    freeze_authority: if has_freeze_authority {
                        COption::Some(SplPubkey::new_from_array(
                            freeze_authority.unwrap_or([7; 32]),
                        ))
                    } else {
                        COption::None
                    },
                };
                SplMint::pack(mint, &mut data).unwrap();
                data
            },
        )
}

fn token_2022_mint_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        8 => (
            token_2022_mint_base_strategy(),
            prop::collection::vec(token_2022_mint_extension_strategy(), 1..6),
        )
            .prop_map(|(mint, extension_types)| token_2022_mint_data(mint, &extension_types)),
        1 => (
            token_2022_mint_base_strategy(),
            prop::collection::vec(any::<u8>(), 0..128),
        )
            .prop_map(|(mint, metadata)| token_2022_mint_token_metadata_data(mint, &metadata)),
    ]
}

fn canonical_token_account_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    (
        any::<[u8; 32]>(),
        any::<[u8; 32]>(),
        any::<u64>(),
        prop::option::of(any::<[u8; 32]>()),
        prop_oneof![
            Just(SplAccountState::Initialized),
            Just(SplAccountState::Frozen),
        ],
        prop::option::of(any::<u64>()),
        any::<u64>(),
        prop::option::of(any::<[u8; 32]>()),
    )
        .prop_map(
            |(
                mint,
                owner,
                amount,
                delegate,
                state,
                is_native,
                delegated_amount,
                close_authority,
            )| {
                let mut data = vec![0; SplTokenAccount::LEN];
                let account = SplTokenAccount {
                    mint: SplPubkey::new_from_array(mint),
                    owner: SplPubkey::new_from_array(owner),
                    amount,
                    delegate: delegate
                        .map(SplPubkey::new_from_array)
                        .map(COption::Some)
                        .unwrap_or(COption::None),
                    state,
                    is_native: is_native.map(COption::Some).unwrap_or(COption::None),
                    delegated_amount,
                    close_authority: close_authority
                        .map(SplPubkey::new_from_array)
                        .map(COption::Some)
                        .unwrap_or(COption::None),
                };
                SplTokenAccount::pack(account, &mut data).unwrap();
                data
            },
        )
}

fn token_2022_token_account_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    (
        token_2022_account_base_strategy(),
        prop::collection::vec(token_2022_account_extension_strategy(), 1..6),
    )
        .prop_map(|(account, extension_types)| {
            token_2022_token_account_data(account, &extension_types)
        })
}

fn token_2022_mint_base_strategy() -> impl Strategy<Value = Token2022Mint> {
    (
        prop::option::of(any::<[u8; 32]>()),
        any::<u64>(),
        any::<u8>(),
        prop::option::of(any::<[u8; 32]>()),
    )
        .prop_map(
            |(mint_authority, supply, decimals, freeze_authority)| Token2022Mint {
                mint_authority: mint_authority
                    .map(Pubkey::new_from_array)
                    .map(Token2022COption::Some)
                    .unwrap_or(Token2022COption::None),
                supply,
                decimals,
                is_initialized: true,
                freeze_authority: freeze_authority
                    .map(Pubkey::new_from_array)
                    .map(Token2022COption::Some)
                    .unwrap_or(Token2022COption::None),
            },
        )
}

fn token_2022_account_base_strategy() -> impl Strategy<Value = Token2022Account> {
    (
        any::<[u8; 32]>(),
        any::<[u8; 32]>(),
        any::<u64>(),
        prop::option::of(any::<[u8; 32]>()),
        prop_oneof![
            Just(Token2022AccountState::Initialized),
            Just(Token2022AccountState::Frozen),
        ],
        prop::option::of(any::<u64>()),
        any::<u64>(),
        prop::option::of(any::<[u8; 32]>()),
    )
        .prop_map(
            |(
                mint,
                owner,
                amount,
                delegate,
                state,
                is_native,
                delegated_amount,
                close_authority,
            )| Token2022Account {
                mint: Pubkey::new_from_array(mint),
                owner: Pubkey::new_from_array(owner),
                amount,
                delegate: delegate
                    .map(Pubkey::new_from_array)
                    .map(Token2022COption::Some)
                    .unwrap_or(Token2022COption::None),
                state,
                is_native: is_native
                    .map(Token2022COption::Some)
                    .unwrap_or(Token2022COption::None),
                delegated_amount,
                close_authority: close_authority
                    .map(Pubkey::new_from_array)
                    .map(Token2022COption::Some)
                    .unwrap_or(Token2022COption::None),
            },
        )
}

fn token_2022_mint_extension_strategy() -> impl Strategy<Value = Token2022ExtensionType> {
    prop_oneof![
        Just(Token2022ExtensionType::TransferFeeConfig),
        Just(Token2022ExtensionType::MintCloseAuthority),
        Just(Token2022ExtensionType::ConfidentialTransferMint),
        Just(Token2022ExtensionType::DefaultAccountState),
        Just(Token2022ExtensionType::NonTransferable),
        Just(Token2022ExtensionType::InterestBearingConfig),
        Just(Token2022ExtensionType::PermanentDelegate),
        Just(Token2022ExtensionType::TransferHook),
        Just(Token2022ExtensionType::ConfidentialTransferFeeConfig),
        Just(Token2022ExtensionType::MetadataPointer),
        Just(Token2022ExtensionType::GroupPointer),
        Just(Token2022ExtensionType::TokenGroup),
        Just(Token2022ExtensionType::GroupMemberPointer),
        Just(Token2022ExtensionType::TokenGroupMember),
        Just(Token2022ExtensionType::ConfidentialMintBurn),
        Just(Token2022ExtensionType::ScaledUiAmount),
        Just(Token2022ExtensionType::Pausable),
    ]
}

fn token_2022_account_extension_strategy() -> impl Strategy<Value = Token2022ExtensionType> {
    prop_oneof![
        Just(Token2022ExtensionType::TransferFeeAmount),
        Just(Token2022ExtensionType::ConfidentialTransferAccount),
        Just(Token2022ExtensionType::ImmutableOwner),
        Just(Token2022ExtensionType::MemoTransfer),
        Just(Token2022ExtensionType::CpiGuard),
        Just(Token2022ExtensionType::NonTransferableAccount),
        Just(Token2022ExtensionType::TransferHookAccount),
        Just(Token2022ExtensionType::ConfidentialTransferFeeAmount),
        Just(Token2022ExtensionType::PausableAccount),
    ]
}

fn token_2022_mint_data(
    mint: Token2022Mint,
    extension_types: &[Token2022ExtensionType],
) -> Vec<u8> {
    let len = Token2022ExtensionType::try_calculate_account_len::<Token2022Mint>(extension_types)
        .unwrap();
    let mut data = vec![0; len];
    Token2022Mint::pack(mint, &mut data[..<Token2022Mint as Token2022Pack>::LEN]).unwrap();
    spl_token_2022_interface::extension::set_account_type::<Token2022Mint>(&mut data).unwrap();

    let mut state = StateWithExtensionsMut::<Token2022Mint>::unpack(&mut data).unwrap();
    for extension_type in extension_types {
        match extension_type {
            Token2022ExtensionType::TransferFeeConfig => {
                state.init_extension::<TransferFeeConfig>(true).unwrap();
            }
            Token2022ExtensionType::MintCloseAuthority => {
                state.init_extension::<MintCloseAuthority>(true).unwrap();
            }
            Token2022ExtensionType::ConfidentialTransferMint => {
                state
                    .init_extension::<ConfidentialTransferMint>(true)
                    .unwrap();
            }
            Token2022ExtensionType::DefaultAccountState => {
                state.init_extension::<DefaultAccountState>(true).unwrap();
            }
            Token2022ExtensionType::NonTransferable => {
                state.init_extension::<NonTransferable>(true).unwrap();
            }
            Token2022ExtensionType::InterestBearingConfig => {
                state.init_extension::<InterestBearingConfig>(true).unwrap();
            }
            Token2022ExtensionType::PermanentDelegate => {
                state.init_extension::<PermanentDelegate>(true).unwrap();
            }
            Token2022ExtensionType::TransferHook => {
                state.init_extension::<TransferHook>(true).unwrap();
            }
            Token2022ExtensionType::ConfidentialTransferFeeConfig => {
                state
                    .init_extension::<ConfidentialTransferFeeConfig>(true)
                    .unwrap();
            }
            Token2022ExtensionType::MetadataPointer => {
                state.init_extension::<MetadataPointer>(true).unwrap();
            }
            Token2022ExtensionType::GroupPointer => {
                state.init_extension::<GroupPointer>(true).unwrap();
            }
            Token2022ExtensionType::TokenGroup => {
                state.init_extension::<TokenGroup>(true).unwrap();
            }
            Token2022ExtensionType::GroupMemberPointer => {
                state.init_extension::<GroupMemberPointer>(true).unwrap();
            }
            Token2022ExtensionType::TokenGroupMember => {
                state.init_extension::<TokenGroupMember>(true).unwrap();
            }
            Token2022ExtensionType::ConfidentialMintBurn => {
                state.init_extension::<ConfidentialMintBurn>(true).unwrap();
            }
            Token2022ExtensionType::ScaledUiAmount => {
                state.init_extension::<ScaledUiAmountConfig>(true).unwrap();
            }
            Token2022ExtensionType::Pausable => {
                state.init_extension::<PausableConfig>(true).unwrap();
            }
            _ => unreachable!(),
        }
    }

    data
}

fn token_2022_mint_token_metadata_data(mint: Token2022Mint, metadata: &[u8]) -> Vec<u8> {
    const BASE_ACCOUNT_LEN: usize = 165;
    const ACCOUNT_TYPE_LEN: usize = 1;
    const TLV_HEADER_LEN: usize = 4;

    let mut data = vec![0; BASE_ACCOUNT_LEN + ACCOUNT_TYPE_LEN + TLV_HEADER_LEN + metadata.len()];
    Token2022Mint::pack(mint, &mut data[..<Token2022Mint as Token2022Pack>::LEN]).unwrap();
    spl_token_2022_interface::extension::set_account_type::<Token2022Mint>(&mut data).unwrap();

    let tlv_offset = BASE_ACCOUNT_LEN + ACCOUNT_TYPE_LEN;
    data[tlv_offset..tlv_offset + 2]
        .copy_from_slice(&(Token2022ExtensionType::TokenMetadata as u16).to_le_bytes());
    data[tlv_offset + 2..tlv_offset + 4].copy_from_slice(&(metadata.len() as u16).to_le_bytes());
    data[tlv_offset + 4..].copy_from_slice(metadata);
    data
}

fn token_2022_token_account_data(
    account: Token2022Account,
    extension_types: &[Token2022ExtensionType],
) -> Vec<u8> {
    let len =
        Token2022ExtensionType::try_calculate_account_len::<Token2022Account>(extension_types)
            .unwrap();
    let mut data = vec![0; len];
    Token2022Account::pack(
        account,
        &mut data[..<Token2022Account as Token2022Pack>::LEN],
    )
    .unwrap();
    spl_token_2022_interface::extension::set_account_type::<Token2022Account>(&mut data).unwrap();

    let mut state = StateWithExtensionsMut::<Token2022Account>::unpack(&mut data).unwrap();
    for extension_type in extension_types {
        match extension_type {
            Token2022ExtensionType::TransferFeeAmount => {
                state.init_extension::<TransferFeeAmount>(true).unwrap();
            }
            Token2022ExtensionType::ConfidentialTransferAccount => {
                state
                    .init_extension::<ConfidentialTransferAccount>(true)
                    .unwrap();
            }
            Token2022ExtensionType::ImmutableOwner => {
                state.init_extension::<ImmutableOwner>(true).unwrap();
            }
            Token2022ExtensionType::MemoTransfer => {
                state.init_extension::<MemoTransfer>(true).unwrap();
            }
            Token2022ExtensionType::CpiGuard => {
                state.init_extension::<CpiGuard>(true).unwrap();
            }
            Token2022ExtensionType::NonTransferableAccount => {
                state
                    .init_extension::<NonTransferableAccount>(true)
                    .unwrap();
            }
            Token2022ExtensionType::TransferHookAccount => {
                state.init_extension::<TransferHookAccount>(true).unwrap();
            }
            Token2022ExtensionType::ConfidentialTransferFeeAmount => {
                state
                    .init_extension::<ConfidentialTransferFeeAmount>(true)
                    .unwrap();
            }
            Token2022ExtensionType::PausableAccount => {
                state.init_extension::<PausableAccount>(true).unwrap();
            }
            _ => unreachable!(),
        }
    }

    data
}

fn malformed_mint_coption_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    (0usize..2, invalid_coption_tag_strategy()).prop_map(|(which, tag)| {
        let mut data = vec![0; SplMint::LEN];
        data[36 + 8 + 1] = 1;
        let offset = if which == 0 { 0 } else { 36 + 8 + 1 + 1 };
        data[offset..offset + 4].copy_from_slice(&tag);
        data
    })
}

fn malformed_token_coption_data_strategy() -> impl Strategy<Value = Vec<u8>> {
    (0usize..3, invalid_coption_tag_strategy()).prop_map(|(which, tag)| {
        let mut data = vec![0; SplTokenAccount::LEN];
        data[32 + 32 + 8 + 4 + 32] = 1;
        let offset = match which {
            0 => 32 + 32 + 8,
            1 => 32 + 32 + 8 + 4 + 32 + 1,
            _ => 32 + 32 + 8 + 4 + 32 + 1 + 4 + 8 + 8,
        };
        data[offset..offset + 4].copy_from_slice(&tag);
        data
    })
}

fn invalid_coption_tag_strategy() -> impl Strategy<Value = [u8; 4]> {
    prop_oneof![
        (2u8..=255, any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(a, b, c, d)| [a, b, c, d]),
        (Just(1u8), 1u8..=255, any::<u8>(), any::<u8>()).prop_map(|(a, b, c, d)| [a, b, c, d]),
        (Just(0u8), 1u8..=255, any::<u8>(), any::<u8>()).prop_map(|(a, b, c, d)| [a, b, c, d]),
    ]
}

fn assert_v1_v2_equivalent(deploy_dir: &std::path::Path, case: &Case) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        run_tx(deploy_dir, OracleVersion::V1, case),
        run_tx(deploy_dir, OracleVersion::V2, case),
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32 * 1024,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    #[ignore = "runs a 32k-case fuzz-style SPL v1/v2 equivalence pass; run explicitly with `-- --ignored`"]
    fn spl_v1_v2_strict_mint_has_equivalent_tx_results(
        case in mint_case_strategy(Operation::StrictMint, strict_owner_strategy()),
    ) {
        let deploy_dir = setup();
        assert_v1_v2_equivalent(&deploy_dir, &case)?;
    }

    #[test]
    #[ignore = "runs a 32k-case fuzz-style SPL v1/v2 equivalence pass; run explicitly with `-- --ignored`"]
    fn spl_v1_v2_strict_token_account_has_equivalent_tx_results(
        case in token_account_case_strategy(Operation::StrictTokenAccount, strict_owner_strategy()),
    ) {
        let deploy_dir = setup();
        assert_v1_v2_equivalent(&deploy_dir, &case)?;
    }

    #[test]
    #[ignore = "runs a 32k-case fuzz-style SPL v1/v2 equivalence pass; run explicitly with `-- --ignored`"]
    fn spl_v1_v2_interface_mint_has_equivalent_tx_results(
        case in mint_case_strategy(Operation::InterfaceMint, interface_owner_strategy()),
    ) {
        let deploy_dir = setup();
        assert_v1_v2_equivalent(&deploy_dir, &case)?;
    }

    #[test]
    #[ignore = "runs a 32k-case fuzz-style SPL v1/v2 equivalence pass; run explicitly with `-- --ignored`"]
    fn spl_v1_v2_interface_token_account_has_equivalent_tx_results(
        case in token_account_case_strategy(Operation::InterfaceTokenAccount, interface_owner_strategy()),
    ) {
        let deploy_dir = setup();
        assert_v1_v2_equivalent(&deploy_dir, &case)?;
    }
}
