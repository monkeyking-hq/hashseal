//! Legacy GPG helpers (delegating temp-dir & status-fd hardening to `signing`).
//!
//! Signed, Sealed, Delivered - I'm Yours.

use crate::error::Result;
use crate::signing::{self, SignFormat, SigningOptions, VerifyOutcome};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// How HashSeal invokes GPG (defaults from git).
#[derive(Debug, Clone)]
pub struct GpgConfig {
    pub program: PathBuf,
    /// Key id / fingerprint / email (git `user.signingKey`).
    pub signing_key: Option<String>,
    /// If non-empty, verify must use a key in this allowlist (fingerprints/ids).
    pub allowed_key_ids: Vec<String>,
}

impl Default for GpgConfig {
    fn default() -> Self {
        Self {
            program: PathBuf::from("gpg"),
            signing_key: None,
            allowed_key_ids: Vec::new(),
        }
    }
}

impl GpgConfig {
    /// Load program + signing key from git config when available.
    pub fn from_git() -> Self {
        let opts = SigningOptions::from_git_and_root(None);
        Self {
            program: opts.gpg_program,
            signing_key: opts.signing_key,
            allowed_key_ids: opts.allowed_key_ids,
        }
    }

    pub fn into_signing_options(self) -> SigningOptions {
        SigningOptions {
            format: SignFormat::Gpg,
            signing_key: self.signing_key,
            gpg_program: self.program,
            allowed_key_ids: self.allowed_key_ids,
            ..Default::default()
        }
    }
}

/// Canonical legacy payload signed by GPG (UTF-8).
pub fn signing_payload(digest_qualified: &str) -> String {
    format!("HASHSEAL-GPG1\ndigest={digest_qualified}\n")
}

/// Detached ASCII-armored signature over `signing_payload(digest)`.
pub fn sign_digest(digest_qualified: &str, cfg: &GpgConfig) -> Result<String> {
    let payload = signing_payload(digest_qualified);
    let opts = cfg.clone().into_signing_options();
    let (armor, _) = signing::sign_bytes(payload.as_bytes(), &opts)?;
    Ok(armor)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpgVerifyStatus {
    Good,
    Bad,
    /// gpg missing or could not run
    Unavailable(String),
}

/// Verify detached armor over the HashSeal signing payload for `digest_qualified`.
pub fn verify_digest(digest_qualified: &str, armor: &str, cfg: &GpgConfig) -> GpgVerifyStatus {
    let payload = signing_payload(digest_qualified);
    let opts = cfg.clone().into_signing_options();
    match signing::verify_bytes(payload.as_bytes(), armor, &opts) {
        VerifyOutcome::Good { .. } => GpgVerifyStatus::Good,
        VerifyOutcome::Bad(_) | VerifyOutcome::UntrustedKey(_) => GpgVerifyStatus::Bad,
        VerifyOutcome::Unavailable(m) => GpgVerifyStatus::Unavailable(m),
    }
}

/// True if `gpg` (or configured program) appears runnable.
pub fn gpg_available(cfg: &GpgConfig) -> bool {
    Command::new(&cfg.program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_format_stable() {
        let p = signing_payload("blake3:abc");
        assert_eq!(p, "HASHSEAL-GPG1\ndigest=blake3:abc\n");
    }
}
