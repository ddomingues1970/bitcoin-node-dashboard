# Agent Instructions — Bitcoin Node Dashboard

## Project purpose

Bitcoin Node Dashboard is a local, read-only web interface for monitoring the user's Bitcoin Core node on Linux Mint. It is a separate project from Frost Lock Wallet. It must not manage wallets, keys, transactions, peers, Tor, Bisq, or Bitcoin Core configuration.

## Product language and naming

- Use US English in source, comments, documentation, UI, errors, logs, and handoff reports.
- Use the display name Bitcoin Node Dashboard.
- Use Bitcoin Core terminology precisely.
- Do not introduce Portuguese strings into the product.

## Architecture

- Use stable Rust, Edition 2024, and a conventional Cargo project layout.
- Use Axum with Tokio for the local HTTP service. Keep the browser interface in local static files with plain HTML, CSS, and JavaScript unless a documented need justifies a build tool.
- Keep node access behind a narrow read-only adapter.
- Use bitcoin-cli for authentication and RPC transport so Bitcoin Core's normal cookie handling remains in effect. Do not read or copy RPC credentials or cookies.
- Use serde and serde_json for typed RPC data.
- Keep presentation separate from node access and system sampling.
- Avoid Docker, databases, external APIs, CDN assets, and third-party runtime services.

## Security and privacy requirements

- Bind only to 127.0.0.1. Never bind to 0.0.0.0 or a LAN address.
- Use a fixed allowlist of read-only Bitcoin Core RPC methods. Never accept an RPC method, CLI argument, command, or executable path from an HTTP request.
- Launch bitcoin-cli with std::process::Command and separate arguments. Never use a shell to construct or execute commands.
- Never call mutating RPCs. Prohibited examples include disconnectnode, addnode, setban, sendrawtransaction, wallet methods, signing, key import, or transaction creation.
- Do not change bitcoin.conf, system services, firewall rules, Tor, Bisq, or Bitcoin Core settings.
- Do not expose RPC stderr, cookie paths, credentials, environment values, wallet names, wallet data, or peer IP addresses through the HTTP API or UI.
- Show aggregate peer counts only. Do not persist node or peer telemetry.
- Use textContent or equivalent safe DOM APIs for dynamic content. Never inject node-derived data with innerHTML.
- Serve only the local UI and documented API routes. Do not enable CORS.
- Do not add authentication secrets or listen on a non-loopback interface.
- Do not make network calls to external hosts.

## Diagnostics and logging

- Logs may contain lifecycle events and redacted error categories only.
- Never log raw RPC output, RPC stderr, node data, peer addresses, or configuration contents.
- Keep normal dashboard operation silent apart from startup, shutdown, and sanitized failures.
- The optional conversation log protocol is inactive unless the user explicitly issues the command reglog on. Reading LOG_RULES.md does not activate logging.

## Workflow and completion

- Inspect the current Git state before editing; preserve unrelated work.
- Keep README.md, CHANGELOG.md, docs, and implementation aligned.
- Add focused tests for RPC parsing, command allowlisting, error redaction, and web/API behavior.
- Validate with cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings, cargo build, cargo test, and a dependency audit where available.
- Do not claim connection to the user's node unless a live query succeeds.
- Report changed files, validation results, blockers, residual risks, and the exact command the Operator should use.
- Do not modify the Frost Lock Wallet repository or its status files.

