# Operator Guide

## Requirements and start

Use Linux Mint or compatible Linux, Rust stable/Cargo, and Bitcoin Core with `bitcoin-cli` available in PATH. No `bitcoin.conf` changes are required. From the project root run:

    cargo run

Open http://127.0.0.1:8890. The server binds only to loopback. Use **Stop dashboard** and confirm, or press Ctrl+C. Shutdown stops only the dashboard; Bitcoin Core, Tor, and Bisq remain running. Release builds use `cargo run --release`.

## Views and update cadence

- **Overview** refreshes every 15 seconds and shows node/host summary data.
- **Mempool** loads this node's local transaction count, BIP 141 virtual size, memory use/capacity, minimum mempool/relay fee, and fee estimates. A local node template is summarized into selected transaction count, virtual size, total fees, and fee-rate bands. The template is a current candidate, not a mined block or a prediction of pool selection. The fee bands cover template-selected transactions, not the entire mempool. Raw transactions and IDs are not returned. The aggregate refreshes every 60 seconds and is cached for 55 seconds.
- **Blocks** loads up to six recent confirmed blocks from the active chain when selected and refreshes every 60 seconds while selected. Cards show non-coinbase transaction size/weight statistics. Selecting a block loads a capped list of up to 100 transaction IDs. These are confirmed blocks seen by this node, not candidate blocks.

The **Refresh now** button refreshes the active view. Fee estimates are node estimates, not promises about confirmation time; BTC/kvB values are converted to sat/vB. Zero or absent estimates display as unavailable. Missing RPC data shows a sanitized diagnostic category. Local mempool contents differ between nodes and can differ from mempool.space. Fee-band bars are sized by template group virtual size and colored from green (lower fee rate) to yellow (higher fee rate). No arrival-history chart is shown because count deltas include removals.

## API and privacy

`GET /api/status`, `GET /api/mempool`, `GET /api/blocks`, and `GET /api/blocks/{height}` return sanitized summaries. The block route accepts only a numeric height. No route accepts an RPC method or arbitrary CLI argument. `POST /api/shutdown` requires the current page's token and exact `http://127.0.0.1:8890` Origin; GET cannot stop the app. The API never returns peer addresses, credentials, raw RPC errors, or raw transactions.

CPU/memory use Linux `/proc`; disk free space is sampled on the filesystem containing conventional `~/.bitcoin`. Custom Bitcoin data directories are not detected.

## Troubleshooting

- If Core is unavailable, confirm Core is running and `bitcoin-cli` works as the same user.
- If partial details are missing, retry; Core may be syncing, busy, pruned, or a chain reorganization may have occurred. The API reports only a sanitized category, never raw RPC errors.
- If port 8890 is occupied, stop the conflicting local application. The bind address and port are fixed.
