# Threat Model

## Scope and assets

Bitcoin Node Dashboard is a single-user, local-only, read-only monitor for Bitcoin Core. Assets include RPC credentials, wallet state, peer identities, node availability, and local system information.

## Controls

- Bind only to 127.0.0.1:8890; serve only local assets and documented API routes; no CORS or external network calls.
- Use a fixed allowlist of read-only Bitcoin Core methods via `bitcoin-cli`, argument arrays, normal cookie handling, a three-second RPC timeout, a four-second process guard, and streaming stdout/stderr caps.
- Never expose raw RPC output/stderr, cookie information, wallet data, peer addresses, or raw transactions.
- Block summaries are capped at six and checked for positive confirmations and matching active-chain height. Failures are categorized without exposing stderr; one failed card does not blank the rest. Selected transaction IDs are capped at 100.
- Do not fetch the full mempool. The bounded getblocktemplate request has a 3-second RPC timeout, a four-second process guard, and an 8 MB streaming output cap; it is aggregated, discarded, and never returned to the browser. Only aggregate candidate fee bands are cached for 55 seconds.
- Render dynamic values with text APIs. Persist no node telemetry.
- Shutdown is POST-only, requires a per-process random token and exact loopback Origin, and affects only this dashboard.

## Residual risks

Loopback access does not authenticate the user against other local processes. RPC requests can add load to a syncing node, although heavier views use low-frequency polling, fixed output caps, a measured template call, and a client timeout. A compromised host is outside this app's protection boundary.
