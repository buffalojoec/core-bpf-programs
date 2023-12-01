# Address Lookup Table

This BPF implementation essentially replaces the following crates in
`github.com:solana-labs/solana/programs`:

- `address-lookup-table`
- `address-lookup-table-tests`

Biggest changes for Core BPF migration:

- Errors are inside of program crate, not SDK
  - The errors from the SDK probably should be removed?
  - If not, we can import them, and just add the necessary new ones to that
  module
- Had to hack around `limited_deserialize` until we decide if and how to
  update the SDK
