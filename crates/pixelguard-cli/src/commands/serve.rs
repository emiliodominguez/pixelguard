//! The `serve` command for serving an existing report.
//!
//! This command serves the HTML report with the decisions API,
//! allowing you to review and approve changes without re-running tests.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Args;
use tower_http::services::ServeDir;

/// Arguments for the serve command.
#[derive(Args)]
pub struct ServeArgs {
    /// Path to config file (default: pixelguard.config.json)
    #[arg(long, short)]
    config: Option<String>,

    /// Port for the local web server
    #[arg(long, default_value = "3333")]
    port: u16,
}

/// State for the serve endpoints.
#[derive(Clone)]
struct ServeState {
    output_dir: PathBuf,
}

/// Runs the serve command.
pub async fn run(args: ServeArgs) -> Result<()> {
    let working_dir = std::env::current_dir()?;

    // Load config to get output_dir
    let config = super::load_config(&working_dir, args.config.as_deref())?;
    let output_dir = working_dir.join(&config.output_dir);

    // Check if report exists
    let report_path = output_dir.join("report.html");

    if !report_path.exists() {
        anyhow::bail!(
            "No report found at '{}'. Run 'pixelguard test' first.",
            report_path.display()
        );
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let url = format!("http://localhost:{}/report.html", args.port);

    let state = Arc::new(ServeState {
        output_dir: output_dir.clone(),
    });

    let app = Router::new()
        .route("/api/decisions", post(save_decisions))
        .route("/api/decisions", get(load_decisions))
        .with_state(state)
        .fallback_service(ServeDir::new(&output_dir));

    println!("Serving report at: {}", url);
    println!("Press Ctrl+C to stop the server\n");

    // Open browser
    if let Err(e) = open::that(&url) {
        eprintln!("Could not open browser: {}. Open {} manually.", e, url);
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind to port {}", args.port))?;

    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler to save decisions to the output directory.
async fn save_decisions(
    State(state): State<Arc<ServeState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let decisions_path = state.output_dir.join("decisions.json");

    let content = serde_json::to_string_pretty(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    std::fs::write(&decisions_path, content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Handler to load decisions from the output directory.
async fn load_decisions(
    State(state): State<Arc<ServeState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let decisions_path = state.output_dir.join("decisions.json");

    if !decisions_path.exists() {
        return Ok(Json(serde_json::json!({
            "version": "1.0",
            "exportedAt": "",
            "decisions": {}
        })));
    }

    let content = std::fs::read_to_string(&decisions_path).map_err(|_| StatusCode::NOT_FOUND)?;

    let data: serde_json::Value =
        serde_json::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(data))
}
