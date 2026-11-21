# Development Plan

## Delivered milestones

1. Cargo app, loopback Axum server, Linux host summaries, and typed read-only bitcoin-cli adapter.
2. Responsive local Overview with in-memory network trend and protected dashboard-only shutdown.
3. Keyboard accessible Overview, Mempool, and Blocks tabs with safe text rendering and view-specific polling.
4. Bounded active-chain block summaries, lazy capped transaction ID details, BIP 141 mempool virtual size, corrected fee conversion, and cached aggregate template fee bands.
5. Documentation and fixture coverage for CLI scalar parsing, error categories, fee units, active-chain checks, partial block failures, template aggregation, output bounds, and shutdown behavior.

## Review gates

Confirm fixed read-only RPC allowlist, no shell or browser-controlled command, loopback-only binding, error redaction, safe DOM rendering, and no external assets. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, `cargo test`, local API/UI checks, and `cargo audit` when installed.

## Deferred

A full-mempool fee histogram and incoming-transaction history remain out of scope: Core does not provide a full-mempool histogram as a compact aggregate, and mempool count deltas combine arrivals with removals. The implemented fee bands describe only bounded template-selected transactions. Future-block simulation, persistent telemetry, wallet features, remote access, and Bitcoin Core control remain out of scope.
