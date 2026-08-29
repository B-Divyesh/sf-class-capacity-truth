use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

const DEFAULT_TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[derive(Clone)]
pub struct AuthVerifier {
    tenant_id: String,
    client_id: String,
    discovery_url: String,
    cache: Arc<RwLock<Option<CachedKeys>>>,
    test_token: Option<String>,
}

struct CachedKeys {
    issuer: String,
    keys: Vec<Jwk>,
    loaded_at: Instant,
}

#[derive(Clone, Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

#[derive(Clone, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Clone, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct Claims {
    aud: String,
    tid: String,
    iss: String,
    oid: String,
    exp: usize,
    nbf: Option<usize>,
}

impl AuthVerifier {
    pub fn from_env() -> Self {
        let tenant_id = env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT_ID.into());
        let subdomain =
            env::var("ENTRA_TENANT_SUBDOMAIN").unwrap_or_else(|_| DEFAULT_SUBDOMAIN.into());
        let client_id = env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.into());
        let discovery_url = format!(
            "https://{subdomain}.ciamlogin.com/{tenant_id}/v2.0/.well-known/openid-configuration"
        );
        Self {
            tenant_id,
            client_id,
            discovery_url,
            cache: Arc::new(RwLock::new(None)),
            test_token: env::var("TEST_AUTH_TOKEN").ok(),
        }
    }

    pub fn for_tests() -> Self {
        let mut verifier = Self::from_env();
        verifier.test_token = Some("test-owner".into());
        verifier
    }

    pub async fn verify(&self, token: &str) -> anyhow::Result<String> {
        // A production release may temporarily use one exact fixture token for
        // a controlled persistence drill. Never accept the broad test-owner
        // prefix there; that convenience is restricted to debug test servers.
        let debug_test_prefix = cfg!(debug_assertions) && token.starts_with("test-owner-");
        if self.test_token.is_some()
            && (self.test_token.as_deref() == Some(token) || debug_test_prefix)
        {
            return Ok(if token == "test-owner" {
                "test-owner-oid".into()
            } else {
                format!("{token}-oid")
            });
        }
        self.refresh_if_needed().await?;
        let cache = self.cache.read().await;
        let cache = cache.as_ref().context("identity keys unavailable")?;
        let header = decode_header(token).context("invalid bearer token")?;
        anyhow::ensure!(
            header.alg == Algorithm::RS256,
            "only RS256 tokens are accepted"
        );
        let kid = header.kid.context("bearer token has no key id")?;
        let jwk = cache
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .context("unknown bearer key")?;
        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&[&cache.issuer]);
        let claims = decode::<Claims>(token, &key, &validation)?.claims;
        anyhow::ensure!(claims.aud == self.client_id, "wrong token audience");
        anyhow::ensure!(claims.tid == self.tenant_id, "wrong token tenant");
        anyhow::ensure!(claims.iss == cache.issuer, "wrong token issuer");
        let _ = (claims.exp, claims.nbf);
        Ok(claims.oid)
    }

    async fn refresh_if_needed(&self) -> anyhow::Result<()> {
        if self
            .cache
            .read()
            .await
            .as_ref()
            .is_some_and(|cache| cache.loaded_at.elapsed() < Duration::from_secs(3600))
        {
            return Ok(());
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()?;
        let discovery = client
            .get(&self.discovery_url)
            .send()
            .await?
            .error_for_status()?
            .json::<Discovery>()
            .await?;
        let keys = client
            .get(&discovery.jwks_uri)
            .send()
            .await?
            .error_for_status()?
            .json::<JwkSet>()
            .await?;
        *self.cache.write().await = Some(CachedKeys {
            issuer: discovery.issuer,
            keys: keys.keys,
            loaded_at: Instant::now(),
        });
        Ok(())
    }
}
