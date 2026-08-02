//! HashSeal core — minimal dependencies (blake3 + optional serde_json).
//!
//! Signed, Sealed, Delivered - I'm Yours.
//! Copyright (c) 2026 MonkeyKing.dev

pub mod bundle;
pub mod config;
pub mod digest;
pub mod error;
pub mod gpg;
pub mod instruct;
pub mod report;
pub mod result;
pub mod timeutil;
pub mod tree;
pub mod walk;

pub use bundle::{
    digest_artifacts, write_bundle, ArtifactDigest, BundleManifest, BundlePaths, BUNDLE_DIR_NAME,
};
pub use config::{
    default_document_includes, default_tree_excludes, HashSealConfig, DEFAULT_DOCUMENT_INCLUDES,
    DEFAULT_TREE_EXCLUDES,
};
pub use digest::{Algorithm, Digest};
pub use error::Error;
pub use gpg::{signing_payload, GpgConfig, GpgVerifyStatus};
pub use instruct::{
    check_instruct_bytes, check_instruct_bytes_with, check_instruct_path, check_instruct_path_with,
    seal_instruct_bytes, seal_instruct_bytes_with, seal_instruct_path, seal_instruct_path_with,
    unseal_instruct_bytes, CanonicalMode, CheckOpts, InstructOptions, SealOpts,
};
pub use report::{write_report, AuditReport, ReportBuilder};
pub use result::{BatchCheckResult, CheckResult, CheckStatus};
pub use tree::{
    clean_tree_artifacts, collect_tree_files, seal_tree, verify_tree, Ledger, TreeSealOptions,
    TreeVerifyResult,
};

#[cfg(feature = "json")]
pub use tree::{read_ledger, write_ledger};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Test helpers for resolving shared monorepo assets (vectors, fixtures).
#[cfg(test)]
pub(crate) mod test_paths {
    use std::path::{Path, PathBuf};

    /// Walk up from this crate until the monorepo root is found
    /// (directory containing workspace `Cargo.toml` and `verify/vectors/`).
    pub fn monorepo_root() -> PathBuf {
        let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut dir = start.clone();
        loop {
            if dir.join("Cargo.toml").is_file() && dir.join("verify").join("vectors").is_dir() {
                return dir;
            }
            if !dir.pop() {
                panic!(
                    "could not find monorepo root from CARGO_MANIFEST_DIR={}",
                    start.display()
                );
            }
        }
    }

    /// Path relative to the monorepo root, e.g. `verify/vectors/instruct-v1.json`.
    pub fn monorepo_path(rel: impl AsRef<Path>) -> PathBuf {
        monorepo_root().join(rel)
    }
}
