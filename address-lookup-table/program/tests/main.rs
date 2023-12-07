//! Address Lookup Table program tests

#[cfg(feature = "test-sbf")]
pub mod harness;

#[cfg(feature = "test-sbf")]
use solana_program_test::tokio;

#[cfg(feature = "test-sbf")]
#[tokio::main]
async fn main() {
    libtest_mimic::run(
        &libtest_mimic::Arguments::from_args(),
        harness::AddressLookupTestHarness::tests(),
    )
    .exit();
}

#[cfg(not(feature = "test-sbf"))]
fn main() {
    println!();
    println!("[TESTS SKIPPED]: Address Lookup Table program tests require `cargo test-sbf`.");
    println!();
}
