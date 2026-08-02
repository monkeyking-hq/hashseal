use crate::digest::Digest;
use std::path::PathBuf;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(rename_all = "snake_case"))]
pub enum CheckStatus {
    Valid,
    MissingSeal,
    Mismatch,
    InvalidFormat,
    IoError,
    MissingSignature,
    BadSignature,
    UntrustedKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct CheckResult {
    pub ok: bool,
    pub status: CheckStatus,
    pub algorithm: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub path: Option<PathBuf>,
    pub message: Option<String>,
}

impl CheckResult {
    pub fn valid(path: Option<PathBuf>, digest: &Digest) -> Self {
        Self {
            ok: true,
            status: CheckStatus::Valid,
            algorithm: Some(digest.algorithm.as_str().to_string()),
            expected: Some(digest.qualified()),
            actual: Some(digest.qualified()),
            path,
            message: None,
        }
    }

    pub fn missing_seal(path: Option<PathBuf>, actual: Option<Digest>) -> Self {
        Self {
            ok: false,
            status: CheckStatus::MissingSeal,
            algorithm: actual.as_ref().map(|d| d.algorithm.as_str().to_string()),
            expected: None,
            actual: actual.map(|d| d.qualified()),
            path,
            message: Some("missing hashseal field".into()),
        }
    }

    pub fn mismatch(path: Option<PathBuf>, expected: &Digest, actual: &Digest) -> Self {
        Self {
            ok: false,
            status: CheckStatus::Mismatch,
            algorithm: Some(expected.algorithm.as_str().to_string()),
            expected: Some(expected.qualified()),
            actual: Some(actual.qualified()),
            path,
            message: None,
        }
    }

    pub fn invalid_format(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            status: CheckStatus::InvalidFormat,
            algorithm: None,
            expected: None,
            actual: None,
            path,
            message: Some(message.into()),
        }
    }

    pub fn io_error(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            status: CheckStatus::IoError,
            algorithm: None,
            expected: None,
            actual: None,
            path,
            message: Some(message.into()),
        }
    }

    pub fn missing_signature(path: Option<PathBuf>, digest: &Digest) -> Self {
        Self {
            ok: false,
            status: CheckStatus::MissingSignature,
            algorithm: Some(digest.algorithm.as_str().to_string()),
            expected: Some(digest.qualified()),
            actual: None,
            path,
            message: Some("missing hashseal_sig".into()),
        }
    }

    pub fn bad_signature(
        path: Option<PathBuf>,
        digest: &Digest,
        message: impl Into<String>,
    ) -> Self {
        Self {
            ok: false,
            status: CheckStatus::BadSignature,
            algorithm: Some(digest.algorithm.as_str().to_string()),
            expected: Some(digest.qualified()),
            actual: None,
            path,
            message: Some(message.into()),
        }
    }

    pub fn untrusted_key(
        path: Option<PathBuf>,
        digest: &Digest,
        message: impl Into<String>,
    ) -> Self {
        Self {
            ok: false,
            status: CheckStatus::UntrustedKey,
            algorithm: Some(digest.algorithm.as_str().to_string()),
            expected: Some(digest.qualified()),
            actual: None,
            path,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct BatchCheckResult {
    pub ok: bool,
    pub summary: BatchSummary,
    pub findings: Vec<CheckResult>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct BatchSummary {
    pub checked: usize,
    pub ok: usize,
    pub mismatch: usize,
    pub missing_seal: usize,
    pub invalid_format: usize,
    pub io_error: usize,
    pub missing_signature: usize,
    pub bad_signature: usize,
    pub untrusted_key: usize,
}

impl BatchCheckResult {
    pub fn from_results(results: Vec<CheckResult>) -> Self {
        let mut summary = BatchSummary {
            checked: results.len(),
            ..Default::default()
        };
        for r in &results {
            match r.status {
                CheckStatus::Valid => summary.ok += 1,
                CheckStatus::Mismatch => summary.mismatch += 1,
                CheckStatus::MissingSeal => summary.missing_seal += 1,
                CheckStatus::InvalidFormat => summary.invalid_format += 1,
                CheckStatus::IoError => summary.io_error += 1,
                CheckStatus::MissingSignature => summary.missing_signature += 1,
                CheckStatus::BadSignature => summary.bad_signature += 1,
                CheckStatus::UntrustedKey => summary.untrusted_key += 1,
            }
        }
        let findings: Vec<_> = results.into_iter().filter(|r| !r.ok).collect();
        let ok = findings.is_empty();
        Self {
            ok,
            summary,
            findings,
        }
    }
}
