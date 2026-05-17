use {
    anchor_lang_v2::accounts::SysvarId,
    pinocchio::{
        address::Address,
        sysvars::{clock::Clock, instructions::Instructions, rent::Rent, slot_hashes::SlotHashes},
    },
};

#[test]
fn sysvar_idl_addresses_match_well_known_accounts() {
    assert_eq!(
        <Clock as SysvarId>::IDL_ADDRESS,
        "SysvarC1ock11111111111111111111111111111111"
    );
    assert_eq!(
        <Rent as SysvarId>::IDL_ADDRESS,
        "SysvarRent111111111111111111111111111111111"
    );
    assert_eq!(
        <Instructions<&'static [u8]> as SysvarId>::IDL_ADDRESS,
        "Sysvar1nstructions1111111111111111111111111"
    );
    assert_eq!(
        <SlotHashes<&'static [u8]> as SysvarId>::IDL_ADDRESS,
        "SysvarS1otHashes111111111111111111111111111"
    );
}

#[test]
fn instructions_sysvar_id_is_not_the_system_program() {
    assert_eq!(
        <Instructions<&'static [u8]> as SysvarId>::SYSVAR_ID,
        Address::from_str_const("Sysvar1nstructions1111111111111111111111111")
    );
    assert_ne!(
        <Instructions<&'static [u8]> as SysvarId>::SYSVAR_ID,
        Address::from_str_const("11111111111111111111111111111111")
    );
}
