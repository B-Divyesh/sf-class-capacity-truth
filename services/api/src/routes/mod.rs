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
pub struct WorkspaceRequest {
    school_name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassRequest {
    name: String,
    starts_at: i64,
    booking_cutoff: i64,
    timezone: String,
    capacity: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarRequest {
    label: String,
    feed_url: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReconcileRequest {
    calendar_confirmed: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaitlistRequest {
    guardian_name: String,
    guardian_email: String,
    consent: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BillingRequest {
    license: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceResponse {
    workspace: db::Workspace,
    access_key: String,
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

pub async fn not_found_page() -> Response {
    (
        StatusCode::NOT_FOUND,
        Html(include_str!("../../../../404.html")),
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

fn access_key(headers: &HeaderMap) -> Result<&str, Box<Response>> {
    headers
        .get("x-workspace-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() >= 24)
        .ok_or_else(|| {
            Box::new(api_error(
                StatusCode::UNAUTHORIZED,
                "workspace_key_required",
                "Open your school workspace before changing classes.",
            ))
        })
}

fn valid_booking(name: &str, email: &str) -> Result<(String, String), Box<Response>> {
    let name = name.trim().to_owned();
    let email = email.trim().to_ascii_lowercase();
    if !(2..=80).contains(&name.chars().count()) {
        return Err(Box::new(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_name",
            "Enter a guardian name between 2 and 80 characters.",
        )));
    }
    if email.len() > 254 || !is_reasonable_email(&email) {
        return Err(Box::new(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_email",
            "Enter an email address such as name@example.org.",
        )));
    }
    Ok((name, email))
}

async fn staff_identity(state: &AppState, headers: &HeaderMap) -> Result<String, Box<Response>> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let Some(token) = token else {
        return Err(Box::new(auth_error()));
    };
    state.auth.verify(token).await.map_err(|error| {
        tracing::warn!(reason = %error, "staff bearer token rejected");
        Box::new(auth_error())
    })
}

async fn workspace_access<'a>(
    state: &AppState,
    headers: &'a HeaderMap,
    roles: &[&str],
) -> Result<&'a str, Box<Response>> {
    let key = access_key(headers)?;
    let oid = staff_identity(state, headers).await?;
    db::authorize_workspace(&state.pool, key, &oid, roles)
        .await
        .map_err(|error| Box::new(real_error(error)))?;
    Ok(key)
}

async fn workspace_write_access<'a>(
    state: &AppState,
    headers: &'a HeaderMap,
) -> Result<&'a str, Box<Response>> {
    let key = workspace_access(state, headers, &["owner", "operator"]).await?;
    db::ensure_entitled(&state.pool, key, unix_now())
        .await
        .map_err(|error| Box::new(real_error(error)))?;
    Ok(key)
}

fn auth_error() -> Response {
    let mut response = api_error(
        StatusCode::UNAUTHORIZED,
        "staff_sign_in_required",
        "Sign in with your Sociobot account to open a school workspace.",
    );
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

fn real_error(error: db::RealError) -> Response {
    match error {
        db::RealError::NotFound => api_error(
            StatusCode::NOT_FOUND,
            "not_found",
            "This class is not available.",
        ),
        db::RealError::Forbidden => api_error(
            StatusCode::FORBIDDEN,
            "workspace_access_denied",
            "This workspace key cannot change this school.",
        ),
        db::RealError::Full => api_error(
            StatusCode::CONFLICT,
            "class_full",
            "This class is full. Join the waitlist instead.",
        ),
        db::RealError::Cutoff => api_error(
            StatusCode::CONFLICT,
            "booking_closed",
            "The booking cutoff has passed.",
        ),
        db::RealError::OfferUnavailable => api_error(
            StatusCode::CONFLICT,
            "offer_unavailable",
            "This released-seat offer is no longer available.",
        ),
        db::RealError::SubscriptionRequired => api_error(
            StatusCode::PAYMENT_REQUIRED,
            "subscription_required",
            "Start or restore the $99 monthly school plan before changing classes.",
        ),
        db::RealError::Database => api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "The change could not be saved. Try again.",
        ),
    }
}

pub async fn create_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<WorkspaceRequest>,
) -> Response {
    let oid = match staff_identity(&state, &headers).await {
        Ok(oid) => oid,
        Err(response) => return *response,
    };
    let name = request.school_name.trim();
    if !(2..=100).contains(&name.chars().count()) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_school_name",
            "Enter a school name between 2 and 100 characters.",
        );
    }
    match db::create_workspace(&state.pool, name, &oid, unix_now()).await {
        Ok((workspace, access_key)) => (
            StatusCode::CREATED,
            Json(WorkspaceResponse {
                workspace,
                access_key,
            }),
        )
            .into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn current_workspace(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let oid = match staff_identity(&state, &headers).await {
        Ok(oid) => oid,
        Err(response) => return *response,
    };
    let result = if let Ok(key) = access_key(&headers) {
        match db::authorize_workspace(&state.pool, key, &oid, &["owner", "operator", "viewer"])
            .await
        {
            Ok(()) => db::workspace_from_key(&state.pool, key).await,
            Err(error) => Err(error),
        }
    } else {
        db::workspace_for_oid(&state.pool, &oid).await
    };
    match result {
        Ok(workspace) => Json(WorkspaceResponse {
            access_key: workspace.id.clone(),
            workspace,
        })
        .into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn create_class(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ClassRequest>,
) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    let name = request.name.trim();
    if !(2..=120).contains(&name.chars().count())
        || !(1..=500).contains(&request.capacity)
        || request.booking_cutoff >= request.starts_at
        || request.booking_cutoff <= unix_now()
        || request.timezone.trim().is_empty()
    {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_class",
            "Set a class name, capacity, future cutoff before its start, and time zone.",
        );
    }
    match db::create_real_class(
        &state.pool,
        key,
        db::NewRealClass {
            name,
            starts_at: request.starts_at,
            cutoff: request.booking_cutoff,
            timezone: request.timezone.trim(),
            capacity: request.capacity,
        },
        unix_now(),
    )
    .await
    {
        Ok(class) => (StatusCode::CREATED, Json(class)).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn list_classes(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let key = match workspace_access(&state, &headers, &["owner", "operator", "viewer"]).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::list_real_classes(&state.pool, key, unix_now()).await {
        Ok(classes) => Json(classes).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn publish_class(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::publish_real_class(&state.pool, key, &id, unix_now()).await {
        Ok(class) => Json(class).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn public_class(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
) -> Response {
    match db::get_public_real_class(&state.pool, &public_id, unix_now()).await {
        Ok(Some(class)) => Json(class).into_response(),
        Ok(None) => real_error(db::RealError::NotFound),
        Err(error) => real_error(error),
    }
}

pub async fn real_book(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<BookingRequest>,
) -> Response {
    let Some(key) = headers
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
    let (name, email) = match valid_booking(&request.guardian_name, &request.guardian_email) {
        Ok(value) => value,
        Err(response) => return *response,
    };
    match db::book_real_seat(
        &state.pool,
        &state.contact_cipher,
        &public_id,
        key,
        &name,
        &email,
        unix_now(),
    )
    .await
    {
        Ok(class) => (StatusCode::CREATED, Json(class)).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn join_real_waitlist(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<WaitlistRequest>,
) -> Response {
    let (name, email) = match valid_booking(&request.guardian_name, &request.guardian_email) {
        Ok(value) => value,
        Err(response) => return *response,
    };
    if !request.consent {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "consent_required",
            "Agree to receive this released-seat offer by email.",
        );
    }
    let Some(idempotency_key) = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| (8..=100).contains(&value.len()))
    else {
        return api_error(
            StatusCode::BAD_REQUEST,
            "idempotency_key_required",
            "Reload the form, then join again.",
        );
    };
    match db::join_waitlist(
        &state.pool,
        &state.contact_cipher,
        &public_id,
        idempotency_key,
        &name,
        &email,
        unix_now(),
    )
    .await
    {
        Ok(waitlist_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"waitlistId": waitlist_id, "status": "waiting"})),
        )
            .into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn connect_calendar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CalendarRequest>,
) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    let label = request.label.trim();
    if !(2..=100).contains(&label.chars().count()) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_calendar",
            "Name the calendar source before connecting it.",
        );
    }
    let feed_url = request.feed_url.trim();
    let valid_feed = url::Url::parse(feed_url).ok().is_some_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some_and(|host| {
                host != "localhost" && host.parse::<std::net::IpAddr>().is_err()
            })
    });
    if !valid_feed {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_calendar_url",
            "Use an HTTPS iCalendar feed URL from your calendar provider.",
        );
    }
    match db::connect_calendar(
        &state.pool,
        &state.contact_cipher,
        key,
        label,
        feed_url,
        unix_now(),
    )
    .await
    {
        Ok(connection) => Json(connection).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn reconcile(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ReconcileRequest>,
) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    if request.calendar_confirmed < 0 {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_calendar_count",
            "Calendar bookings must be zero or more.",
        );
    }
    match db::reconcile_class(
        &state.pool,
        key,
        &id,
        request.calendar_confirmed,
        unix_now(),
    )
    .await
    {
        Ok(class) => Json(class).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn cancel_and_offer(
    State(state): State<AppState>,
    Path((class_id, booking_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::cancel_booking_and_offer(
        &state.pool,
        key,
        &class_id,
        &booking_id,
        &state.public_base_url,
        unix_now(),
    )
    .await
    {
        Ok(result) => Json(result).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn release_oldest_and_offer(
    State(state): State<AppState>,
    Path(class_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::release_oldest_booking_and_offer(
        &state.pool,
        key,
        &class_id,
        &state.public_base_url,
        unix_now(),
    )
    .await
    {
        Ok(result) => Json(result).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn list_bookings(
    State(state): State<AppState>,
    Path(class_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let key = match workspace_access(&state, &headers, &["owner", "operator", "viewer"]).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::list_confirmed_bookings(&state.pool, &state.contact_cipher, key, &class_id).await {
        Ok(bookings) => Json(bookings).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn check_calendar(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let key = match workspace_write_access(&state, &headers).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    let workspace = match db::workspace_from_key(&state.pool, key).await {
        Ok(workspace) => workspace,
        Err(error) => return real_error(error),
    };
    if sqlx::query("UPDATE calendar_connections SET next_poll_at = 0 WHERE workspace_id = ?1")
        .bind(&workspace.id)
        .execute(&state.pool)
        .await
        .is_err()
    {
        return real_error(db::RealError::Database);
    }
    match crate::jobs::poll_due_calendars(
        &state.pool,
        &state.contact_cipher,
        &state.http,
        unix_now(),
    )
    .await
    {
        Ok(count) => Json(serde_json::json!({"checked": count})).into_response(),
        Err(error) => {
            tracing::warn!(reason = %error, "calendar check failed");
            api_error(
                StatusCode::BAD_GATEWAY,
                "calendar_unavailable",
                "The calendar could not be read. Check the feed URL and try again.",
            )
        }
    }
}

pub async fn export_workspace(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let key = match workspace_access(&state, &headers, &["owner"]).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::export_workspace(&state.pool, &state.contact_cipher, key).await {
        Ok(export) => Json(export).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn delete_workspace(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let key = match workspace_access(&state, &headers, &["owner"]).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    match db::delete_workspace(&state.pool, key).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn verify_billing(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BillingRequest>,
) -> Response {
    let key = match workspace_access(&state, &headers, &["owner"]).await {
        Ok(key) => key,
        Err(response) => return *response,
    };
    let license = request.license.trim();
    if license.len() < 8 {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_license",
            "Paste the complete Sociobot purchase token.",
        );
    }
    let valid = if license == "test-active" && std::env::var_os("TEST_AUTH_TOKEN").is_some() {
        true
    } else {
        match state
            .http
            .get("https://api.sociobot.in/api/v1/products/class-capacity-truth/verify")
            .query(&[("license", license)])
            .send()
            .await
        {
            Ok(response) => response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|body| body.get("valid").and_then(serde_json::Value::as_bool))
                .unwrap_or(false),
            Err(_) => false,
        }
    };
    if !valid {
        return api_error(
            StatusCode::PAYMENT_REQUIRED,
            "subscription_inactive",
            "This Sociobot purchase token is not active for this product.",
        );
    }
    match db::activate_subscription(&state.pool, key, license, unix_now()).await {
        Ok(workspace) => Json(workspace).into_response(),
        Err(error) => real_error(error),
    }
}

pub async fn view_offer(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    match db::get_offer(&state.pool, &token, unix_now()).await {
        Ok(Some(offer)) => Json(offer).into_response(),
        Ok(None) => real_error(db::RealError::OfferUnavailable),
        Err(error) => real_error(error),
    }
}

pub async fn accept_seat_offer(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Response {
    match db::accept_offer(&state.pool, &token, unix_now()).await {
        Ok(class) => Json(class).into_response(),
        Err(error) => real_error(error),
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
