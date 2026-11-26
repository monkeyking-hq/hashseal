//! Signed instruction lockfile (`.hashseal.lock`) to detect deleted or unapproved files.

use crate::digest::Digest;
use crate::error::{Error, Result};
use crate::instruct::{
    digest_instruct_bytes, extract_fm_field, CheckOpts, InstructOptions, SCOPE_FIELD, SIGNER_FIELD,
};
use crate::result::CheckResult;
use crate::signing::{self, SigningOptions, VerifyOutcome};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const LOCK_SCHEMA_VERSION: u32 = 2;
pub const LOCKFILE_NAME: &str = ".hashseal.lock";
pub const LOCKFILE_SIG_NAME: &str = ".hashseal.lock.sig";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lockfile {
    pub lock_schema_version: u32,
    pub algorithm: String,
    pub tool_version: String,
    pub entries: Vec<LockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEntry {
    pub path: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer: Option<String>,
}

fn rel_posix(root: &Path, path: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| Error::InvalidFormat(format!("path not under root: {}", path.display())))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

pub fn build_lockfile(root: &Path, files: &[PathBuf], opts: &InstructOptions) -> Result<Lockfile> {
    let mut entries = Vec::with_capacity(files.len());
    for p in files {
        let rel = rel_posix(root, p)?;
        let text = fs::read_to_string(p)?;
        let digest = digest_instruct_bytes(&text, opts);
        let scope = extract_fm_field(&text, SCOPE_FIELD);
        let signer = extract_fm_field(&text, SIGNER_FIELD);
        entries.push(LockEntry {
            path: rel,
            digest: digest.qualified(),
            scope,
            signer,
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Lockfile {
        lock_schema_version: LOCK_SCHEMA_VERSION,
        algorithm: opts.algorithm.as_str().to_string(),
        tool_version: crate::VERSION.to_string(),
        entries,
    })
}

pub fn write_lockfile(path: &Path, lock: &Lockfile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut json = serde_json::to_string_pretty(lock)
        .map_err(|e| Error::InvalidFormat(format!("lockfile json: {e}")))?;
    json.push('\n');
    fs::write(path, json)?;
    Ok(())
}

pub fn read_lockfile(path: &Path) -> Result<Lockfile> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| Error::InvalidFormat(format!("lockfile parse: {e}")))
}

pub fn sign_lockfile(lock_path: &Path, opts: &SigningOptions) -> Result<PathBuf> {
    let bytes = fs::read(lock_path)?;
    let (armor, _) = signing::sign_bytes(&bytes, opts)?;
    let sig_path = PathBuf::from(format!("{}.sig", lock_path.display()));
    let mut out = armor;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(&sig_path, out)?;
    Ok(sig_path)
}

pub fn verify_lockfile(
    root: &Path,
    lock_path: &Path,
    current_files: &[PathBuf],
    check_opts: &CheckOpts,
) -> Result<Vec<CheckResult>> {
    let mut findings = Vec::new();
    let lock_bytes = fs::read(lock_path)?;
    let sig_path = PathBuf::from(format!("{}.sig", lock_path.display()));

    if (sig_path.is_file() || check_opts.require_signature) && !check_opts.digest_only {
        let lock_digest = Digest::compute(check_opts.instruct.algorithm, &lock_bytes);
        if !sig_path.is_file() {
            findings.push(CheckResult::missing_signature(
                Some(lock_path.to_path_buf()),
                &lock_digest,
            ));
        } else {
            let armor = fs::read_to_string(&sig_path)?;
            match signing::verify_bytes(&lock_bytes, &armor, &check_opts.signing) {
                VerifyOutcome::Good { .. } => {}
                VerifyOutcome::Bad(m) => findings.push(CheckResult::bad_signature(
                    Some(lock_path.to_path_buf()),
                    &lock_digest,
                    m,
                )),
                VerifyOutcome::UntrustedKey(m) => findings.push(CheckResult::untrusted_key(
                    Some(lock_path.to_path_buf()),
                    &lock_digest,
                    m,
                )),
                VerifyOutcome::Unavailable(m) => findings.push(CheckResult::unverified_signature(
                    Some(lock_path.to_path_buf()),
                    &lock_digest,
                    m,
                )),
            }
        }
    }

    let lock: Lockfile = serde_json::from_slice(&lock_bytes)
        .map_err(|e| Error::InvalidFormat(format!("lockfile parse: {e}")))?;

    let mut current_map: BTreeMap<String, PathBuf> = BTreeMap::new();
    for p in current_files {
        if let Ok(rel) = rel_posix(root, p) {
            current_map.insert(rel, p.clone());
        }
    }

    let mut locked_paths = BTreeSet::new();
    for entry in &lock.entries {
        locked_paths.insert(entry.path.clone());
        let full = root.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !full.is_file() {
            findings.push(CheckResult::locked_file_removed(
                Some(full),
                Some(entry.digest.clone()),
                "locked instruction file missing on disk",
            ));
            continue;
        }
        let text = fs::read_to_string(&full)?;
        let actual = digest_instruct_bytes(&text, &check_opts.instruct);
        if actual.qualified() != entry.digest {
            if let Ok(exp) = entry.digest.parse::<Digest>() {
                findings.push(CheckResult::mismatch(Some(full), &exp, &actual));
            }
        }
    }

    for (rel, path) in current_map {
        if !locked_paths.contains(&rel) {
            let actual = fs::read_to_string(&path)
                .ok()
                .map(|t| digest_instruct_bytes(&t, &check_opts.instruct).qualified());
            findings.push(CheckResult::unapproved_file(
                Some(path),
                actual,
                "instruction file not present in .hashseal.lock",
            ));
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruct::seal_instruct_path;
    use crate::signing::SignFormat;
    use crate::CheckStatus;
    use std::process::{Command, Stdio};
    use tempfile::tempdir;

    #[test]
    fn lockfile_detects_remove_add_and_tamper() {
        let d = tempdir().unwrap();
        let root = d.path();
        let a = root.join("AGENTS.md");
        fs::write(&a, "# Rules\n").unwrap();
        let iopts = InstructOptions::default();
        seal_instruct_path(&a, &iopts).unwrap();

        let lock_path = root.join(LOCKFILE_NAME);
        let lock = build_lockfile(root, std::slice::from_ref(&a), &iopts).unwrap();
        write_lockfile(&lock_path, &lock).unwrap();

        let copts = CheckOpts {
            instruct: iopts.clone(),
            digest_only: true,
            ..Default::default()
        };

        // Clean
        let ok = verify_lockfile(root, &lock_path, std::slice::from_ref(&a), &copts).unwrap();
        assert!(ok.is_empty(), "{ok:?}");

        // Added unapproved file
        let b = root.join("CLAUDE.md");
        fs::write(&b, "# Extra\n").unwrap();
        let added = verify_lockfile(root, &lock_path, &[a.clone(), b.clone()], &copts).unwrap();
        assert!(added
            .iter()
            .any(|f| f.status == CheckStatus::UnapprovedFile));
        fs::remove_file(&b).unwrap();

        // Removed locked file
        fs::remove_file(&a).unwrap();
        let removed = verify_lockfile(root, &lock_path, &[], &copts).unwrap();
        assert!(removed
            .iter()
            .any(|f| f.status == CheckStatus::LockedFileRemoved));
    }

    #[test]
    fn signed_lockfile_detects_tamper() {
        if Command::new("ssh-keygen")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_err()
        {
            return;
        }
        let d = tempdir().unwrap();
        let root = d.path();
        let key = root.join("id_ed25519");
        assert!(Command::new("ssh-keygen")
            .args([
                "-q",
                "-t",
                "ed25519",
                "-N",
                "",
                "-C",
                "lock@hashseal.test",
                "-f"
            ])
            .arg(&key)
            .status()
            .unwrap()
            .success());
        let pub_s = fs::read_to_string(root.join("id_ed25519.pub")).unwrap();
        let allowed = root.join("allowed_signers");
        fs::write(&allowed, format!("lock@hashseal.test {}\n", pub_s.trim())).unwrap();

        let a = root.join("AGENTS.md");
        fs::write(&a, "# Rules\n").unwrap();
        let iopts = InstructOptions::default();
        seal_instruct_path(&a, &iopts).unwrap();

        let lock_path = root.join(LOCKFILE_NAME);
        let lock = build_lockfile(root, std::slice::from_ref(&a), &iopts).unwrap();
        write_lockfile(&lock_path, &lock).unwrap();

        let sign_opts = SigningOptions {
            format: SignFormat::Ssh,
            signing_key: Some(key.display().to_string()),
            allowed_signers: Some(allowed),
            ..Default::default()
        };
        sign_lockfile(&lock_path, &sign_opts).unwrap();

        let copts = CheckOpts {
            instruct: iopts,
            require_signature: true,
            digest_only: false,
            expected_scope: None,
            signing: sign_opts,
        };
        let ok = verify_lockfile(root, &lock_path, std::slice::from_ref(&a), &copts).unwrap();
        assert!(ok.is_empty(), "{ok:?}");

        // Tamper lockfile whitespace -> BadSignature
        let mut raw = fs::read_to_string(&lock_path).unwrap();
        raw.push(' ');
        fs::write(&lock_path, raw).unwrap();
        let bad = verify_lockfile(root, &lock_path, &[a], &copts).unwrap();
        assert!(bad.iter().any(|f| f.status == CheckStatus::BadSignature));
    }
}
