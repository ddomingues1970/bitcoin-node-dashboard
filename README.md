# Bitcoin Node Dashboard

Bitcoin Node Dashboard is a local web interface for monitoring one Bitcoin Core node from the same Linux computer.

## Views

- **Overview** shows chain synchronization, Bitcoin Core version, aggregate peer counts, local mempool totals, network traffic, host metrics, and fee estimates.
- **Mempool** shows this node's local transaction count, BIP 141 virtual size, memory use/capacity, minimum fee context, and fee estimates. A local `getblocktemplate` candidate is aggregated into transaction fee bands; raw transactions and transaction IDs are discarded before the API response. The bands describe transactions in the candidate template, not the entire mempool.
- **Blocks** shows up to six recent blocks verified against the active best chain, with time, transaction count, fee total when available, and Core's non-coinbase transaction size/weight statistics. Selecting a block loads at most 100 transaction IDs; raw transactions are not returned.

Overview refreshes every 15 seconds. The selected Mempool or Blocks view refreshes every 60 seconds and loads on demand. Mempool data is cached in memory for 55 seconds; only fixed-size aggregates and fee bands reach the browser. Manual refresh updates the active view.

## Data behavior

Bitcoin Core 32.1 returns mempool virtual size in `getmempoolinfo.bytes`; the former adapter read a nonexistent `vsize` field. `bitcoin-cli getblockhash` prints an unquoted hex scalar, which the former JSON-only parser rejected and caused the empty Blocks view. Fee estimates arrive in BTC/kvB and are converted to sat/vB; missing or non-positive estimates remain unavailable. A transaction-arrival chart is omitted because periodic mempool count deltas include removals and cannot honestly represent incoming-transaction counts.

## Design and security

The Rust backend uses Axum and Tokio. A local HTML, CSS, and JavaScript interface calls a small JSON API. Node data comes through fixed read-only `bitcoin-cli` calls, preserving Bitcoin Core's normal authentication behavior. The listener binds only to 127.0.0.1. The dashboard does not change Bitcoin Core, Tor, Bisq, or system settings, and it does not manage wallets, peers, or transactions.

## Requirements

- Stable Rust toolchain, Edition 2024.
- Bitcoin Core 32.1 or compatible, with `bitcoin-cli` available in PATH.
- Linux Mint or another Linux system with `/proc` for host metrics.

## Run

From this directory, run:

    cargo run

Open http://127.0.0.1:8890. Use **Stop dashboard** and confirm, or press Ctrl+C. The shutdown action stops only this dashboard. For a release build, use `cargo run --release`.

The host summary checks free space on the filesystem containing the conventional `~/.bitcoin` directory; custom data directories are not detected. The dashboard remains usable when Bitcoin Core is unavailable. Core 32.1 reports mempool virtual size through `getmempoolinfo.bytes`; the adapter supports `vsize` as a fallback. Fee estimates are converted from BTC/kvB to sat/vB and shown as unavailable when Core returns no positive estimate. A template call has a three-second RPC timeout, a four-second process guard, and an 8 MB streamed output cap; the current local node returned a 6.9 MB result in about 0.23 seconds. No live connectivity is claimed unless an RPC call succeeds.

## License

Licensed under either of the following, at your option: [MIT](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE).

## Documentation

See `docs/architecture.md`, `docs/threat-model.md`, `docs/operator-guide.md`, `docs/development-plan.md`, and `CHANGELOG.md`.
