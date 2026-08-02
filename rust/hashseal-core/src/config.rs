//! Minimal config: defaults + optional JSON overlay (no toml crate).

use crate::digest::Algorithm;
use crate::error::Result;
use crate::instruct::CanonicalMode;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(feature = "json")]
use crate::error::Error;
#[cfg(feature = "json")]
use std::fs;

#[derive(Debug, Clone)]
pub struct HashSealConfig {
    pub version: u32,
    pub tree: TreeConfig,
    pub document: DocumentConfig,
    pub enforce: EnforceConfig,
    pub signing: SigningConfig,
    pub report: ReportConfig,
}

#[derive(Debug, Clone)]
pub struct TreeConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub line_endings: String,
    pub algorithm: String,
    pub ledger: Option<String>,
    pub report: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DocumentConfig {
    pub enable: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub canonical: String,
    pub field: String,
    pub auto_frontmatter: bool,
    pub algorithm: String,
}

#[derive(Debug, Clone)]
pub struct EnforceConfig {
    pub on_tree_mismatch: String,
    pub on_doc_mismatch: String,
    pub on_missing_ledger: String,
}

#[derive(Debug, Clone)]
pub struct SigningConfig {
    pub enable: bool,
    pub require: bool,
    pub signing_key: Option<String>,
    pub gpg_program: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub write: bool,
    pub include_ci: bool,
    pub include_hostname: bool,
    pub max_findings: usize,
    pub pretty: bool,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
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
            line_endings: "lf-text".into(),
            algorithm: "blake3".into(),
            ledger: None,
            report: None,
        }
    }
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            enable: true,
            include: vec!["**/*.md".into()],
            exclude: TreeConfig::default().exclude,
            canonical: "full".into(),
            field: "hashseal".into(),
            auto_frontmatter: true,
            algorithm: "blake3".into(),
        }
    }
}

impl Default for EnforceConfig {
    fn default() -> Self {
        Self {
            on_tree_mismatch: "fail".into(),
            on_doc_mismatch: "fail".into(),
            on_missing_ledger: "fail".into(),
        }
    }
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            enable: false,
            require: false,
            signing_key: None,
            gpg_program: None,
        }
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            write: true,
            include_ci: true,
            include_hostname: false,
            max_findings: 10_000,
            pretty: false,
        }
    }
}

impl Default for HashSealConfig {
    fn default() -> Self {
        Self {
            version: 1,
            tree: TreeConfig::default(),
            document: DocumentConfig::default(),
            enforce: EnforceConfig::default(),
            signing: SigningConfig::default(),
            report: ReportConfig::default(),
        }
    }
}

impl HashSealConfig {
    pub fn load_for_root(root: &Path) -> Result<Self> {
        // Prefer JSON (no toml crate). Fallback: defaults.
        let json_path = root.join(".hashseal.json");
        if json_path.is_file() {
            return Self::load_json_path(&json_path);
        }
        // Optional: ignore .hashseal.toml content for now beyond defaults
        // (full TOML parser would add a dep). Document that JSON is supported.
        Ok(Self::default())
    }

    #[cfg(feature = "json")]
    pub fn load_json_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        let v: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::InvalidFormat(format!("config json: {e}")))?;
        let mut cfg = Self::default();
        cfg.apply_json(&v)?;
        Ok(cfg)
    }

    #[cfg(not(feature = "json"))]
    pub fn load_json_path(_path: &Path) -> Result<Self> {
        Ok(Self::default())
    }

    #[cfg(feature = "json")]
    pub fn merge_overlay_json(&mut self, overlay: &serde_json::Value) -> Result<()> {
        self.apply_json(overlay)
    }

    #[cfg(feature = "json")]
    fn apply_json(&mut self, v: &serde_json::Value) -> Result<()> {
        if let Some(o) = v.get("signing") {
            if let Some(b) = o.get("enable").and_then(|x| x.as_bool()) {
                self.signing.enable = b;
            }
            if let Some(b) = o.get("require").and_then(|x| x.as_bool()) {
                self.signing.require = b;
            }
            if let Some(s) = o.get("signing_key").and_then(|x| x.as_str()) {
                self.signing.signing_key = Some(s.into());
            }
        }
        if let Some(o) = v.get("enforce") {
            if let Some(s) = o.get("on_tree_mismatch").and_then(|x| x.as_str()) {
                self.enforce.on_tree_mismatch = s.into();
            }
            if let Some(s) = o.get("on_doc_mismatch").and_then(|x| x.as_str()) {
                self.enforce.on_doc_mismatch = s.into();
            }
        }
        if let Some(o) = v.get("document") {
            if let Some(b) = o.get("enable").and_then(|x| x.as_bool()) {
                self.document.enable = b;
            }
            if let Some(s) = o.get("canonical").and_then(|x| x.as_str()) {
                self.document.canonical = s.into();
            }
            if let Some(a) = o.get("include").and_then(|x| x.as_array()) {
                self.document.include = a
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect();
            }
        }
        if let Some(o) = v.get("tree") {
            if let Some(s) = o.get("ledger").and_then(|x| x.as_str()) {
                self.tree.ledger = Some(s.into());
            }
            if let Some(s) = o.get("algorithm").and_then(|x| x.as_str()) {
                self.tree.algorithm = s.into();
            }
            if let Some(a) = o.get("exclude").and_then(|x| x.as_array()) {
                self.tree.exclude = a
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect();
            }
        }
        Ok(())
    }

    pub fn tree_algorithm(&self) -> Result<Algorithm> {
        Algorithm::from_str(&self.tree.algorithm)
    }

    pub fn document_algorithm(&self) -> Result<Algorithm> {
        Algorithm::from_str(&self.document.algorithm)
    }

    pub fn document_canonical(&self) -> CanonicalMode {
        match self.document.canonical.as_str() {
            "body-only" => CanonicalMode::BodyOnly,
            _ => CanonicalMode::Full,
        }
    }

    pub fn default_ledger_path(&self, root: &Path) -> PathBuf {
        if let Some(p) = &self.tree.ledger {
            let pb = PathBuf::from(p);
            if pb.is_absolute() {
                pb
            } else {
                root.join(pb)
            }
        } else {
            root.join("hashseal-bundle").join("ledger.json")
        }
    }

    pub fn default_report_path(&self, root: &Path) -> PathBuf {
        if let Some(p) = &self.tree.report {
            let pb = PathBuf::from(p);
            if pb.is_absolute() {
                pb
            } else {
                root.join(pb)
            }
        } else {
            root.join("hashseal-bundle").join("report.json")
        }
    }
}
