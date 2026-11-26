//! Suspicious invisible/BiDi Unicode detector for instruction files (std-only).
//!
//! Catches hidden prompt-injection / Trojan-Source characters such as:
//! - Unicode Tag block (`U+E0000..=U+E007F`)
//! - BiDi override/isolate controls (`U+202A..=U+202E`, `U+2066..=U+2069`, `U+061C`, `U+200E`, `U+200F`)
//! - Zero-width / invisible formatting chars (`U+200B`, `U+2060..=U+2064`)
//! - Mid-file BOM (`U+FEFF` at char index > 0)
//!
//! Intentionally allows `U+200D` (ZWJ) and `U+FE0F` so normal emoji do not false-positive.

use crate::error::{Error, Result};
use std::str::FromStr;

/// Policy for handling suspicious Unicode in instruct files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnicodePolicy {
    #[default]
    Fail,
    Warn,
    Off,
}

impl FromStr for UnicodePolicy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "fail" => Ok(UnicodePolicy::Fail),
            "warn" => Ok(UnicodePolicy::Warn),
            "off" => Ok(UnicodePolicy::Off),
            other => Err(Error::InvalidFormat(format!(
                "invalid unicode_policy '{other}' (expected fail, warn, or off)"
            ))),
        }
    }
}

/// A single suspicious Unicode character occurrence (1-based line and column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeFinding {
    pub line: usize,
    pub col: usize,
    pub codepoint: u32,
    pub kind: &'static str,
}

/// Scan `text` for suspicious invisible or BiDi-control Unicode characters.
pub fn scan_suspicious_unicode(text: &str) -> Vec<UnicodeFinding> {
    let mut findings = Vec::new();
    let mut line = 1usize;
    let mut col = 1usize;

    for (idx, ch) in text.chars().enumerate() {
        let cp = ch as u32;
        let kind = match cp {
            0xE0000..=0xE007F => Some("unicode_tag"),
            0x202A..=0x202E | 0x2066..=0x2069 | 0x061C | 0x200E | 0x200F => Some("bidi_control"),
            0x200B | 0x2060 | 0x2061..=0x2064 => Some("zero_width"),
            0xFEFF if idx > 0 => Some("embedded_bom"),
            _ => None,
        };
        if let Some(kind) = kind {
            findings.push(UnicodeFinding {
                line,
                col,
                codepoint: cp,
                kind,
            });
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    findings
}

/// Format suspicious Unicode findings into a concise human-readable message.
pub fn format_unicode_findings(findings: &[UnicodeFinding]) -> String {
    let Some(first) = findings.first() else {
        return String::new();
    };
    let head = format!(
        "U+{:04X} ({}) at {}:{}",
        first.codepoint, first.kind, first.line, first.col
    );
    if findings.len() > 1 {
        format!("{head} (+{} more)", findings.len() - 1)
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_utf8_and_emoji_allowed() {
        let text = "# Rules\n\nUse Rust 🦀 and family emoji 👨‍👩‍👧‍👦 safely.\n";
        assert!(scan_suspicious_unicode(text).is_empty());
    }

    #[test]
    fn leading_bom_allowed_mid_file_bom_flagged() {
        let leading = "\u{FEFF}# Clean\n";
        assert!(scan_suspicious_unicode(leading).is_empty());

        let embedded = "# Header\n\u{FEFF}hidden\n";
        let f = scan_suspicious_unicode(embedded);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, "embedded_bom");
        assert_eq!(f[0].line, 2);
        assert_eq!(f[0].col, 1);
    }

    #[test]
    fn bidi_rlo_flagged() {
        let text = "ok\nrun \u{202E}evil\n";
        let f = scan_suspicious_unicode(text);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, "bidi_control");
        assert_eq!(f[0].codepoint, 0x202E);
        assert_eq!(f[0].line, 2);
        assert_eq!(f[0].col, 5);
        assert_eq!(format_unicode_findings(&f), "U+202E (bidi_control) at 2:5");
    }

    #[test]
    fn unicode_tag_flagged() {
        let text = "hello\u{E0041}\u{E0042}";
        let f = scan_suspicious_unicode(text);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].kind, "unicode_tag");
        assert_eq!(
            format_unicode_findings(&f),
            "U+E0041 (unicode_tag) at 1:6 (+1 more)"
        );
    }

    #[test]
    fn zwsp_flagged() {
        let text = "do\u{200B}something";
        let f = scan_suspicious_unicode(text);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, "zero_width");
        assert_eq!(f[0].codepoint, 0x200B);
    }
}
