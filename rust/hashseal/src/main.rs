//! HashSeal CLI — Signed, Sealed, Delivered - I'm Yours.
//! Copyright (c) 2026 Nathan A. Chadwick

use clap::{Parser, Subcommand, ValueEnum};
use hashseal_core::bundle::{
    digest_artifacts, sign_bundle, verify_bundle_with_signing, write_bundle, BundlePaths,
};
use hashseal_core::config::HashSealConfig;
use hashseal_core::instruct::{
    check_instruct_path_with, seal_instruct_path_with, unseal_instruct_bytes, CheckOpts,
    InstructOptions, SealOpts,
};
use hashseal_core::lockfile::{
    build_lockfile, sign_lockfile, verify_lockfile, write_lockfile, LOCKFILE_NAME,
};
use hashseal_core::report::{write_report, ReportBuilder};
use hashseal_core::result::{BatchCheckResult, CheckResult};
use hashseal_core::signing::{SignFormat, SigningOptions};
use hashseal_core::tree::{
    clean_tree_artifacts, read_ledger, seal_tree, verify_tree, write_ledger, TreeSealOptions,
};
use hashseal_core::walk::{any_glob, is_skipped_dir_name};
use hashseal_core::{CheckStatus, UnicodePolicy, VERSION};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

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
    /// Seal tree and/or instruct files; optional release bundle and lockfile
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
        #[arg(long, value_enum)]
        sign_format: Option<SignFormatArg>,
        #[arg(long)]
        signing_key: Option<String>,
        /// Bind signature to scope='*' for portable skill/instruction packs
        #[arg(long)]
        portable: bool,
        /// Write .hashseal.lock covering sealed instruct files
        #[arg(long)]
        lock: bool,
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
        #[arg(long, value_enum)]
        unicode_policy: Option<UnicodePolicyArg>,
        /// Respect .gitignore files when walking directories (opt-in; default off)
        #[arg(long)]
        gitignore: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Verify tree ledger and/or integrity bundle
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
        #[arg(long)]
        require_signature: bool,
        #[arg(long)]
        digest_only: bool,
        #[arg(long)]
        allowed_signers: Option<PathBuf>,
        #[arg(long = "allowed-key")]
        allowed_keys: Vec<String>,
        /// Respect .gitignore files when walking directories (opt-in; default off)
        #[arg(long)]
        gitignore: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Fast instruct-file check (supports lockfile and --diff PR gate)
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
        #[arg(long)]
        digest_only: bool,
        #[arg(long)]
        lock: Option<PathBuf>,
        #[arg(long)]
        no_lock: bool,
        #[arg(long)]
        allowed_signers: Option<PathBuf>,
        #[arg(long = "allowed-key")]
        allowed_keys: Vec<String>,
        /// Check only instruct files changed vs git base ref (e.g. origin/main)
        #[arg(long)]
        diff: Option<String>,
        #[arg(long, value_enum)]
        unicode_policy: Option<UnicodePolicyArg>,
        /// Respect .gitignore files when walking directories (opt-in; default off)
        #[arg(long)]
        gitignore: bool,
    },
    Unseal {
        #[arg(long)]
        instruct: bool,
        paths: Vec<PathBuf>,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Respect .gitignore files when walking directories (opt-in; default off)
        #[arg(long)]
        gitignore: bool,
    },
    /// Remove ledger/report/bundle artifacts
    Clean {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        ledger: Option<PathBuf>,
    },
    /// Initialize HashSeal config, hooks, and lockfile in a project
    Init {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        git_hooks: bool,
        #[arg(long)]
        in_git_dir: bool,
        #[arg(long)]
        claude_hooks: bool,
        #[arg(long)]
        lock: bool,
        #[arg(long)]
        sign: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum UnicodePolicyArg {
    Fail,
    Warn,
    Off,
}

impl From<UnicodePolicyArg> for UnicodePolicy {
    fn from(a: UnicodePolicyArg) -> Self {
        match a {
            UnicodePolicyArg::Fail => UnicodePolicy::Fail,
            UnicodePolicyArg::Warn => UnicodePolicy::Warn,
            UnicodePolicyArg::Off => UnicodePolicy::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SignFormatArg {
    Auto,
    Ssh,
    Gpg,
}

impl From<SignFormatArg> for SignFormat {
    fn from(a: SignFormatArg) -> Self {
        match a {
            SignFormatArg::Auto => SignFormat::Auto,
            SignFormatArg::Ssh => SignFormat::Ssh,
            SignFormatArg::Gpg => SignFormat::Gpg,
        }
    }
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
            sign_format,
            signing_key,
            portable,
            lock,
            root,
            config,
            overlay,
            ledger,
            artifacts,
            unicode_policy,
            gitignore,
            format,
        } => {
            let cfg = load_config(&root, config.as_deref(), overlay.as_deref())?;
            let respect_gitignore = gitignore || cfg.walk.respect_gitignore;
            let upol = unicode_policy
                .map(Into::into)
                .unwrap_or(cfg.document_unicode_policy()?);
            let s_opts = signing_from(
                &root,
                &cfg,
                sign_format.map(Into::into),
                signing_key,
                None,
                &[],
            )?;

            let do_instruct = instruct || (!tree && !release && cfg.document.enable);
            let do_tree = tree || release || (!instruct && !release);
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
                run_seal_instruct(
                    &root,
                    &cfg,
                    sign,
                    &s_opts,
                    portable,
                    lock,
                    upol,
                    respect_gitignore,
                    format,
                )?;
            }
            if do_tree || release {
                run_seal_tree(
                    &root,
                    &cfg,
                    ledger.as_deref(),
                    release,
                    sign,
                    &s_opts,
                    &artifacts,
                    respect_gitignore,
                    format,
                )?;
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
            require_signature,
            digest_only,
            allowed_signers,
            allowed_keys,
            gitignore,
            format,
        } => {
            let cfg = load_config(&root, config.as_deref(), None)?;
            let respect_gitignore = gitignore || cfg.walk.respect_gitignore;
            let s_opts = signing_from(
                &root,
                &cfg,
                None,
                None,
                allowed_signers.as_deref(),
                &allowed_keys,
            )?;
            run_verify(
                &root,
                &cfg,
                ledger.as_deref(),
                bundle.as_deref(),
                no_fail,
                require_signature || cfg.signing.require,
                digest_only,
                &s_opts,
                respect_gitignore,
                format,
            )
        }
        Commands::Check {
            paths,
            root,
            config,
            format,
            no_fail,
            require_signature,
            digest_only,
            lock,
            no_lock,
            allowed_signers,
            allowed_keys,
            diff,
            unicode_policy,
            gitignore,
        } => {
            let cfg = load_config(&root, config.as_deref(), None)?;
            let respect_gitignore = gitignore || cfg.walk.respect_gitignore;
            let upol = unicode_policy
                .map(Into::into)
                .unwrap_or(cfg.document_unicode_policy()?);
            let s_opts = signing_from(
                &root,
                &cfg,
                None,
                None,
                allowed_signers.as_deref(),
                &allowed_keys,
            )?;
            run_check(
                &root,
                &paths,
                &cfg,
                format,
                no_fail,
                require_signature || cfg.signing.require,
                digest_only,
                lock.as_deref(),
                no_lock,
                diff.as_deref(),
                &s_opts,
                upol,
                respect_gitignore,
            )
        }
        Commands::Unseal {
            instruct,
            paths,
            root,
            gitignore,
        } => {
            if !instruct {
                return Err("use --instruct with unseal".into());
            }
            let cfg = HashSealConfig::load_for_root(&root).unwrap_or_default();
            let respect_gitignore = gitignore || cfg.walk.respect_gitignore;
            run_unseal(&root, &paths, respect_gitignore)
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
                bundle.dir.join("MANIFEST.json.sig"),
            ])?;
            let _ = fs::remove_dir(bundle.dir);
            println!("HashSeal: cleaned ledger/report/bundle artifacts");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Init {
            root,
            git_hooks,
            in_git_dir,
            claude_hooks,
            lock,
            sign,
        } => run_init(&root, git_hooks, in_git_dir, claude_hooks, lock, sign),
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

fn signing_from(
    root: &Path,
    cfg: &HashSealConfig,
    cli_format: Option<SignFormat>,
    cli_key: Option<String>,
    cli_allowed_signers: Option<&Path>,
    cli_allowed_keys: &[String],
) -> Result<SigningOptions, Box<dyn std::error::Error>> {
    let mut opts = SigningOptions::from_git_and_root(Some(root));
    if let Some(fmt_str) = &cfg.signing.format {
        opts.format = fmt_str.parse()?;
    }
    if let Some(fmt) = cli_format {
        opts.format = fmt;
    }
    if let Some(p) = &cfg.signing.gpg_program {
        opts.gpg_program = PathBuf::from(p);
    }
    if let Some(k) = cli_key.or_else(|| cfg.signing.signing_key.clone()) {
        opts.signing_key = Some(k);
    }
    if let Some(s) = cli_allowed_signers {
        opts.allowed_signers = Some(s.to_path_buf());
    } else if let Some(s) = &cfg.signing.allowed_signers {
        let pb = PathBuf::from(s);
        opts.allowed_signers = Some(if pb.is_absolute() { pb } else { root.join(pb) });
    }
    if !cli_allowed_keys.is_empty() {
        opts.allowed_key_ids = cli_allowed_keys.to_vec();
    } else if !cfg.signing.allowed_key_ids.is_empty() {
        opts.allowed_key_ids = cfg.signing.allowed_key_ids.clone();
    }
    Ok(opts)
}

fn rel_posix(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .map(|r| r.to_string_lossy().replace('\\', "/"))
}

fn collect_md_files(
    root: &Path,
    paths: &[PathBuf],
    include: &[String],
    exclude: &[String],
    respect_gitignore: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if !paths.is_empty() {
        return Ok(paths
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    root.join(p)
                }
            })
            .collect());
    }
    let opts = TreeSealOptions {
        include: if include.is_empty() {
            hashseal_core::default_document_includes()
        } else {
            include.to_vec()
        },
        exclude: exclude.to_vec(),
        respect_gitignore,
        ..TreeSealOptions::default()
    };
    Ok(hashseal_core::collect_tree_files(root, &opts)?)
}

fn collect_diff_md_files(
    root: &Path,
    base_ref: &str,
    include: &[String],
    exclude: &[String],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let range = format!("{base_ref}...HEAD");
    let mut out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-only", "--diff-filter=ACMR", &range])
        .output()?;
    if !out.status.success() {
        out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["diff", "--name-only", "--diff-filter=ACMR", base_ref])
            .output()?;
    }
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git diff against '{base_ref}' failed: {}", err.trim()).into());
    }

    let effective_inc = if include.is_empty() {
        hashseal_core::default_document_includes()
    } else {
        include.to_vec()
    };

    let mut matched = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let rel = line.trim().replace('\\', "/");
        if rel.is_empty() {
            continue;
        }
        if rel.split('/').any(is_skipped_dir_name) || any_glob(exclude, &rel) {
            continue;
        }
        if any_glob(&effective_inc, &rel) {
            let full = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
            if full.is_file() {
                matched.push(full);
            }
        }
    }
    matched.sort();
    Ok(matched)
}

#[allow(clippy::too_many_arguments)]
fn run_seal_instruct(
    root: &Path,
    cfg: &HashSealConfig,
    sign: bool,
    signing: &SigningOptions,
    portable: bool,
    write_lock: bool,
    unicode_policy: UnicodePolicy,
    respect_gitignore: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let sign = sign || cfg.signing.enable;
    let instruct_opts = InstructOptions {
        field: cfg.document.field.clone(),
        algorithm: cfg.document_algorithm()?,
        canonical: cfg.document_canonical(),
        auto_frontmatter: cfg.document.auto_frontmatter,
        unicode_policy,
    };
    let files = collect_md_files(
        root,
        &[],
        &cfg.document.include,
        &cfg.document.exclude,
        respect_gitignore,
    )?;
    let mut digests = Vec::new();
    for path in &files {
        let scope = if portable {
            Some("*".to_string())
        } else {
            rel_posix(root, path)
        };
        let opts = SealOpts {
            instruct: instruct_opts.clone(),
            sign,
            signing: signing.clone(),
            scope,
        };
        let d = seal_instruct_path_with(path, &opts)?;
        digests.push((path.clone(), d.qualified()));
    }
    let lock_path = if write_lock {
        let lp = root.join(LOCKFILE_NAME);
        let lock = build_lockfile(root, &files, &instruct_opts)?;
        write_lockfile(&lp, &lock)?;
        if sign {
            sign_lockfile(&lp, signing)?;
        }
        Some(lp)
    } else {
        None
    };

    match format {
        OutputFormat::Human => {
            let mode = if sign { "sealed+signed" } else { "sealed" };
            println!("HashSeal: {mode} {} instruct file(s)", digests.len());
            for (p, d) in &digests {
                println!("  {d}  {}", p.display());
            }
            if let Some(lp) = lock_path {
                println!("HashSeal: wrote lockfile {}", lp.display());
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "sealed": digests.iter().map(|(p, d)| {
                        serde_json::json!({"path": p, "digest": d, "signed": sign})
                    }).collect::<Vec<_>>(),
                    "lockfile": lock_path,
                }))?
            );
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_seal_tree(
    root: &Path,
    cfg: &HashSealConfig,
    ledger_override: Option<&Path>,
    release: bool,
    sign: bool,
    signing: &SigningOptions,
    artifacts: &[PathBuf],
    respect_gitignore: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let sign = sign || cfg.signing.enable;
    let opts = TreeSealOptions {
        algorithm: cfg.tree_algorithm()?,
        include: cfg.tree.include.clone(),
        exclude: cfg.tree.exclude.clone(),
        line_endings_lf_text: cfg.tree.line_endings == "lf-text",
        respect_gitignore,
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
        let sig_path = if sign {
            Some(sign_bundle(&paths.dir, signing)?)
        } else {
            None
        };
        if format == OutputFormat::Human {
            println!(
                "HashSeal: tree sealed {} file(s) → {}",
                ledger.entries.len(),
                paths.dir.display()
            );
            for a in &arts {
                println!("  artifact {}  {}", a.digest, a.name);
            }
            if let Some(sp) = sig_path {
                println!("  signed   {}", sp.display());
            }
        } else {
            println!(
                "{}",
                serde_json::json!({
                    "entries": ledger.entries.len(),
                    "bundle": paths.dir,
                    "artifacts": arts,
                    "signature": sig_path,
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

#[allow(clippy::too_many_arguments)]
fn run_verify(
    root: &Path,
    cfg: &HashSealConfig,
    ledger: Option<&Path>,
    bundle: Option<&Path>,
    no_fail: bool,
    require_signature: bool,
    digest_only: bool,
    signing: &SigningOptions,
    respect_gitignore: bool,
    format: OutputFormat,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let bp = BundlePaths::under(root);
    let bundle_dir = if let Some(b) = bundle {
        Some(b.to_path_buf())
    } else if ledger.is_none() && bp.manifest.is_file() {
        Some(bp.dir.clone())
    } else {
        None
    };

    let ledger_path = if let Some(b) = &bundle_dir {
        b.join("ledger.json")
    } else if let Some(l) = ledger {
        l.to_path_buf()
    } else if bp.ledger.is_file() {
        bp.ledger
    } else {
        cfg.default_ledger_path(root)
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
        respect_gitignore,
    };
    let result = if let Some(b) = &bundle_dir {
        let s_opt = if digest_only { None } else { Some(signing) };
        verify_bundle_with_signing(root, b, &opts, s_opt, require_signature && !digest_only)?
    } else {
        verify_tree(root, &led, &opts)?
    };
    let exit = if result.ok || no_fail || cfg.enforce.on_tree_mismatch == "warn" {
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

#[allow(clippy::too_many_arguments)]
fn run_check(
    root: &Path,
    paths: &[PathBuf],
    cfg: &HashSealConfig,
    format: OutputFormat,
    no_fail: bool,
    require_signature: bool,
    digest_only: bool,
    lock_override: Option<&Path>,
    no_lock: bool,
    diff_base: Option<&str>,
    signing: &SigningOptions,
    unicode_policy: UnicodePolicy,
    respect_gitignore: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let instruct_opts = InstructOptions {
        field: cfg.document.field.clone(),
        algorithm: cfg.document_algorithm()?,
        canonical: cfg.document_canonical(),
        auto_frontmatter: cfg.document.auto_frontmatter,
        unicode_policy,
    };

    let files = if let Some(base) = diff_base {
        collect_diff_md_files(root, base, &cfg.document.include, &cfg.document.exclude)?
    } else {
        collect_md_files(
            root,
            paths,
            &cfg.document.include,
            &cfg.document.exclude,
            respect_gitignore,
        )?
    };

    let mut results = Vec::new();
    for path in &files {
        let opts = CheckOpts {
            instruct: instruct_opts.clone(),
            require_signature,
            digest_only,
            expected_scope: rel_posix(root, path),
            signing: signing.clone(),
        };
        results.push(check_instruct_path_with(path, &opts));
    }

    if !no_lock && diff_base.is_none() && paths.is_empty() {
        let lock_path = lock_override
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.join(LOCKFILE_NAME));
        if lock_path.is_file() {
            let lock_opts = CheckOpts {
                instruct: instruct_opts,
                require_signature,
                digest_only,
                expected_scope: None,
                signing: signing.clone(),
            };
            let lock_findings = verify_lockfile(root, &lock_path, &files, &lock_opts)?;
            results.extend(lock_findings);
        }
    }

    let batch = BatchCheckResult::from_results(results.clone());
    let exit = if batch.ok || no_fail || cfg.enforce.on_doc_mismatch == "warn" {
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
        CheckStatus::UnverifiedSignature => "UNVERIFIED_SIGNATURE",
        CheckStatus::SuspiciousUnicode => "SUSPICIOUS_UNICODE",
        CheckStatus::LockedFileRemoved => "LOCKED_FILE_REMOVED",
        CheckStatus::UnapprovedFile => "UNAPPROVED_FILE",
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

fn run_unseal(
    root: &Path,
    paths: &[PathBuf],
    respect_gitignore: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let opts = InstructOptions::default();
    let files = if paths.is_empty() {
        collect_md_files(
            root,
            &[],
            &hashseal_core::default_document_includes(),
            &TreeSealOptions::default().exclude,
            respect_gitignore,
        )?
    } else {
        paths
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    root.join(p)
                }
            })
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

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
    }
    #[cfg(not(unix))]
    let _ = path;
}

fn run_init(
    root: &Path,
    git_hooks: bool,
    in_git_dir: bool,
    claude_hooks: bool,
    lock: bool,
    sign: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    fs::create_dir_all(root)?;

    // 1. .hashseal.json (if absent)
    let cfg_path = root.join(".hashseal.json");
    if !cfg_path.exists() {
        let starter = serde_json::json!({
            "version": 1,
            "document": {
                "enable": true,
                "canonical": "full",
                "field": "hashseal",
                "unicode_policy": "fail"
            },
            "signing": {
                "enable": sign,
                "require": sign
            }
        });
        fs::write(&cfg_path, serde_json::to_string_pretty(&starter)? + "\n")?;
        println!("HashSeal: wrote {}", cfg_path.display());
    }

    // 2. .hashseal/allowed_signers (auto-import git user.signingKey .pub if present)
    let hs_dir = root.join(".hashseal");
    fs::create_dir_all(&hs_dir)?;
    let signers_path = hs_dir.join("allowed_signers");
    if !signers_path.exists() {
        let mut content =
            String::from("# HashSeal allowed signers (<principal> ssh-ed25519 <base64>)\n");
        let git_opts = SigningOptions::from_git_and_root(Some(root));
        if let Some(key) = &git_opts.signing_key {
            let pub_candidate = if key.ends_with(".pub") {
                PathBuf::from(key)
            } else {
                PathBuf::from(format!("{key}.pub"))
            };
            if pub_candidate.is_file() {
                if let Ok(pub_line) = fs::read_to_string(&pub_candidate) {
                    let email = Command::new("git")
                        .arg("-C")
                        .arg(root)
                        .args(["config", "--get", "user.email"])
                        .output()
                        .ok()
                        .filter(|o| o.status.success())
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "maintainer@local".to_string());
                    content.push_str(&format!("{email} {}\n", pub_line.trim()));
                }
            }
        }
        fs::write(&signers_path, content)?;
        println!("HashSeal: wrote {}", signers_path.display());
    }

    // 3. Git pre-commit hook
    if git_hooks {
        let hook_body = "#!/usr/bin/env sh\nset -eu\nif command -v hashseal >/dev/null 2>&1; then\n  exec hashseal check --root .\nelif command -v hashseal-check >/dev/null 2>&1; then\n  exec hashseal-check --root .\nelse\n  echo \"hashseal: binary not found on PATH\" >&2\n  exit 1\nfi\n";
        let hook_path = if in_git_dir {
            let p = root.join(".git").join("hooks");
            fs::create_dir_all(&p)?;
            p.join("pre-commit")
        } else {
            let p = root.join(".githooks");
            fs::create_dir_all(&p)?;
            if root.join(".git").exists() {
                let _ = Command::new("git")
                    .arg("-C")
                    .arg(root)
                    .args(["config", "core.hooksPath", ".githooks"])
                    .status();
            }
            p.join("pre-commit")
        };
        fs::write(&hook_path, hook_body)?;
        make_executable(&hook_path);
        println!("HashSeal: installed git hook {}", hook_path.display());
    }

    // 4. Claude Code hooks (SessionStart + PreToolUse guard)
    if claude_hooks {
        let hooks_dir = hs_dir.join("hooks");
        fs::create_dir_all(&hooks_dir)?;
        let guard_path = hooks_dir.join("claude-guard.sh");
        let guard_script = r#"#!/usr/bin/env sh
# HashSeal Claude Code guard: block editing human-signed instruction files and check seals.
set -eu
if command -v hashseal-check >/dev/null 2>&1; then
  hashseal-check --root .
elif command -v hashseal >/dev/null 2>&1; then
  hashseal check --root .
fi
"#;
        fs::write(&guard_path, guard_script)?;
        make_executable(&guard_path);

        let claude_dir = root.join(".claude");
        fs::create_dir_all(&claude_dir)?;
        let settings_path = claude_dir.join("settings.json");
        let mut settings: serde_json::Value = if settings_path.is_file() {
            let raw = fs::read_to_string(&settings_path)?;
            serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let check_cmd = "if command -v hashseal-check >/dev/null 2>&1; then hashseal-check --root .; else hashseal check --root .; fi";
        let hooks_obj = settings
            .as_object_mut()
            .ok_or("invalid .claude/settings.json")?
            .entry("hooks")
            .or_insert_with(|| serde_json::json!({}));
        if let Some(m) = hooks_obj.as_object_mut() {
            m.insert(
                "SessionStart".into(),
                serde_json::json!([{
                    "hooks": [{ "type": "command", "command": check_cmd }]
                }]),
            );
            m.insert(
                "PreToolUse".into(),
                serde_json::json!([{
                    "matcher": "Edit|Write",
                    "hooks": [{ "type": "command", "command": ".hashseal/hooks/claude-guard.sh" }]
                }]),
            );
        }
        fs::write(
            &settings_path,
            serde_json::to_string_pretty(&settings)? + "\n",
        )?;
        println!(
            "HashSeal: configured Claude hooks in {}",
            settings_path.display()
        );
    }

    // 5. Seal instruct files + optional .hashseal.lock (default on init when --lock or no specific hook-only flag)
    let write_lock = lock || (!git_hooks && !claude_hooks);
    let cfg = HashSealConfig::load_for_root(root)?;
    let s_opts = signing_from(root, &cfg, None, None, None, &[])?;
    let upol = cfg.document_unicode_policy()?;
    run_seal_instruct(
        root,
        &cfg,
        sign,
        &s_opts,
        false,
        write_lock,
        upol,
        cfg.walk.respect_gitignore,
        OutputFormat::Human,
    )?;

    Ok(ExitCode::SUCCESS)
}
