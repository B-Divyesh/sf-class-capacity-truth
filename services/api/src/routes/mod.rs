use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceRequest { school_name: String }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassRequest { name: String, starts_at: i64, booking_cutoff: i64, timezone: String, capacity: i64 }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarRequest { label: String }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReconcileRequest { calendar_confirmed: i64 }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaitlistRequest { guardian_name: String, guardian_email: String, consent: bool }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceResponse { workspace: db::Workspace, access_key: String }

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

pub async fn not_found_page() -> Response {
    (StatusCode::NOT_FOUND, Html(include_str!("../../../../404.html"))).into_response()
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

fn access_key(headers: &HeaderMap) -> Result<&str, Response> {
    headers.get("x-workspace-key").and_then(|value| value.to_str().ok()).filter(|value| value.len() >= 24)
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before changing classes."))
}

fn valid_booking(name: &str, email: &str) -> Result<(String, String), Response> {
    let name = name.trim().to_owned(); let email = email.trim().to_ascii_lowercase();
    if !(2..=80).contains(&name.chars().count()) { return Err(api_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_name", "Enter a guardian name between 2 and 80 characters.")); }
    if email.len() > 254 || !is_reasonable_email(&email) { return Err(api_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_email", "Enter an email address such as name@example.org.")); }
    Ok((name, email))
}

fn real_error(error: db::RealError) -> Response {
    match error {
        db::RealError::NotFound => api_error(StatusCode::NOT_FOUND, "not_found", "This class is not available."),
        db::RealError::Forbidden => api_error(StatusCode::FORBIDDEN, "workspace_access_denied", "This workspace key cannot change this school."),
        db::RealError::Full => api_error(StatusCode::CONFLICT, "class_full", "This class is full. Join the waitlist instead."),
        db::RealError::Cutoff => api_error(StatusCode::CONFLICT, "booking_closed", "The booking cutoff has passed."),
        db::RealError::OfferUnavailable => api_error(StatusCode::CONFLICT, "offer_unavailable", "This released-seat offer is no longer available."),
        db::RealError::Database => api_error(StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", "The change could not be saved. Try again."),
    }
}

pub async fn create_workspace(State(state): State<AppState>, Json(request): Json<WorkspaceRequest>) -> Response {
    let name = request.school_name.trim();
    if !(2..=100).contains(&name.chars().count()) { return api_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_school_name", "Enter a school name between 2 and 100 characters."); }
    match db::create_workspace(&state.pool, name, unix_now()).await { Ok((workspace, access_key)) => (StatusCode::CREATED, Json(WorkspaceResponse { workspace, access_key })).into_response(), Err(error) => real_error(error) }
}

pub async fn current_workspace(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Create or reopen a school workspace."); };
    match db::workspace_from_key(&state.pool, key).await { Ok(workspace) => Json(workspace).into_response(), Err(error) => real_error(error) }
}

pub async fn create_class(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<ClassRequest>) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before changing classes."); };
    let name = request.name.trim();
    if !(2..=120).contains(&name.chars().count()) || !(1..=500).contains(&request.capacity) || request.booking_cutoff >= request.starts_at || request.booking_cutoff <= unix_now() || request.timezone.trim().is_empty() { return api_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_class", "Set a class name, capacity, future cutoff before its start, and time zone."); }
    match db::create_real_class(&state.pool, key, name, request.starts_at, request.booking_cutoff, request.timezone.trim(), request.capacity, unix_now()).await { Ok(class) => (StatusCode::CREATED, Json(class)).into_response(), Err(error) => real_error(error) }
}

pub async fn list_classes(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Create or reopen a school workspace."); };
    match db::list_real_classes(&state.pool, key, unix_now()).await { Ok(classes) => Json(classes).into_response(), Err(error) => real_error(error) }
}

pub async fn publish_class(State(state): State<AppState>, Path(id): Path<String>, headers: HeaderMap) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before changing classes."); };
    match db::publish_real_class(&state.pool, key, &id, unix_now()).await { Ok(class) => Json(class).into_response(), Err(error) => real_error(error) }
}

pub async fn public_class(State(state): State<AppState>, Path(public_id): Path<String>) -> Response {
    match db::get_public_real_class(&state.pool, &public_id, unix_now()).await { Ok(Some(class)) => Json(class).into_response(), Ok(None) => real_error(db::RealError::NotFound), Err(error) => real_error(error) }
}

pub async fn real_book(State(state): State<AppState>, Path(public_id): Path<String>, headers: HeaderMap, Json(request): Json<BookingRequest>) -> Response {
    let Some(key) = headers.get("idempotency-key").and_then(|value| value.to_str().ok()).filter(|value| (8..=100).contains(&value.len())) else { return api_error(StatusCode::BAD_REQUEST, "idempotency_key_required", "Reload the form, then book again."); };
    let Ok((name, email)) = valid_booking(&request.guardian_name, &request.guardian_email) else { return valid_booking(&request.guardian_name, &request.guardian_email).unwrap_err(); };
    match db::book_real_seat(&state.pool, &public_id, key, &name, &email, unix_now()).await { Ok(class) => (StatusCode::CREATED, Json(class)).into_response(), Err(error) => real_error(error) }
}

pub async fn join_real_waitlist(State(state): State<AppState>, Path(public_id): Path<String>, Json(request): Json<WaitlistRequest>) -> Response {
    let Ok((name, email)) = valid_booking(&request.guardian_name, &request.guardian_email) else { return valid_booking(&request.guardian_name, &request.guardian_email).unwrap_err(); };
    if !request.consent { return api_error(StatusCode::UNPROCESSABLE_ENTITY, "consent_required", "Agree to receive this released-seat offer by email."); }
    match db::join_waitlist(&state.pool, &public_id, &name, &email, unix_now()).await { Ok(()) => StatusCode::CREATED.into_response(), Err(error) => real_error(error) }
}

pub async fn connect_calendar(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<CalendarRequest>) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before connecting a calendar."); };
    let label = request.label.trim(); if !(2..=100).contains(&label.chars().count()) { return api_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_calendar", "Name the calendar source before connecting it."); }
    match db::connect_calendar(&state.pool, key, label, unix_now()).await { Ok(connection) => Json(connection).into_response(), Err(error) => real_error(error) }
}

pub async fn reconcile(State(state): State<AppState>, Path(id): Path<String>, headers: HeaderMap, Json(request): Json<ReconcileRequest>) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before reconciling."); };
    if request.calendar_confirmed < 0 { return api_error(StatusCode::UNPROCESSABLE_ENTITY, "invalid_calendar_count", "Calendar bookings must be zero or more."); }
    match db::reconcile_class(&state.pool, key, &id, request.calendar_confirmed, unix_now()).await { Ok(class) => Json(class).into_response(), Err(error) => real_error(error) }
}

pub async fn cancel_and_offer(State(state): State<AppState>, Path((class_id, booking_id)): Path<(String, String)>, headers: HeaderMap) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before cancelling."); };
    match db::cancel_booking_and_offer(&state.pool, key, &class_id, &booking_id, unix_now()).await { Ok(token) => Json(serde_json::json!({"offerToken": token})).into_response(), Err(error) => real_error(error) }
}

pub async fn release_oldest_and_offer(State(state): State<AppState>, Path(class_id): Path<String>, headers: HeaderMap) -> Response {
    let Ok(key) = access_key(&headers) else { return api_error(StatusCode::UNAUTHORIZED, "workspace_key_required", "Open your school workspace before releasing a seat."); };
    match db::release_oldest_booking_and_offer(&state.pool, key, &class_id, unix_now()).await { Ok(token) => Json(serde_json::json!({"offerToken": token})).into_response(), Err(error) => real_error(error) }
}

pub async fn view_offer(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    match db::get_offer(&state.pool, &token, unix_now()).await { Ok(Some(offer)) => Json(offer).into_response(), Ok(None) => real_error(db::RealError::OfferUnavailable), Err(error) => real_error(error) }
}

pub async fn accept_seat_offer(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    match db::accept_offer(&state.pool, &token, unix_now()).await { Ok(class) => Json(class).into_response(), Err(error) => real_error(error) }
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
