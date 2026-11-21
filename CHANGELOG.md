# Changelog

All notable project changes are recorded here.

## Unreleased

- Added accessible Overview, Mempool, and Blocks tabs with view-specific refresh cadence.
- Added capped active-chain block summaries with per-card failure diagnostics, reorganization checks, and lazy selected-block transaction ID lists.
- Fixed parsing of bitcoin-cli bare hash output, Core 32.1 mempool virtual-size field handling, and BTC/kvB-to-sat/vB conversion. Added cached local template summaries and bounded candidate fee bands without returning transaction hex or IDs.
- Reworked Mempool and Blocks layouts with local candidate fee bands and recent confirmed cards; retained the same-origin, per-process-token shutdown action and local-only read-only boundary.
- Added the Axum loopback service, fixed bitcoin-cli calls, sanitized status model, Linux host summaries, and responsive local dashboard.
