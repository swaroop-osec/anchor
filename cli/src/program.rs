use anchor_lang::idl::IdlAccount;
use anchor_lang_idl::types::Idl;
use anyhow::{anyhow, bail, Result};
use solana_client::send_and_confirm_transactions_in_parallel::{
    send_and_confirm_transactions_in_parallel_blocking_v2, SendAndConfirmConfigV2,
};
use solana_loader_v3_interface::{
    instruction as loader_v3_instruction, state::UpgradeableLoaderState,
};
use solana_packet::PACKET_DATA_SIZE;
use solana_rpc_client::rpc_client::RpcClient;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::Signer,
    signer::{keypair::Keypair, EncodableKey},
    transaction::Transaction,
};
use solana_sdk_ids::bpf_loader_upgradeable as bpf_loader_upgradeable_id;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

use crate::{
    config::{Config, WithPath},
    ConfigOverride, ProgramCommand,
};

/// Parse priority fee from solana args
fn parse_priority_fee_from_args(args: &[String]) -> Option<u64> {
    args.windows(2)
        .find(|pair| pair[0] == "--with-compute-unit-price")
        .and_then(|pair| pair[1].parse().ok())
}

// Main entry point for all program commands
pub fn program(cfg_override: &ConfigOverride, cmd: ProgramCommand) -> Result<()> {
    match cmd {
        ProgramCommand::Deploy {
            program_filepath,
            program_keypair,
            upgrade_authority,
            program_id,
            buffer,
            max_len,
            no_idl,
            solana_args,
        } => program_deploy(
            cfg_override,
            program_filepath,
            program_keypair,
            upgrade_authority,
            program_id,
            buffer,
            max_len,
            no_idl,
            solana_args,
        ),
        ProgramCommand::WriteBuffer {
            program_filepath,
            buffer,
            buffer_authority,
            max_len,
        } => program_write_buffer(
            cfg_override,
            program_filepath,
            buffer,
            buffer_authority,
            max_len,
        ),
        ProgramCommand::SetBufferAuthority {
            buffer,
            new_buffer_authority,
        } => program_set_buffer_authority(cfg_override, buffer, new_buffer_authority),
        ProgramCommand::SetUpgradeAuthority {
            program_id,
            new_upgrade_authority,
            new_upgrade_authority_signer,
            skip_new_upgrade_authority_signer_check,
            make_final,
            upgrade_authority,
        } => program_set_upgrade_authority(
            cfg_override,
            program_id,
            new_upgrade_authority,
            new_upgrade_authority_signer,
            skip_new_upgrade_authority_signer_check,
            make_final,
            upgrade_authority,
        ),
        ProgramCommand::Show {
            account,
            get_programs,
            get_buffers,
            all,
        } => program_show(cfg_override, account, get_programs, get_buffers, all),
        ProgramCommand::Upgrade {
            program_id,
            buffer,
            upgrade_authority,
        } => program_upgrade(cfg_override, program_id, buffer, upgrade_authority),
        ProgramCommand::Dump {
            account,
            output_file,
        } => program_dump(cfg_override, account, output_file),
        ProgramCommand::Close {
            account,
            authority,
            recipient,
            bypass_warning,
        } => program_close(cfg_override, account, authority, recipient, bypass_warning),
        ProgramCommand::Extend {
            program_id,
            additional_bytes,
        } => program_extend(cfg_override, program_id, additional_bytes),
    }
}

fn get_rpc_client_and_config(
    cfg_override: &ConfigOverride,
) -> Result<(RpcClient, WithPath<Config>)> {
    let config = Config::discover(cfg_override)?.ok_or_else(|| {
        anyhow!(
            "Not in anchor workspace. Run `anchor init` or provide cluster with --provider.cluster"
        )
    })?;

    let url = config.provider.cluster.url().to_string();
    let rpc_client = RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());

    Ok((rpc_client, config))
}

#[allow(clippy::too_many_arguments)]
fn program_deploy(
    cfg_override: &ConfigOverride,
    program_filepath: String,
    program_keypair: Option<String>,
    upgrade_authority: Option<String>,
    program_id: Option<Pubkey>,
    buffer: Option<Pubkey>,
    max_len: Option<usize>,
    no_idl: bool,
    solana_args: Vec<String>,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    // Augment solana_args with recommended defaults (priority fees, max sign attempts, buffer)
    let solana_args = crate::add_recommended_deployment_solana_args(&rpc_client, solana_args)?;

    // Parse priority fee from solana_args
    let priority_fee = parse_priority_fee_from_args(&solana_args);

    // Read program data
    let program_data = fs::read(&program_filepath)
        .map_err(|e| anyhow!("Failed to read program file {}: {}", program_filepath, e))?;

    // Determine program keypair
    let loaded_program_keypair = if let Some(keypair_path) = program_keypair {
        // Load from specified keypair file
        Keypair::read_from_file(&keypair_path).map_err(|e| {
            anyhow!(
                "Failed to read program keypair from {}: {}",
                keypair_path,
                e
            )
        })?
    } else if let Some(_program_id) = program_id {
        return Err(anyhow!(
            "When --program-id is specified, --program-keypair must also be provided"
        ));
    } else {
        // Auto-detect from target/deploy/{program_name}-keypair.json
        let program_name = Path::new(&program_filepath)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid program filepath"))?;

        let keypair_path = format!("target/deploy/{}-keypair.json", program_name);
        Keypair::read_from_file(&keypair_path).map_err(|e| {
            anyhow!(
                "Failed to read program keypair from {}: {}. \
                Use --program-keypair to specify a custom location.",
                keypair_path,
                e
            )
        })?
    };

    let program_id = loaded_program_keypair.pubkey();

    // Determine upgrade authority
    let upgrade_authority = if let Some(auth_path) = upgrade_authority {
        let authority_keypair = Keypair::read_from_file(&auth_path)
            .map_err(|e| anyhow!("Failed to read upgrade authority keypair: {}", e))?;
        println!(
            "Using custom upgrade authority: {}",
            authority_keypair.pubkey()
        );
        authority_keypair
    } else {
        payer.insecure_clone()
    };

    // Check if program already exists
    let program_account = rpc_client.get_account(&program_id);

    if program_account.is_ok() {
        // Program exists - validate it can be upgraded BEFORE writing buffer
        println!("Program already exists, upgrading...");

        // Verify program can be upgraded before doing expensive buffer write
        verify_program_can_be_upgraded(&rpc_client, &program_id, &upgrade_authority)?;

        // Write to buffer
        let buffer_pubkey = if let Some(buffer) = buffer {
            buffer
        } else {
            let buffer_keypair = Keypair::new();
            write_program_buffer(
                &rpc_client,
                &payer,
                &program_data,
                &upgrade_authority.pubkey(),
                &buffer_keypair,
                max_len,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
                    encoding: None,
                    max_retries: None,
                    min_context_slot: None,
                },
            )?
        };

        // Upgrade the program
        upgrade_program(
            &rpc_client,
            &payer,
            &program_id,
            &buffer_pubkey,
            &upgrade_authority,
            priority_fee,
        )?;
    } else {
        // New deployment

        let buffer_pubkey = if let Some(buffer) = buffer {
            buffer
        } else {
            let buffer_keypair = Keypair::new();
            write_program_buffer(
                &rpc_client,
                &payer,
                &program_data,
                &upgrade_authority.pubkey(),
                &buffer_keypair,
                max_len,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
                    encoding: None,
                    max_retries: None,
                    min_context_slot: None,
                },
            )?
        };

        // Deploy from buffer
        let max_data_len = max_len.unwrap_or(program_data.len());
        deploy_program(
            &rpc_client,
            &payer,
            &buffer_pubkey,
            &loaded_program_keypair,
            &upgrade_authority,
            max_data_len,
            priority_fee,
        )?;
    }

    // Print the program ID
    println!("Program ID: {}", program_id);

    // Deploy IDL if not skipped
    if !no_idl {
        // Extract program name from filepath
        let program_name = Path::new(&program_filepath)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid program filepath"))?;

        // Look for IDL file in target/idl/{program_name}.json
        let idl_filepath = format!("target/idl/{}.json", program_name);

        if Path::new(&idl_filepath).exists() {
            println!("Deploying IDL...");

            // Read and update the IDL with the program address
            let idl_content = fs::read_to_string(&idl_filepath)
                .map_err(|e| anyhow!("Failed to read IDL file {}: {}", idl_filepath, e))?;

            let mut idl: Idl = serde_json::from_str(&idl_content)
                .map_err(|e| anyhow!("Failed to parse IDL file {}: {}", idl_filepath, e))?;

            // Update the IDL with the program address
            idl.address = program_id.to_string();

            // Write the updated IDL back to the file
            let idl_json = serde_json::to_string_pretty(&idl)
                .map_err(|e| anyhow!("Failed to serialize IDL: {}", e))?;
            fs::write(&idl_filepath, idl_json)
                .map_err(|e| anyhow!("Failed to write IDL file {}: {}", idl_filepath, e))?;

            // Wait for the program to be confirmed before initializing IDL to prevent
            // race condition where the program isn't yet available in validator cache
            let max_retries = 5;
            let retry_delay = Duration::from_millis(500);
            let cache_delay = Duration::from_secs(2);

            for attempt in 0..max_retries {
                if let Ok(account) = rpc_client.get_account(&program_id) {
                    if account.executable {
                        thread::sleep(cache_delay);
                        break;
                    }
                }

                if attempt == max_retries - 1 {
                    println!("Failed");
                    return Err(anyhow!(
                        "Timeout waiting for program {} to be confirmed",
                        program_id
                    ));
                }

                thread::sleep(retry_delay);
            }

            // Check if IDL account already exists
            let idl_address = IdlAccount::address(&program_id);
            let idl_account_exists = rpc_client.get_account(&idl_address).is_ok();

            if idl_account_exists {
                // IDL account exists, upgrade it
                crate::idl_upgrade(cfg_override, program_id, idl_filepath, None)?;
            } else {
                // IDL account doesn't exist, create it
                crate::idl_init(cfg_override, program_id, idl_filepath, None)?;
            }

            println!("✓ Idl account created: {}", idl_address);
        } else {
            println!(
                "Warning: IDL file not found at {}, skipping IDL deployment",
                idl_filepath
            );
        }
    }

    Ok(())
}

/// Verify that a buffer account is valid for upgrading
fn verify_buffer_account(
    rpc_client: &RpcClient,
    buffer_pubkey: &Pubkey,
    buffer_authority: &Pubkey,
) -> Result<()> {
    let buffer_account = rpc_client
        .get_account(buffer_pubkey)
        .map_err(|e| anyhow!("Buffer account {} not found: {}", buffer_pubkey, e))?;

    // Check if buffer is owned by BPF Upgradeable Loader
    if buffer_account.owner != bpf_loader_upgradeable_id::id() {
        return Err(anyhow!(
            "Buffer account {} is not owned by the BPF Upgradeable Loader",
            buffer_pubkey
        ));
    }

    // Verify it's actually a Buffer account
    match bincode::deserialize::<UpgradeableLoaderState>(&buffer_account.data) {
        Ok(UpgradeableLoaderState::Buffer { authority_address }) => {
            // Check if buffer is immutable
            if authority_address.is_none() {
                return Err(anyhow!("Buffer {} is immutable", buffer_pubkey));
            }
            // Verify the authority matches
            if authority_address != Some(*buffer_authority) {
                return Err(anyhow!(
                    "Buffer's authority {:?} does not match authority provided {}",
                    authority_address,
                    buffer_authority
                ));
            }
        }
        Ok(_) => {
            return Err(anyhow!("Account {} is not a Buffer account", buffer_pubkey));
        }
        Err(e) => {
            return Err(anyhow!(
                "Failed to deserialize buffer account {}: {}",
                buffer_pubkey,
                e
            ));
        }
    }

    Ok(())
}

/// Verify that a program exists, is upgradeable, and the authority matches
/// This should be called BEFORE doing expensive operations like buffer writes
fn verify_program_can_be_upgraded(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    upgrade_authority: &Keypair,
) -> Result<()> {
    // Verify the program exists
    let program_account = rpc_client
        .get_account(program_id)
        .map_err(|e| anyhow!("Failed to get program account {}: {}", program_id, e))?;

    if program_account.owner != bpf_loader_upgradeable_id::id() {
        return Err(anyhow!(
            "Program {} is not an upgradeable program",
            program_id
        ));
    }

    // Check if this is a valid program and get the ProgramData address
    let programdata_address =
        match bincode::deserialize::<UpgradeableLoaderState>(&program_account.data) {
            Ok(UpgradeableLoaderState::Program {
                programdata_address,
            }) => programdata_address,
            _ => {
                return Err(anyhow!(
                    "{} is not an upgradeable program account",
                    program_id
                ));
            }
        };

    // Verify the ProgramData account exists and is valid
    let programdata_account = rpc_client.get_account(&programdata_address).map_err(|e| {
        anyhow!(
            "Failed to get ProgramData account: {}. The program may have been closed.",
            e
        )
    })?;

    // Verify it's a valid ProgramData account
    match bincode::deserialize::<UpgradeableLoaderState>(&programdata_account.data) {
        Ok(UpgradeableLoaderState::ProgramData {
            upgrade_authority_address,
            ..
        }) => {
            // Check if the program is immutable
            if upgrade_authority_address.is_none() {
                return Err(anyhow!(
                    "Program {} is immutable and cannot be upgraded",
                    program_id
                ));
            }
            // Verify the authority matches
            if upgrade_authority_address != Some(upgrade_authority.pubkey()) {
                return Err(anyhow!(
                    "Upgrade authority mismatch. Expected {:?}, but ProgramData has {:?}",
                    Some(upgrade_authority.pubkey()),
                    upgrade_authority_address
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "Program {} has been closed or is in an invalid state",
                program_id
            ));
        }
    }

    Ok(())
}

#[allow(deprecated)]
fn deploy_program(
    rpc_client: &RpcClient,
    payer: &Keypair,
    buffer: &Pubkey,
    program_keypair: &Keypair,
    upgrade_authority: &Keypair,
    max_data_len: usize,
    priority_fee: Option<u64>,
) -> Result<()> {
    println!("Deploying program from buffer...");

    let program_id = program_keypair.pubkey();
    let mut deploy_ixs = loader_v3_instruction::deploy_with_max_program_len(
        &payer.pubkey(),
        &program_id,
        buffer,
        &upgrade_authority.pubkey(),
        rpc_client
            .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::size_of_program())?,
        max_data_len,
    )
    .map_err(|e| anyhow!("Failed to create deploy instruction: {}", e))?;

    // Add priority fee if specified
    deploy_ixs = crate::prepend_compute_unit_ix(deploy_ixs, rpc_client, priority_fee)?;

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let deploy_tx = Transaction::new_signed_with_payer(
        &deploy_ixs,
        Some(&payer.pubkey()),
        &[payer, program_keypair, upgrade_authority],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&deploy_tx)
        .map_err(|e| anyhow!("Failed to deploy program: {}", e))?;

    println!("Program deployed successfully!");
    Ok(())
}

fn upgrade_program(
    rpc_client: &RpcClient,
    payer: &Keypair,
    program_id: &Pubkey,
    buffer: &Pubkey,
    upgrade_authority: &Keypair,
    priority_fee: Option<u64>,
) -> Result<()> {
    // Verify the program can be upgraded
    // Note: This may be redundant if called from program_deploy,
    // but necessary when called directly from program_upgrade command
    verify_program_can_be_upgraded(rpc_client, program_id, upgrade_authority)?;

    // Verify the buffer account is valid
    verify_buffer_account(rpc_client, buffer, &upgrade_authority.pubkey())?;

    println!("Sending upgrade transaction...");

    let upgrade_ix = loader_v3_instruction::upgrade(
        program_id,
        buffer,
        &upgrade_authority.pubkey(),
        &payer.pubkey(),
    );

    // Add priority fee if specified
    let upgrade_ixs = crate::prepend_compute_unit_ix(vec![upgrade_ix], rpc_client, priority_fee)?;

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let upgrade_tx = Transaction::new_signed_with_payer(
        &upgrade_ixs,
        Some(&payer.pubkey()),
        &[payer, upgrade_authority],
        recent_blockhash,
    );

    let signature = rpc_client
        .send_and_confirm_transaction(&upgrade_tx)
        .map_err(|e| anyhow!("Failed to upgrade program: {}", e))?;
    println!("Program upgraded successfully!");
    println!("Signature: {}", signature);
    Ok(())
}

fn program_write_buffer(
    cfg_override: &ConfigOverride,
    program_filepath: String,
    _buffer: Option<String>,
    buffer_authority: Option<String>,
    max_len: Option<usize>,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    // Read program data
    let program_data = fs::read(&program_filepath)
        .map_err(|e| anyhow!("Failed to read program file {}: {}", program_filepath, e))?;

    // Determine buffer authority
    let buffer_authority_keypair = if let Some(auth_path) = buffer_authority {
        Keypair::read_from_file(&auth_path)
            .map_err(|e| anyhow!("Failed to read buffer authority keypair: {}", e))?
    } else {
        payer.insecure_clone()
    };

    let buffer_keypair = Keypair::new();
    let buffer_pubkey = write_program_buffer(
        &rpc_client,
        &payer,
        &program_data,
        &buffer_authority_keypair.pubkey(),
        &buffer_keypair,
        max_len,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        },
    )?;

    println!("Buffer: {}", buffer_pubkey);
    Ok(())
}

fn program_set_buffer_authority(
    cfg_override: &ConfigOverride,
    buffer: Pubkey,
    new_buffer_authority: Pubkey,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    println!("Setting buffer authority...");
    println!("Buffer: {}", buffer);
    println!("New authority: {}", new_buffer_authority);

    let set_authority_ixs = loader_v3_instruction::set_buffer_authority(
        &buffer,
        &payer.pubkey(),
        &new_buffer_authority,
    );

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ixs],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow!("Failed to set buffer authority: {}", e))?;

    println!("Buffer authority updated successfully!");
    Ok(())
}

fn program_set_upgrade_authority(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    new_upgrade_authority: Option<Pubkey>,
    new_upgrade_authority_signer: Option<String>,
    skip_new_upgrade_authority_signer_check: bool,
    make_final: bool,
    current_upgrade_authority: Option<String>,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    // Validate that this is a Program account, not ProgramData
    let program_account = rpc_client
        .get_account(&program_id)
        .map_err(|e| anyhow!("Failed to get account {}: {}", program_id, e))?;

    if program_account.owner != bpf_loader_upgradeable_id::id() {
        return Err(anyhow!(
            "Account {} is not owned by the BPF Upgradeable Loader",
            program_id
        ));
    }

    // Ensure this is a Program account, not ProgramData or Buffer
    match bincode::deserialize::<UpgradeableLoaderState>(&program_account.data) {
        Ok(UpgradeableLoaderState::Program { .. }) => {
            // Valid program account
        }
        Ok(UpgradeableLoaderState::ProgramData { .. }) => {
            return Err(anyhow!(
                "Error: {} is a ProgramData account, not a Program account.\n\n\
                To set the upgrade authority, you must provide the Program ID, not the ProgramData address.\n\
                Use 'anchor program show {}' to find the associated Program ID.",
                program_id,
                program_id
            ));
        }
        Ok(UpgradeableLoaderState::Buffer { .. }) => {
            return Err(anyhow!(
                "{} is a Buffer account, not a Program account. Use set-buffer-authority for buffers.",
                program_id
            ));
        }
        _ => {
            return Err(anyhow!("{} is not a valid upgradeable program", program_id));
        }
    }

    println!("Setting upgrade authority...");
    println!("Program ID: {}", program_id);

    if make_final {
        println!("Making program immutable (cannot be upgraded)");
    } else if let Some(new_auth) = new_upgrade_authority {
        println!("New upgrade authority: {}", new_auth);
    } else {
        bail!("Must provide either --new-upgrade-authority or --final");
    }

    // Determine current authority keypair (must be a signer)
    let current_authority_keypair = if let Some(auth_path) = current_upgrade_authority {
        let keypair = Keypair::read_from_file(&auth_path)
            .map_err(|e| anyhow!("Failed to read current upgrade authority keypair: {}", e))?;
        println!("Using custom current authority: {}", keypair.pubkey());
        keypair
    } else {
        payer.insecure_clone()
    };

    // Validate signer requirements and load keypair
    let new_auth_keypair_opt = if let Some(signer_path) = new_upgrade_authority_signer {
        // Signer provided - use checked mode
        let keypair = Keypair::read_from_file(&signer_path)
            .map_err(|e| anyhow!("Failed to read new upgrade authority signer keypair: {}", e))?;

        // Verify the pubkey matches if both are provided
        if let Some(pubkey) = new_upgrade_authority {
            if pubkey != keypair.pubkey() {
                return Err(anyhow!(
                    "New upgrade authority pubkey mismatch: --new-upgrade-authority ({}) \
                    doesn't match --new-upgrade-authority-signer keypair ({})",
                    pubkey,
                    keypair.pubkey()
                ));
            }
        }

        println!("Using CHECKED mode - both current and new authority will sign (recommended)");
        Some(keypair)
    } else if new_upgrade_authority.is_some() && !make_final {
        // No signer provided but new authority specified
        if skip_new_upgrade_authority_signer_check {
            // User explicitly allowed unchecked mode
            println!("WARNING: Using UNCHECKED mode - only current authority will sign");
            println!("         This is less safe! The new authority won't verify ownership.");
            None
        } else {
            // By default, require the signer for safety
            return Err(anyhow!(
                "New upgrade authority signer is required for safety.\n\
                Please provide --new-upgrade-authority-signer <KEYPAIR_FILE> (recommended),\n\
                or use --skip-new-upgrade-authority-signer-check if you're confident the pubkey is correct."
            ));
        }
    } else {
        // Making program final or no new authority - no signer needed
        None
    };

    // Build instruction based on mode
    let set_authority_ixs = if let Some(ref new_auth_keypair) = new_auth_keypair_opt {
        // Checked mode: both current and new authority sign (safer)
        loader_v3_instruction::set_upgrade_authority_checked(
            &program_id,
            &current_authority_keypair.pubkey(),
            &new_auth_keypair.pubkey(),
        )
    } else {
        // Unchecked mode or final mode: only current authority signs
        loader_v3_instruction::set_upgrade_authority(
            &program_id,
            &current_authority_keypair.pubkey(),
            new_upgrade_authority.as_ref(),
        )
    };

    let recent_blockhash = rpc_client.get_latest_blockhash()?;

    let signature = if let Some(ref new_auth_keypair) = new_auth_keypair_opt {
        // Checked mode with 3 signers
        let tx = Transaction::new_signed_with_payer(
            &[set_authority_ixs],
            Some(&payer.pubkey()),
            &[&payer, &current_authority_keypair, new_auth_keypair],
            recent_blockhash,
        );
        rpc_client
            .send_and_confirm_transaction(&tx)
            .map_err(|e| anyhow!("Failed to set upgrade authority: {}", e))?
    } else {
        // Unchecked mode or final mode with 2 signers
        let tx = Transaction::new_signed_with_payer(
            &[set_authority_ixs],
            Some(&payer.pubkey()),
            &[&payer, &current_authority_keypair],
            recent_blockhash,
        );
        rpc_client
            .send_and_confirm_transaction(&tx)
            .map_err(|e| anyhow!("Failed to set upgrade authority: {}", e))?
    };

    println!();
    println!("Upgrade authority updated successfully!");
    println!("Signature: {}", signature);
    Ok(())
}

fn program_show(
    cfg_override: &ConfigOverride,
    account: Pubkey,
    _get_programs: bool,
    _get_buffers: bool,
    _all: bool,
) -> Result<()> {
    let (rpc_client, _config) = get_rpc_client_and_config(cfg_override)?;

    let account_data = rpc_client
        .get_account(&account)
        .map_err(|e| anyhow!("Failed to get account {}: {}", account, e))?;

    println!("Account: {}", account);
    println!("Owner: {}", account_data.owner);
    println!("Balance: {} lamports", account_data.lamports);
    println!("Data length: {} bytes", account_data.data.len());
    println!("Executable: {}", account_data.executable);

    // Try to parse as upgradeable loader state
    if account_data.owner == bpf_loader_upgradeable_id::id() {
        match bincode::deserialize::<UpgradeableLoaderState>(&account_data.data) {
            Ok(state) => match state {
                UpgradeableLoaderState::Uninitialized => {
                    println!("Type: Uninitialized");
                }
                UpgradeableLoaderState::Buffer { authority_address } => {
                    println!("Type: Buffer");
                    if let Some(authority) = authority_address {
                        println!("Authority: {}", authority);
                    } else {
                        println!("Authority: None (immutable)");
                    }
                }
                UpgradeableLoaderState::Program {
                    programdata_address,
                } => {
                    println!("Type: Program");
                    println!("Program Data Address: {}", programdata_address);

                    // Fetch program data account
                    if let Ok(programdata_account) = rpc_client.get_account(&programdata_address) {
                        if let Ok(UpgradeableLoaderState::ProgramData {
                            slot,
                            upgrade_authority_address,
                        }) = bincode::deserialize::<UpgradeableLoaderState>(
                            &programdata_account.data,
                        ) {
                            println!("Slot: {}", slot);
                            if let Some(authority) = upgrade_authority_address {
                                println!("Upgrade Authority: {}", authority);
                            } else {
                                println!("Upgrade Authority: None (immutable)");
                            }
                        }
                    }
                }
                UpgradeableLoaderState::ProgramData {
                    slot,
                    upgrade_authority_address,
                } => {
                    println!("Type: Program Data");
                    println!("Slot: {}", slot);
                    if let Some(authority) = upgrade_authority_address {
                        println!("Upgrade Authority: {}", authority);
                    } else {
                        println!("Upgrade Authority: None (immutable)");
                    }
                }
            },
            Err(e) => {
                println!("Failed to parse as upgradeable loader state: {}", e);
            }
        }
    }

    Ok(())
}

fn program_upgrade(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    buffer: Pubkey,
    upgrade_authority: Option<String>,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    // Determine upgrade authority
    let upgrade_authority_keypair = if let Some(auth_path) = upgrade_authority {
        let keypair = Keypair::read_from_file(&auth_path)
            .map_err(|e| anyhow!("Failed to read upgrade authority keypair: {}", e))?;
        println!("Using custom upgrade authority: {}", keypair.pubkey());
        keypair
    } else {
        payer.insecure_clone()
    };

    upgrade_program(
        &rpc_client,
        &payer,
        &program_id,
        &buffer,
        &upgrade_authority_keypair,
        None, // No priority fee specified for standalone upgrade command
    )?;

    Ok(())
}

fn program_dump(cfg_override: &ConfigOverride, account: Pubkey, output_file: String) -> Result<()> {
    let (rpc_client, _config) = get_rpc_client_and_config(cfg_override)?;

    println!("Fetching program data...");

    let account_data = rpc_client
        .get_account(&account)
        .map_err(|e| anyhow!("Failed to get account {}: {}", account, e))?;

    // Check if this is a program account
    let program_data = if account_data.owner == bpf_loader_upgradeable_id::id() {
        match bincode::deserialize::<UpgradeableLoaderState>(&account_data.data) {
            Ok(UpgradeableLoaderState::Program {
                programdata_address,
            }) => {
                // Fetch the program data account
                let programdata_account = rpc_client
                    .get_account(&programdata_address)
                    .map_err(|e| anyhow!("Failed to get program data account: {}", e))?;

                // Skip the UpgradeableLoaderState header
                let data_offset = UpgradeableLoaderState::size_of_programdata_metadata();
                programdata_account.data[data_offset..].to_vec()
            }
            Ok(UpgradeableLoaderState::Buffer { .. }) => {
                // Buffer account - skip the header
                let data_offset = UpgradeableLoaderState::size_of_buffer_metadata();
                account_data.data[data_offset..].to_vec()
            }
            Ok(UpgradeableLoaderState::ProgramData { .. }) => {
                // Program data account - skip the header
                let data_offset = UpgradeableLoaderState::size_of_programdata_metadata();
                account_data.data[data_offset..].to_vec()
            }
            _ => account_data.data,
        }
    } else {
        // Regular program or other account
        account_data.data
    };

    println!("Writing {} bytes to {}...", program_data.len(), output_file);

    let mut file =
        File::create(&output_file).map_err(|e| anyhow!("Failed to create output file: {}", e))?;

    file.write_all(&program_data)
        .map_err(|e| anyhow!("Failed to write program data: {}", e))?;

    println!("Program dumped successfully!");
    Ok(())
}

fn program_close(
    cfg_override: &ConfigOverride,
    account: Pubkey,
    authority: Option<String>,
    recipient: Option<Pubkey>,
    bypass_warning: bool,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    // Fetch the account to determine its type
    let account_data = rpc_client
        .get_account(&account)
        .map_err(|e| anyhow!("Failed to get account {}: {}", account, e))?;

    // Check if this is a BPF Loader Upgradeable account
    if account_data.owner != bpf_loader_upgradeable_id::id() {
        return Err(anyhow!(
            "Account {} is not owned by the BPF Loader Upgradeable program",
            account
        ));
    }

    // Determine which account to actually close
    let (account_to_close, account_type, program_account) =
        match bincode::deserialize::<UpgradeableLoaderState>(&account_data.data) {
            Ok(UpgradeableLoaderState::Program {
                programdata_address,
            }) => (programdata_address, "ProgramData", Some(account)),
            Ok(UpgradeableLoaderState::Buffer { .. }) => (account, "Buffer", None),
            Ok(UpgradeableLoaderState::ProgramData { .. }) => (account, "ProgramData", None),
            _ => {
                return Err(anyhow!(
                    "Account {} is not a Buffer, Program, or ProgramData account",
                    account
                ));
            }
        };

    // Determine authority
    let authority_keypair = if let Some(auth_path) = authority {
        Keypair::read_from_file(&auth_path)
            .map_err(|e| anyhow!("Failed to read authority keypair: {}", e))?
    } else {
        payer.insecure_clone()
    };

    // Determine recipient
    let recipient_pubkey = recipient.unwrap_or_else(|| authority_keypair.pubkey());

    if !bypass_warning {
        println!();
        println!(
            "WARNING: This will close the {} account and reclaim all lamports.",
            account_type
        );

        if account_type == "ProgramData" {
            println!();
            println!(
                "IMPORTANT: Closing the ProgramData account will make the program non-upgradeable"
            );
            println!("and the program will become immutable. This action cannot be undone!");
        }

        println!();
        print!("Continue? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    println!("Closing {} account...", account_type);

    let close_ixs = loader_v3_instruction::close_any(
        &account_to_close,
        &recipient_pubkey,
        Some(&authority_keypair.pubkey()),
        program_account.as_ref(),
    );

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[close_ixs],
        Some(&payer.pubkey()),
        &[&payer, &authority_keypair],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow!("Failed to close account: {}", e))?;

    println!("{} account closed successfully!", account_type);
    println!("Reclaimed lamports sent to: {}", recipient_pubkey);
    Ok(())
}

fn program_extend(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    additional_bytes: usize,
) -> Result<()> {
    let (rpc_client, config) = get_rpc_client_and_config(cfg_override)?;
    let payer = config.wallet_kp()?;

    if additional_bytes == 0 {
        return Err(anyhow!("Additional bytes must be greater than zero"));
    }

    println!("Extending program data...");
    println!("Program ID: {}", program_id);
    println!("Additional bytes: {}", additional_bytes);

    // Get the program account to find the ProgramData address
    let program_account = rpc_client
        .get_account(&program_id)
        .map_err(|e| anyhow!("Failed to get program account {}: {}", program_id, e))?;

    if program_account.owner != bpf_loader_upgradeable_id::id() {
        return Err(anyhow!(
            "Account {} is not an upgradeable program",
            program_id
        ));
    }

    // Get the ProgramData address
    let programdata_address =
        match bincode::deserialize::<UpgradeableLoaderState>(&program_account.data) {
            Ok(UpgradeableLoaderState::Program {
                programdata_address,
            }) => programdata_address,
            _ => {
                return Err(anyhow!(
                    "{} is not an upgradeable program account",
                    program_id
                ));
            }
        };

    // Get the ProgramData account to verify upgrade authority
    let programdata_account = rpc_client
        .get_account(&programdata_address)
        .map_err(|e| anyhow!("Program {} is closed: {}", program_id, e))?;

    // Get the upgrade authority address
    let upgrade_authority_address =
        match bincode::deserialize::<UpgradeableLoaderState>(&programdata_account.data) {
            Ok(UpgradeableLoaderState::ProgramData {
                upgrade_authority_address,
                ..
            }) => upgrade_authority_address,
            _ => {
                return Err(anyhow!("Program {} is closed", program_id));
            }
        };

    let upgrade_authority_address = upgrade_authority_address
        .ok_or_else(|| anyhow!("Program {} is not upgradeable", program_id))?;

    // Verify the payer is the upgrade authority
    if upgrade_authority_address != payer.pubkey() {
        return Err(anyhow!(
            "Upgrade authority mismatch. Expected {}, but ProgramData has {}",
            payer.pubkey(),
            upgrade_authority_address
        ));
    }

    // Use the checked version which requires upgrade authority signature
    let extend_ix = loader_v3_instruction::extend_program_checked(
        &program_id,
        &upgrade_authority_address,
        Some(&payer.pubkey()),
        additional_bytes as u32,
    );

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[extend_ix],
        Some(&payer.pubkey()),
        &[&payer], // payer is also the upgrade authority
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow!("Failed to extend program: {}", e))?;

    println!("Program extended successfully!");
    Ok(())
}

// ========== Agave's core parallel deployment functions ==========

pub fn calculate_max_chunk_size(baseline_msg: Message) -> usize {
    let tx_size = bincode::serialized_size(&Transaction {
        signatures: vec![
            solana_sdk::signature::Signature::default();
            baseline_msg.header.num_required_signatures as usize
        ],
        message: baseline_msg,
    })
    .unwrap() as usize;
    // add 1 byte buffer to account for shortvec encoding
    PACKET_DATA_SIZE.saturating_sub(tx_size).saturating_sub(1)
}

#[allow(clippy::too_many_arguments)]
pub fn send_deploy_messages(
    rpc_client: &RpcClient,
    initial_message: Option<Message>,
    write_messages: Vec<Message>,
    final_message: Option<Message>,
    fee_payer_signer: &dyn Signer,
    initial_signer: Option<&dyn Signer>,
    write_signer: Option<&dyn Signer>,
    final_signers: Option<&[&dyn Signer]>,
    max_sign_attempts: usize,
    commitment: CommitmentConfig,
    send_transaction_config: RpcSendTransactionConfig,
) -> Result<Option<solana_sdk::signature::Signature>> {
    // Handle initial message (e.g., buffer creation)
    if let Some(message) = initial_message {
        if let Some(initial_signer) = initial_signer {
            let mut initial_transaction = Transaction::new_unsigned(message.clone());
            let blockhash = rpc_client.get_latest_blockhash()?;

            // Sign based on number of required signatures
            if message.header.num_required_signatures == 3 {
                initial_transaction.try_sign(
                    &[fee_payer_signer, initial_signer, write_signer.unwrap()],
                    blockhash,
                )?;
            } else if message.header.num_required_signatures == 2 {
                initial_transaction.try_sign(&[fee_payer_signer, initial_signer], blockhash)?;
            } else {
                initial_transaction.try_sign(&[fee_payer_signer], blockhash)?;
            }

            rpc_client
                .send_and_confirm_transaction_with_spinner_and_config(
                    &initial_transaction,
                    commitment,
                    send_transaction_config,
                )
                .map_err(|err| anyhow!("Account allocation failed: {}", err))?;
        } else {
            return Err(anyhow!(
                "Buffer account not created yet, must provide a key pair"
            ));
        }
    }

    if !write_messages.is_empty() {
        if let Some(write_signer) = write_signer {
            send_messages_in_batches(
                rpc_client,
                &write_messages,
                &[fee_payer_signer, write_signer],
                max_sign_attempts,
                commitment,
                send_transaction_config,
            )?;
        }
    }

    if let Some(message) = final_message {
        if let Some(final_signers) = final_signers {
            let mut final_tx = Transaction::new_unsigned(message);
            let blockhash = rpc_client.get_latest_blockhash()?;
            let mut signers = final_signers.to_vec();
            signers.push(fee_payer_signer);
            final_tx.try_sign(&signers, blockhash)?;

            return Ok(Some(
                rpc_client
                    .send_and_confirm_transaction_with_spinner_and_config(
                        &final_tx,
                        commitment,
                        send_transaction_config,
                    )
                    .map_err(|e| anyhow!("Deploying program failed: {}", e))?,
            ));
        }
    }

    Ok(None)
}

/// Complete buffer writing implementation
#[allow(clippy::too_many_arguments)]
pub fn write_program_buffer(
    rpc_client: &RpcClient,
    payer: &dyn Signer,
    program_data: &[u8],
    buffer_authority: &Pubkey,
    buffer_keypair: &dyn Signer,
    max_len: Option<usize>,
    commitment: CommitmentConfig,
    send_transaction_config: RpcSendTransactionConfig,
) -> Result<Pubkey> {
    let buffer_pubkey = buffer_keypair.pubkey();

    let program_len = program_data.len();
    let buffer_len = max_len.unwrap_or(program_len);

    // Calculate required lamports for buffer
    let buffer_data_len = UpgradeableLoaderState::size_of_buffer(buffer_len);
    let min_balance = rpc_client
        .get_minimum_balance_for_rent_exemption(buffer_data_len)
        .map_err(|e| anyhow!("Failed to get rent exemption: {}", e))?;

    // Get blockhash for all messages
    let blockhash = rpc_client.get_latest_blockhash()?;

    // Create buffer initialization message
    let initial_instructions = loader_v3_instruction::create_buffer(
        &payer.pubkey(),
        &buffer_pubkey,
        buffer_authority,
        min_balance,
        buffer_len,
    )
    .map_err(|e| anyhow!("Failed to create buffer instruction: {}", e))?;

    let initial_message = Some(Message::new_with_blockhash(
        &initial_instructions,
        Some(&payer.pubkey()),
        &blockhash,
    ));

    // Prepare all write messages upfront
    let write_messages = prepare_write_messages(
        program_data,
        &buffer_pubkey,
        buffer_authority,
        &payer.pubkey(),
        &blockhash,
    );

    const MAX_SIGN_ATTEMPTS: usize = 5;
    send_deploy_messages(
        rpc_client,
        initial_message,
        write_messages,
        None,
        payer,
        Some(buffer_keypair),
        Some(payer),
        None,
        MAX_SIGN_ATTEMPTS,
        commitment,
        send_transaction_config,
    )?;
    Ok(buffer_pubkey)
}

/// Prepare write messages
fn prepare_write_messages(
    program_data: &[u8],
    buffer_pubkey: &Pubkey,
    buffer_authority: &Pubkey,
    fee_payer: &Pubkey,
    blockhash: &Hash,
) -> Vec<Message> {
    let create_msg = |offset: u32, bytes: Vec<u8>| {
        let instruction =
            loader_v3_instruction::write(buffer_pubkey, buffer_authority, offset, bytes);
        Message::new_with_blockhash(&[instruction], Some(fee_payer), blockhash)
    };

    let mut write_messages = Vec::new();
    let chunk_size = calculate_max_chunk_size(create_msg(0, Vec::new()));

    for (chunk, i) in program_data.chunks(chunk_size).zip(0usize..) {
        let offset = i.saturating_mul(chunk_size);
        write_messages.push(create_msg(offset as u32, chunk.to_vec()));
    }

    write_messages
}

/// Send messages in parallel
fn send_messages_in_batches(
    rpc_client: &RpcClient,
    messages: &[Message],
    signers: &[&dyn Signer],
    max_sign_attempts: usize,
    commitment: CommitmentConfig,
    send_config: RpcSendTransactionConfig,
) -> Result<()> {
    // Use parallel send and confirm function
    // Create a new RpcClient with the same URL and wrap in Arc for parallel processing
    let url = rpc_client.url();
    let new_rpc_client = RpcClient::new_with_commitment(url, commitment);
    let rpc_client_arc = Arc::new(new_rpc_client);

    let transaction_errors = send_and_confirm_transactions_in_parallel_blocking_v2(
        rpc_client_arc,
        None,
        messages,
        signers,
        SendAndConfirmConfigV2 {
            resign_txs_count: Some(max_sign_attempts),
            with_spinner: true,
            rpc_send_transaction_config: send_config,
        },
    )
    .map_err(|err| anyhow!("Data writes to account failed: {}", err))?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if !transaction_errors.is_empty() {
        for transaction_error in &transaction_errors {
            eprintln!("{:?}", transaction_error);
        }
        return Err(anyhow!(
            "{} write transactions failed",
            transaction_errors.len()
        ));
    }

    Ok(())
}
