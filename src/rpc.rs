use serde::Serialize;
use serde_json::Value;
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const METHODS: &[&str] = &[
    "getblockchaininfo",
    "getnetworkinfo",
    "getpeerinfo",
    "getmempoolinfo",
    "getnettotals",
    "estimatesmartfee",
    "getblockcount",
    "getblockhash",
    "getblockheader",
    "getblockstats",
    "getblock",
    "getblocktemplate",
];
const MAX_RPC_OUTPUT: usize = 8_000_000;
const MAX_RPC_STDERR: usize = 8_192;
const RPC_PROCESS_TIMEOUT: Duration = Duration::from_secs(4);
const MEMPOOL_CACHE_TTL: Duration = Duration::from_secs(55);

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    UnavailableRpc,
    Timeout,
    UnsupportedMethod,
    PrunedData,
    MalformedResult,
    ActiveChainMismatch,
    OutputLimit,
}

#[derive(Default, Serialize)]
pub struct NodeData {
    pub reachable: bool,
    pub error: Option<&'static str>,
    pub version: Option<String>,
    pub chain: Option<String>,
    pub blocks: Option<u64>,
    pub headers: Option<u64>,
    pub progress: Option<f64>,
    pub ibd: Option<bool>,
    pub background_validation: Option<bool>,
    pub inbound: Option<usize>,
    pub outbound: Option<usize>,
    pub mempool_count: Option<u64>,
    pub mempool_vsize: Option<u64>,
    pub mempool_usage: Option<u64>,
    pub mempool_max: Option<u64>,
    pub upload: Option<u64>,
    pub download: Option<u64>,
    pub fees: Vec<FeeEstimate>,
}

#[derive(Clone, Serialize)]
pub struct FeeEstimate {
    pub target: u16,
    pub sat_per_vbyte: Option<f64>,
}

pub fn allowlisted(method: &str) -> bool {
    METHODS.contains(&method)
}

fn classify_stderr(stderr: &[u8]) -> DiagnosticCategory {
    let text = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    if text.contains("timeout") || text.contains("timed out") {
        DiagnosticCategory::Timeout
    } else if text.contains("pruned") {
        DiagnosticCategory::PrunedData
    } else if text.contains("method not found") || text.contains("unknown command") {
        DiagnosticCategory::UnsupportedMethod
    } else {
        DiagnosticCategory::UnavailableRpc
    }
}

fn parse_cli_output(method: &str, bytes: &[u8]) -> Result<Value, DiagnosticCategory> {
    if bytes.len() > MAX_RPC_OUTPUT {
        return Err(DiagnosticCategory::OutputLimit);
    }
    match serde_json::from_slice(bytes) {
        Ok(value) => Ok(value),
        Err(_) if matches!(method, "getblockhash" | "getbestblockhash") => {
            let raw = std::str::from_utf8(bytes)
                .map_err(|_| DiagnosticCategory::MalformedResult)?
                .trim();
            if raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_hexdigit()) {
                Ok(Value::String(raw.to_owned()))
            } else {
                Err(DiagnosticCategory::MalformedResult)
            }
        }
        Err(_) => Err(DiagnosticCategory::MalformedResult),
    }
}

fn read_bounded<R: Read>(mut stream: R, maximum: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut captured = Vec::with_capacity(maximum.min(64 * 1024));
    let mut buffer = [0_u8; 8192];
    let mut exceeded = false;
    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let available = maximum.saturating_sub(captured.len());
        let retain = read.min(available);
        captured.extend_from_slice(&buffer[..retain]);
        exceeded |= retain < read;
    }
    Ok((captured, exceeded))
}

fn call(method: &str, args: &[&str]) -> Result<Value, DiagnosticCategory> {
    if !allowlisted(method) {
        return Err(DiagnosticCategory::UnsupportedMethod);
    }
    let mut child = Command::new("bitcoin-cli")
        .arg("-rpcclienttimeout=3")
        .arg(method)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| DiagnosticCategory::UnavailableRpc)?;
    let stdout = child
        .stdout
        .take()
        .ok_or(DiagnosticCategory::UnavailableRpc)?;
    let stderr = child
        .stderr
        .take()
        .ok_or(DiagnosticCategory::UnavailableRpc)?;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, MAX_RPC_OUTPUT));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, MAX_RPC_STDERR));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(DiagnosticCategory::UnavailableRpc);
            }
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= RPC_PROCESS_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(DiagnosticCategory::Timeout);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
        }
    };
    let (stdout, stdout_exceeded) = stdout_reader
        .join()
        .map_err(|_| DiagnosticCategory::UnavailableRpc)?
        .map_err(|_| DiagnosticCategory::UnavailableRpc)?;
    let (stderr, _) = stderr_reader
        .join()
        .map_err(|_| DiagnosticCategory::UnavailableRpc)?
        .map_err(|_| DiagnosticCategory::UnavailableRpc)?;
    if stdout_exceeded {
        return Err(DiagnosticCategory::OutputLimit);
    }
    if !status.success() {
        return Err(classify_stderr(&stderr));
    }
    parse_cli_output(method, &stdout)
}

#[derive(Clone, Serialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub time: Option<u64>,
    pub confirmations: u64,
    pub tx_count: Option<u64>,
    pub total_fees_btc: Option<f64>,
    // getblockstats reports transaction statistics excluding the coinbase transaction.
    pub tx_size: Option<u64>,
    pub tx_weight: Option<u64>,
}

#[derive(Serialize)]
pub struct BlockDetails {
    pub block: BlockSummary,
    pub transactions: Vec<String>,
    pub transactions_truncated: bool,
}

#[derive(Serialize)]
pub struct BlocksResponse {
    pub updated_at: String,
    pub tip_height: Option<u64>,
    pub blocks: Vec<BlockSummary>,
    pub unavailable: bool,
    pub skipped_blocks: usize,
    pub diagnostic: Option<DiagnosticCategory>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CandidateTemplate {
    pub height: Option<u64>,
    pub transaction_count: u64,
    pub virtual_size: u64,
    pub total_fees_btc: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FeeBand {
    pub label: &'static str,
    pub transaction_count: u64,
    pub virtual_size: u64,
}

#[derive(Clone, Serialize)]
pub struct MempoolResponse {
    pub updated_at: String,
    pub local_node_notice: &'static str,
    pub transaction_count: Option<u64>,
    pub virtual_size: Option<u64>,
    pub memory_usage: Option<u64>,
    pub max_memory: Option<u64>,
    pub minimum_fee_sat_per_vbyte: Option<f64>,
    pub minimum_relay_fee_sat_per_vbyte: Option<f64>,
    pub status: Option<DiagnosticCategory>,
    pub template: Option<CandidateTemplate>,
    pub template_status: Option<DiagnosticCategory>,
    pub fee_bands: Vec<FeeBand>,
    pub fees: Vec<FeeEstimate>,
}

pub fn timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

fn block_at(height: u64, tip: u64) -> Result<BlockSummary, DiagnosticCategory> {
    if height > tip {
        return Err(DiagnosticCategory::ActiveChainMismatch);
    }
    // Only fields used by the summary are returned. The returned hash is checked
    // against getblockheader, whose positive confirmations prove active-chain membership.
    let height_arg = height.to_string();
    let fields = "[\"height\",\"totalfee\",\"blockhash\",\"txs\",\"total_size\",\"total_weight\"]";
    let stats = call("getblockstats", &[&height_arg, fields])?;
    let hash = stats
        .get("blockhash")
        .and_then(Value::as_str)
        .ok_or(DiagnosticCategory::MalformedResult)?
        .to_owned();
    let header = call("getblockheader", &[hash.as_str(), "true"])?;
    parse_block_values(height, tip, &stats, &header)
}

fn parse_block_values(
    height: u64,
    tip: u64,
    stats: &Value,
    header: &Value,
) -> Result<BlockSummary, DiagnosticCategory> {
    if height > tip {
        return Err(DiagnosticCategory::ActiveChainMismatch);
    }
    let hash = stats
        .get("blockhash")
        .and_then(Value::as_str)
        .ok_or(DiagnosticCategory::MalformedResult)?
        .to_owned();
    if stats.get("height").and_then(Value::as_u64) != Some(height) {
        return Err(DiagnosticCategory::ActiveChainMismatch);
    }
    let confirmations = header
        .get("confirmations")
        .and_then(Value::as_i64)
        .filter(|count| *count > 0)
        .ok_or(DiagnosticCategory::ActiveChainMismatch)? as u64;
    if header.get("height").and_then(Value::as_u64) != Some(height) {
        return Err(DiagnosticCategory::ActiveChainMismatch);
    }
    let fees = stats.get("totalfee").and_then(Value::as_f64);
    Ok(BlockSummary {
        height,
        hash,
        time: header.get("time").and_then(Value::as_u64),
        confirmations,
        tx_count: stats.get("txs").and_then(Value::as_u64),
        total_fees_btc: fees.map(|sats| sats / 100_000_000.0),
        tx_size: stats.get("total_size").and_then(Value::as_u64),
        tx_weight: stats.get("total_weight").and_then(Value::as_u64),
    })
}

fn collect_recent_blocks<F>(
    tip: u64,
    mut get_block: F,
) -> (Vec<BlockSummary>, usize, Option<DiagnosticCategory>)
where
    F: FnMut(u64) -> Result<BlockSummary, DiagnosticCategory>,
{
    let mut blocks = Vec::with_capacity(6);
    let mut skipped = 0;
    let mut diagnostic = None;
    for height in (tip.saturating_sub(5)..=tip).rev() {
        match get_block(height) {
            Ok(block) => blocks.push(block),
            Err(error) => {
                skipped += 1;
                diagnostic.get_or_insert(error);
            }
        }
    }
    (blocks, skipped, diagnostic)
}

pub fn recent_blocks() -> BlocksResponse {
    let mut blocks = Vec::with_capacity(6);
    let mut skipped_blocks = 0;
    let mut diagnostic = None;
    let tip = match call("getblockcount", &[])
        .and_then(|v| v.as_u64().ok_or(DiagnosticCategory::MalformedResult))
    {
        Ok(tip) => Some(tip),
        Err(error) => {
            diagnostic = Some(error);
            None
        }
    };
    if let Some(tip) = tip {
        (blocks, skipped_blocks, diagnostic) =
            collect_recent_blocks(tip, |height| block_at(height, tip));
    }
    BlocksResponse {
        updated_at: timestamp(),
        tip_height: tip,
        unavailable: blocks.is_empty(),
        blocks,
        skipped_blocks,
        diagnostic,
    }
}

pub fn block_details(height: u64) -> Result<BlockDetails, DiagnosticCategory> {
    let tip = call("getblockcount", &[])?
        .as_u64()
        .ok_or(DiagnosticCategory::MalformedResult)?;
    if height > tip {
        return Err(DiagnosticCategory::ActiveChainMismatch);
    }
    let summary = block_at(height, tip)?;
    let hash = summary.hash.as_str();
    let block = call("getblock", &[hash, "1"])?;
    if block.get("hash").and_then(Value::as_str) != Some(hash)
        || block.get("height").and_then(Value::as_u64) != Some(height)
        || block
            .get("confirmations")
            .and_then(Value::as_i64)
            .is_none_or(|confirmations| confirmations <= 0)
    {
        return Err(DiagnosticCategory::ActiveChainMismatch);
    }
    let (transactions, transactions_truncated) = bounded_txids(&block)?;
    Ok(BlockDetails {
        block: summary,
        transactions,
        transactions_truncated,
    })
}

fn bounded_txids(block: &Value) -> Result<(Vec<String>, bool), DiagnosticCategory> {
    let all = block
        .get("tx")
        .and_then(Value::as_array)
        .ok_or(DiagnosticCategory::MalformedResult)?;
    let transactions = all
        .iter()
        .take(100)
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    Ok((transactions, all.len() > 100))
}

fn btc_per_kvb_to_sat_per_vbyte(value: Option<f64>) -> Option<f64> {
    value
        .filter(|amount| amount.is_finite() && *amount > 0.0)
        .map(|btc_per_kvb| btc_per_kvb * 100_000.0)
}

fn fee_estimate(target: u16) -> FeeEstimate {
    let target_arg = target.to_string();
    FeeEstimate {
        target,
        sat_per_vbyte: call("estimatesmartfee", &[&target_arg])
            .ok()
            .and_then(|value| parse_fee_estimate(&value)),
    }
}

#[derive(Default)]
struct ParsedMempoolInfo {
    transaction_count: Option<u64>,
    virtual_size: Option<u64>,
    memory_usage: Option<u64>,
    max_memory: Option<u64>,
    minimum_fee_sat_per_vbyte: Option<f64>,
    minimum_relay_fee_sat_per_vbyte: Option<f64>,
}

fn parse_mempool_info(value: &Value) -> ParsedMempoolInfo {
    ParsedMempoolInfo {
        transaction_count: value.get("size").and_then(Value::as_u64),
        // `bytes` is Core's BIP 141 sum of virtual transaction sizes.
        virtual_size: value
            .get("bytes")
            .or_else(|| value.get("vsize"))
            .and_then(Value::as_u64),
        memory_usage: value.get("usage").and_then(Value::as_u64),
        max_memory: value.get("maxmempool").and_then(Value::as_u64),
        minimum_fee_sat_per_vbyte: btc_per_kvb_to_sat_per_vbyte(
            value.get("mempoolminfee").and_then(Value::as_f64),
        ),
        minimum_relay_fee_sat_per_vbyte: btc_per_kvb_to_sat_per_vbyte(
            value.get("minrelaytxfee").and_then(Value::as_f64),
        ),
    }
}

const BAND_UPPER_LIMITS: [f64; 8] = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, f64::INFINITY];
const BAND_LABELS: [&str; 8] = [
    "<1", "1–2", "2–5", "5–10", "10–20", "20–50", "50–100", "100+",
];

fn summarize_template(
    value: &Value,
) -> Result<(CandidateTemplate, Vec<FeeBand>), DiagnosticCategory> {
    let entries = value
        .get("transactions")
        .and_then(Value::as_array)
        .ok_or(DiagnosticCategory::MalformedResult)?;
    let mut counts = [0_u64; 8];
    let mut sizes = [0_u64; 8];
    let mut total_size = 0_u64;
    let mut total_fees = 0_u64;
    let mut all_fees_known = true;
    for entry in entries {
        let weight = entry.get("weight").and_then(Value::as_u64);
        if let Some(weight) = weight {
            total_size = total_size.saturating_add(weight.saturating_add(3) / 4);
        }
        let fee = entry
            .get("fee")
            .and_then(Value::as_i64)
            .filter(|fee| *fee >= 0)
            .map(|fee| fee as u64);
        match (fee, weight.filter(|weight| *weight > 0)) {
            (Some(fee), Some(weight)) => {
                total_fees = total_fees.saturating_add(fee);
                let vsize = weight.saturating_add(3) / 4;
                let rate = fee as f64 / vsize.max(1) as f64;
                if let Some(index) = BAND_UPPER_LIMITS.iter().position(|limit| rate < *limit) {
                    counts[index] = counts[index].saturating_add(1);
                    sizes[index] = sizes[index].saturating_add(vsize);
                }
            }
            _ => all_fees_known = false,
        }
    }
    let bands = (0..BAND_LABELS.len())
        .map(|index| FeeBand {
            label: BAND_LABELS[index],
            transaction_count: counts[index],
            virtual_size: sizes[index],
        })
        .collect();
    let template = CandidateTemplate {
        height: value.get("height").and_then(Value::as_u64),
        transaction_count: entries.len() as u64,
        virtual_size: total_size,
        total_fees_btc: all_fees_known.then_some(total_fees as f64 / 100_000_000.0),
    };
    Ok((template, bands))
}

#[derive(Default)]
struct MempoolCache {
    cached_at: Option<Instant>,
    response: Option<MempoolResponse>,
}
static MEMPOOL_CACHE: OnceLock<Mutex<MempoolCache>> = OnceLock::new();

pub fn mempool_snapshot() -> MempoolResponse {
    let cache = MEMPOOL_CACHE.get_or_init(|| Mutex::new(MempoolCache::default()));
    let Ok(mut guard) = cache.lock() else {
        return load_mempool_snapshot();
    };
    if let (Some(at), Some(response)) = (guard.cached_at, guard.response.as_ref())
        && at.elapsed() < MEMPOOL_CACHE_TTL
    {
        return response.clone();
    }
    // Keep the lock during the bounded fetch so simultaneous requests share one RPC snapshot.
    let response = load_mempool_snapshot();
    guard.cached_at = Some(Instant::now());
    guard.response = Some(response.clone());
    response
}

fn load_mempool_snapshot() -> MempoolResponse {
    let info = call("getmempoolinfo", &[]);
    let status = info.as_ref().err().copied();
    let info = info.ok();
    let totals = info.as_ref().map(parse_mempool_info).unwrap_or_default();
    let fees = [2, 6, 12].into_iter().map(fee_estimate).collect();
    // Live diagnostic measured <0.25 s and a 6.9 MB JSON payload. CLI timeout and output cap
    // bound it; only aggregated values leave this function, and the aggregate is cached.
    let template_result = call("getblocktemplate", &["{\"rules\":[\"segwit\"]}"])
        .and_then(|value| summarize_template(&value));
    let (template, fee_bands, template_status) = match template_result {
        Ok((template, bands)) => (Some(template), bands, None),
        Err(error) => (None, Vec::new(), Some(error)),
    };
    MempoolResponse {
        updated_at: timestamp(),
        local_node_notice: "This is this node's local mempool. Other nodes, including mempool.space, can have different transactions. Fee bands describe the transactions selected into this node's current template, not the entire mempool.",
        transaction_count: totals.transaction_count,
        virtual_size: totals.virtual_size,
        memory_usage: totals.memory_usage,
        max_memory: totals.max_memory,
        minimum_fee_sat_per_vbyte: totals.minimum_fee_sat_per_vbyte,
        minimum_relay_fee_sat_per_vbyte: totals.minimum_relay_fee_sat_per_vbyte,
        status,
        template,
        template_status,
        fee_bands,
        fees,
    }
}

fn parse_fee_estimate(value: &Value) -> Option<f64> {
    btc_per_kvb_to_sat_per_vbyte(value.get("feerate").and_then(Value::as_f64))
}

pub fn snapshot() -> NodeData {
    let mut data = NodeData::default();
    let (Ok(chain), Ok(network)) = (call("getblockchaininfo", &[]), call("getnetworkinfo", &[]))
    else {
        data.error = Some("Bitcoin Core is unavailable");
        return data;
    };
    data.reachable = true;
    let peers = call("getpeerinfo", &[]).ok();
    let mempool = call("getmempoolinfo", &[]).ok();
    let totals = call("getnettotals", &[]).ok();
    data.version = network
        .get("subversion")
        .and_then(Value::as_str)
        .map(|s| s.trim_matches('/').to_owned())
        .or_else(|| {
            network
                .get("version")
                .and_then(Value::as_i64)
                .map(|v| format!("Core {v}"))
        });
    data.chain = chain
        .get("chain")
        .and_then(Value::as_str)
        .map(str::to_owned);
    data.blocks = chain.get("blocks").and_then(Value::as_u64);
    data.headers = chain.get("headers").and_then(Value::as_u64);
    data.progress = chain.get("verificationprogress").and_then(Value::as_f64);
    data.ibd = chain.get("initialblockdownload").and_then(Value::as_bool);
    data.background_validation = chain
        .get("backgroundvalidation")
        .and_then(|value| value.get("progress"))
        .and_then(Value::as_f64)
        .map(|progress| progress < 1.0);
    if let Some(peers) = peers.as_ref().and_then(Value::as_array) {
        data.inbound = Some(
            peers
                .iter()
                .filter(|p| p.get("inbound").and_then(Value::as_bool) == Some(true))
                .count(),
        );
        data.outbound = Some(peers.len().saturating_sub(data.inbound.unwrap_or(0)));
    }
    data.mempool_count = mempool
        .as_ref()
        .and_then(|v| v.get("size"))
        .and_then(Value::as_u64);
    data.mempool_vsize = mempool.as_ref().and_then(|v| {
        v.get("bytes")
            .or_else(|| v.get("vsize"))
            .and_then(Value::as_u64)
    });
    data.mempool_usage = mempool
        .as_ref()
        .and_then(|v| v.get("usage"))
        .and_then(Value::as_u64);
    data.mempool_max = mempool
        .as_ref()
        .and_then(|v| v.get("maxmempool"))
        .and_then(Value::as_u64);
    data.upload = totals
        .as_ref()
        .and_then(|v| v.get("totalbytessent"))
        .and_then(Value::as_u64);
    data.download = totals
        .as_ref()
        .and_then(|v| v.get("totalbytesrecv"))
        .and_then(Value::as_u64);
    data.fees = [2, 6, 12].into_iter().map(fee_estimate).collect();
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_is_read_only_and_fixed() {
        assert!(allowlisted("getpeerinfo"));
        assert!(allowlisted("getblocktemplate"));
        assert!(!allowlisted("sendrawtransaction"));
        assert!(!allowlisted("getpeerinfo;anything"));
    }

    #[test]
    fn parses_bitcoin_cli_unquoted_hash_scalars() {
        let hash = "a".repeat(64);
        assert_eq!(
            parse_cli_output("getblockhash", hash.as_bytes()).unwrap(),
            Value::String(hash.clone())
        );
        assert_eq!(
            parse_cli_output("getblockhash", format!("{hash}\n").as_bytes()).unwrap(),
            Value::String(hash)
        );
        assert_eq!(
            parse_cli_output("getblockhash", b"not-a-hash"),
            Err(DiagnosticCategory::MalformedResult)
        );
    }

    #[test]
    fn fee_units_are_converted_and_zero_is_unavailable() {
        assert_eq!(btc_per_kvb_to_sat_per_vbyte(Some(1.31e-6)), Some(0.131));
        assert_eq!(btc_per_kvb_to_sat_per_vbyte(Some(0.0)), None);
        assert_eq!(btc_per_kvb_to_sat_per_vbyte(Some(-1.0)), None);
        assert_eq!(
            parse_fee_estimate(&serde_json::json!({"errors":["insufficient data"]})),
            None
        );
    }

    #[test]
    fn parses_core_mempool_virtual_size_from_bytes() {
        let info = serde_json::json!({"size": 7, "bytes": 1200, "usage": 2048, "maxmempool": 300000000,"mempoolminfee":0.000001});
        let parsed = parse_mempool_info(&info);
        assert_eq!(parsed.transaction_count, Some(7));
        assert_eq!(parsed.virtual_size, Some(1200));
        assert_eq!(parsed.memory_usage, Some(2048));
        assert!((parsed.minimum_fee_sat_per_vbyte.unwrap() - 0.1).abs() < 1e-10);
        let old = serde_json::json!({"size": 3, "vsize": 500});
        assert_eq!(parse_mempool_info(&old).virtual_size, Some(500));
    }

    #[test]
    fn block_parser_accepts_core_fixture_and_rejects_chain_mismatch() {
        let stats = serde_json::json!({"height":100,"blockhash":"abc","totalfee":123456,"txs":42,"total_size":9000,"total_weight":36000});
        let header = serde_json::json!({"height":100,"confirmations":2,"time":1700000000});
        let parsed = parse_block_values(100, 101, &stats, &header).unwrap();
        assert_eq!(parsed.tx_count, Some(42));
        assert_eq!(parsed.total_fees_btc, Some(0.00123456));
        assert_eq!(parsed.confirmations, 2);
        let fork = serde_json::json!({"height":100,"confirmations":-1,"time":1700000000});
        assert!(matches!(
            parse_block_values(100, 101, &stats, &fork),
            Err(DiagnosticCategory::ActiveChainMismatch)
        ));
        let wrong_height = serde_json::json!({"height":99,"blockhash":"abc","totalfee":1});
        assert!(matches!(
            parse_block_values(100, 101, &wrong_height, &header),
            Err(DiagnosticCategory::ActiveChainMismatch)
        ));
    }

    #[test]
    fn recent_block_collection_keeps_good_cards_when_one_rpc_call_fails() {
        let fixture = |height| BlockSummary {
            height,
            hash: format!("hash{height}"),
            time: Some(1_700_000_000),
            confirmations: 1,
            tx_count: Some(5),
            total_fees_btc: None,
            tx_size: Some(900),
            tx_weight: Some(3600),
        };
        let (blocks, skipped, diagnostic) = collect_recent_blocks(10, |height| {
            if height == 8 {
                Err(DiagnosticCategory::PrunedData)
            } else {
                Ok(fixture(height))
            }
        });
        assert_eq!(blocks.len(), 5);
        assert_eq!(skipped, 1);
        assert_eq!(diagnostic, Some(DiagnosticCategory::PrunedData));
    }

    #[test]
    fn template_aggregation_is_bounded_and_discards_transaction_hex() {
        let txs: Vec<Value> = (0..150)
            .map(|index| serde_json::json!({"fee":1000+index,"weight":400,"data":"sensitive-hex","txid":format!("id{index}")}))
            .collect();
        let template = serde_json::json!({"height":101,"transactions":txs});
        let (summary, bands) = summarize_template(&template).unwrap();
        assert_eq!(summary.height, Some(101));
        assert_eq!(summary.transaction_count, 150);
        assert_eq!(summary.virtual_size, 15000);
        assert_eq!(bands.len(), 8);
        let encoded =
            serde_json::to_string(&summary).unwrap() + &serde_json::to_string(&bands).unwrap();
        assert!(!encoded.contains("sensitive-hex"));
        assert!(!encoded.contains("id0"));
    }

    #[test]
    fn missing_template_fields_are_classified_as_malformed() {
        assert_eq!(
            summarize_template(&serde_json::json!({})).unwrap_err(),
            DiagnosticCategory::MalformedResult
        );
    }

    #[test]
    fn output_limits_and_diagnostics_are_sanitized() {
        assert_eq!(
            parse_cli_output("getblockcount", &vec![b'0'; MAX_RPC_OUTPUT + 1]),
            Err(DiagnosticCategory::OutputLimit)
        );
        assert_eq!(
            classify_stderr(b"RPC error: block not available (pruned data)"),
            DiagnosticCategory::PrunedData
        );
        assert_eq!(
            classify_stderr(b"error: timeout waiting for response"),
            DiagnosticCategory::Timeout
        );
        assert_eq!(
            classify_stderr(b"method not found"),
            DiagnosticCategory::UnsupportedMethod
        );
        let (captured, exceeded) = read_bounded(&b"0123456789"[..], 4).unwrap();
        assert_eq!(captured, b"0123");
        assert!(exceeded);
    }
}
