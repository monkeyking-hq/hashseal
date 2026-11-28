//! Instruct-file seals (`instruct-v2`: Markdown front matter + lossless raw-FM hash).
//!
//! Supports `signing-v2` (OpenSSH `SSHSIG` and hardened GPG) with context scope binding.
//! Signed, Sealed, Delivered - I'm Yours.

use crate::digest::{Algorithm, Digest};
use crate::error::{Error, Result};
use crate::result::CheckResult;
use crate::signing::{self, SignFormat, SigningOptions, VerifyOutcome};
use crate::unicode::{format_unicode_findings, scan_suspicious_unicode, UnicodePolicy};
use std::fs;
use std::path::{Path, PathBuf};

/// YAML keys excluded from the content hash (seal + signature domain).
pub const SEAL_FIELD: &str = "hashseal";
pub const SIG_FIELD: &str = "hashseal_sig";
pub const SIG_FORMAT_FIELD: &str = "hashseal_sig_format";
pub const SIGNER_FIELD: &str = "hashseal_signer";
pub const SCOPE_FIELD: &str = "hashseal_scope";
pub const KEY_ID_FIELD: &str = "hashseal_key_id";

/// Domain separation prefixes for `instruct-v2` canonical hashing.
pub const INSTRUCT_V2_DOMAIN: &str = "HASHSEAL-INSTRUCT-V2";
pub const INSTRUCT_V2_BODY_DOMAIN: &str = "HASHSEAL-INSTRUCT-V2-BODY";

fn is_reserved_fm_key(key: &str, seal_field: &str) -> bool {
    key == seal_field
        || matches!(
            key,
            SEAL_FIELD | SIG_FIELD | SIG_FORMAT_FIELD | SIGNER_FIELD | SCOPE_FIELD | KEY_ID_FIELD
        )
}

/// How to build the hashed payload for an instruct file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanonicalMode {
    /// Raw front-matter lines (minus reserved seal/sig fields) + body.
    #[default]
    Full,
    /// Body only (after front matter).
    BodyOnly,
}

/// Options for instruct seal/check digests.
#[derive(Debug, Clone)]
pub struct InstructOptions {
    pub field: String,
    pub algorithm: Algorithm,
    pub canonical: CanonicalMode,
    pub auto_frontmatter: bool,
    pub unicode_policy: UnicodePolicy,
}

impl Default for InstructOptions {
    fn default() -> Self {
        Self {
            field: SEAL_FIELD.into(),
            algorithm: Algorithm::Blake3,
            canonical: CanonicalMode::Full,
            auto_frontmatter: true,
            unicode_policy: UnicodePolicy::Fail,
        }
    }
}

/// Seal options including optional `signing-v2` cryptographic signature.
#[derive(Debug, Clone, Default)]
pub struct SealOpts {
    pub instruct: InstructOptions,
    /// When set, write `hashseal_sig*` with detached signature over `HASHSEAL-SIG-V2`.
    pub sign: bool,
    pub signing: SigningOptions,
    /// Context scope bound into signature (`<rel-path>` or `"*"` for `--portable`).
    pub scope: Option<String>,
}

/// Check options including `signing-v2` verification policy.
#[derive(Debug, Clone, Default)]
pub struct CheckOpts {
    pub instruct: InstructOptions,
    /// If true, missing `hashseal_sig` fails.
    pub require_signature: bool,
    /// If true, skip signature verification even when `hashseal_sig` is present.
    pub digest_only: bool,
    /// Expected repo-relative POSIX path for scope-bound signature verification.
    pub expected_scope: Option<String>,
    pub signing: SigningOptions,
}

#[derive(Debug, Clone)]
struct ParsedDoc {
    /// Original front-matter lines excluding delimiters (may be empty).
    fm_lines: Vec<String>,
    /// Whether a `---` front matter block was present.
    had_front_matter: bool,
    /// Body after front matter (or entire file if no FM).
    body_raw: String,
}

fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

/// Normalize to LF for hashing only.
pub fn normalize_lf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn parse_document(text: &str) -> ParsedDoc {
    let text = strip_bom(text);
    let bytes = text.as_bytes();
    if bytes.starts_with(b"---\n") || bytes.starts_with(b"---\r\n") {
        let after_open = if bytes.starts_with(b"---\r\n") {
            &text[5..]
        } else {
            &text[4..]
        };
        let mut search = after_open;
        let mut offset = 0usize;
        while let Some(idx) = search.find("\n---") {
            let after = &search[idx + 1..];
            let rest = &after[3..];
            let closed = rest.is_empty()
                || rest.starts_with('\n')
                || rest.starts_with("\r\n")
                || rest.starts_with('\r');
            if closed {
                let fm_block = &after_open[..offset + idx];
                let body_start_in_after = offset + idx + 1 + 3;
                let mut body = &after_open[body_start_in_after..];
                if let Some(b) = body.strip_prefix("\r\n") {
                    body = b;
                } else if let Some(b) = body.strip_prefix('\n') {
                    body = b;
                } else if let Some(b) = body.strip_prefix('\r') {
                    body = b;
                }
                let fm_lines: Vec<String> = normalize_lf(fm_block)
                    .lines()
                    .map(|l| l.to_string())
                    .collect();
                return ParsedDoc {
                    fm_lines,
                    had_front_matter: true,
                    body_raw: body.to_string(),
                };
            }
            offset += idx + 1;
            search = &search[idx + 1..];
        }
    }
    ParsedDoc {
        fm_lines: Vec::new(),
        had_front_matter: false,
        body_raw: text.to_string(),
    }
}

#[cfg(feature = "json")]
pub(crate) fn extract_fm_field(text: &str, field: &str) -> Option<String> {
    let doc = parse_document(text);
    if !doc.had_front_matter {
        return None;
    }
    extract_reserved_field(&doc.fm_lines, field)
}

fn extract_reserved_field(lines: &[String], field: &str) -> Option<String> {
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].starts_with(' ') && !lines[i].starts_with('\t') {
            let trimmed = lines[i].trim();
            if let Some((k, v)) = trimmed.split_once(':') {
                if k.trim() == field {
                    let rest = v.trim();
                    if rest == "|" || rest == ">" || rest == "|-" || rest == ">-" {
                        let mut val = String::new();
                        i += 1;
                        while i < lines.len()
                            && (lines[i].starts_with(' ')
                                || lines[i].starts_with('\t')
                                || (lines[i].trim().is_empty()
                                    && i + 1 < lines.len()
                                    && (lines[i + 1].starts_with(' ')
                                        || lines[i + 1].starts_with('\t'))))
                        {
                            if lines[i].trim().is_empty() {
                                val.push('\n');
                                i += 1;
                                continue;
                            }
                            if !val.is_empty() {
                                val.push('\n');
                            }
                            val.push_str(lines[i].trim_start());
                            i += 1;
                        }
                        return Some(val);
                    }
                    return Some(rest.trim_matches('"').to_string());
                }
            }
        }
        i += 1;
    }
    None
}

fn extract_seal(
    lines: &[String],
    seal_field: &str,
) -> Option<std::result::Result<Digest, crate::Error>> {
    extract_reserved_field(lines, seal_field).map(|raw| raw.parse())
}

/// Drop reserved keys (and their indented continuation blocks) from FM lines,
/// preserving all other front-matter lines verbatim.
fn strip_reserved_fm_lines(lines: &[String], seal_field: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some((k, _)) = trimmed.split_once(':') {
            if !lines[i].starts_with(' ')
                && !lines[i].starts_with('\t')
                && is_reserved_fm_key(k.trim(), seal_field)
            {
                i += 1;
                while i < lines.len()
                    && (lines[i].starts_with(' ')
                        || lines[i].starts_with('\t')
                        || (lines[i].trim().is_empty()
                            && i + 1 < lines.len()
                            && (lines[i + 1].starts_with(' ') || lines[i + 1].starts_with('\t'))))
                {
                    i += 1;
                }
                continue;
            }
        }
        out.push(lines[i].clone());
        i += 1;
    }
    out
}

/// Build `instruct-v2` domain-separated hash input bytes per canonical mode.
fn hash_payload(doc: &ParsedDoc, opts: &InstructOptions) -> Vec<u8> {
    let body_lf = normalize_lf(&doc.body_raw);
    match opts.canonical {
        CanonicalMode::BodyOnly => format!("{INSTRUCT_V2_BODY_DOMAIN}\n{body_lf}").into_bytes(),
        CanonicalMode::Full => {
            let kept = strip_reserved_fm_lines(&doc.fm_lines, &opts.field);
            if kept.iter().any(|l| !l.trim().is_empty()) {
                format!(
                    "{INSTRUCT_V2_DOMAIN}\nFM\n{}\nBODY\n{body_lf}",
                    kept.join("\n")
                )
                .into_bytes()
            } else {
                format!("{INSTRUCT_V2_DOMAIN}\nBODY\n{body_lf}").into_bytes()
            }
        }
    }
}

fn compute_digest(doc: &ParsedDoc, opts: &InstructOptions) -> Digest {
    let payload = hash_payload(doc, opts);
    Digest::compute(opts.algorithm, &payload)
}

/// Compute the `instruct-v2` digest of raw document text (without modifying it).
pub fn digest_instruct_bytes(text: &str, opts: &InstructOptions) -> Digest {
    let doc = parse_document(text);
    compute_digest(&doc, opts)
}

struct RenderedSig<'a> {
    armor: &'a str,
    format: SignFormat,
    scope: &'a str,
    signer: Option<&'a str>,
}

/// Write seal (+ optional sig metadata) into front matter; returns full file text.
fn render_with_seal(
    doc: &ParsedDoc,
    opts: &InstructOptions,
    digest: &Digest,
    sig: Option<RenderedSig<'_>>,
) -> String {
    let mut fm_lines = strip_reserved_fm_lines(&doc.fm_lines, &opts.field);
    if fm_lines.iter().all(|l| l.trim().is_empty()) {
        fm_lines.clear();
    }
    fm_lines.push(format!("{}: \"{}\"", opts.field, digest.qualified()));
    if let Some(s) = sig {
        fm_lines.push(format!("{SIG_FORMAT_FIELD}: {}", s.format.as_str()));
        fm_lines.push(format!("{SCOPE_FIELD}: \"{}\"", s.scope));
        if let Some(signer) = s.signer {
            fm_lines.push(format!("{SIGNER_FIELD}: \"{signer}\""));
        }
        fm_lines.push(format!("{SIG_FIELD}: |"));
        for line in s.armor.trim().lines() {
            fm_lines.push(format!("  {line}"));
        }
    }

    let mut out = String::from("---\n");
    for line in fm_lines {
        out.push_str(&line);
        if !line.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("---\n");
    out.push_str(&doc.body_raw);
    out
}

/// Seal instruct markdown bytes; returns new file contents and digest.
pub fn seal_instruct_bytes(text: &str, opts: &InstructOptions) -> Result<(String, Digest)> {
    seal_instruct_bytes_with(
        text,
        &SealOpts {
            instruct: opts.clone(),
            sign: false,
            signing: SigningOptions::default(),
            scope: None,
        },
    )
}

/// Seal with optional `signing-v2` signature.
pub fn seal_instruct_bytes_with(text: &str, opts: &SealOpts) -> Result<(String, Digest)> {
    if opts.instruct.unicode_policy == UnicodePolicy::Fail {
        let u = scan_suspicious_unicode(text);
        if !u.is_empty() {
            return Err(Error::InvalidFormat(format!(
                "suspicious unicode: {}",
                format_unicode_findings(&u)
            )));
        }
    }
    let mut doc = parse_document(text);
    if !doc.had_front_matter {
        doc.had_front_matter = true;
        doc.fm_lines = Vec::new();
    }
    doc.fm_lines = strip_reserved_fm_lines(&doc.fm_lines, &opts.instruct.field);
    let digest = compute_digest(&doc, &opts.instruct);
    let sig_bundle = if opts.sign {
        let scope = opts.scope.clone().unwrap_or_else(|| "*".into());
        let (armor, fmt) =
            signing::sign_instruct_digest(&digest.qualified(), &scope, &opts.signing)?;
        Some((armor, fmt, scope))
    } else {
        None
    };
    let rendered_sig = sig_bundle.as_ref().map(|(armor, fmt, scope)| RenderedSig {
        armor: armor.as_str(),
        format: *fmt,
        scope: scope.as_str(),
        signer: None,
    });
    let sealed = render_with_seal(&doc, &opts.instruct, &digest, rendered_sig);
    Ok((sealed, digest))
}

/// Seal a file in place.
pub fn seal_instruct_path(path: &Path, opts: &InstructOptions) -> Result<Digest> {
    seal_instruct_path_with(
        path,
        &SealOpts {
            instruct: opts.clone(),
            sign: false,
            signing: SigningOptions::default(),
            scope: None,
        },
    )
}

pub fn seal_instruct_path_with(path: &Path, opts: &SealOpts) -> Result<Digest> {
    let text = fs::read_to_string(path)?;
    let (sealed, digest) = seal_instruct_bytes_with(&text, opts)?;
    fs::write(path, sealed)?;
    Ok(digest)
}

/// Check instruct markdown bytes (digest only).
pub fn check_instruct_bytes(text: &str, opts: &InstructOptions) -> CheckResult {
    check_instruct_bytes_with(
        text,
        &CheckOpts {
            instruct: opts.clone(),
            require_signature: false,
            digest_only: true,
            expected_scope: None,
            signing: SigningOptions::default(),
        },
        None,
    )
}

/// Check with `signing-v2` policy and fail-closed signature verification.
pub fn check_instruct_bytes_with(
    text: &str,
    opts: &CheckOpts,
    path: Option<PathBuf>,
) -> CheckResult {
    if opts.instruct.unicode_policy == UnicodePolicy::Fail {
        let u = scan_suspicious_unicode(text);
        if !u.is_empty() {
            return CheckResult::suspicious_unicode(path, format_unicode_findings(&u));
        }
    }
    let doc = parse_document(text);
    if !doc.had_front_matter {
        let actual = compute_digest(&doc, &opts.instruct);
        return CheckResult::missing_seal(path, Some(actual));
    }
    match extract_seal(&doc.fm_lines, &opts.instruct.field) {
        None => {
            let actual = compute_digest(&doc, &opts.instruct);
            CheckResult::missing_seal(path, Some(actual))
        }
        Some(Err(e)) => CheckResult::invalid_format(path, e.to_string()),
        Some(Ok(expected)) => {
            let mut opts_use = opts.instruct.clone();
            opts_use.algorithm = expected.algorithm;
            let actual = compute_digest(&doc, &opts_use);
            if actual.hex != expected.hex || actual.algorithm != expected.algorithm {
                return CheckResult::mismatch(path, &expected, &actual);
            }

            let sig = extract_reserved_field(&doc.fm_lines, SIG_FIELD);
            if opts.require_signature && sig.is_none() {
                return CheckResult::missing_signature(path, &actual);
            }

            if let Some(armor) = sig {
                if opts.digest_only {
                    return CheckResult::valid(path, &actual);
                }
                let declared_scope = extract_reserved_field(&doc.fm_lines, SCOPE_FIELD);
                let verify_scope = match (&declared_scope, &opts.expected_scope) {
                    (Some(s), Some(exp)) if s != "*" && s != exp => {
                        return CheckResult::bad_signature(
                            path,
                            &actual,
                            format!(
                                "signature scope mismatch: signed for '{s}', checked at '{exp}'"
                            ),
                        );
                    }
                    (Some(s), _) => s.as_str(),
                    (None, Some(exp)) => exp.as_str(),
                    (None, None) => "*",
                };

                match signing::verify_instruct_digest(
                    &actual.qualified(),
                    verify_scope,
                    &armor,
                    &opts.signing,
                ) {
                    VerifyOutcome::Good { .. } => CheckResult::valid(path, &actual),
                    VerifyOutcome::Bad(msg) => CheckResult::bad_signature(path, &actual, msg),
                    VerifyOutcome::UntrustedKey(msg) => {
                        CheckResult::untrusted_key(path, &actual, msg)
                    }
                    VerifyOutcome::Unavailable(msg) => {
                        CheckResult::unverified_signature(path, &actual, msg)
                    }
                }
            } else {
                CheckResult::valid(path, &actual)
            }
        }
    }
}

/// Check an instruct file on disk (digest only).
pub fn check_instruct_path(path: &Path, opts: &InstructOptions) -> CheckResult {
    match fs::read_to_string(path) {
        Ok(text) => check_instruct_bytes_with(
            &text,
            &CheckOpts {
                instruct: opts.clone(),
                require_signature: false,
                digest_only: true,
                expected_scope: None,
                signing: SigningOptions::default(),
            },
            Some(path.to_path_buf()),
        ),
        Err(e) => CheckResult::io_error(Some(path.to_path_buf()), e.to_string()),
    }
}

pub fn check_instruct_path_with(path: &Path, opts: &CheckOpts) -> CheckResult {
    match fs::read_to_string(path) {
        Ok(text) => check_instruct_bytes_with(&text, opts, Some(path.to_path_buf())),
        Err(e) => CheckResult::io_error(Some(path.to_path_buf()), e.to_string()),
    }
}

/// Remove seal + signature fields from front matter.
pub fn unseal_instruct_bytes(text: &str, opts: &InstructOptions) -> String {
    let doc = parse_document(text);
    if !doc.had_front_matter {
        return text.to_string();
    }
    let fm_lines = strip_reserved_fm_lines(&doc.fm_lines, &opts.field);
    if fm_lines.iter().all(|l| l.trim().is_empty()) {
        return doc.body_raw;
    }
    let mut out = String::from("---\n");
    for line in fm_lines {
        out.push_str(&line);
        if !line.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("---\n");
    out.push_str(&doc.body_raw);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CheckStatus;
    use tempfile::tempdir;

    fn status_name(s: CheckStatus) -> &'static str {
        match s {
            CheckStatus::Valid => "valid",
            CheckStatus::MissingSeal => "missing_seal",
            CheckStatus::Mismatch => "mismatch",
            CheckStatus::InvalidFormat => "invalid_format",
            CheckStatus::IoError => "io_error",
            CheckStatus::MissingSignature => "missing_signature",
            CheckStatus::BadSignature => "bad_signature",
            CheckStatus::UntrustedKey => "untrusted_key",
            CheckStatus::UnverifiedSignature => "unverified_signature",
            CheckStatus::SuspiciousUnicode => "suspicious_unicode",
            CheckStatus::LockedFileRemoved => "locked_file_removed",
            CheckStatus::UnapprovedFile => "unapproved_file",
        }
    }

    /// Frozen multi-language vectors — digests must match core FULL mode (instruct-v2).
    #[test]
    fn official_vectors_instruct_v2() {
        let Some(path) = crate::test_paths::try_monorepo_path("verify/vectors/instruct-v2.json")
        else {
            return;
        };
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()));
        let doc: serde_json::Value =
            serde_json::from_str(&raw).expect("instruct-v2.json must be valid JSON");
        assert_eq!(doc["spec"], "instruct-v2");
        assert_eq!(doc["canonical"], "full");
        assert_eq!(doc["algorithm"], "blake3");

        let opts = InstructOptions {
            field: doc["field"].as_str().unwrap_or(SEAL_FIELD).into(),
            algorithm: Algorithm::Blake3,
            canonical: CanonicalMode::Full,
            auto_frontmatter: true,
            unicode_policy: UnicodePolicy::Fail,
        };

        let cases = doc["cases"].as_array().expect("cases array");
        assert!(!cases.is_empty(), "vector file must have cases");

        for case in cases {
            let id = case["id"].as_str().unwrap_or("?");
            let kind = case["kind"].as_str().unwrap_or("check");
            match kind {
                "raw_digest" => {
                    let bytes = case["bytes_utf8"].as_str().expect("bytes_utf8");
                    let expect = case["expect"]["digest"].as_str().expect("expect.digest");
                    let actual = Digest::compute(Algorithm::Blake3, bytes.as_bytes()).qualified();
                    assert_eq!(actual, expect, "raw_digest case {id}");
                }
                "check" => {
                    let text = case["text"].as_str().expect("text");
                    let r = check_instruct_bytes(text, &opts);
                    let expect = &case["expect"];
                    let want_ok = expect["ok"].as_bool().expect("expect.ok");
                    let want_status = expect["status"].as_str().expect("expect.status");
                    assert_eq!(r.ok, want_ok, "case {id} ok: {r:?}");
                    assert_eq!(
                        status_name(r.status),
                        want_status,
                        "case {id} status: {r:?}"
                    );
                    if let Some(d) = expect["digest"].as_str() {
                        assert_eq!(r.actual.as_deref(), Some(d), "case {id} digest actual");
                        if r.ok {
                            assert_eq!(r.expected.as_deref(), Some(d), "case {id} digest expected");
                        }
                    }
                    if let Some(e) = expect["expected"].as_str() {
                        assert_eq!(r.expected.as_deref(), Some(e), "case {id} expected");
                    }
                    if let Some(a) = expect["actual"].as_str() {
                        assert_eq!(r.actual.as_deref(), Some(a), "case {id} actual");
                    }
                }
                other => panic!("unknown case kind {other} in {id}"),
            }
        }
    }

    #[test]
    fn seal_twice_stable() {
        let opts = InstructOptions::default();
        let src = "# Title\n\nHello agent.\n";
        let (once, d1) = seal_instruct_bytes(src, &opts).unwrap();
        let (twice, d2) = seal_instruct_bytes(&once, &opts).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(once, twice);
        let check = check_instruct_bytes(&twice, &opts);
        assert!(check.ok, "{check:?}");
    }

    #[test]
    fn body_tamper_fails() {
        let opts = InstructOptions::default();
        let (sealed, _) = seal_instruct_bytes("# A\n\nbody\n", &opts).unwrap();
        let tampered = sealed.replace("body", "evil");
        let check = check_instruct_bytes(&tampered, &opts);
        assert!(!check.ok);
        assert_eq!(check.status, CheckStatus::Mismatch);
        assert!(check.expected.is_some());
        assert!(check.actual.is_some());
    }

    #[test]
    fn fm_policy_tamper_fails_in_full_mode() {
        let opts = InstructOptions::default();
        let src = "---\ntitle: agents\n---\n# Hi\n";
        let (sealed, _) = seal_instruct_bytes(src, &opts).unwrap();
        let tampered = sealed.replace("title: agents", "title: pwned");
        let check = check_instruct_bytes(&tampered, &opts);
        assert!(!check.ok);
        assert_eq!(check.status, CheckStatus::Mismatch);
    }

    #[test]
    fn yaml_list_tamper_fails() {
        let opts = InstructOptions::default();
        let src = "---\nglobs:\n  - \"*.ts\"\n  - \"*.tsx\"\n---\n# Rule\n";
        let (sealed, _) = seal_instruct_bytes(src, &opts).unwrap();
        assert!(check_instruct_bytes(&sealed, &opts).ok);
        let tampered = sealed.replace("*.ts\"", "*.js\"");
        let check = check_instruct_bytes(&tampered, &opts);
        assert!(!check.ok);
        assert_eq!(check.status, CheckStatus::Mismatch);
    }

    #[test]
    fn yaml_nested_map_tamper_fails() {
        let opts = InstructOptions::default();
        let src = "---\ntools:\n  bash: false\n  read: true\n---\n# Skill\n";
        let (sealed, _) = seal_instruct_bytes(src, &opts).unwrap();
        assert!(check_instruct_bytes(&sealed, &opts).ok);
        let tampered = sealed.replace("bash: false", "bash: true");
        let check = check_instruct_bytes(&tampered, &opts);
        assert!(!check.ok);
        assert_eq!(check.status, CheckStatus::Mismatch);
    }

    #[test]
    fn custom_seal_field_roundtrip() {
        let opts = InstructOptions {
            field: "my_seal".into(),
            ..Default::default()
        };
        let src = "---\ntitle: custom\n---\n# Body\n";
        let (once, d1) = seal_instruct_bytes(src, &opts).unwrap();
        assert!(once.contains("my_seal: \"blake3:"));
        let (twice, d2) = seal_instruct_bytes(&once, &opts).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(once, twice);
        let check = check_instruct_bytes(&twice, &opts);
        assert!(check.ok, "{check:?}");
        let unsealed = unseal_instruct_bytes(&twice, &opts);
        assert!(!unsealed.contains("my_seal:"));
        assert!(unsealed.contains("title: custom"));
    }

    #[test]
    fn fm_with_embedded_dashes_preserves_body() {
        let opts = InstructOptions::default();
        let src = "---\ntitle: signed\nhashseal_sig: |\n  -----BEGIN PGP SIGNATURE-----\n  abc\n  -----END PGP SIGNATURE-----\n---\n# Body\n\nKeep me.\n";
        let doc = parse_document(src);
        assert!(doc.had_front_matter);
        assert_eq!(doc.body_raw, "# Body\n\nKeep me.\n");
        let (sealed, _) = seal_instruct_bytes(src, &opts).unwrap();
        assert!(sealed.ends_with("---\n# Body\n\nKeep me.\n"));
        assert!(check_instruct_bytes(&sealed, &opts).ok);
    }

    #[test]
    fn fm_body_boundary_smuggling_changes_digest() {
        let opts = InstructOptions::default();
        let a = "---\na: 1\n---\nBODY\nb: 2\n";
        let b = "---\na: 1\nb: 2\n---\n";
        let (_, da) = seal_instruct_bytes(a, &opts).unwrap();
        let (_, db) = seal_instruct_bytes(b, &opts).unwrap();
        assert_ne!(da, db);
    }

    #[test]
    fn suspicious_unicode_fails_by_default_and_respects_off_policy() {
        let mut opts = InstructOptions::default();
        let poisoned = "---\ntitle: ok\n---\n# Hi \u{202E}evil\n";
        assert!(seal_instruct_bytes(poisoned, &opts).is_err());

        opts.unicode_policy = UnicodePolicy::Off;
        let (sealed, _) = seal_instruct_bytes(poisoned, &opts).unwrap();
        assert!(check_instruct_bytes(&sealed, &opts).ok);

        opts.unicode_policy = UnicodePolicy::Fail;
        let check = check_instruct_bytes(&sealed, &opts);
        assert!(!check.ok);
        assert_eq!(check.status, CheckStatus::SuspiciousUnicode);
    }

    #[test]
    fn body_only_ignores_fm_churn() {
        let opts = InstructOptions {
            canonical: CanonicalMode::BodyOnly,
            ..Default::default()
        };
        let src = "---\ntitle: a\n---\n# Hi\n";
        let (sealed, _) = seal_instruct_bytes(src, &opts).unwrap();
        let lines: Vec<&str> = sealed.lines().collect();
        let mut out = String::new();
        for line in lines {
            if line.starts_with("title:") {
                out.push_str("title: b\n");
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        let check = check_instruct_bytes(&out, &opts);
        assert!(check.ok, "{check:?}");
    }

    #[test]
    fn missing_seal_detected() {
        let opts = InstructOptions::default();
        let check = check_instruct_bytes("# no seal\n", &opts);
        assert!(!check.ok);
        assert_eq!(check.status, CheckStatus::MissingSeal);
    }

    #[test]
    fn crlf_body_same_digest_as_lf() {
        let opts = InstructOptions::default();
        let lf = "# T\n\nline\n";
        let crlf = "# T\r\n\r\nline\r\n";
        let (_, d1) = seal_instruct_bytes(lf, &opts).unwrap();
        let doc_lf = parse_document(lf);
        let doc_crlf = parse_document(crlf);
        assert_eq!(
            compute_digest(&doc_lf, &opts),
            compute_digest(&doc_crlf, &opts)
        );
        assert_eq!(d1, compute_digest(&doc_lf, &opts));
    }

    #[test]
    fn seal_path_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "# Rules\n\nDo good.\n").unwrap();
        let opts = InstructOptions::default();
        seal_instruct_path(&path, &opts).unwrap();
        let r = check_instruct_path(&path, &opts);
        assert!(r.ok, "{r:?}");
    }

    #[test]
    fn wrong_seal_field_mismatch() {
        let opts = InstructOptions::default();
        let (mut sealed, _) = seal_instruct_bytes("# x\n", &opts).unwrap();
        sealed = sealed.replace(
            "blake3:",
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
        );
        let check = check_instruct_bytes(&sealed, &opts);
        assert!(!check.ok);
    }

    #[test]
    fn sig_field_excluded_from_hash() {
        let opts = InstructOptions::default();
        let (_, d1) = seal_instruct_bytes("# x\n\nbody\n", &opts).unwrap();
        let (sealed2, d2) = seal_instruct_bytes("# x\n\nbody\n", &opts).unwrap();
        assert_eq!(d1, d2);
        let doc = parse_document(&sealed2);
        let mut lines = doc.fm_lines.clone();
        lines.push("hashseal_sig: |".into());
        lines.push("  -----BEGIN PGP SIGNATURE-----".into());
        lines.push("  fake".into());
        lines.push("  -----END PGP SIGNATURE-----".into());
        lines.push("hashseal_sig_format: gpg".into());
        lines.push("hashseal_signer: build-bot@hashseal.ai".into());
        lines.push("hashseal_scope: AGENTS.md".into());
        let doc2 = ParsedDoc {
            fm_lines: lines,
            had_front_matter: true,
            body_raw: doc.body_raw.clone(),
        };
        assert_eq!(compute_digest(&doc, &opts), compute_digest(&doc2, &opts));
    }

    #[test]
    fn require_signature_missing() {
        let opts = InstructOptions::default();
        let (sealed, _) = seal_instruct_bytes("# x\n", &opts).unwrap();
        let r = check_instruct_bytes_with(
            &sealed,
            &CheckOpts {
                instruct: opts,
                require_signature: true,
                digest_only: false,
                expected_scope: None,
                signing: SigningOptions::default(),
            },
            None,
        );
        assert!(!r.ok);
        assert_eq!(r.status, CheckStatus::MissingSignature);
    }

    #[test]
    fn ssh_signed_instruct_scope_and_fail_closed() {
        let has_ssh = std::process::Command::new("ssh-keygen")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok();
        if !has_ssh {
            return;
        }
        let dir = tempdir().unwrap();
        let key = dir.path().join("id_ed25519");
        let st = std::process::Command::new("ssh-keygen")
            .args([
                "-q",
                "-t",
                "ed25519",
                "-N",
                "",
                "-C",
                "dev@hashseal.test",
                "-f",
            ])
            .arg(&key)
            .status()
            .unwrap();
        assert!(st.success());
        let pub_s = fs::read_to_string(dir.path().join("id_ed25519.pub")).unwrap();
        let allowed = dir.path().join("allowed_signers");
        fs::write(&allowed, format!("dev@hashseal.test {}\n", pub_s.trim())).unwrap();

        let signing = SigningOptions {
            format: SignFormat::Ssh,
            signing_key: Some(key.display().to_string()),
            allowed_signers: Some(allowed),
            ..Default::default()
        };

        let (sealed, _) = seal_instruct_bytes_with(
            "# Rules\n",
            &SealOpts {
                instruct: InstructOptions::default(),
                sign: true,
                signing: signing.clone(),
                scope: Some("AGENTS.md".into()),
            },
        )
        .unwrap();

        // Matching scope passes
        let ok = check_instruct_bytes_with(
            &sealed,
            &CheckOpts {
                instruct: InstructOptions::default(),
                require_signature: true,
                digest_only: false,
                expected_scope: Some("AGENTS.md".into()),
                signing: signing.clone(),
            },
            None,
        );
        assert!(ok.ok, "{ok:?}");

        // Replay at different path fails scope binding
        let replay = check_instruct_bytes_with(
            &sealed,
            &CheckOpts {
                instruct: InstructOptions::default(),
                require_signature: true,
                digest_only: false,
                expected_scope: Some("CLAUDE.md".into()),
                signing,
            },
            None,
        );
        assert!(!replay.ok);
        assert_eq!(replay.status, CheckStatus::BadSignature);

        // Fail closed when allowed_signers missing
        let unverified = check_instruct_bytes_with(
            &sealed,
            &CheckOpts {
                instruct: InstructOptions::default(),
                require_signature: false,
                digest_only: false,
                expected_scope: Some("AGENTS.md".into()),
                signing: SigningOptions::default(),
            },
            None,
        );
        assert!(!unverified.ok);
        assert_eq!(unverified.status, CheckStatus::UnverifiedSignature);

        // digest_only opts out
        let dig_ok = check_instruct_bytes_with(
            &sealed,
            &CheckOpts {
                instruct: InstructOptions::default(),
                require_signature: false,
                digest_only: true,
                expected_scope: Some("AGENTS.md".into()),
                signing: SigningOptions::default(),
            },
            None,
        );
        assert!(dig_ok.ok, "{dig_ok:?}");
    }

    #[test]
    fn official_vectors_signing_v2() {
        let has_ssh = std::process::Command::new("ssh-keygen")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok();
        if !has_ssh {
            return;
        }
        let Some(path) = crate::test_paths::try_monorepo_path("verify/vectors/signing-v2.json")
        else {
            return;
        };
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()));
        let doc: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(doc["spec"], "signing-v2");
        let dir = tempdir().unwrap();

        for (idx, case) in doc["cases"].as_array().unwrap().iter().enumerate() {
            let id = case["id"].as_str().unwrap();
            let allowed_path = dir.path().join(format!("allowed_{idx}"));
            fs::write(&allowed_path, case["allowed_signers"].as_str().unwrap()).unwrap();
            let opts = CheckOpts {
                instruct: InstructOptions::default(),
                require_signature: case["require_signature"].as_bool().unwrap_or(false),
                digest_only: false,
                expected_scope: case["path"].as_str().map(str::to_string),
                signing: SigningOptions {
                    format: SignFormat::Ssh,
                    allowed_signers: Some(allowed_path),
                    ..Default::default()
                },
            };
            let r = check_instruct_bytes_with(case["text"].as_str().unwrap(), &opts, None);
            let want_ok = case["expect"]["ok"].as_bool().unwrap();
            let want_status = case["expect"]["status"].as_str().unwrap();
            assert_eq!(r.ok, want_ok, "case {id}: {r:?}");
            assert_eq!(status_name(r.status), want_status, "case {id}: {r:?}");
        }
    }
}
