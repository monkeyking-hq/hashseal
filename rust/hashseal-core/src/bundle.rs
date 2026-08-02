//! Integrity bundle layout.

use crate::error::Result;
use crate::report::AuditReport;
use crate::tree::Ledger;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "json")]
use crate::error::Error;
#[cfg(feature = "json")]
use crate::timeutil::utc_now_rfc3339;

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
    let manifest = BundleManifest {
        bundle_schema_version: BUNDLE_SCHEMA_VERSION,
        created_at: utc_now_rfc3339(),
        tool_version: crate::VERSION.to_string(),
        root: ledger.root.clone(),
        ledger: "ledger.json".into(),
        report: "report.json".into(),
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

pub fn digest_artifacts(
    paths: &[PathBuf],
    algorithm: crate::digest::Algorithm,
) -> Result<Vec<ArtifactDigest>> {
    let mut out = Vec::new();
    for p in paths {
        let bytes = fs::read(p)?;
        let d = crate::digest::Digest::compute(algorithm, &bytes);
        out.push(ArtifactDigest {
            name: p
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string()),
            path: p.to_string_lossy().replace('\\', "/"),
            digest: d.qualified(),
            size: bytes.len() as u64,
        });
    }
    Ok(out)
}
