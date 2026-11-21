use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    token: Arc<String>,
    shutdown: tokio::sync::watch::Sender<bool>,
    data: Arc<dyn DashboardDataSource>,
}

trait DashboardDataSource: Send + Sync {
    fn node(&self) -> crate::rpc::NodeData;
    fn host(&self) -> crate::metrics::HostMetrics;
    fn blocks(&self) -> crate::rpc::BlocksResponse;
    fn block(
        &self,
        height: u64,
    ) -> Result<crate::rpc::BlockDetails, crate::rpc::DiagnosticCategory>;
    fn mempool(&self) -> crate::rpc::MempoolResponse;
}

struct LiveDataSource;

impl DashboardDataSource for LiveDataSource {
    fn node(&self) -> crate::rpc::NodeData {
        crate::rpc::snapshot()
    }
    fn host(&self) -> crate::metrics::HostMetrics {
        crate::metrics::sample()
    }
    fn blocks(&self) -> crate::rpc::BlocksResponse {
        crate::rpc::recent_blocks()
    }
    fn block(
        &self,
        height: u64,
    ) -> Result<crate::rpc::BlockDetails, crate::rpc::DiagnosticCategory> {
        crate::rpc::block_details(height)
    }
    fn mempool(&self) -> crate::rpc::MempoolResponse {
        crate::rpc::mempool_snapshot()
    }
}

#[derive(Serialize)]
struct Status {
    updated_at: String,
    node: crate::rpc::NodeData,
    host: crate::metrics::HostMetrics,
}
async fn index(State(state): State<AppState>) -> Response {
    let html = include_str!("../static/index.html").replace("__SHUTDOWN_TOKEN__", &state.token);
    ([(header::CACHE_CONTROL, "no-store")], Html(html)).into_response()
}
async fn css() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/styles.css"),
    )
}
async fn js() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/javascript; charset=utf-8",
        )],
        include_str!("../static/app.js"),
    )
}
async fn status(State(state): State<AppState>) -> Json<Status> {
    let updated_at = crate::rpc::timestamp();
    Json(Status {
        updated_at,
        node: state.data.node(),
        host: state.data.host(),
    })
}
async fn blocks(State(state): State<AppState>) -> Json<crate::rpc::BlocksResponse> {
    Json(state.data.blocks())
}
async fn block(
    State(state): State<AppState>,
    Path(height): Path<u64>,
) -> Result<Json<crate::rpc::BlockDetails>, StatusCode> {
    state
        .data
        .block(height)
        .map(Json)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}
async fn mempool(State(state): State<AppState>) -> Json<crate::rpc::MempoolResponse> {
    Json(state.data.mempool())
}
async fn shutdown(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let origin_ok = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        == Some("http://127.0.0.1:8890");
    let token_ok = headers
        .get("x-dashboard-shutdown-token")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|provided| constant_time_eq(provided.as_bytes(), state.token.as_bytes()));
    if !origin_ok || !token_ok {
        return StatusCode::FORBIDDEN;
    }
    state.shutdown.send_replace(true);
    StatusCode::ACCEPTED
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

pub fn router(token: Arc<String>, shutdown_tx: tokio::sync::watch::Sender<bool>) -> Router {
    router_with_source(token, shutdown_tx, Arc::new(LiveDataSource))
}

fn router_with_source(
    token: Arc<String>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    data: Arc<dyn DashboardDataSource>,
) -> Router {
    let state = AppState {
        token,
        shutdown: shutdown_tx,
        data,
    };
    Router::new()
        .route("/", get(index))
        .route("/styles.css", get(css))
        .route("/app.js", get(js))
        .route("/api/status", get(status))
        .route("/api/blocks", get(blocks))
        .route("/api/blocks/{height}", get(block))
        .route("/api/mempool", get(mempool))
        .route("/api/shutdown", post(shutdown))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use serde_json::Value;
    use tower::ServiceExt;

    struct FixtureData;

    impl FixtureData {
        fn summary() -> crate::rpc::BlockSummary {
            crate::rpc::BlockSummary {
                height: 123,
                hash: "a".repeat(64),
                time: Some(1_700_000_000),
                confirmations: 3,
                tx_count: Some(50),
                total_fees_btc: Some(0.001),
                tx_size: Some(12_000),
                tx_weight: Some(48_000),
            }
        }
    }

    impl DashboardDataSource for FixtureData {
        fn node(&self) -> crate::rpc::NodeData {
            crate::rpc::NodeData {
                reachable: true,
                version: Some("Bitcoin Core 32.1".to_owned()),
                ..Default::default()
            }
        }
        fn host(&self) -> crate::metrics::HostMetrics {
            crate::metrics::HostMetrics::default()
        }
        fn blocks(&self) -> crate::rpc::BlocksResponse {
            crate::rpc::BlocksResponse {
                updated_at: "1".to_owned(),
                tip_height: Some(125),
                blocks: vec![Self::summary()],
                unavailable: false,
                skipped_blocks: 0,
                diagnostic: None,
            }
        }
        fn block(
            &self,
            _: u64,
        ) -> Result<crate::rpc::BlockDetails, crate::rpc::DiagnosticCategory> {
            Ok(crate::rpc::BlockDetails {
                block: Self::summary(),
                transactions: vec!["fixture-txid".to_owned()],
                transactions_truncated: false,
            })
        }
        fn mempool(&self) -> crate::rpc::MempoolResponse {
            crate::rpc::MempoolResponse {
                updated_at: "1".to_owned(),
                local_node_notice: "Fixture data is local to this node.",
                transaction_count: Some(5),
                virtual_size: Some(2_000),
                memory_usage: Some(4_000),
                max_memory: Some(8_000),
                minimum_fee_sat_per_vbyte: Some(0.1),
                minimum_relay_fee_sat_per_vbyte: Some(0.1),
                status: None,
                template: Some(crate::rpc::CandidateTemplate {
                    height: Some(126),
                    transaction_count: 4,
                    virtual_size: 1_000,
                    total_fees_btc: Some(0.0001),
                }),
                template_status: None,
                fee_bands: vec![crate::rpc::FeeBand {
                    label: "1–2",
                    transaction_count: 4,
                    virtual_size: 1_000,
                }],
                fees: vec![crate::rpc::FeeEstimate {
                    target: 2,
                    sat_per_vbyte: Some(1.25),
                }],
            }
        }
    }

    async fn get_json(app: &Router, uri: &str) -> (StatusCode, Value) {
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn routes_serve_ui_and_fixture_backed_data_apis() {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        let app = router_with_source(
            Arc::new("test-secret".to_owned()),
            tx,
            Arc::new(FixtureData),
        );
        let page = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page.status(), 200);
        assert_eq!(page.headers()[header::CACHE_CONTROL], "no-store");
        let page_html = axum::body::to_bytes(page.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(
            std::str::from_utf8(&page_html)
                .unwrap()
                .contains("name=\"dashboard-shutdown-token\" content=\"test-secret\"")
        );
        let (status_code, status) = get_json(&app, "/api/status").await;
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(status["node"]["reachable"], true);
        assert!(status.get("host").is_some());
        assert!(!status.to_string().contains("test-secret"));

        let (status_code, blocks) = get_json(&app, "/api/blocks").await;
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(blocks["blocks"].as_array().unwrap().len(), 1);
        assert_eq!(blocks["blocks"][0]["confirmations"], 3);

        let (status_code, detail) = get_json(&app, "/api/blocks/123").await;
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(detail["transactions"].as_array().unwrap().len(), 1);

        let (status_code, mempool) = get_json(&app, "/api/mempool").await;
        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(mempool["virtual_size"], 2_000);
        assert_eq!(mempool["template"]["transaction_count"], 4);
        assert!(!mempool.to_string().contains("fixture-txid"));
    }

    async fn shutdown_request(origin: Option<&str>, token: Option<&str>) -> (StatusCode, bool) {
        let (tx, mut rx) = tokio::sync::watch::channel(false);
        let app = router(Arc::new("test-secret".to_owned()), tx);
        let mut builder = axum::http::Request::builder()
            .method("POST")
            .uri("/api/shutdown");
        if let Some(origin) = origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        if let Some(token) = token {
            builder = builder.header("x-dashboard-shutdown-token", token);
        }
        let response = app
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        (response.status(), *rx.borrow_and_update())
    }

    #[tokio::test]
    async fn shutdown_rejects_missing_or_invalid_token() {
        assert_eq!(
            shutdown_request(Some("http://127.0.0.1:8890"), None).await,
            (StatusCode::FORBIDDEN, false)
        );
        assert_eq!(
            shutdown_request(Some("http://127.0.0.1:8890"), Some("wrong")).await,
            (StatusCode::FORBIDDEN, false)
        );
    }

    #[tokio::test]
    async fn shutdown_rejects_missing_or_foreign_origin() {
        assert_eq!(
            shutdown_request(None, Some("test-secret")).await,
            (StatusCode::FORBIDDEN, false)
        );
        assert_eq!(
            shutdown_request(Some("http://localhost:8890"), Some("test-secret")).await,
            (StatusCode::FORBIDDEN, false)
        );
        assert_eq!(
            shutdown_request(Some("https://127.0.0.1:8890"), Some("test-secret")).await,
            (StatusCode::FORBIDDEN, false)
        );
    }

    #[tokio::test]
    async fn authorized_shutdown_signals_dashboard_once() {
        assert_eq!(
            shutdown_request(Some("http://127.0.0.1:8890"), Some("test-secret")).await,
            (StatusCode::ACCEPTED, true)
        );
    }

    #[tokio::test]
    async fn shutdown_get_request_does_not_signal() {
        let (tx, mut rx) = tokio::sync::watch::channel(false);
        let app = router(Arc::new("test-secret".to_owned()), tx);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/shutdown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert!(!*rx.borrow_and_update());
    }
}
