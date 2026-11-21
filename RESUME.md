# Project Resume

## Project context

Bitcoin Node Dashboard is a local, read-only monitor for Bitcoin Core. The listener is fixed to 127.0.0.1:8890. Do not modify Bitcoin Core, Tor, Bisq, system settings, or the separate Frost Lock Wallet repository. Keep all product text in US English. The optional conversation log remains inactive unless the Operator explicitly types `reglog on`.

## Verified implementation state

- Overview retains the node and host summary and refreshes every 15 seconds.
- Mempool reads local `getmempoolinfo` data (`bytes` is BIP 141 virtual size), `estimatesmartfee`, and `getblocktemplate`. It aggregates template-selected transaction fee bands; raw transactions/IDs are not included in browser API responses. Template output has a 3-second RPC timeout, 4-second process guard, 8 MB streaming cap, and 55-second in-memory aggregate cache.
- Blocks displays up to six recent blocks. The adapter handles bitcoin-cli's unquoted hash scalar, verifies the stats/header height and positive confirmations, and preserves successful cards when another block fails. Selected details return at most 100 transaction IDs.
- The local Bitcoin Core RPC query succeeded during the 2026-11-20 run. Local API checks returned blocks and aggregate mempool/template data. This result is time-specific; rerun the API smoke check to reconfirm connectivity.
- Screenshot references were described in the prompt but were not present as workspace files or accessible attachments. The computer-use browser host was unavailable, so rendered desktop/mobile preview could not be inspected.
- `cargo fmt`, Clippy, build, 14 tests including deterministic fixtures, JavaScript syntax validation, local live API checks, and `cargo audit` passed for this work.

## Current blockers and next work

No implementation blocker remains. Browser preview could not be completed because no browser surface was available. Next, review the rendered view at desktop and narrow widths when a browser is available. Fee bands are based on template-selected transactions, not a full mempool histogram. Block size/weight shown in cards are Core's non-coinbase transaction statistics.

## Run

From this project directory: `cargo run`, then open `http://127.0.0.1:8890`. The protected Stop dashboard action stops only this application.
