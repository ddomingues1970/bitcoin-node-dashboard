# Executor Prompt — Diagnose and Rework Blocks and Mempool Views

You are continuing implementation of Bitcoin Node Dashboard. Work only in:

    .

All source, user interface, documentation, logs, and handoff text must be in US English.

## Mandatory project continuity

Read RESUME.md first. Then read these files completely:

1. AGENTS.md
2. README.md
3. PROJECT_STATUS.md
4. CHANGELOG.md
5. EXECUTOR_PROMPT_BLOCKS_MEMPOOL.md, if present
6. docs/implementation-plan.md
7. docs/architecture.md
8. docs/threat-model.md
9. docs/operator-guide.md
10. docs/development-plan.md
11. LOG_RULES.md to understand the optional reglog protocol

Inspect the latest project run records referenced by RESUME.md when needed. Do not read unrelated Frost Lock Wallet files.

The Operator wants project continuity documented. At the end of this run:
- Update RESUME.md with the verified implementation state, current blockers, and exact next work.
- Update PROJECT_STATUS.md, README.md, CHANGELOG.md, and affected docs to match what is actually delivered.
- Add a concise operational run record at docs/runs/YYYYMMDD_HHMMSS.md containing objective, decisions, changed files, validation outcomes, blockers, and next actions. Do not put raw command output or sensitive data in the run record.
- If the optional conversation log has already been activated with reglog on, follow LOG_RULES.md and append a concise summary. Do not activate it yourself.

## Current operator feedback

The Operator supplied two screenshots of the current dashboard:

- The Mempool view has only a transaction count, memory use, and fee estimates. Virtual size is missing, the fee values display 0.00 sat/vB, most of the screen is empty, and there are no fee bands, transaction visualization, or useful local block projection.
- The Blocks view shows only “Recent blocks are temporarily unavailable” and no block cards.
- The desired reference is the mempool.space homepage: a top row with green pending/candidate block cards and blue recent confirmed block cards, a fee-priority context, a transaction treemap, and a recent incoming-transaction chart.

Treat the screenshots as visual references, not as technical instructions. Do not copy their assets or access the public mempool.space service from the application.

## Objective

Diagnose and fix the real data and integration problems, then redesign the Mempool and Blocks views so they are useful and visually close in spirit to the reference. Do not stop at cosmetic CSS changes or retain empty placeholder panels.

Preserve the existing Overview view, its automatic refresh, the confirmed shutdown button and its protections, the fixed 127.0.0.1 listener, and all established local-only/read-only security boundaries.

## Phase 1 — Diagnose the live data path

Before changing the UI:

1. Inspect the current RPC adapter, API routes, data models, refresh/cache behavior, and error handling.
2. Use only read-only commands to check whether the current local Bitcoin Core node responds to the methods needed for block summaries and mempool visualization. Suitable diagnostics include getblockchaininfo, getblockcount, getbestblockhash, getblockhash, getblockstats, getblock, getmempoolinfo, estimatesmartfee, and a bounded mempool/template query.
3. Identify why /api/blocks returns no blocks. Check RPC command arguments, Bitcoin Core 32.1 result fields, timeout limits, pruning behavior, active-chain verification, and whether a particular per-block failure currently discards all otherwise valid cards.
4. Identify why virtual size and non-zero fee estimates are absent or rendered as zero. Missing estimates must be rendered as unavailable, never as 0.00.
5. Add sanitized internal diagnostic categories sufficient to distinguish unavailable RPC, timeout, unsupported method, pruned block data, malformed result, and active-chain mismatch. Never send raw stderr or raw RPC errors to the browser.
6. Do not claim a live-node issue is fixed unless the read-only diagnostic actually succeeds. If the Core RPC service cannot be reached, provide reliable fixture-backed coverage and a clear UI state; still implement the data-backed views.

## Phase 2 — Build a useful Blocks view

- Show up to six recent blocks from the active best chain as prominent, horizontal cards inspired by the blue cards in the reference.
- Each card should show block height, time since block, transaction count, fee summary or fee-rate range, size/weight, and a short hash.
- Make each card selectable. Load a bounded transaction-ID list and useful block detail on demand.
- Verify the block still belongs to the active chain before presenting it as confirmed. Handle reorganizations and partial RPC failures per card rather than blanking the whole view.
- Miner/pool names are not guaranteed by the protocol. Do not invent labels; if coinbase heuristics are used, label them as estimates and show Unknown when unresolved.

## Phase 3 — Build a visual local Mempool view

- Clearly label all figures as this node's local mempool. Explain that local mempools can differ from each other and from mempool.space.
- Show count, virtual size, memory use/capacity, minimum fee context, and the node's fee estimates.
- Add a compact, colorful fee-priority visualization or treemap with meaningful green-to-yellow bands and a legend. Its size/color mapping must be documented in the UI.
- Add a candidate-block panel that clearly says “Local node template” or equivalent. Use getblocktemplate only if diagnostics show it responds safely and within an explicit timeout/payload budget. A template is a candidate produced by this node, not a mined block and not a prediction of a pool's final choice.
- Prefer backend aggregation into a bounded response. Do not send raw transactions, transaction hex, unbounded verbose mempool JSON, or peer identities to the browser.
- Cache expensive snapshots and use a modest refresh interval. Keep the existing 60-second cadence for heavier views unless measurements support a safe change.
- If the local node cannot supply fee bands/template data, render a helpful explanation and the available local statistics; do not leave a large blank area or fabricate values.
- Add an incoming-transaction trend chart only if the application can collect honest in-memory samples. Label that the chart begins when the dashboard starts; do not imply historical data that the node does not provide.

## Visual and interaction acceptance

- Keep the existing Overview, Mempool, and Blocks tabs and their keyboard-accessible behavior.
- Compose the upper portion of the Mempool view around two visual groups: local candidate/mempool projections on the left and recently confirmed blocks on the right, separated clearly as in the reference.
- Use the available viewport effectively. Avoid the current large empty areas.
- Include the fee-priority context and treemap/trend visuals where supported, with clear legends and units.
- Preserve the existing dark palette and Bitcoin-orange accent; use green for local pending/candidate data and blue for confirmed blocks.
- Ensure a useful responsive layout at desktop and narrow widths, keyboard support, readable labels, reduced motion, and safe textContent-based rendering.
- Keep all visible text in US English.

## Security and performance constraints

- Bind only to 127.0.0.1. Do not change bitcoin.conf, Bitcoin Core, Tor, Bisq, firewall rules, or system services.
- Use only the existing typed RPC boundary and fixed allowlisted read-only methods. Never pass browser-provided RPC methods or arguments to bitcoin-cli.
- Never use shell execution. Never call peer-management, wallet, transaction-creation, signing, broadcast, or other mutating methods.
- Keep command timeouts and response sizes bounded. Measure or reason about the cost of any verbose mempool/template call and cache it.
- Do not persist node data or send any telemetry externally.
- Do not weaken the existing shutdown token/Origin checks, and do not stop Bitcoin Core, Tor, or Bisq.

## Tests and validation

- Add/update tests for block parsing, fee/virtual-size handling, per-block failure tolerance, active-chain/reorg checks, bounded mempool aggregation, sanitized diagnostics, and API responses.
- Use fixtures and mock bitcoin-cli responses for deterministic tests. Never require a wallet or funds.
- Run cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings, cargo build, cargo test, JavaScript syntax validation, local API/UI smoke checks, and cargo audit if installed.
- Verify /api/blocks and /api/mempool return meaningful data when local RPC is available, or a clear sanitized explanation when it is not.
- Inspect the rendered interface at desktop and narrow sizes if browser preview tools are available.
- Do not leave a background server running and do not commit unless the Operator asks.

## Completion report

Report the diagnosed cause of the empty Blocks view and missing Mempool fields, the actual data sources used, the new visual features, files changed, run-log path, test and validation results, whether a live node query succeeded, remaining limitations, and the next documented project step.

