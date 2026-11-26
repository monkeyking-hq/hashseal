//! Unified signing & verification (`signing-v2`): OpenSSH `SSHSIG` + hardened GPG.
//!
//! Zero external crates (std only) so `hashseal-check` stays blake3 + std.
//! Signed, Sealed, Delivered - I'm Yours.

use crate::error::{Error, Result};
use crate::timeutil::unix_ms;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQ: AtomicU64 = AtomicU64::new(1);

/// Signing backend format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignFormat {
    #[default]
    Auto,
    Ssh,
    Gpg,
}

impl SignFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            SignFormat::Auto => "auto",
            SignFormat::Ssh => "ssh",
            SignFormat::Gpg => "gpg",
        }
    }
}

impl FromStr for SignFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(SignFormat::Auto),
            "ssh" => Ok(SignFormat::Ssh),
            "gpg" | "openpgp" => Ok(SignFormat::Gpg),
            other => Err(Error::InvalidFormat(format!(
                "invalid sign format '{other}' (expected auto, ssh, or gpg)"
            ))),
        }
    }
}

/// Unified signing & verification options (git-aligned + repo trust roots).
#[derive(Debug, Clone)]
pub struct SigningOptions {
    pub format: SignFormat,
    /// SSH key path or GPG key id/fingerprint/email (git `user.signingKey`).
    pub signing_key: Option<String>,
    /// GPG binary (git `gpg.program`, default `gpg`).
    pub gpg_program: PathBuf,
    /// OpenSSH keygen binary (default `ssh-keygen`).
    pub ssh_keygen_program: PathBuf,
    /// Path to OpenSSH `allowed_signers` file.
    pub allowed_signers: Option<PathBuf>,
    /// Directory containing pinned GPG public keys (`*.asc` / `*.pub`).
    pub keys_dir: Option<PathBuf>,
    /// Allowlisted GPG key fingerprints / key IDs (case-insensitive suffix or exact match).
    pub allowed_key_ids: Vec<String>,
    /// SSHSIG namespace (default `hashseal`).
    pub namespace: String,
}

impl Default for SigningOptions {
    fn default() -> Self {
        Self {
            format: SignFormat::Auto,
            signing_key: None,
            gpg_program: PathBuf::from("gpg"),
            ssh_keygen_program: PathBuf::from("ssh-keygen"),
            allowed_signers: None,
            keys_dir: None,
            allowed_key_ids: Vec::new(),
            namespace: "hashseal".into(),
        }
    }
}

impl SigningOptions {
    /// Load defaults from git config (`gpg.format`, `user.signingKey`, `gpg.program`,
    /// `gpg.ssh.allowedSignersFile`) and auto-discover `<root>/.hashseal/allowed_signers`
    /// and `<root>/.hashseal/keys`.
    pub fn from_git_and_root(root: Option<&Path>) -> Self {
        let mut opts = Self::default();
        if let Some(fmt) = git_config(root, "gpg.format") {
            if let Ok(parsed) = SignFormat::from_str(&fmt) {
                opts.format = parsed;
            }
        }
        if let Some(prog) = git_config(root, "gpg.program") {
            opts.gpg_program = PathBuf::from(prog);
        }
        if let Some(key) = git_config(root, "user.signingKey") {
            opts.signing_key = Some(key);
        }
        if let Some(r) = root {
            let repo_signers = r.join(".hashseal").join("allowed_signers");
            if repo_signers.is_file() {
                opts.allowed_signers = Some(repo_signers);
            } else if let Some(s) = git_config(root, "gpg.ssh.allowedSignersFile") {
                let pb = PathBuf::from(s);
                if pb.is_file() {
                    opts.allowed_signers = Some(pb);
                }
            }
            let repo_keys = r.join(".hashseal").join("keys");
            if repo_keys.is_dir() {
                opts.keys_dir = Some(repo_keys);
            }
        } else if let Some(s) = git_config(None, "gpg.ssh.allowedSignersFile") {
            let pb = PathBuf::from(s);
            if pb.is_file() {
                opts.allowed_signers = Some(pb);
            }
        }
        opts
    }

    /// Resolve `Auto` to concrete `Ssh` or `Gpg` based on key shape or default `Gpg`.
    pub fn resolve_sign_format(&self) -> SignFormat {
        match self.format {
            SignFormat::Ssh => SignFormat::Ssh,
            SignFormat::Gpg => SignFormat::Gpg,
            SignFormat::Auto => {
                if let Some(k) = &self.signing_key {
                    if k.starts_with("ssh-") || k.ends_with(".pub") || Path::new(k).is_file() {
                        return SignFormat::Ssh;
                    }
                }
                SignFormat::Gpg
            }
        }
    }
}

fn git_config(root: Option<&Path>, key: &str) -> Option<String> {
    let mut cmd = Command::new("git");
    if let Some(r) = root {
        cmd.arg("-C").arg(r);
    }
    let out = cmd
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

/// Canonical `signing-v2` payload bound to digest and context scope.
pub fn signing_payload_v2(digest_qualified: &str, scope: &str) -> String {
    format!("HASHSEAL-SIG-V2\ndigest={digest_qualified}\nscope={scope}\n")
}

/// Verification outcome for detached signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    Good { signer: Option<String> },
    Bad(String),
    UntrustedKey(String),
    Unavailable(String),
}

/// RAII unique `0o700` temporary directory for cryptographic operations.
pub(crate) struct TempDirGuard {
    pub(crate) path: PathBuf,
}

impl TempDirGuard {
    pub(crate) fn new(prefix: &str) -> std::io::Result<Self> {
        let base = std::env::temp_dir();
        for _ in 0..32 {
            let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
            let dir = base.join(format!(
                "{prefix}-{}-{}-{seq}",
                std::process::id(),
                unix_ms()
            ));
            match fs::create_dir(&dir) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
                    }
                    return Ok(Self { path: dir });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "failed to allocate unique temp directory",
        ))
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Sign arbitrary bytes using the configured or auto-detected backend.
/// Returns `(armored_signature, concrete_format)`.
pub fn sign_bytes(payload: &[u8], opts: &SigningOptions) -> Result<(String, SignFormat)> {
    let fmt = opts.resolve_sign_format();
    let armor = match fmt {
        SignFormat::Ssh => sign_bytes_ssh(payload, opts)?,
        SignFormat::Gpg | SignFormat::Auto => sign_bytes_gpg(payload, opts)?,
    };
    Ok((armor, fmt))
}

/// Verify detached signature over `payload`, auto-detecting SSH vs GPG from armor header.
pub fn verify_bytes(payload: &[u8], armor: &str, opts: &SigningOptions) -> VerifyOutcome {
    if armor.contains("BEGIN SSH SIGNATURE") {
        verify_bytes_ssh(payload, armor, opts)
    } else if armor.contains("BEGIN PGP SIGNATURE") {
        verify_bytes_gpg(payload, armor, opts)
    } else {
        VerifyOutcome::Bad("unrecognized signature armor".into())
    }
}

/// Sign an instruct digest bound to `scope`.
pub fn sign_instruct_digest(
    digest_qualified: &str,
    scope: &str,
    opts: &SigningOptions,
) -> Result<(String, SignFormat)> {
    let payload = signing_payload_v2(digest_qualified, scope);
    sign_bytes(payload.as_bytes(), opts)
}

/// Verify an instruct digest signature bound to `scope`.
pub fn verify_instruct_digest(
    digest_qualified: &str,
    scope: &str,
    armor: &str,
    opts: &SigningOptions,
) -> VerifyOutcome {
    let payload = signing_payload_v2(digest_qualified, scope);
    verify_bytes(payload.as_bytes(), armor, opts)
}

fn sign_bytes_ssh(payload: &[u8], opts: &SigningOptions) -> Result<String> {
    let key = opts.signing_key.as_deref().ok_or_else(|| {
        Error::InvalidFormat(
            "SSH signing requires a key path (--signing-key or git user.signingKey)".into(),
        )
    })?;
    let tmp = TempDirGuard::new("hashseal-ssh")?;
    let data_path = tmp.path.join("payload.bin");
    fs::write(&data_path, payload)?;

    let out = Command::new(&opts.ssh_keygen_program)
        .args(["-Y", "sign", "-f", key, "-n", &opts.namespace])
        .arg(&data_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            Error::InvalidFormat(format!(
                "failed to run {}: {e}",
                opts.ssh_keygen_program.display()
            ))
        })?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(Error::InvalidFormat(format!(
            "ssh-keygen -Y sign failed (exit {:?}): {}",
            out.status.code(),
            err.trim()
        )));
    }

    let sig_path = tmp.path.join("payload.bin.sig");
    let armor = fs::read_to_string(&sig_path)
        .map_err(|e| Error::InvalidFormat(format!("read ssh signature: {e}")))?;
    let trimmed = armor.trim().to_string();
    if !trimmed.contains("BEGIN SSH SIGNATURE") {
        return Err(Error::InvalidFormat(
            "ssh-keygen did not produce an SSH SIGNATURE block".into(),
        ));
    }
    Ok(trimmed)
}

fn verify_bytes_ssh(payload: &[u8], armor: &str, opts: &SigningOptions) -> VerifyOutcome {
    let Some(allowed_signers) = &opts.allowed_signers else {
        return VerifyOutcome::Unavailable(
            "missing allowed_signers file (configure .hashseal/allowed_signers or --allowed-signers)"
                .into(),
        );
    };
    if !allowed_signers.is_file() {
        return VerifyOutcome::Unavailable(format!(
            "allowed_signers not found at {}",
            allowed_signers.display()
        ));
    }

    let tmp = match TempDirGuard::new("hashseal-ssh-v") {
        Ok(t) => t,
        Err(e) => return VerifyOutcome::Unavailable(e.to_string()),
    };
    let sig_path = tmp.path.join("payload.sig");
    let mut a = armor.trim().to_string();
    a.push('\n');
    if let Err(e) = fs::write(&sig_path, a.as_bytes()) {
        return VerifyOutcome::Unavailable(e.to_string());
    }

    let find = match Command::new(&opts.ssh_keygen_program)
        .args(["-Y", "find-principals", "-f"])
        .arg(allowed_signers)
        .arg("-s")
        .arg(&sig_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return VerifyOutcome::Unavailable(format!(
                "failed to run {}: {e}",
                opts.ssh_keygen_program.display()
            ))
        }
    };

    if !find.status.success() {
        return VerifyOutcome::UntrustedKey("signer key not in allowed_signers".into());
    }
    let principals: Vec<String> = String::from_utf8_lossy(&find.stdout)
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if principals.is_empty() {
        return VerifyOutcome::UntrustedKey("no matching principal in allowed_signers".into());
    }

    for principal in principals {
        let mut child = match Command::new(&opts.ssh_keygen_program)
            .args(["-Y", "verify", "-f"])
            .arg(allowed_signers)
            .args(["-I", &principal, "-n", &opts.namespace, "-s"])
            .arg(&sig_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return VerifyOutcome::Unavailable(format!(
                    "failed to run {}: {e}",
                    opts.ssh_keygen_program.display()
                ))
            }
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(payload);
        }
        if let Ok(out) = child.wait_with_output() {
            if out.status.success() {
                return VerifyOutcome::Good {
                    signer: Some(principal),
                };
            }
        }
    }

    VerifyOutcome::Bad("ssh-keygen -Y verify failed".into())
}

fn sign_bytes_gpg(payload: &[u8], opts: &SigningOptions) -> Result<String> {
    let mut cmd = Command::new(&opts.gpg_program);
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
    if let Some(key) = &opts.signing_key {
        cmd.args(["--local-user", key]);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        Error::InvalidFormat(format!(
            "failed to run {}: {e} (is GPG installed? same setup as git commit -S)",
            opts.gpg_program.display()
        ))
    })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::InvalidFormat("gpg stdin missing".into()))?;
        stdin.write_all(payload)?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| Error::InvalidFormat(format!("gpg wait failed: {e}")))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(Error::InvalidFormat(format!(
            "gpg sign failed (exit {:?}): {}",
            out.status.code(),
            err.trim()
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

fn collect_key_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("asc" | "pub" | "gpg")
                )
        })
        .collect();
    files.sort();
    files
}

fn key_id_matches(fpr: &str, allowed: &str) -> bool {
    let f = fpr.trim().to_ascii_uppercase();
    let a = allowed
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_ascii_uppercase();
    !a.is_empty() && (f == a || f.ends_with(&a))
}

fn verify_bytes_gpg(payload: &[u8], armor: &str, opts: &SigningOptions) -> VerifyOutcome {
    let tmp = match TempDirGuard::new("hashseal-gpg-v") {
        Ok(t) => t,
        Err(e) => return VerifyOutcome::Unavailable(e.to_string()),
    };

    let data_path = tmp.path.join("payload.bin");
    let sig_path = tmp.path.join("payload.bin.asc");
    if let Err(e) = fs::write(&data_path, payload) {
        return VerifyOutcome::Unavailable(e.to_string());
    }
    let mut a = armor.trim().to_string();
    a.push('\n');
    if let Err(e) = fs::write(&sig_path, a.as_bytes()) {
        return VerifyOutcome::Unavailable(e.to_string());
    }

    // If repo-pinned keys exist in keys_dir, isolate verification to those keys only.
    let mut isolated_home: Option<PathBuf> = None;
    if let Some(kdir) = &opts.keys_dir {
        let key_files = collect_key_files(kdir);
        if !key_files.is_empty() {
            let home = tmp.path.join("gnupghome");
            if let Err(e) = fs::create_dir(&home) {
                return VerifyOutcome::Unavailable(e.to_string());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&home, fs::Permissions::from_mode(0o700));
            }
            for kf in &key_files {
                let imp = Command::new(&opts.gpg_program)
                    .arg("--homedir")
                    .arg(&home)
                    .args(["--batch", "--quiet", "--import"])
                    .arg(kf)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                match imp {
                    Ok(s) if s.success() => {}
                    Ok(_) => {}
                    Err(e) => {
                        return VerifyOutcome::Unavailable(format!(
                            "failed to run {}: {e}",
                            opts.gpg_program.display()
                        ))
                    }
                }
            }
            isolated_home = Some(home);
        }
    }

    let mut cmd = Command::new(&opts.gpg_program);
    if let Some(home) = &isolated_home {
        cmd.arg("--homedir").arg(home);
    }
    cmd.args(["--batch", "--status-fd", "1", "--verify"])
        .arg(&sig_path)
        .arg(&data_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return VerifyOutcome::Unavailable(format!(
                "failed to run {}: {e}",
                opts.gpg_program.display()
            ))
        }
    };

    let status_text = String::from_utf8_lossy(&out.stdout);
    let mut validsig_fpr: Option<String> = None;
    for line in status_text.lines() {
        let Some(rest) = line.strip_prefix("[GNUPG:] ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(code) = parts.next() else {
            continue;
        };
        match code {
            "BADSIG" => return VerifyOutcome::Bad("gpg BADSIG".into()),
            "EXPSIG" => return VerifyOutcome::Bad("gpg EXPSIG (expired signature)".into()),
            "EXPKEYSIG" => return VerifyOutcome::Bad("gpg EXPKEYSIG (expired signing key)".into()),
            "REVKEYSIG" => return VerifyOutcome::Bad("gpg REVKEYSIG (revoked signing key)".into()),
            "NO_PUBKEY" => {
                return VerifyOutcome::UntrustedKey("gpg NO_PUBKEY (untrusted/unknown key)".into())
            }
            "ERRSIG" => return VerifyOutcome::Bad("gpg ERRSIG".into()),
            "VALIDSIG" => {
                if let Some(fpr) = parts.next() {
                    validsig_fpr = Some(fpr.to_ascii_uppercase());
                }
            }
            _ => {}
        }
    }

    if !out.status.success() {
        return VerifyOutcome::Bad("gpg --verify failed".into());
    }

    let Some(fpr) = validsig_fpr else {
        return VerifyOutcome::Bad("gpg --verify did not emit VALIDSIG".into());
    };

    if !opts.allowed_key_ids.is_empty()
        && !opts
            .allowed_key_ids
            .iter()
            .any(|allowed| key_id_matches(&fpr, allowed))
    {
        return VerifyOutcome::UntrustedKey(format!("gpg key {fpr} is not in allowed_key_ids"));
    }

    VerifyOutcome::Good { signer: Some(fpr) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_exists(bin: &str) -> bool {
        Command::new(bin)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }

    #[test]
    fn payload_v2_format_stable() {
        let p = signing_payload_v2("blake3:abc", "AGENTS.md");
        assert_eq!(p, "HASHSEAL-SIG-V2\ndigest=blake3:abc\nscope=AGENTS.md\n");
    }

    #[test]
    fn ssh_sign_verify_roundtrip_and_untrusted_key() {
        if !tool_exists("ssh-keygen") {
            return;
        }
        let dir = TempDirGuard::new("hashseal-test-ssh").unwrap();
        let key1 = dir.path.join("id_ed25519_1");
        let key2 = dir.path.join("id_ed25519_2");
        for (k, c) in [
            (&key1, "signer1@hashseal.test"),
            (&key2, "signer2@hashseal.test"),
        ] {
            let st = Command::new("ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-C", c, "-f"])
                .arg(k)
                .status()
                .unwrap();
            assert!(st.success());
        }
        let pub1 = fs::read_to_string(dir.path.join("id_ed25519_1.pub")).unwrap();
        let allowed_path = dir.path.join("allowed_signers");
        fs::write(
            &allowed_path,
            format!("signer1@hashseal.test {}\n", pub1.trim()),
        )
        .unwrap();

        let sign_opts = SigningOptions {
            format: SignFormat::Ssh,
            signing_key: Some(key1.display().to_string()),
            allowed_signers: Some(allowed_path.clone()),
            ..Default::default()
        };

        let (sig, fmt) = sign_instruct_digest("blake3:deadbeef", "AGENTS.md", &sign_opts).unwrap();
        assert_eq!(fmt, SignFormat::Ssh);
        assert!(sig.contains("BEGIN SSH SIGNATURE"));

        // Valid verify
        let ok = verify_instruct_digest("blake3:deadbeef", "AGENTS.md", &sig, &sign_opts);
        assert_eq!(
            ok,
            VerifyOutcome::Good {
                signer: Some("signer1@hashseal.test".into())
            }
        );

        // Wrong scope -> Bad
        let bad_scope = verify_instruct_digest("blake3:deadbeef", "OTHER.md", &sig, &sign_opts);
        assert!(matches!(bad_scope, VerifyOutcome::Bad(_)), "{bad_scope:?}");

        // Untrusted key2 -> UntrustedKey
        let sign_opts2 = SigningOptions {
            format: SignFormat::Ssh,
            signing_key: Some(key2.display().to_string()),
            allowed_signers: Some(allowed_path),
            ..Default::default()
        };
        let (sig2, _) = sign_instruct_digest("blake3:deadbeef", "AGENTS.md", &sign_opts2).unwrap();
        let untrusted = verify_instruct_digest("blake3:deadbeef", "AGENTS.md", &sig2, &sign_opts);
        assert!(
            matches!(untrusted, VerifyOutcome::UntrustedKey(_)),
            "{untrusted:?}"
        );

        // Missing allowed_signers -> Unavailable (fail closed)
        let no_trust = SigningOptions::default();
        let unavail = verify_instruct_digest("blake3:deadbeef", "AGENTS.md", &sig, &no_trust);
        assert!(
            matches!(unavail, VerifyOutcome::Unavailable(_)),
            "{unavail:?}"
        );
    }

    #[test]
    fn gpg_isolated_keys_and_allowlist() {
        if !tool_exists("gpg") {
            return;
        }
        let dir = TempDirGuard::new("hashseal-test-gpg").unwrap();
        let gpghome = dir.path.join("gpghome");
        fs::create_dir(&gpghome).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&gpghome, fs::Permissions::from_mode(0o700));
        }

        let gen = Command::new("gpg")
            .arg("--homedir")
            .arg(&gpghome)
            .args([
                "--batch",
                "--passphrase",
                "",
                "--quick-generate-key",
                "Test Bot <bot@hashseal.invalid>",
                "ed25519",
                "sign",
                "1d",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !matches!(gen, Ok(s) if s.success()) {
            return;
        }

        // Export public key to .hashseal/keys/bot.pub.asc
        let keys_dir = dir.path.join("keys");
        fs::create_dir(&keys_dir).unwrap();
        let pub_out = Command::new("gpg")
            .arg("--homedir")
            .arg(&gpghome)
            .args(["--batch", "--armor", "--export", "bot@hashseal.invalid"])
            .output()
            .unwrap();
        assert!(pub_out.status.success());
        fs::write(keys_dir.join("bot.pub.asc"), &pub_out.stdout).unwrap();

        // Sign using the isolated gpghome via GNUPGHOME env? Use a wrapper or check payload via gpg directly
        let payload = signing_payload_v2("blake3:cafe", "AGENTS.md");
        let mut child = Command::new("gpg")
            .arg("--homedir")
            .arg(&gpghome)
            .args([
                "--batch",
                "--yes",
                "--pinentry-mode",
                "loopback",
                "--armor",
                "--detach-sign",
                "-o",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.as_bytes())
            .unwrap();
        let sig_out = child.wait_with_output().unwrap();
        assert!(sig_out.status.success());
        let armor = String::from_utf8_lossy(&sig_out.stdout).trim().to_string();

        let verify_opts = SigningOptions {
            format: SignFormat::Gpg,
            keys_dir: Some(keys_dir.clone()),
            ..Default::default()
        };
        let res = verify_instruct_digest("blake3:cafe", "AGENTS.md", &armor, &verify_opts);
        let fpr = match res {
            VerifyOutcome::Good { signer: Some(f) } => f,
            other => panic!("expected Good, got {other:?}"),
        };

        // Allowed key IDs match
        let allow_ok = SigningOptions {
            keys_dir: Some(keys_dir.clone()),
            allowed_key_ids: vec![fpr.clone()],
            ..Default::default()
        };
        assert!(matches!(
            verify_instruct_digest("blake3:cafe", "AGENTS.md", &armor, &allow_ok),
            VerifyOutcome::Good { .. }
        ));

        // Disallowed key ID -> UntrustedKey
        let allow_bad = SigningOptions {
            keys_dir: Some(keys_dir),
            allowed_key_ids: vec!["DEADBEEFDEADBEEF".into()],
            ..Default::default()
        };
        assert!(matches!(
            verify_instruct_digest("blake3:cafe", "AGENTS.md", &armor, &allow_bad),
            VerifyOutcome::UntrustedKey(_)
        ));
    }
}
