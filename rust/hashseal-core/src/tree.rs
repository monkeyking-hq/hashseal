//! Tree seal / verify ledger (std walk only).

use crate::digest::{Algorithm, Digest};
use crate::error::{Error, Result};
use crate::instruct::normalize_lf;
use crate::timeutil::utc_now_rfc3339;
use crate::walk::{any_glob, walk_files};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

pub const LEDGER_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Ledger {
    pub schema_version: u32,
    pub sealed_at: String,
    pub root: String,
    pub algorithm: String,
    pub tool_version: String,
    pub entries: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct LedgerEntry {
    pub path: String,
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(rename_all = "snake_case"))]
pub enum TreeEntryStatus {
    Ok,
    Mismatch,
    Added,
    Removed,
    BrokenSymlink,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct TreeFinding {
    pub path: String,
    pub status: TreeEntryStatus,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct TreeVerifyResult {
    pub ok: bool,
    pub findings: Vec<TreeFinding>,
    pub checked: usize,
}

#[derive(Debug, Clone)]
pub struct TreeSealOptions {
    pub algorithm: Algorithm,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub line_endings_lf_text: bool,
}

impl Default for TreeSealOptions {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Blake3,
            include: vec!["**/*".into()],
            exclude: vec![
                "**/.git/**".into(),
                "**/target/**".into(),
                "**/node_modules/**".into(),
                "**/build/**".into(),
                "**/dist/**".into(),
                "**/vendor/**".into(),
                "**/.hashseal/**".into(),
                "**/hashseal-bundle/**".into(),
            ],
            line_endings_lf_text: true,
        }
    }
}

fn is_text_ext(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).unwrap_or(""),
        "md" | "txt"
            | "toml"
            | "yml"
            | "yaml"
            | "json"
            | "rs"
            | "java"
            | "go"
            | "py"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "css"
            | "html"
            | "xml"
            | "sh"
            | "ps1"
            | "c"
            | "h"
            | "cpp"
            | "cs"
            | "rb"
            | "svg"
    )
}

fn rel_posix(root: &Path, path: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| Error::InvalidFormat(format!("path not under root: {}", path.display())))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn hash_file(path: &Path, algorithm: Algorithm, lf_text: bool) -> Result<(Digest, u64)> {
    let bytes = fs::read(path)?;
    let size = bytes.len() as u64;
    let data = if lf_text && is_text_ext(path) {
        let s = String::from_utf8_lossy(&bytes);
        let s = s.strip_prefix('\u{feff}').unwrap_or(&s);
        normalize_lf(s).into_bytes()
    } else {
        bytes
    };
    Ok((Digest::compute(algorithm, &data), size))
}

fn path_excluded(rel: &str, exclude: &[String]) -> bool {
    if any_glob(exclude, rel) {
        return true;
    }
    // component-level skip
    rel.split('/').any(|c| {
        matches!(
            c,
            ".git"
                | "target"
                | "node_modules"
                | "dist"
                | "build"
                | "vendor"
                | ".hashseal"
                | "hashseal-bundle"
        )
    })
}

pub fn collect_tree_files(root: &Path, opts: &TreeSealOptions) -> Result<Vec<PathBuf>> {
    let include = opts.include.clone();
    let exclude = opts.exclude.clone();
    Ok(walk_files(root, |rel| {
        if path_excluded(rel, &exclude) {
            return false;
        }
        any_glob(&include, rel)
    })?)
}

pub fn seal_tree(root: &Path, opts: &TreeSealOptions) -> Result<Ledger> {
    let files = collect_tree_files(root, opts)?;
    let mut entries = Vec::with_capacity(files.len());
    for path in files {
        let rel = rel_posix(root, &path)?;
        let (digest, size) = hash_file(&path, opts.algorithm, opts.line_endings_lf_text)?;
        entries.push(LedgerEntry {
            path: rel,
            digest: digest.qualified(),
            size,
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    let root_s = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    Ok(Ledger {
        schema_version: LEDGER_SCHEMA_VERSION,
        sealed_at: utc_now_rfc3339(),
        root: root_s,
        algorithm: opts.algorithm.as_str().to_string(),
        tool_version: crate::VERSION.to_string(),
        entries,
    })
}

#[cfg(feature = "json")]
pub fn write_ledger(path: &Path, ledger: &Ledger) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(ledger)
        .map_err(|e| Error::InvalidFormat(format!("ledger json: {e}")))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(feature = "json")]
pub fn read_ledger(path: &Path) -> Result<Ledger> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| Error::InvalidFormat(format!("ledger parse: {e}")))
}

pub fn verify_tree(
    root: &Path,
    ledger: &Ledger,
    opts: &TreeSealOptions,
) -> Result<TreeVerifyResult> {
    let algorithm = ledger.algorithm.parse().unwrap_or(opts.algorithm);
    let mut opts = opts.clone();
    opts.algorithm = algorithm;

    let current_files = collect_tree_files(root, &opts)?;
    let mut current_map = std::collections::BTreeMap::new();
    for path in current_files {
        let rel = rel_posix(root, &path)?;
        match hash_file(&path, opts.algorithm, opts.line_endings_lf_text) {
            Ok((d, _)) => {
                current_map.insert(rel, d.qualified());
            }
            Err(_) => {
                current_map.insert(rel, String::new());
            }
        }
    }

    let mut findings = Vec::new();
    let mut expected_paths = std::collections::BTreeSet::new();

    for e in &ledger.entries {
        expected_paths.insert(e.path.clone());
        match current_map.get(&e.path) {
            None => findings.push(TreeFinding {
                path: e.path.clone(),
                status: TreeEntryStatus::Removed,
                expected: Some(e.digest.clone()),
                actual: None,
            }),
            Some(actual) if actual.is_empty() || actual != &e.digest => {
                findings.push(TreeFinding {
                    path: e.path.clone(),
                    status: TreeEntryStatus::Mismatch,
                    expected: Some(e.digest.clone()),
                    actual: if actual.is_empty() {
                        None
                    } else {
                        Some(actual.clone())
                    },
                });
            }
            Some(_) => {}
        }
    }

    for (path, digest) in &current_map {
        if !expected_paths.contains(path) {
            findings.push(TreeFinding {
                path: path.clone(),
                status: TreeEntryStatus::Added,
                expected: None,
                actual: Some(digest.clone()),
            });
        }
    }

    findings.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(TreeVerifyResult {
        ok: findings.is_empty(),
        findings,
        checked: ledger.entries.len(),
    })
}

pub fn clean_tree_artifacts(paths: &[PathBuf]) -> Result<()> {
    for p in paths {
        if p.is_file() {
            let _ = fs::remove_file(p);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn seal_verify_roundtrip() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("a.txt"), "hello\n").unwrap();
        fs::create_dir_all(d.path().join("src")).unwrap();
        fs::write(d.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        let opts = TreeSealOptions::default();
        let ledger = seal_tree(d.path(), &opts).unwrap();
        assert!(ledger.entries.len() >= 2);
        let v = verify_tree(d.path(), &ledger, &opts).unwrap();
        assert!(v.ok, "{:?}", v.findings);
    }

    #[test]
    fn detect_modify_add_remove() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("a.txt"), "one\n").unwrap();
        fs::write(d.path().join("b.txt"), "two\n").unwrap();
        let opts = TreeSealOptions::default();
        let ledger = seal_tree(d.path(), &opts).unwrap();
        fs::write(d.path().join("a.txt"), "changed\n").unwrap();
        fs::remove_file(d.path().join("b.txt")).unwrap();
        fs::write(d.path().join("c.txt"), "new\n").unwrap();
        let v = verify_tree(d.path(), &ledger, &opts).unwrap();
        assert!(!v.ok);
    }

    #[test]
    fn crlf_lf_same_text_hash() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("a.txt"), "hello\r\nworld\r\n").unwrap();
        let opts = TreeSealOptions::default();
        let ledger = seal_tree(d.path(), &opts).unwrap();
        fs::write(d.path().join("a.txt"), "hello\nworld\n").unwrap();
        let v = verify_tree(d.path(), &ledger, &opts).unwrap();
        assert!(v.ok, "{:?}", v.findings);
    }

    fn status_name(s: TreeEntryStatus) -> &'static str {
        match s {
            TreeEntryStatus::Ok => "ok",
            TreeEntryStatus::Mismatch => "mismatch",
            TreeEntryStatus::Added => "added",
            TreeEntryStatus::Removed => "removed",
            TreeEntryStatus::BrokenSymlink => "broken_symlink",
        }
    }

    fn hash_content_like_core(path: &str, content: &str, lf_text: bool) -> (String, u64) {
        let size = content.len() as u64;
        let data = if lf_text && is_text_ext(Path::new(path)) {
            let s = content.strip_prefix('\u{feff}').unwrap_or(content);
            normalize_lf(s).into_bytes()
        } else {
            content.as_bytes().to_vec()
        };
        (Digest::compute(Algorithm::Blake3, &data).qualified(), size)
    }

    fn write_files_map(root: &Path, files: &serde_json::Map<String, serde_json::Value>) {
        for (rel, val) in files {
            let content = val.as_str().expect("file content string");
            let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, content).unwrap();
        }
    }

    /// Frozen multi-language tree vectors — digests must match core seal/verify.
    #[test]
    fn official_vectors_tree_v1() {
        let path = crate::test_paths::monorepo_path("verify/vectors/tree-v1.json");
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()));
        let doc: serde_json::Value =
            serde_json::from_str(&raw).expect("tree-v1.json must be valid JSON");
        assert_eq!(doc["spec"], "tree-v1");
        assert_eq!(doc["algorithm"], "blake3");
        let lf_text = doc["line_endings_lf_text"].as_bool().unwrap_or(true);

        let cases = doc["cases"].as_array().expect("cases array");
        assert!(!cases.is_empty(), "vector file must have cases");

        for case in cases {
            let id = case["id"].as_str().unwrap_or("?");
            let kind = case["kind"].as_str().unwrap_or("");
            match kind {
                "raw_file_digest" => {
                    let p = case["path"].as_str().expect("path");
                    let content = case["content"].as_str().expect("content");
                    let expect = &case["expect"];
                    let want_d = expect["digest"].as_str().expect("expect.digest");
                    let want_size = expect["size"].as_u64().expect("expect.size");
                    let (actual, size) = hash_content_like_core(p, content, lf_text);
                    assert_eq!(actual, want_d, "raw_file_digest {id} digest");
                    assert_eq!(size, want_size, "raw_file_digest {id} size");
                }
                "verify_tree" => {
                    let d = tempdir().unwrap();
                    let root = d.path();
                    if let Some(files) = case["files"].as_object() {
                        write_files_map(root, files);
                    }
                    let include: Vec<String> = case["include"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_else(|| vec!["**/*".into()]);
                    let exclude: Vec<String> = case["exclude"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let opts = TreeSealOptions {
                        algorithm: Algorithm::Blake3,
                        include,
                        exclude,
                        line_endings_lf_text: lf_text,
                    };
                    let mut entries = Vec::new();
                    for e in case["ledger_entries"].as_array().expect("ledger_entries") {
                        entries.push(LedgerEntry {
                            path: e["path"].as_str().unwrap().to_string(),
                            digest: e["digest"].as_str().unwrap().to_string(),
                            size: e["size"].as_u64().unwrap_or(0),
                        });
                    }
                    let ledger = Ledger {
                        schema_version: LEDGER_SCHEMA_VERSION,
                        sealed_at: "1970-01-01T00:00:00Z".into(),
                        root: root.to_string_lossy().replace('\\', "/"),
                        algorithm: "blake3".into(),
                        tool_version: crate::VERSION.to_string(),
                        entries,
                    };
                    // Frozen digests must match a fresh seal of the same paths when ok is true
                    // and files match ledger (roundtrip cases). Always compare verify findings.
                    let v = verify_tree(root, &ledger, &opts).unwrap();
                    let expect = &case["expect"];
                    let want_ok = expect["ok"].as_bool().expect("expect.ok");
                    let want_checked = expect["checked"].as_u64().expect("expect.checked") as usize;
                    assert_eq!(v.ok, want_ok, "case {id} ok: {:?}", v.findings);
                    assert_eq!(v.checked, want_checked, "case {id} checked");

                    let want_findings = expect["findings"].as_array().expect("expect.findings");
                    assert_eq!(
                        v.findings.len(),
                        want_findings.len(),
                        "case {id} findings count: {:?}",
                        v.findings
                    );
                    for (i, (got, want)) in v.findings.iter().zip(want_findings.iter()).enumerate()
                    {
                        assert_eq!(
                            got.path,
                            want["path"].as_str().unwrap(),
                            "case {id} finding[{i}] path"
                        );
                        assert_eq!(
                            status_name(got.status),
                            want["status"].as_str().unwrap(),
                            "case {id} finding[{i}] status"
                        );
                        let want_exp = want["expected"].as_str();
                        assert_eq!(
                            got.expected.as_deref(),
                            want_exp,
                            "case {id} finding[{i}] expected"
                        );
                        let want_act = want["actual"].as_str();
                        assert_eq!(
                            got.actual.as_deref(),
                            want_act,
                            "case {id} finding[{i}] actual"
                        );
                    }

                    // Cross-check: seal of current files produces digests matching frozen raw hashes
                    if want_ok {
                        let sealed = seal_tree(root, &opts).unwrap();
                        assert_eq!(
                            sealed.entries.len(),
                            ledger.entries.len(),
                            "case {id} seal entry count"
                        );
                        for (s, e) in sealed.entries.iter().zip(ledger.entries.iter()) {
                            assert_eq!(s.path, e.path, "case {id} seal path");
                            assert_eq!(s.digest, e.digest, "case {id} seal digest for {}", e.path);
                        }
                    }
                }
                other => panic!("unknown case kind {other} in {id}"),
            }
        }
    }
}
