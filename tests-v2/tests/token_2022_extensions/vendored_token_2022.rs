use {
    super::common::*,
    sha2::{Digest, Sha256},
    solana_loader_v3_interface::get_program_data_address,
    solana_rpc_client::rpc_client::RpcClient,
};

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
