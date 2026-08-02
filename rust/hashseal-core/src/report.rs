//! Enterprise audit report (JSON when `json` feature enabled).

use crate::result::{BatchCheckResult, CheckResult};
use crate::timeutil::{bundle_id, unix_ms, utc_now_rfc3339};
use crate::tree::TreeVerifyResult;
use std::env;
use std::path::Path;

#[cfg(feature = "json")]
use std::fs;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct AuditReport {
    pub report_schema_version: u32,
    pub tool_name: String,
    pub tool_version: String,
    pub command: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub root: String,
    pub algorithm: Option<String>,
    pub mode: String,
    pub result: String,
    pub exit_code: i32,
    pub bundle_id: String,
    pub hostname: Option<String>,
    pub ci: Option<CiInfo>,
    pub counts: ReportCounts,
    #[cfg(feature = "json")]
    pub findings: Vec<serde_json::Value>,
    #[cfg(not(feature = "json"))]
    pub findings_count: usize,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct ReportCounts {
    pub files_sealed: usize,
    pub files_checked: usize,
    pub ok: usize,
    pub mismatch: usize,
    pub added: usize,
    pub removed: usize,
    pub missing_seal: usize,
    pub missing_signature: usize,
    pub bad_signature: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct CiInfo {
    pub system: Option<String>,
    pub pipeline_id: Option<String>,
    pub job_id: Option<String>,
    pub commit: Option<String>,
    pub ref_name: Option<String>,
}

pub fn detect_ci() -> Option<CiInfo> {
    if env::var_os("CI").is_none() && env::var_os("GITHUB_ACTIONS").is_none() {
        return None;
    }
    let system = if env::var_os("GITHUB_ACTIONS").is_some() {
        Some("github_actions".into())
    } else {
        Some("ci".into())
    };
    Some(CiInfo {
        system,
        pipeline_id: env::var("GITHUB_RUN_ID").ok(),
        job_id: env::var("GITHUB_JOB").ok(),
        commit: env::var("GITHUB_SHA").ok(),
        ref_name: env::var("GITHUB_REF").ok(),
    })
}

pub struct ReportBuilder {
    pub command: String,
    pub root: String,
    pub algorithm: Option<String>,
    pub mode: String,
    pub started_ms: u128,
    pub started_at: String,
    pub include_hostname: bool,
    pub include_ci: bool,
    pub max_findings: usize,
}

impl ReportBuilder {
    pub fn new(command: impl Into<String>, root: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            root: root.into(),
            algorithm: None,
            mode: "enforce".into(),
            started_ms: unix_ms(),
            started_at: utc_now_rfc3339(),
            include_hostname: false,
            include_ci: true,
            max_findings: 10_000,
        }
    }

    pub fn finish_instruct(
        self,
        batch: &BatchCheckResult,
        _all: &[CheckResult],
        exit_code: i32,
    ) -> AuditReport {
        let finished_at = utc_now_rfc3339();
        let duration_ms = unix_ms().saturating_sub(self.started_ms) as u64;
        #[cfg(feature = "json")]
        let findings: Vec<serde_json::Value> = batch
            .findings
            .iter()
            .take(self.max_findings)
            .filter_map(|f| serde_json::to_value(f).ok())
            .collect();
        AuditReport {
            report_schema_version: REPORT_SCHEMA_VERSION,
            tool_name: "hashseal".into(),
            tool_version: crate::VERSION.to_string(),
            command: self.command,
            started_at: self.started_at,
            finished_at,
            duration_ms,
            root: self.root,
            algorithm: self.algorithm,
            mode: self.mode,
            result: if batch.ok {
                "pass".into()
            } else if exit_code == 0 {
                "warn".into()
            } else {
                "fail".into()
            },
            exit_code,
            bundle_id: bundle_id(),
            hostname: if self.include_hostname {
                env::var("COMPUTERNAME")
                    .or_else(|_| env::var("HOSTNAME"))
                    .ok()
            } else {
                None
            },
            ci: if self.include_ci { detect_ci() } else { None },
            counts: ReportCounts {
                files_checked: batch.summary.checked,
                ok: batch.summary.ok,
                mismatch: batch.summary.mismatch,
                missing_seal: batch.summary.missing_seal,
                missing_signature: batch.summary.missing_signature,
                bad_signature: batch.summary.bad_signature,
                ..Default::default()
            },
            #[cfg(feature = "json")]
            findings,
            #[cfg(not(feature = "json"))]
            findings_count: batch.findings.len(),
        }
    }

    pub fn finish_tree(
        self,
        sealed_count: usize,
        verify: Option<&TreeVerifyResult>,
        exit_code: i32,
    ) -> AuditReport {
        let finished_at = utc_now_rfc3339();
        let duration_ms = unix_ms().saturating_sub(self.started_ms) as u64;
        let (ok, checked) = if let Some(v) = verify {
            (v.ok, v.checked)
        } else {
            (true, sealed_count)
        };
        let mut mismatch = 0;
        let mut added = 0;
        let mut removed = 0;
        #[cfg(feature = "json")]
        let findings: Vec<serde_json::Value> = if let Some(v) = verify {
            for f in &v.findings {
                match f.status {
                    crate::tree::TreeEntryStatus::Mismatch => mismatch += 1,
                    crate::tree::TreeEntryStatus::Added => added += 1,
                    crate::tree::TreeEntryStatus::Removed => removed += 1,
                    _ => {}
                }
            }
            v.findings
                .iter()
                .take(self.max_findings)
                .filter_map(|f| serde_json::to_value(f).ok())
                .collect()
        } else {
            Vec::new()
        };
        #[cfg(not(feature = "json"))]
        let findings_count = if let Some(v) = verify {
            for f in &v.findings {
                match f.status {
                    crate::tree::TreeEntryStatus::Mismatch => mismatch += 1,
                    crate::tree::TreeEntryStatus::Added => added += 1,
                    crate::tree::TreeEntryStatus::Removed => removed += 1,
                    _ => {}
                }
            }
            v.findings.len()
        } else {
            0
        };
        AuditReport {
            report_schema_version: REPORT_SCHEMA_VERSION,
            tool_name: "hashseal".into(),
            tool_version: crate::VERSION.to_string(),
            command: self.command,
            started_at: self.started_at,
            finished_at,
            duration_ms,
            root: self.root,
            algorithm: self.algorithm,
            mode: self.mode,
            result: if ok {
                "pass".into()
            } else if exit_code == 0 {
                "warn".into()
            } else {
                "fail".into()
            },
            exit_code,
            bundle_id: bundle_id(),
            hostname: if self.include_hostname {
                env::var("COMPUTERNAME")
                    .or_else(|_| env::var("HOSTNAME"))
                    .ok()
            } else {
                None
            },
            ci: if self.include_ci { detect_ci() } else { None },
            counts: ReportCounts {
                files_sealed: sealed_count,
                files_checked: checked,
                ok: checked.saturating_sub(mismatch + added + removed),
                mismatch,
                added,
                removed,
                ..Default::default()
            },
            #[cfg(feature = "json")]
            findings,
            #[cfg(not(feature = "json"))]
            findings_count,
        }
    }
}

#[cfg(feature = "json")]
pub fn write_report(path: &Path, report: &AuditReport, pretty: bool) -> crate::error::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    }
    .map_err(|e| crate::error::Error::InvalidFormat(format!("report json: {e}")))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(not(feature = "json"))]
pub fn write_report(
    _path: &Path,
    _report: &AuditReport,
    _pretty: bool,
) -> crate::error::Result<()> {
    Ok(())
}
