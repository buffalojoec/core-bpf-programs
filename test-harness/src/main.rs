//! CLI-based test harness for Core BPF program implementations.
//! 
//! This test harness is completely implementation-agnostic. It is designed to
//! be used with any BPF program implementation that can be compiled to a
//! shared object file.
//! 
//! The harness will test any program against a suite of test cases that
//! pertain to the specification the program is implementing.
//! 
//! For example, the Solana Labs implementation of the Address Lookup Table
//! program, written in Rust, can be tested against the `address-lookup-table`
//! Core BPF specification by running:
//! 
//! ```bash
//! solana-core-bpf-test \
//!     --program-path path/to/address-lookup-table.so \
//!     --spec address-lookup-table
//! ```
fn main() {
    println!("Hello, world!");
}
