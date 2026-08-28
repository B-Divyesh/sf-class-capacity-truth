use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use class_capacity_truth_api::{app, db, AppState};
use http_body_util::BodyExt;
use serde_json::{json, Value};
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
