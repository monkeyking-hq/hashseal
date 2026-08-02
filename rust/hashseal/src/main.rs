//! HashSeal CLI — Signed, Sealed, Delivered - I'm Yours.
//! Copyright (c) 2026 MonkeyKing.dev

use clap::{Parser, Subcommand, ValueEnum};
use hashseal_core::bundle::{digest_artifacts, write_bundle, BundlePaths};
use hashseal_core::config::HashSealConfig;
use hashseal_core::gpg::GpgConfig;
use hashseal_core::instruct::{
    check_instruct_path_with, seal_instruct_path_with, unseal_instruct_bytes, CheckOpts,
    InstructOptions, SealOpts,
};
use hashseal_core::report::{write_report, ReportBuilder};
use hashseal_core::result::{BatchCheckResult, CheckResult};
use hashseal_core::tree::{
    clean_tree_artifacts, read_ledger, seal_tree, verify_tree, write_ledger, TreeSealOptions,
};
use hashseal_core::{CheckStatus, VERSION};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
#[derive(Parser, Debug)]
#[command(
    name = "hashseal",
    version = VERSION,
    about = "HashSeal — Signed, Sealed, Delivered - I'm Yours."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Version,
    /// Seal tree and/or instruct files; optional release bundle
    Seal {
        #[arg(long)]
        instruct: bool,
        /// Tree ledger seal (default if neither --instruct nor --release alone)
        #[arg(long)]
        tree: bool,
        /// Write integrity bundle + optional artifact digests
        #[arg(long)]
        release: bool,
        #[arg(long)]
        sign: bool,
        #[arg(long)]
        signing_key: Option<String>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        overlay: Option<PathBuf>,
        #[arg(long)]
        ledger: Option<PathBuf>,
        /// Artifact paths to digest into the release bundle
        #[arg(long = "artifact")]
        artifacts: Vec<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Verify tree ledger and/or instruct files
    Verify {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        ledger: Option<PathBuf>,
        /// Integrity bundle directory
        #[arg(long)]
        bundle: Option<PathBuf>,
        #[arg(long)]
        no_fail: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Fast instruct-file check
    Check {
        paths: Vec<PathBuf>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
        #[arg(long)]
        no_fail: bool,
        #[arg(long)]
        require_signature: bool,
    },
    Unseal {
        #[arg(long)]
        instruct: bool,
        paths: Vec<PathBuf>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Remove ledger/report/bundle artifacts
    Clean {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        ledger: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("hashseal: {e}");
            ExitCode::from(3)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Version => {
            println!("hashseal {VERSION}");
            println!("core {}", hashseal_core::VERSION);
            println!("Signed, Sealed, Delivered - I'm Yours.");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Seal {
            instruct,
            tree,
            release,
            sign,
            signing_key,
            root,
            config,
            overlay,
            ledger,
            artifacts,
            format,
        } => {
            let cfg = load_config(&root, config.as_deref(), overlay.as_deref())?;
            let do_instruct = instruct || (!tree && !release && cfg.document.enable);
            let do_tree = tree || release || (!instruct && !release);
            // if only --instruct, skip tree; if only --tree, skip instruct unless document.enable and not exclusive
            let do_instruct = if instruct && !tree && !release {
                true
            } else if tree && !instruct && !release {
                false
            } else {
                do_instruct
            };
            let do_tree = if instruct && !tree && !release {
                false
            } else {
                do_tree || release
            };

            if do_instruct {
                run_seal_instruct(&root, &cfg, sign, signing_key.clone(), format)?;
            }
            if do_tree || release {
                run_seal_tree(&root, &cfg, ledger.as_deref(), release, &artifacts, format)?;
            }
            if !do_instruct && !do_tree && !release {
                return Err("nothing to seal; use --instruct, --tree, and/or --release".into());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Verify {
            root,
            config,
            ledger,
            bundle,
            no_fail,
            format,
        } => run_verify(
            &root,
            config.as_deref(),
            ledger.as_deref(),
            bundle.as_deref(),
            no_fail,
            format,
        ),
        Commands::Check {
            paths,
            root,
            config,
            format,
            no_fail,
            require_signature,
        } => {
            let cfg = load_config(&root, config.as_deref(), None)?;
            run_check(
                &root,
                &paths,
                &cfg,
                format,
                no_fail,
                require_signature || cfg.signing.require,
            )
        }
        Commands::Unseal {
            instruct,
            paths,
            root,
        } => {
            if !instruct {
                return Err("use --instruct with unseal".into());
            }
            run_unseal(&root, &paths)
        }
        Commands::Clean { root, ledger } => {
            let cfg = load_config(&root, None, None)?;
            let ledger_path = ledger.unwrap_or_else(|| cfg.default_ledger_path(&root));
            let report_path = cfg.default_report_path(&root);
            let bundle = BundlePaths::under(&root);
            clean_tree_artifacts(&[
                ledger_path,
                report_path,
                bundle.ledger,
                bundle.report,
                bundle.manifest,
            ])?;
            // remove bundle dir if empty
            let _ = fs::remove_dir(bundle.dir);
            println!("HashSeal: cleaned ledger/report/bundle artifacts");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn load_config(
    root: &Path,
    config: Option<&Path>,
    overlay: Option<&Path>,
) -> Result<HashSealConfig, Box<dyn std::error::Error>> {
    let mut cfg = if let Some(p) = config {
        HashSealConfig::load_json_path(p)?
    } else {
        HashSealConfig::load_for_root(root)?
    };
    if let Some(o) = overlay {
        let text = fs::read_to_string(o)?;
        let v: serde_json::Value = serde_json::from_str(&text)?;
        cfg.merge_overlay_json(&v)?;
    }
    Ok(cfg)
}

fn gpg_from(cfg: &HashSealConfig, signing_key: Option<String>) -> GpgConfig {
    let mut gpg = GpgConfig::from_git();
    if let Some(p) = &cfg.signing.gpg_program {
        gpg.program = PathBuf::from(p);
    }
    if let Some(k) = signing_key.or_else(|| cfg.signing.signing_key.clone()) {
        gpg.signing_key = Some(k);
    }
    gpg
}

fn collect_md_files(
    root: &Path,
    paths: &[PathBuf],
    include: &[String],
    exclude: &[String],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if !paths.is_empty() {
        return Ok(paths
            .iter()
            .map(|p| if p.is_absolute() { p.clone() } else { root.join(p) })
            .collect());
    }
    // reuse tree collector with md-focused include
    let opts = TreeSealOptions {
        include: if include.is_empty() {
            vec!["**/*.md".into()]
        } else {
            include.to_vec()
        },
        exclude: exclude.to_vec(),
        ..TreeSealOptions::default()
    };
    Ok(hashseal_core::collect_tree_files(root, &opts)?)
}

fn run_seal_instruct(
    root: &Path,
    cfg: &HashSealConfig,
    sign: bool,
    signing_key: Option<String>,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let sign = sign || cfg.signing.enable;
    let opts = SealOpts {
        instruct: InstructOptions {
            field: cfg.document.field.clone(),
            algorithm: cfg.document_algorithm()?,
            canonical: cfg.document_canonical(),
            auto_frontmatter: cfg.document.auto_frontmatter,
        },
        sign,
        gpg: gpg_from(cfg, signing_key),
    };
    let files = collect_md_files(root, &[], &cfg.document.include, &cfg.document.exclude)?;
    let mut digests = Vec::new();
    for path in &files {
        let d = seal_instruct_path_with(path, &opts)?;
        digests.push((path.clone(), d.qualified()));
    }
    match format {
        OutputFormat::Human => {
            let mode = if sign { "sealed+signed" } else { "sealed" };
            println!("HashSeal: {mode} {} instruct file(s)", digests.len());
            for (p, d) in &digests {
                println!("  {d}  {}", p.display());
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&digests.iter().map(|(p, d)| {
                    serde_json::json!({"path": p, "digest": d, "signed": sign})
                }).collect::<Vec<_>>())?
            );
        }
    }
    Ok(())
}

fn run_seal_tree(
    root: &Path,
    cfg: &HashSealConfig,
    ledger_override: Option<&Path>,
    release: bool,
    artifacts: &[PathBuf],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = TreeSealOptions {
        algorithm: cfg.tree_algorithm()?,
        include: cfg.tree.include.clone(),
        exclude: cfg.tree.exclude.clone(),
        line_endings_lf_text: cfg.tree.line_endings == "lf-text",
    };
    let ledger = seal_tree(root, &opts)?;
    let ledger_path = ledger_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| cfg.default_ledger_path(root));

    let builder = ReportBuilder {
        algorithm: Some(opts.algorithm.as_str().to_string()),
        include_hostname: cfg.report.include_hostname,
        include_ci: cfg.report.include_ci,
        max_findings: cfg.report.max_findings,
        mode: cfg.enforce.on_tree_mismatch.clone(),
        ..ReportBuilder::new(
            if release { "seal-release" } else { "seal" },
            root.display().to_string(),
        )
    };
    let report = builder.finish_tree(ledger.entries.len(), None, 0);

    if release || ledger_path.starts_with(root.join("hashseal-bundle")) {
        let arts = if artifacts.is_empty() {
            vec![]
        } else {
            digest_artifacts(artifacts, opts.algorithm)?
        };
        let paths = write_bundle(root, &ledger, &report, &arts)?;
        if format == OutputFormat::Human {
            println!(
                "HashSeal: tree sealed {} file(s) → {}",
                ledger.entries.len(),
                paths.dir.display()
            );
            for a in &arts {
                println!("  artifact {}  {}", a.digest, a.name);
            }
        } else {
            println!(
                "{}",
                serde_json::json!({
                    "entries": ledger.entries.len(),
                    "bundle": paths.dir,
                    "artifacts": arts,
                })
            );
        }
    } else {
        write_ledger(&ledger_path, &ledger)?;
        if cfg.report.write {
            write_report(&cfg.default_report_path(root), &report, cfg.report.pretty)?;
        }
        if format == OutputFormat::Human {
            println!(
                "HashSeal: tree sealed {} file(s) → {}",
                ledger.entries.len(),
                ledger_path.display()
            );
        } else {
            println!(
                "{}",
                serde_json::json!({
                    "entries": ledger.entries.len(),
                    "ledger": ledger_path,
                })
            );
        }
    }
    Ok(())
}

fn run_verify(
    root: &Path,
    config: Option<&Path>,
    ledger: Option<&Path>,
    bundle: Option<&Path>,
    no_fail: bool,
    format: OutputFormat,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let cfg = load_config(root, config, None)?;
    let ledger_path = if let Some(b) = bundle {
        b.join("ledger.json")
    } else if let Some(l) = ledger {
        l.to_path_buf()
    } else {
        let bp = BundlePaths::under(root);
        if bp.ledger.is_file() {
            bp.ledger
        } else {
            cfg.default_ledger_path(root)
        }
    };

    if !ledger_path.is_file() {
        eprintln!("hashseal: missing ledger at {}", ledger_path.display());
        if cfg.enforce.on_missing_ledger == "warn" || no_fail {
            return Ok(ExitCode::SUCCESS);
        }
        return Ok(ExitCode::from(1));
    }

    let led = read_ledger(&ledger_path)?;
    let opts = TreeSealOptions {
        algorithm: cfg.tree_algorithm()?,
        include: cfg.tree.include.clone(),
        exclude: cfg.tree.exclude.clone(),
        line_endings_lf_text: cfg.tree.line_endings == "lf-text",
    };
    let result = verify_tree(root, &led, &opts)?;
    let exit = if result.ok {
        0
    } else if no_fail || cfg.enforce.on_tree_mismatch == "warn" {
        0
    } else {
        1
    };

    let builder = ReportBuilder {
        algorithm: Some(led.algorithm.clone()),
        include_hostname: cfg.report.include_hostname,
        include_ci: cfg.report.include_ci,
        max_findings: cfg.report.max_findings,
        mode: cfg.enforce.on_tree_mismatch.clone(),
        ..ReportBuilder::new("verify", root.display().to_string())
    };
    let report = builder.finish_tree(0, Some(&result), exit);
    if cfg.report.write {
        let report_path = if let Some(b) = bundle {
            b.join("report.json")
        } else {
            cfg.default_report_path(root)
        };
        write_report(&report_path, &report, cfg.report.pretty)?;
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Human => {
            if result.ok {
                println!(
                    "HashSeal verify passed: {} ledger path(s) ok",
                    result.checked
                );
            } else {
                println!(
                    "HashSeal verify failed: {} issue(s)\n",
                    result.findings.len()
                );
                for f in &result.findings {
                    let status = format!("{:?}", f.status).to_ascii_uppercase();
                    println!("  {status}  {}", f.path);
                    if let (Some(e), Some(a)) = (&f.expected, &f.actual) {
                        if e != a {
                            println!("            expected: {e}");
                            println!("            actual:   {a}");
                        }
                    }
                    println!();
                }
            }
        }
    }
    Ok(ExitCode::from(exit as u8))
}

fn run_check(
    root: &Path,
    paths: &[PathBuf],
    cfg: &HashSealConfig,
    format: OutputFormat,
    no_fail: bool,
    require_signature: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = CheckOpts {
        instruct: InstructOptions {
            field: cfg.document.field.clone(),
            algorithm: cfg.document_algorithm()?,
            canonical: cfg.document_canonical(),
            auto_frontmatter: cfg.document.auto_frontmatter,
        },
        require_signature,
        verify_signature: true,
        gpg: gpg_from(cfg, None),
    };
    let files = collect_md_files(root, paths, &cfg.document.include, &cfg.document.exclude)?;
    let mut results = Vec::new();
    for path in &files {
        results.push(check_instruct_path_with(path, &opts));
    }
    let batch = BatchCheckResult::from_results(results.clone());
    let exit = if batch.ok {
        0
    } else if no_fail || cfg.enforce.on_doc_mismatch == "warn" {
        0
    } else {
        1
    };

    if cfg.report.write {
        let builder = ReportBuilder {
            algorithm: Some(cfg.document.algorithm.clone()),
            include_hostname: cfg.report.include_hostname,
            include_ci: cfg.report.include_ci,
            max_findings: cfg.report.max_findings,
            mode: cfg.enforce.on_doc_mismatch.clone(),
            ..ReportBuilder::new("check", root.display().to_string())
        };
        let report = builder.finish_instruct(&batch, &results, exit);
        write_report(&cfg.default_report_path(root), &report, cfg.report.pretty)?;
    }

    print_check_output(&batch, format);
    Ok(ExitCode::from(exit as u8))
}

fn print_check_output(batch: &BatchCheckResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(batch).unwrap());
        }
        OutputFormat::Human => {
            if batch.ok {
                println!("HashSeal check passed: {} file(s) ok", batch.summary.ok);
            } else {
                println!("HashSeal check failed: {} issue(s)\n", batch.findings.len());
                for f in &batch.findings {
                    print_finding_human(f);
                }
            }
        }
    }
}

fn print_finding_human(f: &CheckResult) {
    let path = f
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<bytes>".into());
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
    if let (Some(exp), Some(act)) = (&f.expected, &f.actual) {
        if exp != act {
            println!("            expected: {exp}");
            println!("            actual:   {act}");
        }
    } else if let Some(msg) = &f.message {
        println!("            {msg}");
    }
    println!();
}

fn run_unseal(root: &Path, paths: &[PathBuf]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = InstructOptions::default();
    let files = if paths.is_empty() {
        collect_md_files(root, &[], &["**/*.md".into()], &TreeSealOptions::default().exclude)?
    } else {
        paths
            .iter()
            .map(|p| if p.is_absolute() { p.clone() } else { root.join(p) })
            .collect()
    };
    for path in files {
        let text = fs::read_to_string(&path)?;
        let out = unseal_instruct_bytes(&text, &opts);
        fs::write(&path, out)?;
        println!("unsealed {}", path.display());
    }
    Ok(ExitCode::SUCCESS)
}
