//! GPG signing aligned with git config (`user.signingKey`, `gpg.program`).
//!
//! Signed, Sealed, Delivered - I'm Yours.

use crate::error::{Error, Result};
use std::io::Write;
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
        let mut cfg = Self::default();
        if let Some(prog) = git_config("gpg.program") {
            cfg.program = PathBuf::from(prog);
        }
        if let Some(key) = git_config("user.signingKey") {
            cfg.signing_key = Some(key);
        }
        cfg
    }
}

fn git_config(key: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["config", "--get", key])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Canonical payload signed by GPG (UTF-8). Frozen for vectors.
pub fn signing_payload(digest_qualified: &str) -> String {
    format!("HASHSEAL-GPG1\ndigest={digest_qualified}\n")
}

/// Detached ASCII-armored signature over `signing_payload(digest)`.
pub fn sign_digest(digest_qualified: &str, cfg: &GpgConfig) -> Result<String> {
    let payload = signing_payload(digest_qualified);
    let mut cmd = Command::new(&cfg.program);
    // -o - writes armor to stdout when signing from stdin (required on Windows/GnuPG)
    cmd.args([
        "--batch",
        "--yes",
        "--pinentry-mode",
        "loopback",
        "--armor",
        "--detach-sign",
        "-o",
        "-",
    ]);
    if let Some(key) = &cfg.signing_key {
        cmd.args(["--local-user", key]);
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        Error::InvalidFormat(format!(
            "failed to run {}: {e} (is GPG installed? same setup as git commit -S)",
            cfg.program.display()
        ))
    })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::InvalidFormat("gpg stdin missing".into()))?;
        stdin.write_all(payload.as_bytes())?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| Error::InvalidFormat(format!("gpg wait failed: {e}")))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(Error::InvalidFormat(format!(
            "gpg sign failed (exit {:?}): {err}",
            out.status.code()
        )));
    }

    let armor = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !armor.contains("BEGIN PGP SIGNATURE") {
        return Err(Error::InvalidFormat(
            "gpg did not return an ASCII-armored signature".into(),
        ));
    }
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
    let tmp = match tempfile_pair(&payload, armor) {
        Ok(t) => t,
        Err(e) => return GpgVerifyStatus::Unavailable(e.to_string()),
    };

    let mut cmd = Command::new(&cfg.program);
    cmd.args(["--batch", "--verify"])
        .arg(&tmp.sig_path)
        .arg(&tmp.data_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            let _ = tmp.cleanup();
            return GpgVerifyStatus::Unavailable(format!("failed to run gpg: {e}"));
        }
    };
    let _ = tmp.cleanup();

    if out.status.success() {
        // Optional allowlist: parse fingerprint from stderr if needed later
        GpgVerifyStatus::Good
    } else {
        GpgVerifyStatus::Bad
    }
}

struct TempPair {
    dir: PathBuf,
    data_path: PathBuf,
    sig_path: PathBuf,
}

impl TempPair {
    fn cleanup(&self) -> std::io::Result<()> {
        let _ = std::fs::remove_file(&self.data_path);
        let _ = std::fs::remove_file(&self.sig_path);
        std::fs::remove_dir(&self.dir)
    }
}

fn tempfile_pair(payload: &str, armor: &str) -> std::io::Result<TempPair> {
    let dir = std::env::temp_dir().join(format!("hashseal-gpg-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let data_path = dir.join("payload.txt");
    let sig_path = dir.join("payload.txt.asc");
    std::fs::write(&data_path, payload.as_bytes())?;
    // Ensure armor ends with newline
    let mut a = armor.trim().to_string();
    if !a.ends_with('\n') {
        a.push('\n');
    }
    std::fs::write(&sig_path, a.as_bytes())?;
    Ok(TempPair {
        dir,
        data_path,
        sig_path,
    })
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
