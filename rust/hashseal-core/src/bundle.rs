//! Integrity bundle layout and verification.

use crate::digest::{Algorithm, Digest};
use crate::error::Result;
use crate::report::AuditReport;
use crate::tree::Ledger;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "json")]
use crate::error::Error;
#[cfg(feature = "json")]
use crate::signing::{self, SigningOptions, VerifyOutcome};
#[cfg(feature = "json")]
use crate::timeutil::utc_now_rfc3339;
#[cfg(feature = "json")]
use crate::tree::{
    read_ledger, verify_tree, TreeEntryStatus, TreeFinding, TreeSealOptions, TreeVerifyResult,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

pub const BUNDLE_SCHEMA_VERSION: u32 = 1;
pub const BUNDLE_DIR_NAME: &str = "hashseal-bundle";

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct BundleManifest {
    pub bundle_schema_version: u32,
    pub created_at: String,
    pub tool_version: String,
    pub root: String,
    pub ledger: String,
    pub report: String,
    #[cfg_attr(feature = "json", serde(default))]
    pub ledger_digest: Option<String>,
    #[cfg_attr(feature = "json", serde(default))]
    pub report_digest: Option<String>,
    pub artifacts: Vec<ArtifactDigest>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct ArtifactDigest {
    pub name: String,
    pub path: String,
    pub digest: String,
    pub size: u64,
}

pub struct BundlePaths {
    pub dir: PathBuf,
    pub ledger: PathBuf,
    pub report: PathBuf,
    pub manifest: PathBuf,
}

impl BundlePaths {
    pub fn under(root: &Path) -> Self {
        let dir = root.join(BUNDLE_DIR_NAME);
        Self {
            ledger: dir.join("ledger.json"),
            report: dir.join("report.json"),
            manifest: dir.join("MANIFEST.json"),
            dir,
        }
    }
}

#[cfg(feature = "json")]
pub fn write_bundle(
    root: &Path,
    ledger: &Ledger,
    report: &AuditReport,
    artifacts: &[ArtifactDigest],
) -> Result<BundlePaths> {
    let paths = BundlePaths::under(root);
    fs::create_dir_all(&paths.dir)?;
    crate::tree::write_ledger(&paths.ledger, ledger)?;
    crate::report::write_report(&paths.report, report, true)?;
    let alg: Algorithm = ledger.algorithm.parse().unwrap_or(Algorithm::Blake3);
    let (ledger_d, _) = Digest::compute_reader(alg, fs::File::open(&paths.ledger)?)?;
    let (report_d, _) = Digest::compute_reader(alg, fs::File::open(&paths.report)?)?;
    let manifest = BundleManifest {
        bundle_schema_version: BUNDLE_SCHEMA_VERSION,
        created_at: utc_now_rfc3339(),
        tool_version: crate::VERSION.to_string(),
        root: ledger.root.clone(),
        ledger: "ledger.json".into(),
        report: "report.json".into(),
        ledger_digest: Some(ledger_d.qualified()),
        report_digest: Some(report_d.qualified()),
        artifacts: artifacts.to_vec(),
    };
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| Error::InvalidFormat(format!("manifest: {e}")))?;
    fs::write(&paths.manifest, json)?;
    Ok(paths)
}

#[cfg(not(feature = "json"))]
pub fn write_bundle(
    root: &Path,
    _ledger: &Ledger,
    _report: &AuditReport,
    _artifacts: &[ArtifactDigest],
) -> Result<BundlePaths> {
    Ok(BundlePaths::under(root))
}

#[cfg(feature = "json")]
pub fn read_manifest(path: &Path) -> Result<BundleManifest> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| Error::InvalidFormat(format!("manifest parse: {e}")))
}

#[cfg(feature = "json")]
pub fn sign_bundle(bundle_dir: &Path, opts: &SigningOptions) -> Result<PathBuf> {
    let manifest_path = bundle_dir.join("MANIFEST.json");
    let bytes = fs::read(&manifest_path)?;
    let (armor, _) = signing::sign_bytes(&bytes, opts)?;
    let sig_path = bundle_dir.join("MANIFEST.json.sig");
    let mut out = armor;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(&sig_path, out)?;
    Ok(sig_path)
}

#[cfg(feature = "json")]
pub fn verify_bundle(
    root: &Path,
    bundle_dir: &Path,
    opts: &TreeSealOptions,
) -> Result<TreeVerifyResult> {
    verify_bundle_with_signing(root, bundle_dir, opts, None, false)
}

#[cfg(feature = "json")]
pub fn verify_bundle_with_signing(
    root: &Path,
    bundle_dir: &Path,
    opts: &TreeSealOptions,
    signing_opts: Option<&SigningOptions>,
    require_signature: bool,
) -> Result<TreeVerifyResult> {
    let ledger_path = bundle_dir.join("ledger.json");
    let ledger = read_ledger(&ledger_path)?;
    let mut tree_res = verify_tree(root, &ledger, opts)?;
    let manifest_path = bundle_dir.join("MANIFEST.json");
    let sig_path = bundle_dir.join("MANIFEST.json.sig");

    if manifest_path.is_file() {
        if let Some(sopts) = signing_opts.filter(|_| sig_path.is_file() || require_signature) {
            tree_res.checked += 1;
            let manifest_bytes = fs::read(&manifest_path)?;
            let (m_dig, _) =
                Digest::compute_reader(opts.algorithm, std::io::Cursor::new(&manifest_bytes))?;
            if !sig_path.is_file() {
                tree_res.findings.push(TreeFinding {
                    path: format!("{BUNDLE_DIR_NAME}/MANIFEST.json.sig"),
                    status: TreeEntryStatus::Removed,
                    expected: Some(m_dig.qualified()),
                    actual: None,
                });
            } else {
                let armor = fs::read_to_string(&sig_path)?;
                match signing::verify_bytes(&manifest_bytes, &armor, sopts) {
                    VerifyOutcome::Good { .. } => {}
                    VerifyOutcome::Bad(msg)
                    | VerifyOutcome::UntrustedKey(msg)
                    | VerifyOutcome::Unavailable(msg) => {
                        tree_res.findings.push(TreeFinding {
                            path: format!("{BUNDLE_DIR_NAME}/MANIFEST.json"),
                            status: TreeEntryStatus::Mismatch,
                            expected: Some("valid signature".into()),
                            actual: Some(msg),
                        });
                    }
                }
            }
        }
        let manifest = read_manifest(&manifest_path)?;
        if let Some(exp_ledger) = &manifest.ledger_digest {
            tree_res.checked += 1;
            let alg = exp_ledger
                .parse::<Digest>()
                .map(|d| d.algorithm)
                .unwrap_or(opts.algorithm);
            let (actual, _) = Digest::compute_reader(alg, fs::File::open(&ledger_path)?)?;
            let act_q = actual.qualified();
            if &act_q != exp_ledger {
                tree_res.findings.push(TreeFinding {
                    path: format!("{BUNDLE_DIR_NAME}/{}", manifest.ledger),
                    status: TreeEntryStatus::Mismatch,
                    expected: Some(exp_ledger.clone()),
                    actual: Some(act_q),
                });
            }
        }

        for art in &manifest.artifacts {
            tree_res.checked += 1;
            let art_pb = PathBuf::from(&art.path);
            let resolved = if art_pb.is_absolute() {
                art_pb
            } else {
                root.join(&art_pb)
            };
            if !resolved.is_file() {
                tree_res.findings.push(TreeFinding {
                    path: art.path.clone(),
                    status: TreeEntryStatus::Removed,
                    expected: Some(art.digest.clone()),
                    actual: None,
                });
                continue;
            }
            let alg = art
                .digest
                .parse::<Digest>()
                .map(|d| d.algorithm)
                .unwrap_or(opts.algorithm);
            let (actual, _) = Digest::compute_reader(alg, fs::File::open(&resolved)?)?;
            let act_q = actual.qualified();
            if act_q != art.digest {
                tree_res.findings.push(TreeFinding {
                    path: art.path.clone(),
                    status: TreeEntryStatus::Mismatch,
                    expected: Some(art.digest.clone()),
                    actual: Some(act_q),
                });
            }
        }
    }

    tree_res.findings.sort_by(|a, b| a.path.cmp(&b.path));
    tree_res.ok = tree_res.findings.is_empty();
    Ok(tree_res)
}

pub fn digest_artifacts(paths: &[PathBuf], algorithm: Algorithm) -> Result<Vec<ArtifactDigest>> {
    let mut out = Vec::new();
    for p in paths {
        let file = fs::File::open(p)?;
        let (d, size) = Digest::compute_reader(algorithm, file)?;
        out.push(ArtifactDigest {
            name: p
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string()),
            path: p.to_string_lossy().replace('\\', "/"),
            digest: d.qualified(),
            size,
        });
    }
    Ok(out)
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use super::*;
    use crate::report::ReportBuilder;
    use crate::tree::seal_tree;
    use tempfile::tempdir;

    #[test]
    fn verify_bundle_checks_manifest_ledger_and_artifacts() {
        let d = tempdir().unwrap();
        let root = d.path();
        fs::write(root.join("src.txt"), "hello\n").unwrap();
        let art_path = root.join("dist.bin");
        fs::write(&art_path, b"\x00\x01\x02\x03").unwrap();

        let opts = TreeSealOptions::default();
        let ledger = seal_tree(root, &opts).unwrap();
        let report =
            ReportBuilder::new("seal-release", ".").finish_tree(ledger.entries.len(), None, 0);
        let arts = digest_artifacts(std::slice::from_ref(&art_path), Algorithm::Blake3).unwrap();
        let paths = write_bundle(root, &ledger, &report, &arts).unwrap();

        let ok_res = verify_bundle(root, &paths.dir, &opts).unwrap();
        assert!(ok_res.ok, "{:?}", ok_res.findings);

        // Tamper artifact
        fs::write(&art_path, b"\x00\x01\x02\x99").unwrap();
        let bad_art = verify_bundle(root, &paths.dir, &opts).unwrap();
        assert!(!bad_art.ok);
        assert!(bad_art
            .findings
            .iter()
            .any(|f| f.status == TreeEntryStatus::Mismatch));

        // Restore artifact, tamper ledger.json directly
        fs::write(&art_path, b"\x00\x01\x02\x03").unwrap();
        let mut led_txt = fs::read_to_string(&paths.ledger).unwrap();
        led_txt.push('\n');
        fs::write(&paths.ledger, led_txt).unwrap();
        let bad_led = verify_bundle(root, &paths.dir, &opts).unwrap();
        assert!(!bad_led.ok);
        assert!(bad_led
            .findings
            .iter()
            .any(|f| f.path == "hashseal-bundle/ledger.json"
                && f.status == TreeEntryStatus::Mismatch));
    }
}
