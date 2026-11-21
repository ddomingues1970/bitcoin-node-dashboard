# Executor Prompt — Bitcoin Node Dashboard

You are the Executor implementing Bitcoin Node Dashboard.

Work only in:

    .

The Operator explicitly requested this implementation and asked for it to be verified. All source, documentation, UI text, logs, and handoff notes must be in US English.

## Read first

Read these files completely before editing:

1. AGENTS.md
2. README.md
3. PROJECT_STATUS.md
4. RESUME.md
5. docs/implementation-plan.md
6. docs/architecture.md
7. docs/threat-model.md
8. docs/operator-guide.md
9. docs/development-plan.md

Read LOG_RULES.md to understand the optional protocol. Do not activate reglog unless the Operator explicitly types reglog on.

## Objective

Implement a polished, lightweight Rust web dashboard for the Operator's local Bitcoin Core 32.1 node on Linux Mint. Follow the feature scope and acceptance criteria in docs/implementation-plan.md. Use Axum and Tokio for the local server, typed Serde models for data, and a local HTML/CSS/JavaScript frontend without a frontend build system unless there is a clear, documented reason.

## Required functionality

- Node version, chain, reachable/disconnected state, current block height, headers, sync progress, initial block download, and background validation status when provided by the node.
- Inbound/outbound/total peer counts without exposing peer addresses or node identifiers.
- Mempool transaction count, virtual size, memory usage, capacity, and available fee estimates.
- Bitcoin Core network byte counters and live rates.
- Read-only local CPU, memory, disk-free, and uptime summaries where reliable.
- Responsive, accessible dark dashboard with restrained Bitcoin-orange accents, sync visualization, status cards, network and mempool sections, compact in-memory trend charts, manual refresh, and a clear automatic refresh cadence.
- Clear loading, partial data, offline, and recovery states.

## Security boundaries

- Bind strictly to 127.0.0.1. Do not expose LAN access or add a configurable bind address.
- Use a fixed allowlist of read-only Bitcoin Core RPC methods.
- Invoke bitcoin-cli only with std::process::Command and separate arguments. Never use a shell, arbitrary arguments, an HTTP-provided method, or an HTTP-provided executable path.
- Do not call or expose peer-management, wallet, transaction-creation, signing, broadcast, or any other mutating RPC.
- Do not read or copy RPC credentials or cookie contents. Use bitcoin-cli's normal authentication.
- Never modify bitcoin.conf, Bitcoin Core services, Tor, Bisq, firewall rules, or operating-system settings.
- Do not return raw stderr, raw RPC responses, cookie/configuration paths, peer addresses, wallet information, or sensitive local paths.
- Render dynamic browser data via safe DOM APIs, not HTML string injection.
- No CORS, external requests, CDN assets, telemetry, persistence, autostart service, or database.
- Keep logs to sanitized lifecycle/error categories.

## Workflow

1. Inspect Git status, branch, and existing files. Preserve unrelated changes.
2. Implement the application using the narrow module boundaries described in docs/architecture.md. Keep dependencies minimal and pin direct dependency versions; commit Cargo.lock.
3. Add unit and integration tests for method allowlisting, JSON parsing, redaction, offline behavior, and the HTTP routes. Use fixtures or a mock bitcoin-cli executable; do not require wallet access or real funds.
4. Keep docs and CHANGELOG.md accurate as implementation evolves.
5. Run cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings, cargo build, cargo test, and cargo audit if installed. If dependency downloads or missing tools block validation, report the exact blocker.
6. Verify the local API by running the app temporarily and requesting it through loopback. Stop the process afterward. Do not leave a background process or install a system service.
7. Do not commit changes unless the Operator separately asks for a commit.

## Completion report

Report:

- the implemented features and design;
- changed files;
- exact build/run command;
- all validation results and any unavailable checks;
- whether a live Bitcoin Core query succeeded (never infer success);
- residual limitations and any Operator action required.

Do not modify the Frost Lock Wallet project or its output snapshots.

