use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

const COOKIE_NAME: &str = "cct_demo";

pub fn tenant_from_headers_or_new(headers: &HeaderMap, key: &[u8]) -> (String, bool) {
    verified_tenant(headers, key)
        .map(|id| (id, false))
        .unwrap_or_else(|| (Uuid::new_v4().to_string(), true))
}

pub fn verified_tenant(headers: &HeaderMap, key: &[u8]) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    let value = cookies.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == COOKIE_NAME).then_some(value)
    })?;
    verify(value, key)
}

pub fn set_cookie_value(tenant_id: &str, key: &[u8], secure: bool) -> String {
    let secure_attr = if secure { "; Secure" } else { "" };
    format!(
        "{COOKIE_NAME}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400{secure_attr}",
        sign(tenant_id, key)
    )
}

fn sign(tenant_id: &str, key: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(tenant_id.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{tenant_id}.{signature}")
}

fn verify(value: &str, key: &[u8]) -> Option<String> {
    let (tenant_id, signature) = value.rsplit_once('.')?;
    Uuid::parse_str(tenant_id).ok()?;
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let mut mac = Hmac::<Sha256>::new_from_slice(key).ok()?;
    mac.update(tenant_id.as_bytes());
    mac.verify_slice(&signature).ok()?;
    Some(tenant_id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_cookie_round_trips_and_rejects_changes() {
        let key = [7_u8; 32];
        let tenant = Uuid::new_v4().to_string();
        let value = sign(&tenant, &key);
        assert_eq!(verify(&value, &key), Some(tenant));
        assert_eq!(verify(&format!("{}x", value), &key), None);
    }
}
