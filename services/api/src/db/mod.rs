use std::{collections::HashMap, env, fs, io, path::Path, str::FromStr, time::Duration};

use anyhow::Context;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use thiserror::Error;
use uuid::Uuid;

use crate::crypto::ContactCipher;

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
    pub subscription_status: String,
    pub trial_ends_at: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingSummary {
    pub id: String,
    pub guardian_name: String,
    pub guardian_email: String,
    pub created_at: i64,
}

pub struct NewRealClass<'a> {
    pub name: &'a str,
    pub starts_at: i64,
    pub cutoff: i64,
    pub timezone: &'a str,
    pub capacity: i64,
}

pub struct OfferDelivery<'a> {
    pub cipher: &'a ContactCipher,
    pub public_base_url: &'a str,
    pub email_configured: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseResult {
    pub offer_token: Option<String>,
    pub offer_url: Option<String>,
    pub expires_at: Option<i64>,
    pub delivery_status: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferReceipt {
    pub id: String,
    pub class_id: String,
    pub class_name: String,
    pub recipient_name: String,
    pub offer_url: String,
    pub expires_at: i64,
    pub offer_status: String,
    pub delivery_status: String,
    pub created_at: i64,
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
    pub last_polled_at: Option<i64>,
    pub last_error: Option<String>,
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
    #[error("school subscription is not active")]
    SubscriptionRequired,
    #[error("database error")]
    Database,
}

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    // Do not issue `PRAGMA journal_mode = …` by default. A Container Apps
    // revision briefly overlaps the revision it replaces, and changing (or
    // even reasserting) the journal mode requires an exclusive SQLite lock.
    // Opening the already-durable database without that write lets the new
    // one-replica revision become healthy before the old revision exits. New
    // databases use SQLite's safe DELETE journal default. Existing WAL files
    // are converted by the post-readiness task after their prior revision has
    // stopped; an explicit local override remains available for development.
    let journal_mode = env::var("SQLITE_JOURNAL_MODE")
        .ok()
        .map(|value| value.to_ascii_lowercase());
    let max_connections = env::var("SQLITE_MAX_CONNECTIONS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        // SQLite state is single-writer and production deliberately has one
        // replica. One connection gives the Azure Files-mounted database a
        // predictable lock owner while still allowing caller-level async
        // concurrency.
        .unwrap_or(1);
    let mut options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        // Azure Files does not implement SQLite's POSIX shared-lock protocol
        // reliably. This product has exactly one replica and sequential
        // revision restarts, so the lockless built-in Unix VFS is safe here
        // and prevents the share from reporting false SQLITE_BUSY failures.
        .vfs("unix-none")
        .busy_timeout(Duration::from_secs(30));
    if let Some(journal_mode) = journal_mode.as_deref() {
        options = match journal_mode {
            "delete" => options.journal_mode(sqlx::sqlite::SqliteJournalMode::Delete),
            "wal" => options.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
            _ => options,
        };
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;
    migrate_or_validate(&pool).await?;
    Ok(pool)
}

/// Move a pre-existing WAL database to SQLite's Azure-Files-safe rollback
/// journal once the prior revision has stopped. This intentionally runs after
/// the listener is ready: journal-mode changes need an exclusive lock, while
/// a healthy replacement revision causes Container Apps to retire the old
/// fallback process.
pub async fn normalize_durable_journal_mode(pool: &SqlitePool) -> anyhow::Result<bool> {
    let current_mode = sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?
        .to_ascii_lowercase();
    if current_mode != "wal" {
        return Ok(false);
    }
    sqlx::query_scalar::<_, String>("PRAGMA journal_mode = DELETE")
        .fetch_one(pool)
        .await?;
    Ok(true)
}

async fn migrate_or_validate(pool: &SqlitePool) -> anyhow::Result<()> {
    let migrator = sqlx::migrate!("./migrations");
    let has_migration_table = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?
        > 0;

    if !has_migration_table {
        migrator.run(pool).await?;
        return Ok(());
    }

    // `Migrator::run` always executes CREATE TABLE IF NOT EXISTS. Azure Files
    // requires an exclusive lock for that no-op DDL, so a replacement revision
    // cannot become ready while a current revision is serving. Once a database
    // has all shipped migrations, validate its exact applied versions and
    // checksums with read-only queries instead.
    let applied = sqlx::query_as::<_, (i64, Vec<u8>, bool)>(
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await?;
    let expected = migrator
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .collect::<Vec<_>>();
    let every_shipped_version_is_applied = expected.iter().all(|migration| {
        applied
            .iter()
            .any(|(version, _checksum, success)| *success && *version == migration.version)
    });
    if every_shipped_version_is_applied {
        let exact_checksums_match = applied.len() == expected.len()
            && applied
                .iter()
                .zip(expected.iter())
                .all(|(actual, migration)| {
                    let (version, checksum, success) = actual;
                    *success
                        && *version == migration.version
                        && checksum.as_slice() == migration.checksum.as_ref()
                });
        if !exact_checksums_match {
            // Earlier deployed SQLx releases recorded compatible migration
            // checksums in a representation that differs from the current
            // embedded macro. The versions are successfully applied, so do
            // not turn a no-op startup into Azure-Files-exclusive DDL.
            tracing::warn!("durable SQLite migration checksums differ from the embedded representation; using applied schema versions");
        }
        return Ok(());
    }

    // First boot and intentional schema upgrades still use SQLx's full,
    // checksummed migration engine. A schema upgrade is a controlled release
    // operation and must obtain SQLite's exclusive lock before serving.
    migrator.run(pool).await?;
    Ok(())
}

pub fn restore_durable_snapshot(backup_path: &Path, database_path: &Path) -> anyhow::Result<()> {
    if backup_path
        .metadata()
        .is_ok_and(|metadata| metadata.len() > 0)
    {
        copy_file_contents(backup_path, database_path)?;
    }
    Ok(())
}

fn copy_file_contents(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let mut input = fs::File::open(source)
        .with_context(|| format!("open checkpoint source {}", source.display()))?;
    let mut output = fs::File::create(destination)
        .with_context(|| format!("create checkpoint destination {}", destination.display()))?;
    io::copy(&mut input, &mut output)
        .with_context(|| format!("copy checkpoint bytes to {}", destination.display()))?;
    output
        .sync_all()
        .with_context(|| format!("sync checkpoint {}", destination.display()))?;
    Ok(())
}

pub async fn persist_durable_snapshot(pool: &SqlitePool, backup_path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let local_snapshot = env::temp_dir().join(format!("cct-{}.db", Uuid::new_v4()));
    let statement = format!(
        "VACUUM INTO '{}'",
        local_snapshot.to_string_lossy().replace('\'', "''")
    );
    if let Err(error) = sqlx::query(&statement).execute(pool).await {
        let _ = fs::remove_file(&local_snapshot);
        return Err(error.into());
    }

    let next_path = backup_path.with_extension("db.next");
    copy_file_contents(&local_snapshot, &next_path)?;
    fs::rename(&next_path, backup_path).with_context(|| {
        format!(
            "atomically replace durable checkpoint {}",
            backup_path.display()
        )
    })?;
    fs::remove_file(&local_snapshot)
        .with_context(|| format!("remove temporary checkpoint {}", local_snapshot.display()))?;
    Ok(())
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

pub async fn create_workspace(
    pool: &SqlitePool,
    school_name: &str,
    owner_oid: &str,
    now: i64,
) -> Result<(Workspace, String), RealError> {
    let workspace = Workspace {
        id: Uuid::new_v4().to_string(),
        school_name: school_name.to_owned(),
        subscription_status: "trial".to_owned(),
        trial_ends_at: Some(now + 14 * 86_400),
    };
    let access_key = opaque_id("cct_owner_");
    let mut tx = pool.begin().await.map_err(|_| RealError::Database)?;
    sqlx::query("INSERT INTO workspaces (id, school_name, access_key_hash, created_at, subscription_status, trial_ends_at) VALUES (?1, ?2, ?3, ?4, 'trial', ?5)")
        .bind(&workspace.id).bind(&workspace.school_name).bind(digest(&access_key)).bind(now).bind(workspace.trial_ends_at)
        .execute(&mut *tx).await.map_err(|_| RealError::Database)?;
    sqlx::query("INSERT INTO workspace_members (workspace_id, oid, role, created_at) VALUES (?1, ?2, 'owner', ?3)")
        .bind(&workspace.id).bind(owner_oid).bind(now).execute(&mut *tx).await.map_err(|_| RealError::Database)?;
    tx.commit().await.map_err(|_| RealError::Database)?;
    Ok((workspace, access_key))
}

async fn workspace_for_key(pool: &SqlitePool, access_key: &str) -> Result<Workspace, RealError> {
    let row = sqlx::query("SELECT id, school_name, subscription_status, trial_ends_at FROM workspaces WHERE access_key_hash = ?1 OR id = ?2")
        .bind(digest(access_key)).bind(access_key)
        .fetch_optional(pool)
        .await
        .map_err(|_| RealError::Database)?
        .ok_or(RealError::Forbidden)?;
    Ok(Workspace {
        id: row.get("id"),
        school_name: row.get("school_name"),
        subscription_status: row.get("subscription_status"),
        trial_ends_at: row.get("trial_ends_at"),
    })
}

pub async fn workspace_for_oid(pool: &SqlitePool, oid: &str) -> Result<Workspace, RealError> {
    let row = sqlx::query("SELECT w.id, w.school_name, w.subscription_status, w.trial_ends_at FROM workspaces w JOIN workspace_members m ON m.workspace_id = w.id WHERE m.oid = ?1 ORDER BY w.created_at LIMIT 1")
        .bind(oid).fetch_optional(pool).await.map_err(|_| RealError::Database)?.ok_or(RealError::NotFound)?;
    Ok(Workspace {
        id: row.get("id"),
        school_name: row.get("school_name"),
        subscription_status: row.get("subscription_status"),
        trial_ends_at: row.get("trial_ends_at"),
    })
}

pub async fn authorize_workspace(
    pool: &SqlitePool,
    access_key: &str,
    oid: &str,
    allowed_roles: &[&str],
) -> Result<(), RealError> {
    let role = sqlx::query_scalar::<_, String>("SELECT m.role FROM workspace_members m JOIN workspaces w ON w.id = m.workspace_id WHERE (w.access_key_hash = ?1 OR w.id = ?2) AND m.oid = ?3")
        .bind(digest(access_key)).bind(access_key).bind(oid).fetch_optional(pool).await.map_err(|_| RealError::Database)?
        .ok_or(RealError::Forbidden)?;
    if allowed_roles.contains(&role.as_str()) {
        Ok(())
    } else {
        Err(RealError::Forbidden)
    }
}

pub async fn ensure_entitled(
    pool: &SqlitePool,
    access_key: &str,
    now: i64,
) -> Result<(), RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    if workspace.subscription_status == "active"
        || workspace.subscription_status == "grace"
        || (workspace.subscription_status == "trial"
            && workspace.trial_ends_at.is_some_and(|end| end > now))
    {
        Ok(())
    } else {
        Err(RealError::SubscriptionRequired)
    }
}

pub async fn workspace_from_key(
    pool: &SqlitePool,
    access_key: &str,
) -> Result<Workspace, RealError> {
    workspace_for_key(pool, access_key).await
}

pub async fn create_real_class(
    pool: &SqlitePool,
    access_key: &str,
    input: NewRealClass<'_>,
    now: i64,
) -> Result<RealClass, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let id = Uuid::new_v4().to_string();
    let public_id = opaque_id("class_");
    sqlx::query("INSERT INTO real_classes (id, workspace_id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, ?9)")
        .bind(&id).bind(&workspace.id).bind(&public_id).bind(input.name).bind(input.starts_at).bind(input.cutoff).bind(input.timezone).bind(input.capacity).bind(now)
        .execute(pool).await.map_err(|_| RealError::Database)?;
    get_real_class_by_id(pool, &workspace.id, &id, now)
        .await?
        .ok_or(RealError::Database)
}

pub async fn publish_real_class(
    pool: &SqlitePool,
    access_key: &str,
    id: &str,
    now: i64,
) -> Result<RealClass, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let changed =
        sqlx::query("UPDATE real_classes SET published = 1 WHERE id = ?1 AND workspace_id = ?2")
            .bind(id)
            .bind(&workspace.id)
            .execute(pool)
            .await
            .map_err(|_| RealError::Database)?;
    if changed.rows_affected() != 1 {
        return Err(RealError::NotFound);
    }
    get_real_class_by_id(pool, &workspace.id, id, now)
        .await?
        .ok_or(RealError::NotFound)
}

pub async fn list_real_classes(
    pool: &SqlitePool,
    access_key: &str,
    now: i64,
) -> Result<Vec<RealClass>, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let rows = sqlx::query("SELECT c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published, r.calendar_confirmed, r.status AS reconciliation_status FROM real_classes c LEFT JOIN reconciliation_runs r ON r.id = (SELECT id FROM reconciliation_runs WHERE class_id = c.id ORDER BY created_at DESC LIMIT 1) WHERE c.workspace_id = ?1 ORDER BY c.starts_at")
        .bind(&workspace.id).fetch_all(pool).await.map_err(|_| RealError::Database)?;
    Ok(rows
        .iter()
        .map(|row| real_class_from_row(row, now))
        .collect())
}

async fn get_real_class_by_id(
    pool: &SqlitePool,
    workspace_id: &str,
    id: &str,
    now: i64,
) -> Result<Option<RealClass>, RealError> {
    let row = sqlx::query("SELECT c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published, r.calendar_confirmed, r.status AS reconciliation_status FROM real_classes c LEFT JOIN reconciliation_runs r ON r.id = (SELECT id FROM reconciliation_runs WHERE class_id = c.id ORDER BY created_at DESC LIMIT 1) WHERE c.workspace_id = ?1 AND c.id = ?2")
        .bind(workspace_id).bind(id).fetch_optional(pool).await.map_err(|_| RealError::Database)?;
    Ok(row.as_ref().map(|row| real_class_from_row(row, now)))
}

fn real_class_from_row(row: &sqlx::sqlite::SqliteRow, now: i64) -> RealClass {
    let capacity = row.get::<i64, _>("capacity");
    let confirmed = row.get::<i64, _>("confirmed");
    let cutoff = row.get::<i64, _>("booking_cutoff");
    RealClass {
        id: row.get("id"),
        public_id: row.get("public_id"),
        name: row.get("name"),
        starts_at: row.get("starts_at"),
        booking_cutoff: cutoff,
        timezone: row.get("timezone"),
        capacity,
        confirmed,
        open_seats: capacity - confirmed,
        availability: availability_at(capacity, confirmed, cutoff, now),
        published: row.get::<i64, _>("published") == 1,
        calendar_confirmed: row.try_get("calendar_confirmed").ok(),
        reconciliation_status: row.try_get("reconciliation_status").ok(),
    }
}

pub async fn get_public_real_class(
    pool: &SqlitePool,
    public_id: &str,
    now: i64,
) -> Result<Option<RealClass>, RealError> {
    let row = sqlx::query("SELECT c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published, r.calendar_confirmed, r.status AS reconciliation_status FROM real_classes c LEFT JOIN reconciliation_runs r ON r.id = (SELECT id FROM reconciliation_runs WHERE class_id = c.id ORDER BY created_at DESC LIMIT 1) WHERE c.public_id = ?1 AND c.published = 1")
        .bind(public_id).fetch_optional(pool).await.map_err(|_| RealError::Database)?;
    Ok(row.as_ref().map(|row| real_class_from_row(row, now)))
}

pub async fn book_real_seat(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    public_id: &str,
    idempotency_key: &str,
    name: &str,
    email: &str,
    now: i64,
) -> Result<RealClass, RealError> {
    let encrypted_name = cipher.encrypt(name).map_err(|_| RealError::Database)?;
    let encrypted_email = cipher.encrypt(email).map_err(|_| RealError::Database)?;
    let mut conn = pool.acquire().await.map_err(|_| RealError::Database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(|_| RealError::Database)?;
    let result = async {
        let row = sqlx::query("SELECT id, capacity, confirmed, booking_cutoff, published FROM real_classes WHERE public_id = ?1").bind(public_id).fetch_optional(&mut *conn).await.map_err(|_| RealError::Database)?.ok_or(RealError::NotFound)?;
        let id: String = row.get("id");
        if row.get::<i64, _>("published") != 1 { return Err(RealError::NotFound); }
        if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM real_bookings WHERE class_id = ?1 AND idempotency_key = ?2").bind(&id).bind(idempotency_key).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)? > 0 { let current = sqlx::query("SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published FROM real_classes WHERE id = ?1").bind(&id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?; return Ok(real_class_from_row(&current, now)); }
        if row.get::<i64, _>("confirmed") >= row.get::<i64, _>("capacity") { return Err(RealError::Full); }
        if row.get::<i64, _>("booking_cutoff") <= now { return Err(RealError::Cutoff); }
        let changed = sqlx::query("UPDATE real_classes SET confirmed = confirmed + 1 WHERE id = ?1 AND confirmed < capacity AND booking_cutoff > ?2").bind(&id).bind(now).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        if changed.rows_affected() != 1 { return Err(RealError::Full); }
        sqlx::query("INSERT INTO real_bookings (id, class_id, idempotency_key, guardian_name, guardian_email, status, created_at, contact_expires_at) VALUES (?1, ?2, ?3, ?4, ?5, 'confirmed', ?6, ?7)").bind(Uuid::new_v4().to_string()).bind(&id).bind(idempotency_key).bind(&encrypted_name).bind(&encrypted_email).bind(now).bind(now + 90 * 86_400).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        let current = sqlx::query("SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published FROM real_classes WHERE id = ?1").bind(&id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?;
        Ok(real_class_from_row(&current, now))
    }.await;
    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(|_| RealError::Database)?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
}

pub async fn connect_calendar(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    access_key: &str,
    label: &str,
    feed_url: &str,
    now: i64,
) -> Result<CalendarConnection, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let encrypted_url = cipher.encrypt(feed_url).map_err(|_| RealError::Database)?;
    sqlx::query("INSERT INTO calendar_connections (id, workspace_id, label, provider, enabled, created_at, feed_url_encrypted, next_poll_at) VALUES (?1, ?2, ?3, 'ical_feed', 1, ?4, ?5, ?4) ON CONFLICT(workspace_id) DO UPDATE SET label = excluded.label, provider = 'ical_feed', enabled = 1, feed_url_encrypted = excluded.feed_url_encrypted, next_poll_at = excluded.next_poll_at, last_error = NULL")
        .bind(Uuid::new_v4().to_string()).bind(&workspace.id).bind(label).bind(now).bind(encrypted_url).execute(pool).await.map_err(|_| RealError::Database)?;
    Ok(CalendarConnection {
        label: label.to_owned(),
        provider: "ical_feed".to_owned(),
        enabled: true,
        last_polled_at: None,
        last_error: None,
    })
}

pub async fn reconcile_class(
    pool: &SqlitePool,
    access_key: &str,
    id: &str,
    calendar_confirmed: i64,
    now: i64,
) -> Result<RealClass, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let local = get_real_class_by_id(pool, &workspace.id, id, now)
        .await?
        .ok_or(RealError::NotFound)?;
    let status = if local.confirmed == calendar_confirmed {
        "matched"
    } else {
        "attention"
    };
    sqlx::query("INSERT INTO reconciliation_runs (id, class_id, calendar_confirmed, local_confirmed, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
        .bind(Uuid::new_v4().to_string()).bind(id).bind(calendar_confirmed).bind(local.confirmed).bind(status).bind(now).execute(pool).await.map_err(|_| RealError::Database)?;
    get_real_class_by_id(pool, &workspace.id, id, now)
        .await?
        .ok_or(RealError::NotFound)
}

pub async fn join_waitlist(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    public_id: &str,
    idempotency_key: &str,
    name: &str,
    email: &str,
    now: i64,
) -> Result<String, RealError> {
    let class = get_public_real_class(pool, public_id, now)
        .await?
        .ok_or(RealError::NotFound)?;
    if let Some(id) = sqlx::query_scalar::<_, String>(
        "SELECT id FROM waitlist_entries WHERE class_id = ?1 AND idempotency_key = ?2",
    )
    .bind(&class.id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
    .map_err(|_| RealError::Database)?
    {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    let encrypted_name = cipher.encrypt(name).map_err(|_| RealError::Database)?;
    let encrypted_email = cipher.encrypt(email).map_err(|_| RealError::Database)?;
    sqlx::query("INSERT INTO waitlist_entries (id, class_id, guardian_name, guardian_email, consented_at, status, created_at, contact_expires_at, idempotency_key) VALUES (?1, ?2, ?3, ?4, ?5, 'waiting', ?5, ?6, ?7)")
        .bind(&id).bind(class.id).bind(encrypted_name).bind(encrypted_email).bind(now).bind(now + 90 * 86_400).bind(idempotency_key).execute(pool).await.map_err(|_| RealError::Database)?;
    Ok(id)
}

pub async fn cancel_booking_and_offer(
    pool: &SqlitePool,
    delivery: OfferDelivery<'_>,
    access_key: &str,
    class_id: &str,
    booking_id: &str,
    now: i64,
) -> Result<ReleaseResult, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let mut conn = pool.acquire().await.map_err(|_| RealError::Database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(|_| RealError::Database)?;
    let result = async {
        let owned = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM real_classes WHERE id = ?1 AND workspace_id = ?2").bind(class_id).bind(&workspace.id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?;
        if owned != 1 { return Err(RealError::NotFound); }
        let changed = sqlx::query("UPDATE real_bookings SET status = 'cancelled' WHERE id = ?1 AND class_id = ?2 AND status = 'confirmed'").bind(booking_id).bind(class_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        if changed.rows_affected() != 1 { return Err(RealError::NotFound); }
        sqlx::query("UPDATE real_classes SET confirmed = confirmed - 1 WHERE id = ?1 AND confirmed > 0").bind(class_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
        let next = sqlx::query("SELECT id, guardian_email FROM waitlist_entries WHERE class_id = ?1 AND status = 'waiting' ORDER BY created_at LIMIT 1").bind(class_id).fetch_optional(&mut *conn).await.map_err(|_| RealError::Database)?;
        if let Some(row) = next {
            let entry: String = row.get("id");
            let recipient: String = row.get("guardian_email");
            let token = opaque_id("offer_");
            let offer_id = Uuid::new_v4().to_string();
            let expires_at = now + 86_400;
            let offer_url = format!("{}/offer/{token}", delivery.public_base_url.trim_end_matches('/'));
            let token_encrypted = delivery.cipher.encrypt(&token).map_err(|_| RealError::Database)?;
            let delivery_status = if delivery.email_configured { "email_queued" } else { "ready_to_copy" };
            sqlx::query("INSERT INTO seat_offers (id, waitlist_entry_id, class_id, token_hash, expires_at, status, created_at, token_encrypted, delivery_status) VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6, ?7, ?8)").bind(&offer_id).bind(&entry).bind(class_id).bind(digest(&token)).bind(expires_at).bind(now).bind(token_encrypted).bind(delivery_status).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
            sqlx::query("UPDATE waitlist_entries SET status = 'offered' WHERE id = ?1").bind(entry).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
            if delivery.email_configured {
                let body = format!("A class seat is available for 24 hours. Accept it at {offer_url}");
                sqlx::query("INSERT INTO email_outbox (id, workspace_id, recipient_encrypted, subject, text_body, status, attempts, next_attempt_at, created_at, seat_offer_id) VALUES (?1, ?2, ?3, 'A class seat is available', ?4, 'pending', 0, ?5, ?5, ?6)")
                    .bind(Uuid::new_v4().to_string()).bind(&workspace.id).bind(recipient).bind(body).bind(now).bind(&offer_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?;
            }
            Ok(ReleaseResult { offer_token: Some(token), offer_url: Some(offer_url), expires_at: Some(expires_at), delivery_status })
        } else {
            Ok(ReleaseResult { offer_token: None, offer_url: None, expires_at: None, delivery_status: "not_needed" })
        }
    }.await;
    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(|_| RealError::Database)?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
}

pub async fn release_oldest_booking_and_offer(
    pool: &SqlitePool,
    delivery: OfferDelivery<'_>,
    access_key: &str,
    class_id: &str,
    now: i64,
) -> Result<ReleaseResult, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let booking = sqlx::query("SELECT b.id FROM real_bookings b JOIN real_classes c ON c.id = b.class_id WHERE b.class_id = ?1 AND c.workspace_id = ?2 AND b.status = 'confirmed' ORDER BY b.created_at LIMIT 1")
        .bind(class_id).bind(&workspace.id).fetch_optional(pool).await.map_err(|_| RealError::Database)?
        .ok_or(RealError::NotFound)?;
    cancel_booking_and_offer(
        pool,
        delivery,
        access_key,
        class_id,
        &booking.get::<String, _>("id"),
        now,
    )
    .await
}

pub async fn list_offer_receipts(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    access_key: &str,
    offer_base_url: &str,
    now: i64,
) -> Result<Vec<OfferReceipt>, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let rows = sqlx::query("SELECT o.id, o.class_id, c.name AS class_name, w.guardian_name, o.token_encrypted, o.expires_at, o.status, o.delivery_status, o.created_at FROM seat_offers o JOIN real_classes c ON c.id = o.class_id JOIN waitlist_entries w ON w.id = o.waitlist_entry_id WHERE c.workspace_id = ?1 AND o.token_encrypted IS NOT NULL ORDER BY o.created_at DESC LIMIT 50")
        .bind(workspace.id).fetch_all(pool).await.map_err(|_| RealError::Database)?;
    rows.iter()
        .map(|row| {
            let token = cipher
                .decrypt(&row.get::<String, _>("token_encrypted"))
                .map_err(|_| RealError::Database)?;
            let status: String = row.get("status");
            let expires_at: i64 = row.get("expires_at");
            let stored_delivery: String = row.get("delivery_status");
            let delivery_status = if status == "accepted" {
                "accepted".to_owned()
            } else if expires_at <= now {
                "expired".to_owned()
            } else {
                stored_delivery
            };
            Ok(OfferReceipt {
                id: row.get("id"),
                class_id: row.get("class_id"),
                class_name: row.get("class_name"),
                recipient_name: cipher
                    .decrypt(&row.get::<String, _>("guardian_name"))
                    .map_err(|_| RealError::Database)?,
                offer_url: format!("{}/offer/{token}", offer_base_url.trim_end_matches('/')),
                expires_at,
                offer_status: status,
                delivery_status,
                created_at: row.get("created_at"),
            })
        })
        .collect()
}

pub async fn workspace_operational_metrics(
    pool: &SqlitePool,
    access_key: &str,
    now: i64,
) -> Result<crate::metrics::WorkspaceMetrics, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let calendar_job_lag_seconds = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(?1 - next_poll_at), 0) FROM calendar_connections WHERE workspace_id = ?2 AND enabled = 1 AND next_poll_at IS NOT NULL AND next_poll_at <= ?1",
    )
    .bind(now)
    .bind(&workspace.id)
    .fetch_one(pool)
    .await
    .map_err(|_| RealError::Database)?;
    let unresolved_discrepancies = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM real_classes c WHERE c.workspace_id = ?1 AND (SELECT status FROM reconciliation_runs r WHERE r.class_id = c.id ORDER BY r.created_at DESC LIMIT 1) = 'attention'",
    )
    .bind(&workspace.id)
    .fetch_one(pool)
    .await
    .map_err(|_| RealError::Database)?;
    let (offers_created, offers_accepted) = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN o.status = 'accepted' THEN 1 ELSE 0 END), 0) FROM seat_offers o JOIN real_classes c ON c.id = o.class_id WHERE c.workspace_id = ?1",
    )
    .bind(&workspace.id)
    .fetch_one(pool)
    .await
    .map_err(|_| RealError::Database)?;
    Ok(crate::metrics::WorkspaceMetrics {
        calendar_job_lag_seconds,
        unresolved_discrepancies,
        offers_created,
        offers_accepted,
    })
}

pub async fn list_confirmed_bookings(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    access_key: &str,
    class_id: &str,
) -> Result<Vec<BookingSummary>, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let rows = sqlx::query("SELECT b.id, b.guardian_name, b.guardian_email, b.created_at FROM real_bookings b JOIN real_classes c ON c.id = b.class_id WHERE b.class_id = ?1 AND c.workspace_id = ?2 AND b.status = 'confirmed' ORDER BY b.created_at")
        .bind(class_id).bind(workspace.id).fetch_all(pool).await.map_err(|_| RealError::Database)?;
    rows.iter()
        .map(|row| {
            Ok(BookingSummary {
                id: row.get("id"),
                guardian_name: cipher
                    .decrypt(&row.get::<String, _>("guardian_name"))
                    .map_err(|_| RealError::Database)?,
                guardian_email: cipher
                    .decrypt(&row.get::<String, _>("guardian_email"))
                    .map_err(|_| RealError::Database)?,
                created_at: row.get("created_at"),
            })
        })
        .collect()
}

pub async fn export_workspace(
    pool: &SqlitePool,
    cipher: &ContactCipher,
    access_key: &str,
) -> Result<serde_json::Value, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    let classes = list_real_classes(pool, access_key, i64::MAX / 4).await?;
    let mut bookings = HashMap::new();
    for class in &classes {
        bookings.insert(
            class.id.clone(),
            list_confirmed_bookings(pool, cipher, access_key, &class.id).await?,
        );
    }
    Ok(
        serde_json::json!({"workspace": workspace, "classes": classes, "confirmedBookings": bookings}),
    )
}

pub async fn delete_workspace(pool: &SqlitePool, access_key: &str) -> Result<(), RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    sqlx::query("DELETE FROM workspaces WHERE id = ?1")
        .bind(workspace.id)
        .execute(pool)
        .await
        .map_err(|_| RealError::Database)?;
    Ok(())
}

pub async fn activate_subscription(
    pool: &SqlitePool,
    access_key: &str,
    external_reference: &str,
    now: i64,
) -> Result<Workspace, RealError> {
    let workspace = workspace_for_key(pool, access_key).await?;
    sqlx::query("INSERT OR IGNORE INTO billing_events (id, workspace_id, external_reference_hash, status, created_at) VALUES (?1, ?2, ?3, 'active', ?4)")
        .bind(Uuid::new_v4().to_string()).bind(&workspace.id).bind(digest(external_reference)).bind(now).execute(pool).await.map_err(|_| RealError::Database)?;
    sqlx::query("UPDATE workspaces SET subscription_status = 'active' WHERE id = ?1")
        .bind(&workspace.id)
        .execute(pool)
        .await
        .map_err(|_| RealError::Database)?;
    workspace_from_key(pool, access_key).await
}

pub async fn cleanup_retained_contacts(pool: &SqlitePool, now: i64) -> anyhow::Result<u64> {
    let bookings = sqlx::query("UPDATE real_bookings SET guardian_name = '[deleted]', guardian_email = '[deleted]' WHERE contact_expires_at IS NOT NULL AND contact_expires_at <= ?1 AND guardian_name != '[deleted]'").bind(now).execute(pool).await?;
    let waitlist = sqlx::query("UPDATE waitlist_entries SET guardian_name = '[deleted]', guardian_email = '[deleted]' WHERE contact_expires_at IS NOT NULL AND contact_expires_at <= ?1 AND guardian_name != '[deleted]'").bind(now).execute(pool).await?;
    Ok(bookings.rows_affected() + waitlist.rows_affected())
}

pub async fn get_offer(
    pool: &SqlitePool,
    token: &str,
    now: i64,
) -> Result<Option<WaitlistOffer>, RealError> {
    let row = sqlx::query("SELECT o.expires_at, c.id, c.public_id, c.name, c.starts_at, c.booking_cutoff, c.timezone, c.capacity, c.confirmed, c.published FROM seat_offers o JOIN real_classes c ON c.id = o.class_id WHERE o.token_hash = ?1 AND o.status = 'open' AND o.expires_at > ?2").bind(digest(token)).bind(now).fetch_optional(pool).await.map_err(|_| RealError::Database)?;
    Ok(row.as_ref().map(|row| WaitlistOffer {
        offer_token: token.to_owned(),
        class: real_class_from_row(row, now),
        expires_at: row.get("expires_at"),
    }))
}

pub async fn accept_offer(
    pool: &SqlitePool,
    token: &str,
    now: i64,
) -> Result<RealClass, RealError> {
    let mut conn = pool.acquire().await.map_err(|_| RealError::Database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(|_| RealError::Database)?;
    let result = async { let row = sqlx::query("SELECT o.id, o.waitlist_entry_id, o.class_id, o.expires_at, c.public_id, c.capacity, c.confirmed, c.booking_cutoff FROM seat_offers o JOIN real_classes c ON c.id = o.class_id WHERE o.token_hash = ?1 AND o.status = 'open'").bind(digest(token)).fetch_optional(&mut *conn).await.map_err(|_| RealError::Database)?.ok_or(RealError::OfferUnavailable)?; if row.get::<i64, _>("expires_at") <= now || row.get::<i64, _>("confirmed") >= row.get::<i64, _>("capacity") || row.get::<i64, _>("booking_cutoff") <= now { return Err(RealError::OfferUnavailable); } let class_id: String = row.get("class_id"); let changed = sqlx::query("UPDATE real_classes SET confirmed = confirmed + 1 WHERE id = ?1 AND confirmed < capacity").bind(&class_id).execute(&mut *conn).await.map_err(|_| RealError::Database)?; if changed.rows_affected() != 1 { return Err(RealError::OfferUnavailable); } sqlx::query("UPDATE seat_offers SET status = 'accepted' WHERE id = ?1").bind(row.get::<String, _>("id")).execute(&mut *conn).await.map_err(|_| RealError::Database)?; sqlx::query("UPDATE waitlist_entries SET status = 'accepted' WHERE id = ?1").bind(row.get::<String, _>("waitlist_entry_id")).execute(&mut *conn).await.map_err(|_| RealError::Database)?; let current = sqlx::query("SELECT id, public_id, name, starts_at, booking_cutoff, timezone, capacity, confirmed, published FROM real_classes WHERE id = ?1").bind(&class_id).fetch_one(&mut *conn).await.map_err(|_| RealError::Database)?; Ok(real_class_from_row(&current, now)) }.await;
    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(|_| RealError::Database)?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
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
