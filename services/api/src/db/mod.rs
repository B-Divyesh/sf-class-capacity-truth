use std::{str::FromStr, time::Duration};

use serde::Serialize;
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
    let mut tx = pool.begin().await?;
    let active = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM demo_tenants WHERE id = ?1 AND expires_at > ?2",
    )
    .bind(tenant_id)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;

    if active == 0 {
        sqlx::query("DELETE FROM demo_tenants WHERE id = ?1")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO demo_tenants (id, created_at, expires_at) VALUES (?1, ?2, ?3)")
            .bind(tenant_id)
            .bind(now)
            .bind(now + DEMO_TTL_SECONDS)
            .execute(&mut *tx)
            .await?;
        seed_sessions(&mut tx, tenant_id, now).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn reset_demo(pool: &SqlitePool, tenant_id: &str, now: i64) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM demo_tenants WHERE id = ?1")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO demo_tenants (id, created_at, expires_at) VALUES (?1, ?2, ?3)")
        .bind(tenant_id)
        .bind(now)
        .bind(now + DEMO_TTL_SECONDS)
        .execute(&mut *tx)
        .await?;
    seed_sessions(&mut tx, tenant_id, now).await?;
    tx.commit().await?;
    Ok(())
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
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
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
    let availability = if confirmed >= capacity {
        Availability::Full
    } else if cutoff <= now {
        Availability::Cutoff
    } else {
        Availability::Available
    };
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
