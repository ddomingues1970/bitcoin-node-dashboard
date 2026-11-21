# Implementation Plan

## Product goal

A polished, lightweight local dashboard for one Bitcoin Core node. The app is observational only and never controls the node.

## Delivered views

### Overview

Shows chain and synchronization status, Core version, aggregate inbound/outbound peer counts, local mempool totals, network byte rates, host CPU/memory/disk/uptime, and conservative fee estimates. Peer addresses and identifiers are never returned.

### Mempool

Shows this node's local `getmempoolinfo` count, BIP 141 virtual size (`bytes`), memory use/capacity, minimum fee context, and `estimatesmartfee` values converted from BTC/kvB to sat/vB. A bounded `getblocktemplate` request (3-second RPC timeout, four-second process guard, 8 MB streaming output cap) is aggregated into a local candidate summary and fee bands. The raw template is discarded, and only a fixed-size aggregate is returned and cached in memory for 55 seconds. Bands describe candidate-template transactions, not the whole mempool. The live local node returned a template in approximately 0.23 seconds with a response around 6.9 MB during validation. An incoming-transaction trend is omitted because mempool count deltas include removals.

### Blocks

Shows at most six recent blocks. Each candidate is checked for matching height/hash and positive confirmations before display. Per-block RPC failures are counted and do not discard successful cards. Cards include time/age, transaction count, total fees when available, and Core's non-coinbase transaction size/weight statistics. Details are lazy-loaded on selection; only up to 100 transaction IDs are returned. Raw transactions and miner identities are not requested or inferred.

## Refresh and operation

Overview refreshes every 15 seconds. Blocks and Mempool load when selected and refresh every 60 seconds while selected; manual refresh updates the active view. The API has fixed routes and fixed RPC methods. No endpoint accepts an arbitrary method or command. The shutdown endpoint remains POST-only and protected by the page's process token and exact loopback Origin.

## Security and acceptance

The server binds only to 127.0.0.1:8890. bitcoin-cli is invoked with fixed argument arrays, a short client timeout, and sanitized errors. Dynamic node data is rendered with text APIs. No CORS, remote assets, telemetry persistence, wallet methods, or mutation calls are present. Tests and validation are recorded in the project handoff.
