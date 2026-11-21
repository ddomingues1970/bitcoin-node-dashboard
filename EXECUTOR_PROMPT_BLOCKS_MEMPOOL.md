# Executor Prompt — Add Blocks and Mempool Monitoring Views

You are the Executor implementing a focused dashboard evolution.

Work only in:

    .

All source, UI text, documentation, logs, and the completion report must be in US English.

## Read first

Read:

1. AGENTS.md
2. README.md
3. PROJECT_STATUS.md
4. RESUME.md
5. docs/implementation-plan.md
6. docs/architecture.md
7. docs/threat-model.md
8. docs/operator-guide.md
9. docs/development-plan.md
10. CHANGELOG.md

Read LOG_RULES.md only to understand the optional protocol. Do not activate reglog unless the Operator explicitly types reglog on.

Inspect the current Git status and implementation before editing. Preserve existing work, including the shutdown button and its protections. Do not commit changes.

## Objective

Evolve the existing local Bitcoin Node Dashboard into a tabbed monitoring interface while preserving its current Overview functionality and local-only, read-only security boundary.

Add:

- An Overview tab for the existing node and host summary.
- A Blocks tab modeled on the confirmed-block cards shown in the Operator's mempool.space reference screenshot.
- A Mempool tab that visualizes pending transactions and, where safely supported, a local candidate block view based only on the Operator's own Bitcoin Core node.

Implement the confirmed Blocks view first. Implement the local Mempool view in the same change if it can be added without excessive RPC load or weakening security. Clearly report any part deferred due to measured cost or unavailable data.

## Blocks view requirements

- Display a small, bounded set of recent confirmed blocks from the node's active best chain.
- Each block card should show height, time/age, transaction count, total fees or available fee summary, and block size/weight.
- On selection, show useful block details and a transaction list. Fetch transaction details lazily rather than loading full details for every displayed block.
- Use Bitcoin Core read-only RPC data. Suitable calls include getblockcount, getblockhash, getblock with verbosity 1 or 2, and getblockstats.
- Handle a new tip, transient RPC failure, and a possible reorganization without presenting stale data as confirmed. Use confirmations or verify that each displayed hash still matches its height on the current best chain.
- Miner/pool identity is not a guaranteed block field. Do not invent a miner label. If coinbase-based identification is attempted, label it as a heuristic and show Unknown when no reliable match exists.

## Mempool view requirements

- Make clear that data comes from this node's local mempool and can differ from mempool.space or another node.
- Show useful local mempool totals, fee-rate bands, pending virtual size/transaction count, and the current fee estimates when available.
- If rendering transaction tiles, aggregate in the backend to bounded summaries or return only a capped set of safe fields. Do not send raw transactions or unbounded mempool data to the browser.
- A candidate block/template preview may use Bitcoin Core getblocktemplate if available and safely bounded. Label it as this node's current template/candidate, not a mined block or a prediction of what a pool will select. Handle unsupported/slow RPCs with a clear unavailable state.
- Do not simulate multiple future blocks unless the implementation can explain and validate its local policy assumptions. Prefer a modest, clearly labeled local estimate.

## Interface requirements

- Keep the existing Overview view and preserve its metrics and shutdown action.
- Add keyboard-accessible tabs labeled Overview, Mempool, and Blocks. Use appropriate tab roles and arrow-key behavior or native accessible navigation.
- Preserve the current dark visual style and local English UI.
- Match the reference's broad visual idea: compact cards for recent mined blocks and a separate local mempool/candidate visualization. Do not copy external assets or rely on an external service.
- Keep automatic refresh behavior, but use a lower frequency or lazy loading for heavier block and mempool details. Show the update cadence and per-view last-updated state.
- Keep a manual refresh action if it already exists.
- Ensure the UI remains responsive on smaller screens and safe DOM APIs are used for node-derived data.

## Security and operational constraints

- Preserve the fixed 127.0.0.1 listener and the existing safe shutdown flow.
- Use only the project's typed, fixed RPC adapter and allowlisted read-only methods. Never accept RPC methods, arguments, or CLI commands from browser input.
- Never use shell execution or expose raw RPC stderr/output, wallet data, peer IPs, or sensitive local paths.
- Do not add CORS, external APIs, remote scripts, CDN assets, persistent telemetry, or arbitrary transaction lookup.
- Do not modify bitcoin.conf, Bitcoin Core, Tor, Bisq, firewall rules, or system services.
- Keep polling and payloads bounded so the dashboard does not add unnecessary load while Bitcoin Core is syncing.

## Tests, documentation, and validation

- Add tests for block/mempool parsing, output bounds, route behavior, safe handling of missing fields, and chain/reorg status as practical.
- Update README.md, PROJECT_STATUS.md, CHANGELOG.md, docs/implementation-plan.md, docs/architecture.md, docs/threat-model.md, docs/operator-guide.md, and docs/development-plan.md to match the delivered implementation.
- Run cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings, cargo build, cargo test, and cargo audit if installed.
- Exercise the local API and UI behavior without leaving a background server running. Use mock fixtures or the local node only; do not claim live node verification unless a query succeeds.
- Report dependency/network blockers precisely.

## Completion report

Summarize the new tabs, what each data source means, changed files, validation outcomes, any deferred feature, and the exact command to run the application. Explicitly distinguish confirmed blocks seen by this node from candidate template or local mempool estimates.

