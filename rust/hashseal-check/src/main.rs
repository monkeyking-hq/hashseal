//! Tiny HashSeal check — no clap, no walkdir; core `check` feature only.
//! Copyright (c) 2026 MonkeyKing.dev

use hashseal_core::gpg::GpgConfig;
use hashseal_core::instruct::{check_instruct_path_with, CheckOpts, InstructOptions};
use hashseal_core::result::BatchCheckResult;
use hashseal_core::walk::{any_glob_str, walk_files};
use hashseal_core::{CheckStatus, DEFAULT_DOCUMENT_INCLUDES, VERSION};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut no_fail = false;
    let mut require_signature = false;
    let mut root = PathBuf::from(".");

    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("hashseal-check {VERSION}");
                return ExitCode::SUCCESS;
            }
            "--no-fail" => no_fail = true,
            "--require-signature" => require_signature = true,
            "--root" => {
                if let Some(r) = args.next() {
                    root = PathBuf::from(r);
                }
            }
            p if p.starts_with('-') => {
                eprintln!("hashseal-check: unknown flag {p}");
                return ExitCode::from(2);
            }
            p => paths.push(PathBuf::from(p)),
        }
    }

    let opts = CheckOpts {
        instruct: InstructOptions::default(),
        require_signature,
        verify_signature: true,
        gpg: GpgConfig::from_git(),
    };

    let files = if paths.is_empty() {
        walk_files(&root, |rel| any_glob_str(DEFAULT_DOCUMENT_INCLUDES, rel)).unwrap_or_default()
    } else {
        paths
            .into_iter()
            .map(|p| if p.is_absolute() { p } else { root.join(p) })
            .collect()
    };

    let mut results = Vec::new();
    for p in files {
        results.push(check_instruct_path_with(&p, &opts));
    }
    let batch = BatchCheckResult::from_results(results);

    if batch.ok {
        println!("HashSeal check passed: {} file(s) ok", batch.summary.ok);
    } else {
        println!("HashSeal check failed: {} issue(s)\n", batch.findings.len());
        for f in &batch.findings {
            let path = f
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let status = match f.status {
                CheckStatus::Mismatch => "MISMATCH",
                CheckStatus::MissingSeal => "MISSING_SEAL",
                CheckStatus::InvalidFormat => "INVALID_FORMAT",
                CheckStatus::IoError => "IO_ERROR",
                CheckStatus::MissingSignature => "MISSING_SIGNATURE",
                CheckStatus::BadSignature => "BAD_SIGNATURE",
                CheckStatus::UntrustedKey => "UNTRUSTED_KEY",
                CheckStatus::Valid => "OK",
            };
            println!("  {status}  {path}");
            if let (Some(e), Some(a)) = (&f.expected, &f.actual) {
                if e != a {
                    println!("            expected: {e}");
                    println!("            actual:   {a}");
                }
            } else if let Some(msg) = &f.message {
                println!("            {msg}");
            }
            println!();
        }
    }

    if batch.ok || no_fail {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn print_help() {
    println!(
        "hashseal-check {VERSION} — tiny instruct verify (blake3 + std)\n\n\
         Usage: hashseal-check [OPTIONS] [PATH]...\n\n\
         Options:\n\
           --root DIR              Root to scan for default instruct globs (default .)\n\
           --no-fail               Exit 0 even on mismatch\n\
           --require-signature     Require hashseal_sig (GPG)\n\
           -h, --help              Help\n\
           -V, --version           Version\n\n\
         With no PATH args, scans agent instruction files (AGENTS.md, CLAUDE.md,\n\
         Copilot/Cursor/skill dirs, …) — same defaults as hashseal check."
    );
}
