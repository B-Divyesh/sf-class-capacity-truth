pub mod auth;
pub mod cookies;
pub mod crypto;
pub mod db;
pub mod jobs;
pub mod metrics;
pub mod routes;

use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    http::{header, HeaderName, HeaderValue, Method, Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post, put},
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
    pub contact_cipher: crypto::ContactCipher,
    pub auth: auth::AuthVerifier,
    pub public_base_url: Arc<String>,
    pub http: reqwest::Client,
    pub email_delivery_configured: bool,
    pub durable_backup_path: Option<Arc<PathBuf>>,
    pub backup_lock: Arc<tokio::sync::Mutex<()>>,
    pub metrics: metrics::AppMetrics,
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

    let demo_api = Router::new()
        .route("/demo/session", get(routes::demo_session))
        .route("/demo/reset", post(routes::reset_demo))
        .route("/demo/leave", post(routes::leave_demo))
        .route("/demo/classes/{public_id}/book", post(routes::book))
        .layer(governor)
        .layer(GovernorLayer::new(hourly_limiter));

    // A complete real-school flow makes several safe, related reads and writes
    // in one session. It has its own bounded allowance; the stricter demo
    // creation allowance remains unchanged for anonymous traffic.
    let mut school_builder = GovernorConfigBuilder::default();
    let school_limiter = school_builder
        .per_millisecond((rate_period_ms / 2).max(1))
        .burst_size(rate_burst.max(40))
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("positive school rate limit values");
    let school_api = Router::new()
        .route("/runtime", get(routes::runtime_status))
        .route("/metrics", get(routes::workspace_metrics))
        .route("/workspaces/metrics", get(routes::workspace_metrics))
        .route(
            "/workspaces",
            post(routes::create_workspace).get(routes::current_workspace),
        )
        .route(
            "/workspaces/classes",
            get(routes::list_classes).post(routes::create_class),
        )
        .route(
            "/workspaces/classes/{id}/bookings",
            get(routes::list_bookings),
        )
        .route(
            "/workspaces/classes/{id}/publish",
            post(routes::publish_class),
        )
        .route(
            "/workspaces/classes/{id}/reconcile",
            post(routes::reconcile),
        )
        .route(
            "/workspaces/classes/{class_id}/bookings/{booking_id}/cancel",
            post(routes::cancel_and_offer),
        )
        .route(
            "/workspaces/classes/{class_id}/release-seat",
            post(routes::release_oldest_and_offer),
        )
        .route("/workspaces/offers", get(routes::list_offer_receipts))
        .route("/workspaces/calendar", put(routes::connect_calendar))
        .route("/workspaces/calendar/check", post(routes::check_calendar))
        .route("/workspaces/export", get(routes::export_workspace))
        .route(
            "/workspaces/data",
            axum::routing::delete(routes::delete_workspace),
        )
        .route("/workspaces/billing/verify", post(routes::verify_billing))
        .route("/classes/{public_id}", get(routes::public_class))
        .route("/classes/{public_id}/book", post(routes::real_book))
        .route(
            "/classes/{public_id}/waitlist",
            post(routes::join_real_waitlist),
        )
        .route("/offers/{token}", get(routes::view_offer))
        .route("/offers/{token}/accept", post(routes::accept_seat_offer))
        .layer(GovernorLayer::new(school_limiter));
    let api = demo_api.merge(school_api);

    let index = frontend_dist.join("index.html");
    let static_files = ServeDir::new(&frontend_dist);

    Router::new()
        .route("/health", get(routes::health))
        .route("/metrics", get(routes::workspace_metrics))
        .route_service("/", ServeFile::new(index.clone()))
        .route_service("/demo", ServeFile::new(index.clone()))
        .route_service("/privacy", ServeFile::new(index.clone()))
        .route_service("/terms", ServeFile::new(index.clone()))
        .route_service("/app", ServeFile::new(index.clone()))
        .route_service("/app/classes/{id}", ServeFile::new(index.clone()))
        .route_service("/app/reconciliation", ServeFile::new(index.clone()))
        .route_service("/app/waitlist", ServeFile::new(index.clone()))
        .route_service("/app/settings", ServeFile::new(index.clone()))
        .route_service("/app/settings/billing", ServeFile::new(index.clone()))
        .route_service("/app/settings/data", ServeFile::new(index.clone()))
        .route_service("/app/operations", ServeFile::new(index.clone()))
        .route_service("/auth/callback", ServeFile::new(index.clone()))
        .route_service("/book/{id}", ServeFile::new(index.clone()))
        .route_service("/offer/{token}", ServeFile::new(index))
        .route_service("/assets/{*path}", static_files.clone())
        .route_service("/favicon.svg", ServeFile::new(frontend_dist.join("favicon.svg")))
        .route_service("/apple-touch-icon.svg", ServeFile::new(frontend_dist.join("apple-touch-icon.svg")))
        .route_service("/social-card.svg", ServeFile::new(frontend_dist.join("social-card.svg")))
        .route_service("/foundation-404.css", ServeFile::new(frontend_dist.join("foundation-404.css")))
        .route_service("/robots.txt", ServeFile::new(frontend_dist.join("robots.txt")))
        .route_service("/sitemap.xml", ServeFile::new(frontend_dist.join("sitemap.xml")))
        .nest("/api", api)
        .fallback(get(routes::not_found_page))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            persist_successful_mutation,
        ))
        .layer(middleware::from_fn_with_state(
            state,
            record_metrics,
        ))
        .layer(middleware::from_fn(cache_headers))
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
            HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self' https://sociobotcustomers.ciamlogin.com https://login.microsoftonline.com https://api.sociobot.in; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"),
        ))
        .layer(CorsLayer::new().allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]).allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            origin.as_bytes().ends_with(b".sociobot.in") || origin.as_bytes() == b"https://class-capacity-truth.sociobot.in"
        })))
}

async fn record_metrics(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let route = metrics::route_group(request.uri().path());
    let started = Instant::now();
    let response = next.run(request).await;
    state
        .metrics
        .record(route, response.status(), started.elapsed());
    response
}

async fn persist_successful_mutation(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let persist = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let response = next.run(request).await;
    if persist && response.status().is_success() {
        if let Some(path) = state.durable_backup_path.as_deref() {
            let _guard = state.backup_lock.lock().await;
            if let Err(error) = db::persist_durable_snapshot(&state.pool, path).await {
                tracing::error!(error = %error, "durable snapshot failed");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "code": "durability_failed",
                        "message": "The change could not be stored durably. Try again."
                    })),
                )
                    .into_response();
            }
        }
    }
    response
}

async fn cache_headers(request: Request<Body>, next: Next) -> axum::response::Response {
    let assets = request.uri().path().starts_with("/assets/");
    let mut response = next.run(request).await;
    if response.status().is_success() {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static(if assets {
                "public, max-age=31536000, immutable"
            } else {
                "no-cache, max-age=0"
            }),
        );
    }
    response
}

async fn persist_if_configured(state: &AppState) {
    if let Some(path) = state.durable_backup_path.as_deref() {
        let _guard = state.backup_lock.lock().await;
        if let Err(error) = db::persist_durable_snapshot(&state.pool, path).await {
            tracing::error!(error = %error, "background durable snapshot failed");
        }
    }
}

pub async fn cleanup_task(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(15 * 60));
    loop {
        interval.tick().await;
        match db::cleanup_expired(&state.pool, routes::unix_now()).await {
            Ok(count) if count > 0 => {
                tracing::info!(expired_demo_tenants = count, "cleaned expired demos")
            }
            Ok(_) => {}
            Err(error) => tracing::warn!(error = %error, "demo cleanup failed"),
        }
        if let Err(error) = db::cleanup_retained_contacts(&state.pool, routes::unix_now()).await {
            tracing::warn!(error = %error, "contact retention cleanup failed");
        }
        persist_if_configured(&state).await;
    }
}

pub async fn integration_task(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = routes::unix_now();
        if let Err(error) =
            jobs::poll_due_calendars(&state.pool, &state.contact_cipher, &state.http, now).await
        {
            tracing::warn!(error = %error, "calendar polling failed");
        }
        if let Err(error) = jobs::deliver_due_email(&state.pool, &state.contact_cipher, now).await {
            tracing::warn!(error = %error, "email delivery failed");
        }
        persist_if_configured(&state).await;
    }
}
