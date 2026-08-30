use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine};
use class_capacity_truth_api::{app, cleanup_task, crypto, db, integration_task, AppState};
use rand::RngCore;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let data_dir = env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data"));
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("create data directory {}", data_dir.display()))?;
    let durable_backup_path = env::var("DURABLE_BACKUP_PATH").ok().map(PathBuf::from);
    let default_database_path = if durable_backup_path.is_some() {
        PathBuf::from("/tmp/class-capacity-truth.db")
    } else {
        // The failed pre-repair revision left a never-ready WAL bootstrap at
        // this share's original filename. Keep that file intact for recovery
        // rather than deleting or overwriting it; all serving durable state
        // starts in this rollback-journal database on the mounted /data share.
        data_dir.join("class-capacity-truth-state-v4.db")
    };
    if let Some(backup_path) = durable_backup_path.as_deref() {
        db::restore_durable_snapshot(backup_path, &default_database_path)?;
    }
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| format!("sqlite://{}", default_database_path.display()));
    let (cookie_key, key_source) = load_cookie_key(&data_dir)?;
    let (contact_cipher, contact_key_source) = crypto::load_or_create_key(&data_dir)?;
    let pool = db::connect(&database_url).await?;
    if let Some(backup_path) = durable_backup_path.as_deref() {
        db::persist_durable_snapshot(&pool, backup_path).await?;
    }
    let frontend_dist = env::var("FRONTEND_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/app/dist"));

    tracing::info!(
        port,
        database_config = if env::var_os("DATABASE_URL").is_some() {
            "supplied"
        } else {
            "generated-default"
        },
        sqlite_journal_mode = env::var("SQLITE_JOURNAL_MODE").unwrap_or_else(|_| "delete".into()),
        sqlite_max_connections = env::var("SQLITE_MAX_CONNECTIONS").unwrap_or_else(|_| "1".into()),
        durable_backup = if durable_backup_path.is_some() {
            "supplied"
        } else {
            "disabled"
        },
        cookie_signing_key = key_source,
        contact_encryption_key = contact_key_source,
        smtp = if env::var_os("SMTP_RELAY").is_some() {
            "supplied"
        } else {
            "local-capture"
        },
        "configuration ready"
    );

    let state = AppState {
        pool: pool.clone(),
        cookie_key: Arc::new(cookie_key),
        contact_cipher,
        auth: class_capacity_truth_api::auth::AuthVerifier::from_env(),
        public_base_url: Arc::new(
            env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "https://class-capacity-truth.sociobot.in".into()),
        ),
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?,
        email_delivery_configured: env::var_os("SMTP_RELAY").is_some(),
        durable_backup_path: durable_backup_path.map(Arc::new),
        backup_lock: Arc::new(tokio::sync::Mutex::new(())),
        metrics: class_capacity_truth_api::metrics::AppMetrics::default(),
    };
    let router = app(state.clone(), frontend_dist, 6_000, 10);
    tokio::spawn(cleanup_task(state.clone()));
    tokio::spawn(integration_task(state));
    if env::var_os("SQLITE_JOURNAL_MODE").is_none() {
        tokio::spawn(normalize_durable_journal_task(pool.clone()));
    }
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "server listening");
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

async fn normalize_durable_journal_task(pool: sqlx::SqlitePool) {
    // Give the start-up probe a chance to mark this replacement ready before
    // taking an exclusive journal-mode lock. Retrying is deliberate: the old
    // revision may still be draining its final request against the same /data
    // mount.
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    for attempt in 1..=60 {
        match db::normalize_durable_journal_mode(&pool).await {
            Ok(true) => {
                tracing::info!(attempt, "normalized durable SQLite journal to DELETE");
                return;
            }
            Ok(false) => {
                tracing::info!("durable SQLite journal already uses DELETE");
                return;
            }
            Err(error) if attempt < 60 => {
                tracing::warn!(attempt, error = %error, "waiting to normalize durable SQLite journal");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(error) => {
                tracing::error!(error = %error, "could not normalize durable SQLite journal after revision handoff");
            }
        }
    }
}

fn load_cookie_key(data_dir: &Path) -> anyhow::Result<(Vec<u8>, &'static str)> {
    if let Ok(value) = env::var("COOKIE_SIGNING_KEY") {
        let key = STANDARD
            .decode(value)
            .context("COOKIE_SIGNING_KEY must be base64")?;
        anyhow::ensure!(
            key.len() >= 32,
            "COOKIE_SIGNING_KEY must decode to at least 32 bytes"
        );
        return Ok((key, "supplied"));
    }
    let path = data_dir.join("demo-cookie.key");
    if let Ok(value) = fs::read(&path) {
        anyhow::ensure!(value.len() >= 32, "persisted cookie key is too short");
        return Ok((value, "persisted-generated"));
    }
    let mut value = vec![0_u8; 32];
    rand::rng().fill_bytes(&mut value);
    fs::write(&path, &value).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Azure Files is mounted with the Container App's access controls and
        // rejects POSIX chmod. The key is still private to the mounted share;
        // local filesystems retain the stricter 0600 mode.
        crypto::allow_azure_files_permission_denied(fs::set_permissions(
            &path,
            fs::Permissions::from_mode(0o600),
        ))?;
    }
    Ok((value, "generated-and-persisted"))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler")
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
