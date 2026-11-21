mod metrics;
mod rpc;
mod web;

use std::{net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8890));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let token = shutdown_token()?;
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    eprintln!("Bitcoin Node Dashboard started on 127.0.0.1:8890");
    axum::serve(listener, web::router(Arc::new(token), shutdown_tx))
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = shutdown_rx.changed() => {},
            }
            eprintln!("Bitcoin Node Dashboard stopped");
        })
        .await?;
    Ok(())
}

fn shutdown_token() -> std::io::Result<String> {
    let mut bytes = [0_u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

use std::io::Read;
