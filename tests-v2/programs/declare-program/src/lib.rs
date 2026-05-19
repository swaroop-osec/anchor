use anchor_lang_v2::prelude::*;

declare_program!(external);

#[cfg(feature = "cpi")]
pub fn cpi_account_type_is_generated<'a>(
    accounts: external::cpi::accounts::Composite<'a>,
) -> external::cpi::accounts::Composite<'a> {
    accounts
}
