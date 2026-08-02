//! HashSeal WASM / embedded verify API.
//!
//! Copyright (c) 2026 MonkeyKing.dev

use hashseal_core::instruct::{check_instruct_bytes, InstructOptions};
use hashseal_core::result::CheckResult;

/// Check instruct markdown text; returns structured result (findings-capable).
pub fn check_text(text: &str) -> CheckResult {
    check_instruct_bytes(text, &InstructOptions::default())
}

/// Library version.
pub fn version() -> &'static str {
    hashseal_core::VERSION
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
    }
}
