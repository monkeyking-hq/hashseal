//! HashSeal WASM / embedded verify API (`wasm-bindgen`).
//!
//! Copyright (c) 2026 Nathan A. Chadwick

use hashseal_core::instruct::{check_instruct_bytes, InstructOptions};
use hashseal_core::result::CheckResult;
use hashseal_core::CheckStatus;
use wasm_bindgen::prelude::*;

/// Check instruct markdown text; returns structured result (Rust API).
pub fn check_text(text: &str) -> CheckResult {
    check_instruct_bytes(text, &InstructOptions::default())
}

fn status_str(s: CheckStatus) -> &'static str {
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

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn opt_json(v: Option<&str>) -> String {
    match v {
        Some(s) => json_escape(s),
        None => "null".into(),
    }
}

/// Check instruct markdown text and return JSON string (WASM export).
#[wasm_bindgen]
pub fn check_text_json(text: &str) -> String {
    let r = check_text(text);
    format!(
        "{{\"ok\":{},\"status\":{},\"algorithm\":{},\"expected\":{},\"actual\":{},\"message\":{}}}",
        r.ok,
        json_escape(status_str(r.status)),
        opt_json(r.algorithm.as_deref()),
        opt_json(r.expected.as_deref()),
        opt_json(r.actual.as_deref()),
        opt_json(r.message.as_deref()),
    )
}

/// Library version (WASM export).
#[wasm_bindgen]
pub fn version() -> String {
    hashseal_core::VERSION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashseal_core::instruct::seal_instruct_bytes;

    #[test]
    fn check_after_seal() {
        let opts = InstructOptions::default();
        let (sealed, _) = seal_instruct_bytes("# hi\n", &opts).unwrap();
        let r = check_text(&sealed);
        assert!(r.ok);
        let j = check_text_json(&sealed);
        assert!(j.contains("\"ok\":true"));
        assert!(j.contains("\"status\":\"valid\""));
    }

    #[test]
    fn check_missing_seal_and_mismatch() {
        let missing = check_text_json("# no seal here\n");
        assert!(missing.contains("\"ok\":false"));
        assert!(missing.contains("\"status\":\"missing_seal\""));

        let opts = InstructOptions::default();
        let (sealed, _) = seal_instruct_bytes("# original\n", &opts).unwrap();
        let tampered = sealed.replace("# original", "# tampered");
        let mismatch = check_text_json(&tampered);
        assert!(mismatch.contains("\"ok\":false"));
        assert!(mismatch.contains("\"status\":\"mismatch\""));
        assert!(mismatch.contains("\"expected\":\"blake3:"));
        assert!(mismatch.contains("\"actual\":\"blake3:"));
    }

    #[test]
    fn check_suspicious_unicode_and_json_escaping() {
        let bidi = check_text_json("# hello \u{202e}world\n");
        assert!(bidi.contains("\"ok\":false"));
        assert!(bidi.contains("\"status\":\"suspicious_unicode\""));

        let escaped = json_escape("a\"b\\c\nd\re\tf");
        assert_eq!(escaped, "\"a\\\"b\\\\c\\nd\\re\\tf\"");
    }

    #[test]
    fn version_matches_pkg_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
