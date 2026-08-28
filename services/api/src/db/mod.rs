use std::{str::FromStr, time::Duration};

use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use thiserror::Error;
use uuid::Uuid;

const DEMO_TTL_SECONDS: i64 = 24 * 60 * 60;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassSession {
    pub public_id: String,
    pub name: String,
    pub starts_at: i64,
    pub booking_cutoff: i64,
    pub timezone: String,
    pub capacity: i64,
    pub confirmed: i64,
    pub open_seats: i64,
    pub availability: Availability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Full,
    Cutoff,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum BookingError {
    #[error("class not found")]
    NotFound,
    #[error("this class is full")]
    Full,
    #[error("the booking cutoff has passed")]
    Cutoff,
    #[error("database error")]
    Database,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingResult {
    pub booking_id: String,
    pub class: ClassSession,
    pub repeated: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub school_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealClass {
    pub id: String,
    pub public_id: String,
    pub name: String,
    pub starts_at: i64,
    pub booking_cutoff: i64,
    pub timezone: String,
    pub capacity: i64,
    pub confirmed: i64,
    pub open_seats: i64,
    pub availability: Availability,
    pub published: bool,
    pub calendar_confirmed: Option<i64>,
    pub reconciliation_status: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarConnection {
    pub label: String,
    pub provider: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitlistOffer {
    pub offer_token: String,
    pub class: RealClass,
    pub expires_at: i64,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RealError {
    #[error("not found")]
    NotFound,
    #[error("access denied")]
    Forbidden,
    #[error("class is full")]
    Full,
    #[error("booking cutoff has passed")]
    Cutoff,
    #[error("offer is no longer available")]
    OfferUnavailable,
    #[error("database error")]
    Database,
}

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(10));
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn create_or_refresh_demo(
    pool: &SqlitePool,
    tenant_id: &str,
    now: i64,
) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let result = async {
        let active = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM demo_tenants WHERE id = ?1 AND expires_at > ?2",
        )
        .bind(tenant_id)
        .bind(now)
        .fetch_one(&mut *conn)
        .await?;

        if active == 0 {
            sqlx::query("DELETE FROM demo_tenants WHERE id = ?1")
                .bind(tenant_id)
                .execute(&mut *conn)
                .await?;
            sqlx::query(
                "INSERT INTO demo_tenants (id, created_at, expires_at) VALUES (?1, ?2, ?3)",
            )
            .bind(tenant_id)
            .bind(now)
            .bind(now + DEMO_TTL_SECONDS)
            .execute(&mut *conn)
            .await?;
            seed_sessions(&mut conn, tenant_id, now).await?;
        }
        anyhow::Ok(())
    }
    .await;
    finish_immediate(&mut conn, result).await
}

pub async fn reset_demo(pool: &SqlitePool, tenant_id: &str, now: i64) -> anyhow::Result<()> {
    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let result = async {
        sqlx::query("DELETE FROM demo_tenants WHERE id = ?1")
            .bind(tenant_id)
            .execute(&mut *conn)
            .await?;
        sqlx::query("INSERT INTO demo_tenants (id, created_at, expires_at) VALUES (?1, ?2, ?3)")
            .bind(tenant_id)
            .bind(now)
            .bind(now + DEMO_TTL_SECONDS)
            .execute(&mut *conn)
            .await?;
        seed_sessions(&mut conn, tenant_id, now).await?;
        anyhow::Ok(())
    }
    .await;
    finish_immediate(&mut conn, result).await
}

pub async fn destroy_demo(pool: &SqlitePool, tenant_id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM demo_tenants WHERE id = ?1")
        .bind(tenant_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired(pool: &SqlitePool, now: i64) -> anyhow::Result<u64> {
    let result = sqlx::query("DELETE FROM demo_tenants WHERE expires_at <= ?1")
        .bind(now)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

async fn seed_sessions(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    tenant_id: &str,
    now: i64,
) -> anyhow::Result<()> {
    let seeds = [
        (
            "Level check: upper primary",
            now + 2 * 86_400,
            now + 86_400,
            8_i64,
            6_i64,
        ),
        (
            "Friday conversation group",
            now + 3 * 86_400,
            now + 2 * 86_400,
            6,
            6,
        ),
        ("Saturday assessment", now + 7 * 86_400, now - 3_600, 10, 4),
    ];

    for (index, (name, starts_at, cutoff, capacity, confirmed)) in seeds.into_iter().enumerate() {
        sqlx::query(
            "INSERT INTO class_sessions
             (id, demo_tenant_id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'Europe/London', ?7, ?8, ?9)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(format!("demo-{}", Uuid::new_v4().simple()))
        .bind(name)
        .bind(starts_at)
        .bind(cutoff)
        .bind(capacity)
        .bind(confirmed)
        .bind(index as i64)
        .execute(&mut **conn)
        .await?;
    }
    Ok(())
}

async fn finish_immediate(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    result: anyhow::Result<()>,
) -> anyhow::Result<()> {
    match result {
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut **conn).await?;
            Ok(())
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut **conn).await;
            Err(error)
        }
    }
}

pub async fn list_sessions(
    pool: &SqlitePool,
    tenant_id: &str,
    now: i64,
) -> anyhow::Result<Vec<ClassSession>> {
    let rows = sqlx::query(
        "SELECT public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed
         FROM class_sessions WHERE demo_tenant_id = ?1 ORDER BY sort_order",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| session_from_row(row, now)).collect())
}

pub async fn get_session(
    pool: &SqlitePool,
    tenant_id: &str,
    public_id: &str,
    now: i64,
) -> anyhow::Result<Option<ClassSession>> {
    let row = sqlx::query(
        "SELECT public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed
         FROM class_sessions WHERE demo_tenant_id = ?1 AND public_id = ?2",
    )
    .bind(tenant_id)
    .bind(public_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.as_ref().map(|row| session_from_row(row, now)))
}

fn session_from_row(row: &sqlx::sqlite::SqliteRow, now: i64) -> ClassSession {
    let capacity = row.get::<i64, _>("capacity");
    let confirmed = row.get::<i64, _>("confirmed");
    let cutoff = row.get::<i64, _>("booking_cutoff");
    let availability = availability_at(capacity, confirmed, cutoff, now);
    ClassSession {
        public_id: row.get("public_id"),
        name: row.get("name"),
        starts_at: row.get("starts_at"),
        booking_cutoff: cutoff,
        timezone: row.get("timezone"),
        capacity,
        confirmed,
        open_seats: capacity - confirmed,
        availability,
    }
}

fn availability_at(capacity: i64, confirmed: i64, cutoff: i64, now: i64) -> Availability {
    if confirmed >= capacity {
        Availability::Full
    } else if cutoff <= now {
        Availability::Cutoff
    } else {
        Availability::Available
    }
}

pub async fn book_seat(
    pool: &SqlitePool,
    tenant_id: &str,
    public_id: &str,
    idempotency_key: &str,
    _guardian_name: &str,
    _guardian_email: &str,
    now: i64,
) -> Result<BookingResult, BookingError> {
    let mut conn = pool.acquire().await.map_err(|_| BookingError::Database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(|_| BookingError::Database)?;

    let result = book_in_transaction(&mut conn, tenant_id, public_id, idempotency_key, now).await;

    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(|_| BookingError::Database)?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
}

async fn book_in_transaction(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    tenant_id: &str,
    public_id: &str,
    idempotency_key: &str,
    now: i64,
) -> Result<BookingResult, BookingError> {
    if let Some(row) = sqlx::query(
        "SELECT b.id AS booking_id, s.public_id, s.name, s.starts_at, s.booking_cutoff,
                s.timezone, s.capacity, s.confirmed
         FROM bookings b JOIN class_sessions s ON s.id = b.class_session_id
         WHERE b.demo_tenant_id = ?1 AND b.idempotency_key = ?2",
    )
    .bind(tenant_id)
    .bind(idempotency_key)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|_| BookingError::Database)?
    {
        return Ok(BookingResult {
            booking_id: row.get("booking_id"),
            class: session_from_row(&row, now),
            repeated: true,
        });
    }

    let row = sqlx::query(
        "SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed
         FROM class_sessions WHERE demo_tenant_id = ?1 AND public_id = ?2",
    )
    .bind(tenant_id)
    .bind(public_id)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|_| BookingError::Database)?
    .ok_or(BookingError::NotFound)?;

    let session = session_from_row(&row, now);
    if session.confirmed >= session.capacity {
        return Err(BookingError::Full);
    }
    if session.booking_cutoff <= now {
        return Err(BookingError::Cutoff);
    }

    let session_id: String = row.get("id");
    let updated = sqlx::query(
        "UPDATE class_sessions SET confirmed = confirmed + 1
         WHERE id = ?1 AND demo_tenant_id = ?2 AND confirmed < capacity AND booking_cutoff > ?3",
    )
    .bind(&session_id)
    .bind(tenant_id)
    .bind(now)
    .execute(&mut **conn)
    .await
    .map_err(|_| BookingError::Database)?;
    if updated.rows_affected() != 1 {
        return Err(BookingError::Full);
    }

    let booking_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bookings
         (id, demo_tenant_id, class_session_id, idempotency_key, guardian_name, guardian_email, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&booking_id)
    .bind(tenant_id)
    .bind(&session_id)
    .bind(idempotency_key)
    .bind("[demo input not retained]")
    .bind("[demo input not retained]")
    .bind(now)
    .execute(&mut **conn)
    .await
    .map_err(|_| BookingError::Database)?;

    let updated_row = sqlx::query(
        "SELECT public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed
         FROM class_sessions WHERE id = ?1",
    )
    .bind(&session_id)
    .fetch_one(&mut **conn)
    .await
    .map_err(|_| BookingError::Database)?;

    Ok(BookingResult {
        booking_id,
        class: session_from_row(&updated_row, now),
        repeated: false,
    })
}

fn opaque_id(prefix: &str) -> String {
    format!("{}{}", prefix, Uuid::new_v4().simple())
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

pub async fn create_workspace(pool: &SqlitePool, school_name: &str, now: i64) -> Result<(Workspace, String), RealError> {
    let workspace = Workspace { id: Uuid::new_v4().to_string(), school_name: school_name.to_owned() };
    let access_key = opaque_id("cct_owner_");
    sqlx::query("INSERT INTO workspaces (id, school_name, access_key_hash, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(&workspace.id).bind(&workspace.school_name).bind(digest(&access_key)).bind(now)
        .execute(pool).await.map_err(|_| RealError::Database)?;
    Ok((workspace, access_key))
}

async fn workspace_for_key(pool: &SqlitePool, access_key: &str) -> Result<Workspace, RealError> {
    let row = sqlx::query("SELECT id, school_name FROM workspaces WHERE access_key_hash = ?1")
        .bind(digest(access_key)).fetch_optional(pool).await.map_err(|_| RealError::Database)?
        .ok_or(RealError::Forbidden)?;
    Ok(Workspace { id: row.get("id"), school_name: row.get("school_name") })
}

pub async fn workspace_from_key(pool: &SqlitePool, access_key: &str) -> Result<Workspace, RealError> {
    workspace_for_key(pool, access_key).await
}

pub async fn create_real_class(pool: &SqlitePool, access_key: &str, name: &str, starts_at: i64, cutoff: i64, timezone: &str, capacity: i64, now: i64) -> Result<RealClass, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let id = Uuid::new_v4().to_string();
    let public_id = opaque_id("class_");
    sqlx::query("INSERT INTO real_classes (id, workspace_id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, ?9)")
        .bind(&id).bind(&workspace.id).bind(&public_id).bind(name).bind(starts_at).bind(cutoff).bind(timezone).bind(capacity).bind(now)
        .execute(pool).await.map_err(|_| RealError::Database)?;
    get_real_class_by_id(pool, &workspace.id, &id, now).await?.ok_or(RealError::Database)
}

pub async fn publish_real_class(pool: &SqlitePool, access_key: &str, id: &str, now: i64) -> Result<RealClass, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let changed = sqlx::query("UPDATE real_classes SET published = 1 WHERE id = ?1 AND workspace_id = ?2")
        .bind(id).bind(&workspace.id).execute(pool).await.map_err(|_| RealError::Database)?;
    if changed.rows_affected() != 1 { return Err(RealError::NotFound); }
    get_real_class_by_id(pool, &workspace.id, id, now).await?.ok_or(RealError::NotFound)
}

pub async fn list_real_classes(pool: &SqlitePool, access_key: &str, now: i64) -> Result<Vec<RealClass>, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let rows = sqlx::query("SELECT c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published, r.calendar_confirmed, r.status AS reconciliation_status FROM real_classes c LEFT JOIN reconciliation_runs r ON r.id = (SELECT id FROM reconciliation_runs WHERE class_id = c.id ORDER BY created_at DESC LIMIT 1) WHERE c.workspace_id = ?1 ORDER BY c.starts_at")
        .bind(&workspace.id).fetch_all(pool).await.map_err(|_| RealError::Database)?;
    Ok(rows.iter().map(|row| real_class_from_row(row, now)).collect())
}

async fn get_real_class_by_id(pool: &SqlitePool, workspace_id: &str, id: &str, now: i64) -> Result<Option<RealClass>, RealError> {
    let row = sqlx::query("SELECT c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published, r.calendar_confirmed, r.status AS reconciliation_status FROM real_classes c LEFT JOIN reconciliation_runs r ON r.id = (SELECT id FROM reconciliation_runs WHERE class_id = c.id ORDER BY created_at DESC LIMIT 1) WHERE c.workspace_id = ?1 AND c.id = ?2")
        .bind(workspace_id).bind(id).fetch_optional(pool).await.map_err(|_| RealError::Database)?;
    Ok(row.as_ref().map(|row| real_class_from_row(row, now)))
}

fn real_class_from_row(row: &sqlx::sqlite::SqliteRow, now: i64) -> RealClass {
    let capacity = row.get::<i64, _>("capacity");
    let confirmed = row.get::<i64, _>("confirmed");
    let cutoff = row.get::<i64, _>("booking_cutoff");
    RealClass { id: row.get("id"), public_id: row.get("public_id"), name: row.get("name"), starts_at: row.get("starts_at"), booking_cutoff: cutoff, timezone: row.get("timezone"), capacity, confirmed, open_seats: capacity - confirmed, availability: availability_at(capacity, confirmed, cutoff, now), published: row.get::<i64, _>("published") == 1, calendar_confirmed: row.try_get("calendar_confirmed").ok(), reconciliation_status: row.try_get("reconciliation_status").ok() }
}

pub async fn get_public_real_class(pool: &SqlitePool, public_id: &str, now: i64) -> Result<Option<RealClass>, RealError> {
    let row = sqlx::query("SELECT c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published, r.calendar_confirmed, r.status AS reconciliation_status FROM real_classes c LEFT JOIN reconciliation_runs r ON r.id = (SELECT id FROM reconciliation_runs WHERE class_id = c.id ORDER BY created_at DESC LIMIT 1) WHERE c.public_id = ?1 AND c.published = 1")
        .bind(public_id).fetch_optional(pool).await.map_err(|_| RealError::Database)?;
    Ok(row.as_ref().map(|row| real_class_from_row(row, now)))
}

pub async fn book_real_seat(pool: &SqlitePool, public_id: &str, idempotency_key: &str, name: &str, email: &str, now: i64) -> Result<RealClass, RealError> {
    let mut conn = pool.acquire().await.map_err(|_| RealError::Database)?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await.map_err(|_| RealError::Database)?;
    let result = async {
        let row = sqlx::query("SELECT id, capacity, confirmed, booking_cutoff, published FROM real_classes WHERE public_id = ?1").bind(public_id).fetch_optional(&mut *conn).await.map_err(|_| RealError::Database)?.ok_or(RealError::NotFound)?;
        let id: String = row.get("id");
        if row.get::<i64, _>("published") != 1 { return Err(RealError::NotFound); }
        if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM real_bookings WHERE class_id = ?1 AND idempotency_key = ?2").bind(&id).bind(idempotency_key).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)? > 0 { let current = sqlx::query("SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published FROM real_classes WHERE id = ?1").bind(&id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?; return Ok(real_class_from_row(&current, now)); }
        if row.get::<i64, _>("confirmed") >= row.get::<i64, _>("capacity") { return Err(RealError::Full); }
        if row.get::<i64, _>("booking_cutoff") <= now { return Err(RealError::Cutoff); }
        let changed = sqlx::query("UPDATE real_classes SET confirmed = confirmed + 1 WHERE id = ?1 AND confirmed < capacity AND booking_cutoff > ?2").bind(&id).bind(now).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        if changed.rows_affected() != 1 { return Err(RealError::Full); }
        sqlx::query("INSERT INTO real_bookings (id, class_id, idempotency_key, guardian_name, guardian_email, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 'confirmed', ?6)").bind(Uuid::new_v4().to_string()).bind(&id).bind(idempotency_key).bind(name).bind(email).bind(now).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        let current = sqlx::query("SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published FROM real_classes WHERE id = ?1").bind(&id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?;
        Ok(real_class_from_row(&current, now))
    }.await;
    match result { Ok(value) => { sqlx::query("COMMIT").execute(&mut *conn).await.map_err(|_| RealError::Database)?; Ok(value) }, Err(error) => { let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await; Err(error) } }
}

pub async fn connect_calendar(pool: &SqlitePool, access_key: &str, label: &str, now: i64) -> Result<CalendarConnection, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    sqlx::query("INSERT INTO calendar_connections (id, workspace_id, label, provider, enabled, created_at) VALUES (?1, ?2, ?3, 'manual_calendar', 1, ?4) ON CONFLICT(workspace_id) DO UPDATE SET label = excluded.label, enabled = 1")
        .bind(Uuid::new_v4().to_string()).bind(&workspace.id).bind(label).bind(now).execute(pool).await.map_err(|_| RealError::Database)?;
    Ok(CalendarConnection { label: label.to_owned(), provider: "manual_calendar".to_owned(), enabled: true })
}

pub async fn reconcile_class(pool: &SqlitePool, access_key: &str, id: &str, calendar_confirmed: i64, now: i64) -> Result<RealClass, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let local = get_real_class_by_id(pool, &workspace.id, id, now).await?.ok_or(RealError::NotFound)?;
    let status = if local.confirmed == calendar_confirmed { "matched" } else { "attention" };
    sqlx::query("INSERT INTO reconciliation_runs (id, class_id, calendar_confirmed, local_confirmed, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .bind(Uuid::new_v4().to_string()).bind(id).bind(calendar_confirmed).bind(local.confirmed).bind(status).bind(now).execute(pool).await.map_err(|_| RealError::Database)?;
    get_real_class_by_id(pool, &workspace.id, id, now).await?.ok_or(RealError::NotFound)
}

pub async fn join_waitlist(pool: &SqlitePool, public_id: &str, name: &str, email: &str, now: i64) -> Result<(), RealError> {
    let class = get_public_real_class(pool, public_id, now).await?.ok_or(RealError::NotFound)?;
    sqlx::query("INSERT INTO waitlist_entries (id, class_id, guardian_name, guardian_email, consented_at, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 'waiting', ?5)")
        .bind(Uuid::new_v4().to_string()).bind(class.id).bind(name).bind(email).bind(now).execute(pool).await.map_err(|_| RealError::Database)?;
    Ok(())
}

pub async fn cancel_booking_and_offer(pool: &SqlitePool, access_key: &str, class_id: &str, booking_id: &str, now: i64) -> Result<Option<String>, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let mut conn = pool.acquire().await.map_err(|_| RealError::Database)?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await.map_err(|_| RealError::Database)?;
    let result = async {
        let owned = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM real_classes WHERE id = ?1 AND workspace_id = ?2").bind(class_id).bind(&workspace.id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?;
        if owned != 1 { return Err(RealError::NotFound); }
        let changed = sqlx::query("UPDATE real_bookings SET status = 'cancelled' WHERE id = ?1 AND class_id = ?2 AND status = 'confirmed'").bind(booking_id).bind(class_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        if changed.rows_affected() != 1 { return Err(RealError::NotFound); }
        sqlx::query("UPDATE real_classes SET confirmed = confirmed - 1 WHERE id = ?1 AND confirmed > 0").bind(class_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        let next = sqlx::query("SELECT id FROM waitlist_entries WHERE class_id = ?1 AND status = 'waiting' ORDER BY created_at LIMIT 1").bind(class_id).fetch_optional(&mut *conn).await.map_err(|_| RealError::Database)?;
        if let Some(row) = next { let entry: String = row.get("id"); let token = opaque_id("offer_"); sqlx::query("INSERT INTO seat_offers (id, waitlist_entry_id, class_id, token_hash, expires_at, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6)").bind(Uuid::new_v4().to_string()).bind(&entry).bind(class_id).bind(digest(&token)).bind(now + 86_400).bind(now).execute(&mut *conn).await.map_err(|_| RealError::Database)?; sqlx::query("UPDATE waitlist_entries SET status = 'offered' WHERE id = ?1").bind(entry).execute(&mut *conn).await.map_err(|_| RealError::Database)?; Ok(Some(token)) } else { Ok(None) }
    }.await;
    match result { Ok(value) => { sqlx::query("COMMIT").execute(&mut *conn).await.map_err(|_| RealError::Database)?; Ok(value) }, Err(error) => { let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await; Err(error) } }
}

pub async fn release_oldest_booking_and_offer(pool: &SqlitePool, access_key: &str, class_id: &str, now: i64) -> Result<Option<String>, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let booking = sqlx::query("SELECT b.id FROM real_bookings b JOIN real_classes c ON c.id = b.class_id WHERE b.class_id = ?1 AND c.workspace_id = ?2 AND b.status = 'confirmed' ORDER BY b.created_at LIMIT 1")
        .bind(class_id).bind(&workspace.id).fetch_optional(pool).await.map_err(|_| RealError::Database)?
        .ok_or(RealError::NotFound)?;
    cancel_booking_and_offer(pool, access_key, class_id, &booking.get::<String, _>("id"), now).await
}

pub async fn get_offer(pool: &SqlitePool, token: &str, now: i64) -> Result<Option<WaitlistOffer>, RealError> {
    let row = sqlx::query("SELECT o.expires_at, c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published FROM seat_offers o JOIN real_classes c ON c.id = o.class_id WHERE o.token_hash = ?1 AND o.status = 'open' AND o.expires_at > ?2").bind(digest(token)).bind(now).fetch_optional(pool).await.map_err(|_| RealError::Database)?;
    Ok(row.as_ref().map(|row| WaitlistOffer { offer_token: token.to_owned(), class: real_class_from_row(row, now), expires_at: row.get("expires_at") }))
}

pub async fn accept_offer(pool: &SqlitePool, token: &str, now: i64) -> Result<RealClass, RealError> {
    let mut conn = pool.acquire().await.map_err(|_| RealError::Database)?; sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await.map_err(|_| RealError::Database)?;
    let result = async { let row = sqlx::query("SELECT o.id, o.waitlist_entry_id, o.class_id, o.expires_at, c.public_id, c.capacity, c.confirmed, c.booking_cutoff FROM seat_offers o JOIN real_classes c ON c.id = o.class_id WHERE o.token_hash = ?1 AND o.status = 'open'").bind(digest(token)).fetch_optional(&mut *conn).await.map_err(|_| RealError::Database)?.ok_or(RealError::OfferUnavailable)?; if row.get::<i64, _>("expires_at") <= now || row.get::<i64, _>("confirmed") >= row.get::<i64, _>("capacity") || row.get::<i64, _>("booking_cutoff") <= now { return Err(RealError::OfferUnavailable); } let class_id: String = row.get("class_id"); let changed = sqlx::query("UPDATE real_classes SET confirmed = confirmed + 1 WHERE id = ?1 AND confirmed < capacity").bind(&class_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?; if changed.rows_affected() != 1 { return Err(RealError::OfferUnavailable); } sqlx::query("UPDATE seat_offers SET status = 'accepted' WHERE id = ?1").bind(row.get::<String, _>("id")).execute(&mut *conn).await.map_err(|_| RealError::Database)?; sqlx::query("UPDATE waitlist_entries SET status = 'accepted' WHERE id = ?1").bind(row.get::<String, _>("waitlist_entry_id")).execute(&mut *conn).await.map_err(|_| RealError::Database)?; let current = sqlx::query("SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published FROM real_classes WHERE id = ?1").bind(&class_id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?; Ok(real_class_from_row(&current, now)) }.await;
    match result { Ok(value) => { sqlx::query("COMMIT").execute(&mut *conn).await.map_err(|_| RealError::Database)?; Ok(value) }, Err(error) => { let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await; Err(error) } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn availability_closes_at_the_exact_cutoff_instant() {
        let cutoff = 1_900_000_000;
        assert_eq!(
            availability_at(8, 6, cutoff, cutoff - 1),
            Availability::Available
        );
        assert_eq!(availability_at(8, 6, cutoff, cutoff), Availability::Cutoff);
        assert_eq!(
            availability_at(8, 6, cutoff, cutoff + 1),
            Availability::Cutoff
        );
    }

    #[test]
    fn full_capacity_takes_precedence_over_a_future_cutoff() {
        assert_eq!(
            availability_at(6, 6, 1_900_000_100, 1_900_000_000),
            Availability::Full
        );
    }
}
