use std::{fs, path::Path};

use anyhow::Context;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};

/// Azure Files intentionally does not implement POSIX chmod. Container Apps
/// protects the mounted share, while local files still receive mode 0600.
pub fn allow_azure_files_permission_denied(result: std::io::Result<()>) -> anyhow::Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[derive(Clone)]
pub struct ContactCipher(XChaCha20Poly1305);

impl ContactCipher {
    pub fn from_key(key: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(key.len() == 32, "contact encryption key must be 32 bytes");
        Ok(Self(
            XChaCha20Poly1305::new_from_slice(key).expect("validated key length"),
        ))
    }

    pub fn encrypt(&self, value: &str) -> anyhow::Result<String> {
        let mut nonce = [0_u8; 24];
        OsRng.fill_bytes(&mut nonce);
        let ciphertext = self
            .0
            .encrypt(XNonce::from_slice(&nonce), value.as_bytes())
            .map_err(|_| anyhow::anyhow!("contact encryption failed"))?;
        Ok(format!(
            "v1.{}.{}",
            URL_SAFE_NO_PAD.encode(nonce),
            URL_SAFE_NO_PAD.encode(ciphertext)
        ))
    }

    pub fn decrypt(&self, value: &str) -> anyhow::Result<String> {
        let mut parts = value.split('.');
        anyhow::ensure!(parts.next() == Some("v1"), "contact value is not encrypted");
        let nonce = URL_SAFE_NO_PAD.decode(parts.next().context("missing nonce")?)?;
        let ciphertext = URL_SAFE_NO_PAD.decode(parts.next().context("missing ciphertext")?)?;
        anyhow::ensure!(nonce.len() == 24, "invalid contact nonce");
        let plaintext = self
            .0
            .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("contact decryption failed"))?;
        String::from_utf8(plaintext).context("contact plaintext is not UTF-8")
    }
}

pub fn load_or_create_key(data_dir: &Path) -> anyhow::Result<(ContactCipher, &'static str)> {
    let path = data_dir.join("contact-data.key");
    let (key, source) = if let Ok(encoded) = std::env::var("CONTACT_ENCRYPTION_KEY") {
        (
            URL_SAFE_NO_PAD
                .decode(encoded)
                .context("CONTACT_ENCRYPTION_KEY must be base64url")?,
            "supplied",
        )
    } else if let Ok(value) = fs::read(&path) {
        (value, "persisted-generated")
    } else {
        let mut value = vec![0_u8; 32];
        OsRng.fill_bytes(&mut value);
        fs::write(&path, &value).with_context(|| format!("write {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Azure Files rejects POSIX chmod; its mounted share access policy
            // protects this generated key. Keep 0600 on local filesystems.
            allow_azure_files_permission_denied(fs::set_permissions(
                &path,
                fs::Permissions::from_mode(0o600),
            ))?;
        }
        (value, "generated-and-persisted")
    };
    Ok((ContactCipher::from_key(&key)?, source))
}

#[cfg(test)]
mod tests {
    use super::allow_azure_files_permission_denied;

    #[test]
    fn azure_files_permission_denied_does_not_block_persisted_key_creation() {
        // Regression for the first Azure Files revision: mounted shares reject
        // chmod even though the file was created successfully.
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert!(allow_azure_files_permission_denied(Err(denied)).is_ok());
    }
}
