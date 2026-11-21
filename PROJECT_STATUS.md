# Project Status

Status: Overview, data-backed Mempool and active-chain Blocks views implemented and validated against the local Bitcoin Core node.

## Current objective

Maintain a lightweight, attractive, local-only, read-only Bitcoin Core dashboard in Rust for Linux Mint.

## Decisions

- Project location: Projects/Rust/bitcoin-node-dashboard.
- Backend: Rust stable, Edition 2024, Axum, and Tokio.
- Frontend: local HTML, CSS, and plain JavaScript.
- Node access: fixed read-only bitcoin-cli calls using Bitcoin Core's existing authentication.
- Listener: 127.0.0.1:8890 only.
- Stop dashboard requires confirmation, per-process token, and exact loopback Origin; it stops only the web server gracefully.
- All project content is US English.

## Delivered views

- Overview retains node, peer, host, traffic, and fee metrics.
- Mempool uses local `getmempoolinfo` totals (`bytes` for BIP 141 virtual size), fee estimates converted to sat/vB, minimum fee context, and an aggregated `getblocktemplate` candidate. The candidate response is streamed with an 8 MB cap, 3-second RPC timeout, 4-second process guard, and 55-second aggregate cache. Fee bands describe only candidate-template transactions; raw tx data is not returned.
- Blocks shows up to six active-chain blocks using block hash, height, and positive confirmations. A single block failure does not discard successful cards. Details load on selection and return at most 100 transaction IDs. Size/weight fields are non-coinbase transaction statistics.

## Validation

Validation completed: formatting, Clippy with warnings denied, build, 14 tests including deterministic parsing/aggregation fixtures, JavaScript syntax check, local live API smoke checks, and `cargo audit` passed. Read-only local queries succeeded for status, block summary/header, mempool info, fee estimates, and getblocktemplate. The API returned verified block cards and aggregate mempool/template bands without raw transaction fields. Unauthorized shutdown returned 403. Browser preview was unavailable in this environment.
