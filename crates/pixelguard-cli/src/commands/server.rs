//! Local HTTP server for the HTML report and its decisions API.
//!
//! The server binds to loopback only, but that alone does not stop a web page
//! from talking to it: DNS rebinding lets a malicious site reach 127.0.0.1
//! under its own host name, and any site can frame the report to trick clicks.
//! Every request is therefore checked for a loopback `Host`, writes also need a
//! same-origin `Origin`, and responses forbid framing.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_http::services::ServeDir;

use super::decisions::{DecisionsFile, DECISIONS_FILE};

/// Largest decisions payload the API accepts.
const MAX_DECISIONS_BYTES: usize = 1024 * 1024;

/// State shared by the server handlers.
struct ServerState {
    output_dir: PathBuf,
    port: u16,
}

/// Serves `output_dir` at `http://localhost:{port}/report.html` until interrupted.
///
/// # Errors
///
/// Returns an error if the port cannot be bound.
pub async fn serve_report(output_dir: &Path, port: u16, open_browser: bool) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
        .await
        .with_context(|| {
            format!(
                "Failed to bind to port {}. Use --port to choose another one.",
                port
            )
        })?;
    let port = listener.local_addr()?.port();
    let url = format!("http://localhost:{}/report.html", port);

    println!("🌐 Serving report at: {}", url);
    println!("   Press Ctrl+C to stop the server\n");

    if open_browser {
        if let Err(e) = open::that(&url) {
            eprintln!(
                "⚠️  Could not open browser automatically: {}\n   Open this URL manually: {}",
                e, url
            );
        }
    }

    axum::serve(listener, router(output_dir.to_path_buf(), port)).await?;
    Ok(())
}

/// Builds the report router with all security checks applied.
fn router(output_dir: PathBuf, port: u16) -> Router {
    let state = Arc::new(ServerState {
        output_dir: output_dir.clone(),
        port,
    });

    Router::new()
        .route("/api/decisions", get(load_decisions).post(save_decisions))
        .fallback_service(ServeDir::new(output_dir))
        .layer(DefaultBodyLimit::max(MAX_DECISIONS_BYTES))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state)
}

/// Rejects requests that did not come from the report itself.
async fn guard(State(state): State<Arc<ServerState>>, request: Request, next: Next) -> Response {
    let headers = request.headers();

    if !is_loopback_host(headers, state.port) {
        return (StatusCode::FORBIDDEN, "Forbidden host").into_response();
    }

    let is_read = matches!(*request.method(), Method::GET | Method::HEAD);
    if !is_read && !is_same_origin(headers, state.port) {
        return (StatusCode::FORBIDDEN, "Cross-origin request rejected").into_response();
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("frame-ancestors 'none'"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// True when `Host` names this server on a loopback address.
fn is_loopback_host(headers: &HeaderMap, port: u16) -> bool {
    let Some(host) = headers.get(header::HOST).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    loopback_authorities(port)
        .iter()
        .any(|allowed| host.eq_ignore_ascii_case(allowed))
}

/// True when `Origin` is this server's own origin.
fn is_same_origin(headers: &HeaderMap, port: u16) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    loopback_authorities(port)
        .iter()
        .any(|allowed| origin.eq_ignore_ascii_case(&format!("http://{}", allowed)))
}

fn loopback_authorities(port: u16) -> [String; 3] {
    [
        format!("localhost:{}", port),
        format!("127.0.0.1:{}", port),
        format!("[::1]:{}", port),
    ]
}

/// Validates and atomically stores decisions from the report.
async fn save_decisions(State(state): State<Arc<ServerState>>, body: String) -> Response {
    let decisions = match DecisionsFile::parse(&body) {
        Ok(decisions) => decisions,
        Err(error) => return (StatusCode::BAD_REQUEST, format!("{:#}", error)).into_response(),
    };

    match write_atomically(&state.output_dir, &decisions) {
        Ok(()) => StatusCode::OK.into_response(),
        Err(error) => {
            eprintln!("⚠️  Could not save decisions: {:#}", error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn write_atomically(output_dir: &Path, decisions: &DecisionsFile) -> Result<()> {
    let content = serde_json::to_string_pretty(decisions)?;
    let mut temp = tempfile::NamedTempFile::new_in(output_dir)?;
    std::io::Write::write_all(&mut temp, content.as_bytes())?;
    temp.persist(output_dir.join(DECISIONS_FILE))?;
    Ok(())
}

/// Returns stored decisions, or an empty set when none were saved yet.
async fn load_decisions(State(state): State<Arc<ServerState>>) -> Response {
    let path = state.output_dir.join(DECISIONS_FILE);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Json(serde_json::json!({
                "version": "1.0",
                "exportedAt": "",
                "decisions": {}
            }))
            .into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match DecisionsFile::parse(&content) {
        Ok(decisions) => Json(decisions).into_response(),
        Err(_) => Response::builder()
            .status(StatusCode::CONFLICT)
            .body(Body::from("Stored decisions.json is invalid"))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request as HttpRequest;
    use tower::ServiceExt;

    const PORT: u16 = 3333;

    fn app() -> (tempfile::TempDir, Router) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("report.html"), "<html></html>").unwrap();
        let router = router(dir.path().to_path_buf(), PORT);
        (dir, router)
    }

    fn approval_body() -> String {
        r#"{"version":"1.0","exportedAt":"x","decisions":{"a":{"action":"approve"}}}"#.to_string()
    }

    #[tokio::test]
    async fn rejects_foreign_host_headers() {
        let (_dir, app) = app();
        let response = app
            .oneshot(
                HttpRequest::get("/report.html")
                    .header("host", "attacker.example:3333")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn serves_report_to_loopback_with_anti_framing_headers() {
        let (_dir, app) = app();
        let response = app
            .oneshot(
                HttpRequest::get("/report.html")
                    .header("host", "localhost:3333")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::X_FRAME_OPTIONS], "DENY");
    }

    #[tokio::test]
    async fn rejects_cross_origin_writes() {
        let (dir, app) = app();
        let response = app
            .oneshot(
                HttpRequest::post("/api/decisions")
                    .header("host", "localhost:3333")
                    .header("origin", "http://attacker.example")
                    .header("content-type", "application/json")
                    .body(Body::from(approval_body()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(!dir.path().join(DECISIONS_FILE).exists());
    }

    #[tokio::test]
    async fn rejects_invalid_decisions() {
        let (dir, app) = app();
        let response = app
            .oneshot(
                HttpRequest::post("/api/decisions")
                    .header("host", "localhost:3333")
                    .header("origin", "http://localhost:3333")
                    .body(Body::from(r#"{"decisions":{"../x":{"action":"approve"}}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(!dir.path().join(DECISIONS_FILE).exists());
    }

    #[tokio::test]
    async fn saves_same_origin_decisions() {
        let (dir, app) = app();
        let response = app
            .oneshot(
                HttpRequest::post("/api/decisions")
                    .header("host", "127.0.0.1:3333")
                    .header("origin", "http://127.0.0.1:3333")
                    .header("content-type", "application/json")
                    .body(Body::from(approval_body()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let saved = std::fs::read_to_string(dir.path().join(DECISIONS_FILE)).unwrap();
        assert!(DecisionsFile::parse(&saved).is_ok());
    }
}
