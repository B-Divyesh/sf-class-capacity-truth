use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use class_capacity_truth_api::{app, db, AppState};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::Row;
use tempfile::TempDir;
use tower::ServiceExt;
use uuid::Uuid;

async fn test_app(period_ms: u64, burst: u32) -> (Router, TempDir, sqlx::SqlitePool) {
    let directory = tempfile::tempdir().unwrap();
    let url = format!("sqlite://{}", directory.path().join("test.db").display());
    let pool = db::connect(&url).await.unwrap();
    let state = AppState {
        pool: pool.clone(),
        cookie_key: Arc::new(vec![13_u8; 32]),
        contact_cipher: class_capacity_truth_api::crypto::ContactCipher::from_key(&[17_u8; 32])
            .unwrap(),
        auth: class_capacity_truth_api::auth::AuthVerifier::for_tests(),
        public_base_url: Arc::new("https://example.test".into()),
        http: reqwest::Client::new(),
        email_delivery_configured: false,
    };
    (
        app(state, PathBuf::from("does-not-exist"), period_ms, burst),
        directory,
        pool,
    )
}

fn get(path: &str, ip: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri(path).header("x-forwarded-for", ip);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).unwrap()
}

fn post(path: &str, ip: &str, cookie: Option<&str>, body: Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(path)
        .header("x-forwarded-for", ip)
        .header(header::CONTENT_TYPE, "application/json")
        .header("idempotency-key", Uuid::new_v4().to_string());
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn body_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn create_demo(router: &Router, ip: &str) -> (String, Value) {
    let response = router
        .clone()
        .oneshot(get("/api/demo/session", ip, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    (cookie, body_json(response).await)
}

#[tokio::test]
async fn health_reports_build_without_rate_limit() {
    let (router, _directory, _pool) = test_app(60_000, 1).await;
    for _ in 0..3 {
        let response = router
            .clone()
            .oneshot(get("/health", "192.0.2.1", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["status"], "ok");
        assert!(body["build"].as_str().is_some());
    }
}

#[tokio::test]
async fn demo_cookie_isolates_bookings_and_rejects_tenant_input() {
    let (router, _directory, _pool) = test_app(1, 100).await;
    let (first_cookie, first) = create_demo(&router, "192.0.2.2").await;
    let (second_cookie, second) = create_demo(&router, "192.0.2.3").await;
    let class_id = first["classes"][0]["publicId"].as_str().unwrap();
    let response = router
        .clone()
        .oneshot(post(
            &format!("/api/demo/classes/{class_id}/book"),
            "192.0.2.2",
            Some(&first_cookie),
            json!({"guardianName":"Alex Morgan","guardianEmail":"alex@example.org"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(body_json(response).await["class"]["openSeats"], 1);

    let response = router
        .clone()
        .oneshot(get("/api/demo/session", "192.0.2.3", Some(&second_cookie)))
        .await
        .unwrap();
    assert_eq!(body_json(response).await["classes"][0]["openSeats"], 2);
    assert_eq!(second["classes"][0]["openSeats"], 2);

    let response = router.clone().oneshot(post(
        &format!("/api/demo/classes/{class_id}/book"),
        "192.0.2.2",
        Some(&first_cookie),
        json!({"guardianName":"Alex Morgan","guardianEmail":"alex@example.org","organizationId":"other-school"}),
    )).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn validation_and_signed_cookie_are_enforced() {
    let (router, _directory, _pool) = test_app(1, 100).await;
    let (cookie, body) = create_demo(&router, "192.0.2.4").await;
    let class_id = body["classes"][0]["publicId"].as_str().unwrap();

    let response = router
        .clone()
        .oneshot(post(
            &format!("/api/demo/classes/{class_id}/book"),
            "192.0.2.4",
            Some("cct_demo=changed.invalid"),
            json!({"guardianName":"Alex Morgan","guardianEmail":"alex@example.org"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = router
        .clone()
        .oneshot(post(
            &format!("/api/demo/classes/{class_id}/book"),
            "192.0.2.4",
            Some(&cookie),
            json!({"guardianName":"A","guardianEmail":"not-an-email"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rate_limit_uses_forwarded_ip_and_returns_retry_after() {
    let (router, _directory, _pool) = test_app(60_000, 2).await;
    for _ in 0..2 {
        let response = router
            .clone()
            .oneshot(get("/api/demo/session", "198.51.100.8", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    let blocked = router
        .clone()
        .oneshot(get("/api/demo/session", "198.51.100.8", None))
        .await
        .unwrap();
    assert_eq!(blocked.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(blocked.headers().get(header::RETRY_AFTER).is_some());

    let other_ip = router
        .oneshot(get("/api/demo/session", "198.51.100.9", None))
        .await
        .unwrap();
    assert_eq!(other_ip.status(), StatusCode::OK);
}

#[tokio::test]
async fn school_routes_require_entra_bearer_and_return_auth_challenge() {
    let (router, _directory, _pool) = test_app(1, 100).await;
    let denied = router
        .clone()
        .oneshot(post(
            "/api/workspaces",
            "198.51.100.44",
            None,
            json!({"schoolName":"Protected School"}),
        ))
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(denied.headers()[header::WWW_AUTHENTICATE], "Bearer");

    let request = Request::builder()
        .method("POST")
        .uri("/api/workspaces")
        .header("x-forwarded-for", "198.51.100.45")
        .header(header::AUTHORIZATION, "Bearer test-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"schoolName":"Protected School"}).to_string(),
        ))
        .unwrap();
    assert_eq!(
        router.oneshot(request).await.unwrap().status(),
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn capacity_cutoff_idempotency_reset_and_concurrent_race() {
    let (_router, _directory, pool) = test_app(1, 100).await;
    let tenant = Uuid::new_v4().to_string();
    let now = 1_900_000_000;
    db::create_or_refresh_demo(&pool, &tenant, now)
        .await
        .unwrap();
    let sessions = db::list_sessions(&pool, &tenant, now).await.unwrap();
    let open = sessions
        .iter()
        .find(|item| item.availability == db::Availability::Available)
        .unwrap();
    let full = sessions
        .iter()
        .find(|item| item.availability == db::Availability::Full)
        .unwrap();
    let cutoff = sessions
        .iter()
        .find(|item| item.availability == db::Availability::Cutoff)
        .unwrap();
    assert!(sessions.iter().all(|item| item.timezone == "Europe/London"));

    assert_eq!(
        db::book_seat(
            &pool,
            &tenant,
            &full.public_id,
            "full-booking",
            "Alex Morgan",
            "alex@example.org",
            now
        )
        .await
        .unwrap_err(),
        db::BookingError::Full
    );
    assert_eq!(
        db::book_seat(
            &pool,
            &tenant,
            &cutoff.public_id,
            "cutoff-booking",
            "Alex Morgan",
            "alex@example.org",
            now
        )
        .await
        .unwrap_err(),
        db::BookingError::Cutoff
    );

    let first = db::book_seat(
        &pool,
        &tenant,
        &open.public_id,
        "same-booking",
        "Alex Morgan",
        "alex@example.org",
        now,
    )
    .await
    .unwrap();
    let repeat = db::book_seat(
        &pool,
        &tenant,
        &open.public_id,
        "same-booking",
        "Alex Morgan",
        "alex@example.org",
        now,
    )
    .await
    .unwrap();
    assert_eq!(first.booking_id, repeat.booking_id);
    assert!(repeat.repeated);
    assert_eq!(repeat.class.open_seats, 1);

    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let tenant_a = tenant.clone();
    let tenant_b = tenant.clone();
    let class_a = open.public_id.clone();
    let class_b = open.public_id.clone();
    let a = tokio::spawn(async move {
        db::book_seat(
            &pool_a,
            &tenant_a,
            &class_a,
            "race-a",
            "One Guardian",
            "one@example.org",
            now,
        )
        .await
    });
    let b = tokio::spawn(async move {
        db::book_seat(
            &pool_b,
            &tenant_b,
            &class_b,
            "race-b",
            "Two Guardian",
            "two@example.org",
            now,
        )
        .await
    });
    let outcomes = [a.await.unwrap(), b.await.unwrap()];
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| matches!(result, Err(db::BookingError::Full)))
            .count(),
        1
    );

    db::reset_demo(&pool, &tenant, now + 1).await.unwrap();
    assert_eq!(
        db::list_sessions(&pool, &tenant, now + 1).await.unwrap()[0].open_seats,
        2
    );
}

#[tokio::test]
async fn regression_real_school_flow_configures_books_reconciles_and_converts_released_waitlist_seat(
) {
    // Regression for verifier P0: the old artifact had only a cookie-scoped
    // sample and no durable school class, public booking, reconciliation, or offer.
    let (_router, _directory, pool) = test_app(1, 100).await;
    let now = 1_900_000_000;
    let cipher = class_capacity_truth_api::crypto::ContactCipher::from_key(&[19_u8; 32]).unwrap();
    let (workspace, key) = db::create_workspace(&pool, "Harbour Languages", "test-owner-oid", now)
        .await
        .unwrap();
    assert_eq!(
        db::workspace_from_key(&pool, &key).await.unwrap().id,
        workspace.id
    );
    let class = db::create_real_class(
        &pool,
        &key,
        db::NewRealClass {
            name: "Saturday level check",
            starts_at: now + 86_400,
            cutoff: now + 43_200,
            timezone: "Europe/London",
            capacity: 2,
        },
        now,
    )
    .await
    .unwrap();
    let class = db::publish_real_class(&pool, &key, &class.id, now)
        .await
        .unwrap();
    assert!(class.published);
    assert!(db::get_public_real_class(&pool, &class.public_id, now)
        .await
        .unwrap()
        .is_some());
    let booked = db::book_real_seat(
        &pool,
        &cipher,
        &class.public_id,
        "first-parent",
        "A Parent",
        "a@example.org",
        now,
    )
    .await
    .unwrap();
    assert_eq!(booked.open_seats, 1);
    let booked = db::book_real_seat(
        &pool,
        &cipher,
        &class.public_id,
        "second-parent",
        "B Parent",
        "b@example.org",
        now,
    )
    .await
    .unwrap();
    assert_eq!(booked.open_seats, 0);
    assert_eq!(
        db::book_real_seat(
            &pool,
            &cipher,
            &class.public_id,
            "oversell",
            "C Parent",
            "c@example.org",
            now
        )
        .await
        .unwrap_err(),
        db::RealError::Full
    );
    let waitlist_id = db::join_waitlist(
        &pool,
        &cipher,
        &class.public_id,
        "waitlist-request",
        "Waiting Parent",
        "waiting@example.org",
        now,
    )
    .await
    .unwrap();
    let repeated_waitlist_id = db::join_waitlist(
        &pool,
        &cipher,
        &class.public_id,
        "waitlist-request",
        "Waiting Parent",
        "waiting@example.org",
        now,
    )
    .await
    .unwrap();
    assert_eq!(waitlist_id, repeated_waitlist_id);
    let encrypted_booking = sqlx::query(
        "SELECT guardian_name, guardian_email FROM real_bookings WHERE class_id = ?1 LIMIT 1",
    )
    .bind(&class.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_ne!(
        encrypted_booking.get::<String, _>("guardian_name"),
        "A Parent"
    );
    assert_ne!(
        encrypted_booking.get::<String, _>("guardian_email"),
        "a@example.org"
    );
    let visible = db::list_confirmed_bookings(&pool, &cipher, &key, &class.id)
        .await
        .unwrap();
    assert_eq!(visible[0].guardian_name, "A Parent");
    let checked = db::reconcile_class(&pool, &key, &class.id, 1, now)
        .await
        .unwrap();
    assert_eq!(checked.reconciliation_status.as_deref(), Some("attention"));
    let token =
        db::release_oldest_booking_and_offer(&pool, &key, &class.id, "https://example.test", now)
            .await
            .unwrap()
            .offer_token
            .expect("one fair offer");
    let outbox = sqlx::query("SELECT status, text_body FROM email_outbox WHERE workspace_id = ?1")
        .bind(&workspace.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(outbox.get::<String, _>("status"), "pending");
    assert!(outbox
        .get::<String, _>("text_body")
        .contains(&format!("/offer/{token}")));
    let offer = db::get_offer(&pool, &token, now)
        .await
        .unwrap()
        .expect("offer is viewable");
    assert_eq!(offer.class.public_id, class.public_id);
    let accepted = db::accept_offer(&pool, &token, now).await.unwrap();
    assert_eq!(accepted.confirmed, 2);
    assert_eq!(accepted.open_seats, 0);
    assert_eq!(
        db::accept_offer(&pool, &token, now).await.unwrap_err(),
        db::RealError::OfferUnavailable
    );
    sqlx::query("INSERT INTO workspace_members (workspace_id, oid, role, created_at) VALUES (?1, 'viewer-oid', 'viewer', ?2)")
        .bind(&workspace.id).bind(now).execute(&pool).await.unwrap();
    assert!(
        db::authorize_workspace(&pool, &key, "viewer-oid", &["viewer"])
            .await
            .is_ok()
    );
    assert_eq!(
        db::authorize_workspace(&pool, &key, "viewer-oid", &["owner"])
            .await
            .unwrap_err(),
        db::RealError::Forbidden
    );

    let scrubbed = db::cleanup_retained_contacts(&pool, now + 91 * 86_400)
        .await
        .unwrap();
    assert!(scrubbed >= 3);
    let remaining_plaintext = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM real_bookings WHERE guardian_name LIKE '%Parent%' OR guardian_email LIKE '%example.org%'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(remaining_plaintext, 0);
}

#[tokio::test]
async fn migration_has_a_working_down_path() {
    let (_router, _directory, pool) = test_app(1, 100).await;
    sqlx::raw_sql(include_str!("../migrations/0001_demo.down.sql"))
        .execute(&pool)
        .await
        .unwrap();

    for table in ["bookings", "class_sessions", "demo_tenants"] {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 0, "{table} should be removed by the down migration");
    }
}

#[tokio::test]
async fn claim_demo_expiry_and_input_disposal() {
    // @claim:demo-expiry-input-disposal
    let (_router, _directory, pool) = test_app(1, 100).await;
    let tenant = "claim-demo-expiry";
    let now = 1_900_000_000;
    db::create_or_refresh_demo(&pool, tenant, now)
        .await
        .unwrap();
    let class = db::list_sessions(&pool, tenant, now)
        .await
        .unwrap()
        .remove(0);
    db::book_seat(
        &pool,
        tenant,
        &class.public_id,
        "claim-demo-booking",
        "Private Demo Name",
        "private-demo@example.org",
        now,
    )
    .await
    .unwrap();

    let stored =
        sqlx::query("SELECT guardian_name, guardian_email FROM bookings WHERE demo_tenant_id = ?1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        stored.get::<String, _>("guardian_name"),
        "[demo input not retained]"
    );
    assert_eq!(
        stored.get::<String, _>("guardian_email"),
        "[demo input not retained]"
    );

    assert_eq!(db::cleanup_expired(&pool, now + 86_399).await.unwrap(), 0);
    assert_eq!(db::cleanup_expired(&pool, now + 86_400).await.unwrap(), 1);
    let remaining =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bookings WHERE demo_tenant_id = ?1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn claim_reconciliation_never_mutates_confirmed_seats() {
    // @claim:reconciliation-does-not-change-seats
    let (_router, _directory, pool) = test_app(1, 100).await;
    let now = 1_900_000_000;
    let (_workspace, key) =
        db::create_workspace(&pool, "Reconciliation School", "owner-reconcile", now)
            .await
            .unwrap();
    let class = db::create_real_class(
        &pool,
        &key,
        db::NewRealClass {
            name: "Tuesday class",
            starts_at: now + 86_400,
            cutoff: now + 43_200,
            timezone: "Europe/London",
            capacity: 8,
        },
        now,
    )
    .await
    .unwrap();
    assert_eq!(class.confirmed, 0);
    let reconciled = db::reconcile_class(&pool, &key, &class.id, 7, now)
        .await
        .unwrap();
    assert_eq!(reconciled.confirmed, 0);
    assert_eq!(reconciled.open_seats, 8);
    assert_eq!(reconciled.calendar_confirmed, Some(7));
    assert_eq!(
        reconciled.reconciliation_status.as_deref(),
        Some("attention")
    );
}

#[tokio::test]
async fn claim_contact_encryption_and_retention() {
    // @claim:contact-encryption-retention
    let (_router, _directory, pool) = test_app(1, 100).await;
    let now = 1_900_000_000;
    let cipher = class_capacity_truth_api::crypto::ContactCipher::from_key(&[29_u8; 32]).unwrap();
    let (_workspace, key) = db::create_workspace(&pool, "Privacy School", "owner-privacy", now)
        .await
        .unwrap();
    let class = db::create_real_class(
        &pool,
        &key,
        db::NewRealClass {
            name: "Privacy class",
            starts_at: now + 86_400,
            cutoff: now + 43_200,
            timezone: "Europe/London",
            capacity: 2,
        },
        now,
    )
    .await
    .unwrap();
    let class = db::publish_real_class(&pool, &key, &class.id, now)
        .await
        .unwrap();
    db::book_real_seat(
        &pool,
        &cipher,
        &class.public_id,
        "privacy-booking",
        "Private Parent",
        "private-parent@example.org",
        now,
    )
    .await
    .unwrap();
    let row = sqlx::query("SELECT guardian_name, guardian_email FROM real_bookings LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(row.get::<String, _>("guardian_name"), "Private Parent");
    assert_ne!(
        row.get::<String, _>("guardian_email"),
        "private-parent@example.org"
    );
    assert_eq!(
        db::cleanup_retained_contacts(&pool, now + 90 * 86_400 - 1)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        db::cleanup_retained_contacts(&pool, now + 90 * 86_400)
            .await
            .unwrap(),
        1
    );
    let scrubbed = sqlx::query("SELECT guardian_name, guardian_email FROM real_bookings LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(scrubbed.get::<String, _>("guardian_name"), "[deleted]");
    assert_eq!(scrubbed.get::<String, _>("guardian_email"), "[deleted]");
}

#[tokio::test]
async fn claim_staff_roles_enforce_owner_actions() {
    // @claim:staff-role-access
    let (_router, _directory, pool) = test_app(1, 100).await;
    let now = 1_900_000_000;
    let (workspace, key) = db::create_workspace(&pool, "Role School", "owner-oid", now)
        .await
        .unwrap();
    sqlx::query("INSERT INTO workspace_members (workspace_id, oid, role, created_at) VALUES (?1, 'viewer-oid', 'viewer', ?2)")
        .bind(&workspace.id)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        db::authorize_workspace(&pool, &key, "viewer-oid", &["viewer"])
            .await
            .is_ok()
    );
    assert_eq!(
        db::authorize_workspace(&pool, &key, "viewer-oid", &["owner"])
            .await
            .unwrap_err(),
        db::RealError::Forbidden
    );
}
