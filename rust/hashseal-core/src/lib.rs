//! HashSeal core — minimal dependencies (blake3 + optional serde_json).
//!
//! Signed, Sealed, Delivered - I'm Yours.
//! Copyright (c) 2026 Nathan A. Chadwick

pub mod bundle;
pub mod config;
pub mod digest;
pub mod error;
pub mod gpg;
pub mod instruct;
#[cfg(feature = "json")]
pub mod lockfile;
pub mod report;
pub mod result;
pub mod signing;
pub mod timeutil;
pub mod tree;
pub mod unicode;
pub mod walk;

pub use bundle::{
    digest_artifacts, write_bundle, ArtifactDigest, BundleManifest, BundlePaths, BUNDLE_DIR_NAME,
};
pub use config::{
    default_document_includes, default_tree_excludes, HashSealConfig, WalkConfig,
    DEFAULT_DOCUMENT_INCLUDES, DEFAULT_TREE_EXCLUDES,
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
pub use signing::{
    sign_bytes, sign_instruct_digest, signing_payload_v2, verify_bytes, verify_instruct_digest,
    SignFormat, SigningOptions, VerifyOutcome,
};
pub use tree::{
    clean_tree_artifacts, collect_tree_files, seal_tree, verify_tree, Ledger, TreeSealOptions,
    TreeVerifyResult,
};
pub use unicode::{
    format_unicode_findings, scan_suspicious_unicode, UnicodeFinding, UnicodePolicy,
};
pub use walk::WalkOptions;

#[cfg(feature = "json")]
pub use bundle::{read_manifest, sign_bundle, verify_bundle, verify_bundle_with_signing};
#[cfg(feature = "json")]
pub use lockfile::{
    build_lockfile, read_lockfile, sign_lockfile, verify_lockfile, write_lockfile, LockEntry,
    Lockfile, LOCKFILE_NAME, LOCKFILE_SIG_NAME,
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
    /// Returns `None` when running outside the monorepo (e.g. unpacked crates.io tarball).
    pub fn try_monorepo_root() -> Option<PathBuf> {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            if dir.join("Cargo.toml").is_file() && dir.join("verify").join("vectors").is_dir() {
                return Some(dir);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Path relative to the monorepo root, e.g. `verify/vectors/instruct-v2.json`.
    pub fn try_monorepo_path(rel: impl AsRef<Path>) -> Option<PathBuf> {
        try_monorepo_root().map(|root| root.join(rel))
    }
}
