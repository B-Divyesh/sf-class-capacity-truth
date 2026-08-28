use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{cookies, db, AppState, BUILD_SHA};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: &'static str,
    build: &'static str,
    database: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoResponse {
    school_name: &'static str,
    expires_at: i64,
    classes: Vec<db::ClassSession>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookingRequest {
    guardian_name: String,
    guardian_email: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

pub async fn health(State(state): State<AppState>) -> Response {
    let database = if sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        "ready"
    } else {
        "unavailable"
    };
    let status = if database == "ready" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(HealthResponse {
            status: if status == StatusCode::OK {
                "ok"
            } else {
                "error"
            },
            build: BUILD_SHA,
            database,
        }),
    )
        .into_response()
}

pub async fn demo_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let now = unix_now();
    let (tenant_id, new_cookie) = cookies::tenant_from_headers_or_new(&headers, &state.cookie_key);
    if let Err(error) = db::create_or_refresh_demo(&state.pool, &tenant_id, now).await {
        tracing::error!(error = %error, "failed to initialize demo");
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "demo_unavailable",
            "The sample could not load. Try again.",
        );
    }
    match demo_payload(&state, &tenant_id, now).await {
        Ok(payload) => with_optional_cookie(
            Json(payload).into_response(),
            new_cookie,
            &tenant_id,
            &headers,
            &state.cookie_key,
        ),
        Err(error) => {
            tracing::error!(error = %error, "failed to list demo classes");
            api_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "demo_unavailable",
                "The sample could not load. Try again.",
            )
        }
    }
}

pub async fn reset_demo(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let now = unix_now();
    let Some(tenant_id) = cookies::verified_tenant(&headers, &state.cookie_key) else {
        return api_error(
            StatusCode::UNAUTHORIZED,
            "demo_cookie_missing",
            "Reload the sample before resetting it.",
        );
    };
    if let Err(error) = db::reset_demo(&state.pool, &tenant_id, now).await {
        tracing::error!(error = %error, "failed to reset demo");
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "reset_failed",
            "The sample did not reset. Try again.",
        );
    }
    match demo_payload(&state, &tenant_id, now).await {
        Ok(payload) => Json(payload).into_response(),
        Err(_) => api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "reset_failed",
            "The sample did not reset. Try again.",
        ),
    }
}

pub async fn leave_demo(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(tenant_id) = cookies::verified_tenant(&headers, &state.cookie_key) {
        if let Err(error) = db::destroy_demo(&state.pool, &tenant_id).await {
            tracing::warn!(error = %error, "failed to destroy demo while leaving");
        }
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("cct_demo=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0"),
    );
    response
}

pub async fn book(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<BookingRequest>,
) -> Response {
    let Some(tenant_id) = cookies::verified_tenant(&headers, &state.cookie_key) else {
        return api_error(
            StatusCode::UNAUTHORIZED,
            "demo_cookie_missing",
            "Reload the sample before booking.",
        );
    };
    let Some(idempotency_key) = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| (8..=100).contains(&value.len()))
    else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "idempotency_key_required",
            "Reload the form, then book again.",
        );
    };

    let name = request.guardian_name.trim();
    let email = request.guardian_email.trim().to_ascii_lowercase();
    if !(2..=80).contains(&name.chars().count()) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_name",
            "Enter a guardian name between 2 and 80 characters.",
        );
    }
    if email.len() > 254 || !is_reasonable_email(&email) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_email",
            "Enter an email address such as name@example.org.",
        );
    }

    match db::book_seat(
        &state.pool,
        &tenant_id,
        &public_id,
        idempotency_key,
        name,
        &email,
        unix_now(),
    )
    .await
    {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(db::BookingError::NotFound) => api_error(
            StatusCode::NOT_FOUND,
            "class_not_found",
            "This sample class is not available. Reset the demo.",
        ),
        Err(db::BookingError::Full) => api_error(
            StatusCode::CONFLICT,
            "class_full",
            "This class is full. Choose another sample class.",
        ),
        Err(db::BookingError::Cutoff) => api_error(
            StatusCode::CONFLICT,
            "booking_closed",
            "The booking cutoff has passed. Choose another sample class.",
        ),
        Err(db::BookingError::Database) => api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "booking_unavailable",
            "The booking could not be saved. Try again.",
        ),
    }
}

async fn demo_payload(state: &AppState, tenant_id: &str, now: i64) -> anyhow::Result<DemoResponse> {
    Ok(DemoResponse {
        school_name: "Bright Path Languages",
        expires_at: now + 86_400,
        classes: db::list_sessions(&state.pool, tenant_id, now).await?,
    })
}

fn with_optional_cookie(
    mut response: Response,
    new_cookie: bool,
    tenant_id: &str,
    headers: &HeaderMap,
    key: &[u8],
) -> Response {
    if new_cookie {
        let secure = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            == Some("https");
        if let Ok(value) = HeaderValue::from_str(&cookies::set_cookie_value(tenant_id, key, secure))
        {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}

fn api_error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (status, Json(ErrorBody { code, message })).into_response()
}

fn is_reasonable_email(value: &str) -> bool {
    let mut parts = value.split('@');
    matches!((parts.next(), parts.next(), parts.next()), (Some(local), Some(domain), None) if !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.'))
}

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
