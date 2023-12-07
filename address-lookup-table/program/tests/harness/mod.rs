//! Address Lookup Table program test harness
#![cfg(feature = "test-sbf")]

pub mod close_lookup_table;
pub mod common;
pub mod create_lookup_table;
pub mod deactivate_lookup_table;
pub mod extend_lookup_table;
pub mod freeze_lookup_table;

use {libtest_mimic::Trial, solana_program_test::tokio};

const PROGRAM_IMPLEMENTATIONS: &[&str] = &[
    "solana_address_lookup_table_program",
    // More program implementations...
];

macro_rules! async_trial {
    ($test_func:path, $program_file:ident) => {{
        Trial::test(stringify!($test_func), move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { $test_func($program_file).await });
            Ok(())
        })
    }};
}

pub struct AddressLookupTestHarness;
impl AddressLookupTestHarness {
    pub fn tests() -> Vec<Trial> {
        PROGRAM_IMPLEMENTATIONS
            .iter()
            .flat_map(|program_file| {
                vec![
                    async_trial!(
                        create_lookup_table::test_create_lookup_table_idempotent,
                        program_file
                    ),
                    async_trial!(
                        create_lookup_table::test_create_lookup_table_not_idempotent,
                        program_file
                    ),
                    async_trial!(
                        create_lookup_table::test_create_lookup_table_use_payer_as_authority,
                        program_file
                    ),
                    async_trial!(
                        create_lookup_table::test_create_lookup_table_missing_signer,
                        program_file
                    ),
                    async_trial!(
                        create_lookup_table::test_create_lookup_table_not_recent_slot,
                        program_file
                    ),
                    async_trial!(
                        create_lookup_table::test_create_lookup_table_pda_mismatch,
                        program_file
                    ),
                    async_trial!(close_lookup_table::test_close_lookup_table, program_file),
                    async_trial!(
                        close_lookup_table::test_close_lookup_table_not_deactivated,
                        program_file
                    ),
                    async_trial!(
                        close_lookup_table::test_close_lookup_table_deactivated_in_current_slot,
                        program_file
                    ),
                    async_trial!(
                        close_lookup_table::test_close_lookup_table_recently_deactivated,
                        program_file
                    ),
                    async_trial!(
                        close_lookup_table::test_close_immutable_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        close_lookup_table::test_close_lookup_table_with_wrong_authority,
                        program_file
                    ),
                    async_trial!(
                        close_lookup_table::test_close_lookup_table_without_signing,
                        program_file
                    ),
                    async_trial!(
                        deactivate_lookup_table::test_deactivate_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        deactivate_lookup_table::test_deactivate_immutable_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        deactivate_lookup_table::test_deactivate_already_deactivated,
                        program_file
                    ),
                    async_trial!(
                        deactivate_lookup_table::test_deactivate_lookup_table_with_wrong_authority,
                        program_file
                    ),
                    async_trial!(
                        deactivate_lookup_table::test_deactivate_lookup_table_without_signing,
                        program_file
                    ),
                    async_trial!(extend_lookup_table::test_extend_lookup_table, program_file),
                    async_trial!(
                        extend_lookup_table::test_extend_lookup_table_with_wrong_authority,
                        program_file
                    ),
                    async_trial!(
                        extend_lookup_table::test_extend_lookup_table_without_signing,
                        program_file
                    ),
                    async_trial!(
                        extend_lookup_table::test_extend_deactivated_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        extend_lookup_table::test_extend_immutable_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        extend_lookup_table::test_extend_lookup_table_without_payer,
                        program_file
                    ),
                    async_trial!(
                        extend_lookup_table::test_extend_prepaid_lookup_table_without_payer,
                        program_file
                    ),
                    async_trial!(freeze_lookup_table::test_freeze_lookup_table, program_file),
                    async_trial!(
                        freeze_lookup_table::test_freeze_immutable_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        freeze_lookup_table::test_freeze_deactivated_lookup_table,
                        program_file
                    ),
                    async_trial!(
                        freeze_lookup_table::test_freeze_lookup_table_with_wrong_authority,
                        program_file
                    ),
                    async_trial!(
                        freeze_lookup_table::test_freeze_lookup_table_without_signing,
                        program_file
                    ),
                    async_trial!(
                        freeze_lookup_table::test_freeze_empty_lookup_table,
                        program_file
                    ),
                ]
            })
            .collect()
    }
}
