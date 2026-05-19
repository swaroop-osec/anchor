use {
    anchor_lang_v2::{Discriminator, Id, InstructionData, ToAccountMetas},
    declare_program::{cpi_account_type_is_generated, external},
    sha2::{Digest, Sha256},
    solana_pubkey::Pubkey,
};

fn program_id() -> Pubkey {
    "Externa111111111111111111111111111111111111"
        .parse()
        .unwrap()
}

fn disc_hash(name: &str) -> [u8; 8] {
    let hash = Sha256::digest(format!("global:{name}").as_bytes());
    hash[..8].try_into().unwrap()
}

#[test]
fn declare_program_exports_program_marker_and_id() {
    assert_eq!(external::ID, program_id());
    assert_eq!(external::program::External::id(), program_id());
}

#[test]
fn declared_instruction_builders_use_external_program_id() {
    let ix =
        external::instruction::Update { value: 42 }.to_instruction(external::accounts::Update {
            authority: Pubkey::new_unique(),
            data: Pubkey::new_unique(),
        });

    assert_eq!(ix.program_id, program_id());
}

#[test]
fn declared_default_discriminator_uses_snake_case_anchor_hash() {
    let ix =
        external::instruction::DefaultDisc {}.to_instruction(external::accounts::DefaultDisc {});

    assert_eq!(
        external::instruction::DefaultDisc::DISCRIMINATOR,
        &disc_hash("default_disc")
    );
    assert_eq!(ix.data, disc_hash("default_disc"));
}

#[test]
fn declared_explicit_discriminators_are_preserved() {
    assert_eq!(external::instruction::Update::DISCRIMINATOR, &[9, 8, 7, 6]);
    assert_eq!(
        external::instruction::DefinedArgs::DISCRIMINATOR,
        &[1, 3, 5, 7, 9, 11, 13, 15]
    );
    assert_eq!(
        external::instruction::BytesAndString::DISCRIMINATOR,
        &[2, 4, 6, 8]
    );
    assert_eq!(
        external::instruction::Composite::DISCRIMINATOR,
        &[44, 45, 46]
    );
}

#[test]
fn declared_scalar_and_defined_args_serialize_after_discriminator() {
    let update_data = external::instruction::Update { value: 0x1122_3344 }.data();
    assert_eq!(&update_data[..4], &[9, 8, 7, 6]);
    assert_eq!(&update_data[4..], &0x1122_3344u32.to_le_bytes());

    let owner = Pubkey::new_unique();
    let defined_data = external::instruction::DefinedArgs {
        args: external::MyArgs {
            amount: 0x0102_0304_0506_0708,
            tag: *b"id2",
            owner,
        },
    }
    .data();
    assert_eq!(&defined_data[..8], &[1, 3, 5, 7, 9, 11, 13, 15]);
    assert_eq!(
        &defined_data[8..16],
        &0x0102_0304_0506_0708u64.to_le_bytes()
    );
    assert_eq!(&defined_data[16..19], b"id2");
    assert_eq!(&defined_data[19..51], owner.as_ref());
}

#[test]
fn declared_vec_and_string_args_compile_and_include_discriminator() {
    let data = external::instruction::BytesAndString {
        payload: vec![1, 2, 3],
        label: "anchor".to_string(),
    }
    .data();

    assert_eq!(&data[..4], &[2, 4, 6, 8]);
    assert!(data.len() > 4);
}

#[test]
fn declared_account_metas_preserve_flags_and_composites() {
    let authority = Pubkey::new_unique();
    let vault = Pubkey::new_unique();
    let payer = Pubkey::new_unique();
    let metas = external::accounts::Composite {
        inner: external::__client_accounts_inner::Inner { authority, vault }.into(),
        payer,
    }
    .to_account_metas(None);

    assert_eq!(metas.len(), 3);
    assert_eq!(metas[0].pubkey, authority);
    assert!(metas[0].is_signer);
    assert!(!metas[0].is_writable);
    assert_eq!(metas[1].pubkey, vault);
    assert!(!metas[1].is_signer);
    assert!(metas[1].is_writable);
    assert_eq!(metas[2].pubkey, payer);
    assert!(metas[2].is_signer);
    assert!(metas[2].is_writable);
}

#[test]
fn declared_cpi_account_surface_is_generated() {
    let _ = cpi_account_type_is_generated;
}
