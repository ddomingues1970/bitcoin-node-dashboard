# Architecture

## Overview

A single Rust process serves local static assets and JSON routes. `rpc.rs` owns the fixed Bitcoin Core method allowlist and response shaping; `metrics.rs` samples Linux host data; `web.rs` serves the API and UI. The browser renders with plain JavaScript and safe text APIs.

## Data flow

    Browser -> local Axum routes -> fixed bitcoin-cli invocations -> Bitcoin Core local RPC
                                  -> Linux /proc and statvfs host sampling

The server binds exclusively to 127.0.0.1:8890. It does not enable CORS or contact external hosts. No telemetry is persisted.

## RPC behavior

RPC methods are fixed in the adapter and never supplied by the browser. Calls use `std::process::Command` with argument arrays, Bitcoin Core's normal authentication, a three-second CLI RPC timeout, a four-second process guard, an 8 MB streaming stdout cap, and sanitized diagnostic categories. bitcoin-cli hash results are parsed as bare hex scalars when they are not JSON-quoted; this fixes the former empty Blocks response. The Overview uses fixed status calls. Blocks requests at most six compact `getblockstats` summaries, obtains headers by returned hash, and requires matching height and positive confirmations. It preserves valid cards if another block fails. Size and weight describe non-coinbase transactions. Transaction IDs load only for a selected height and are capped at 100. Mempool reads local aggregate fields and three fee estimates; `bytes` is used for BIP 141 virtual size, with a compatibility fallback. A `getblocktemplate` response (maximum 8 MB, three-second client timeout/four-second process guard) is immediately reduced to candidate totals and eight fixed fee bands. Only the aggregate is cached in memory for 55 seconds and serialized; transaction hex, IDs, and peer metadata are not exposed.

## HTTP and shutdown

Routes serve `/`, local CSS/JS, `/api/status`, `/api/blocks`, `/api/blocks/{height}`, `/api/mempool`, and the protected `/api/shutdown`. Block height is parsed as an integer and used only with fixed allowlisted calls; no RPC method or command is accepted. Shutdown is POST-only, requires exact loopback Origin and a per-process OS-random token, and gracefully stops Axum only.

Overview polls every 15 seconds. The selected Mempool or Blocks view polls every 60 seconds and is loaded lazily. Mempool aggregates are cached in memory for 55 seconds. An incoming-transaction chart is omitted because count deltas cannot distinguish arrivals from removals. Logs contain lifecycle events and sanitized error categories only.
