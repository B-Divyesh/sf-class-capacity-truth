pub mod cookies;
pub mod db;
pub mod routes;

use std::{path::PathBuf, sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{header, HeaderName, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

pub const BUILD_SHA: &str = match option_env!("BUILD_SHA") {
    Some(value) => value,
    None => "dev",
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cookie_key: Arc<Vec<u8>>,
}

pub fn app(
    state: AppState,
    frontend_dist: PathBuf,
    rate_period_ms: u64,
    rate_burst: u32,
) -> Router {
    let mut builder = GovernorConfigBuilder::default();
    let limiter = builder
        .per_millisecond(rate_period_ms)
        .burst_size(rate_burst)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("positive rate limit values");
    let governor = GovernorLayer::new(limiter).error_handler(|error| {
        let (status, headers) = match error {
            GovernorError::TooManyRequests { headers, .. } => (StatusCode::TOO_MANY_REQUESTS, headers),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        };
        let mut response = (status, Json(json!({
            "code": if status == StatusCode::TOO_MANY_REQUESTS { "rate_limited" } else { "client_ip_unavailable" },
            "message": if status == StatusCode::TOO_MANY_REQUESTS { "Too many requests. Wait, then try again." } else { "The request could not be checked." }
        }))).into_response();
        if let Some(headers) = headers {
            response.headers_mut().extend(headers);
        }
        response.map(Body::new)
    });

    let mut hourly_builder = GovernorConfigBuilder::default();
    let hourly_limiter = hourly_builder
        .per_millisecond(120_000)
        .burst_size(30)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .expect("positive hourly demo rate limit");

    let api = Router::new()
        .route("/demo/session", get(routes::demo_session))
        .route("/demo/reset", post(routes::reset_demo))
        .route("/demo/leave", post(routes::leave_demo))
        .route("/demo/classes/{public_id}/book", post(routes::book))
        .layer(governor)
        .layer(GovernorLayer::new(hourly_limiter));

    let spa =
        ServeDir::new(&frontend_dist).fallback(ServeFile::new(frontend_dist.join("index.html")));

    Router::new()
        .route("/health", get(routes::health))
        .nest("/api", api)
        .fallback_service(spa)
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static(
                "camera=(), geolocation=(), microphone=(), payment=(), usb=()",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"),
        ))
        .layer(CorsLayer::new().allow_methods([Method::GET, Method::POST]).allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            origin.as_bytes().ends_with(b".sociobot.in") || origin.as_bytes() == b"https://class-capacity-truth.sociobot.in"
        })))
}

pub async fn cleanup_task(pool: SqlitePool) {
    let mut interval = tokio::time::interval(Duration::from_secs(15 * 60));
    loop {
        interval.tick().await;
        match db::cleanup_expired(&pool, routes::unix_now()).await {
            Ok(count) if count > 0 => {
                tracing::info!(expired_demo_tenants = count, "cleaned expired demos")
            }
            Ok(_) => {}
            Err(error) => tracing::warn!(error = %error, "demo cleanup failed"),
        }
    }
}
