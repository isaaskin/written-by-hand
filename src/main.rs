mod badge;
mod config;
mod filter;
mod markers;
mod paths;
mod readme;
mod report;
mod scan;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use config::{BadgeStyle, CONFIG_FILE, Config};
use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// A tiny badge for code made the old-fashioned way.
///
/// Handwritten status is self-declared: wbh performs no verification or AI detection.
#[derive(Parser)]
#[command(name = "wbh", version)]
struct Cli {
    /// Project root (defaults to the current directory)
    #[arg(long, global = true, value_name = "PATH")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create .wbh.toml, generate the badge and reports, add the README badge
    Init {
        /// Badge style (prompted for in an interactive terminal, otherwise stamp)
        #[arg(long)]
        style: Option<BadgeStyle>,
        /// Never touch README.md
        #[arg(long)]
        no_readme: bool,
    },
    /// Declare whole files handwritten (edits .wbh.toml only, never the files)
    Mark {
        /// Files relative to the project root
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Remove whole-file declarations from .wbh.toml
    Unmark {
        /// Files relative to the project root
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Scan sources and regenerate the badge, reports and README block
    Scan,
    /// Show the current badge settings, or list and change styles
    Badge {
        #[command(subcommand)]
        action: Option<BadgeAction>,
    },
    /// Validate markers and config without writing anything (for CI)
    Check {
        /// Also fail when generated files are out of date
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Subcommand)]
enum BadgeAction {
    /// List the available styles
    List,
    /// Change the badge and regenerate (e.g. `wbh badge set pixel --compact`)
    #[command(group(clap::ArgGroup::new("change").required(true).multiple(true)))]
    Set {
        /// New style
        #[arg(group = "change")]
        style: Option<BadgeStyle>,
        /// Use the smaller variant
        #[arg(long, group = "change", conflicts_with = "no_compact")]
        compact: bool,
        /// Use the regular size
        #[arg(long, group = "change")]
        no_compact: bool,
        /// Show the percentage
        #[arg(long, group = "change", conflicts_with = "no_percentage")]
        percentage: bool,
        /// Show only the label
        #[arg(long, group = "change")]
        no_percentage: bool,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let root = cli.root.unwrap_or_else(|| PathBuf::from("."));
    let root = fs::canonicalize(&root)
        .with_context(|| format!("cannot open project root {}", root.display()))?;
    if !root.is_dir() {
        bail!("project root {} is not a directory", root.display());
    }
    match cli.command {
        Command::Init { style, no_readme } => init(&root, style, no_readme),
        Command::Mark { paths } => mark(&root, &paths),
        Command::Unmark { paths } => unmark(&root, &paths),
        Command::Scan => scan_cmd(&root),
        Command::Badge { action: None } => badge_show(&root),
        Command::Badge {
            action: Some(BadgeAction::List),
        } => {
            for s in BadgeStyle::ALL {
                println!("{:<9} {}", s.key(), s.description());
            }
            println!("\nChange it with: wbh badge set <style> [--compact] [--no-percentage]");
            Ok(())
        }
        Command::Badge {
            action:
                Some(BadgeAction::Set {
                    style,
                    compact,
                    no_compact,
                    percentage,
                    no_percentage,
                }),
        } => {
            edit_config(&root, |doc| {
                let badge = &mut doc["badge"];
                if let Some(s) = style {
                    badge["style"] = toml_edit::value(s.key());
                }
                if compact || no_compact {
                    badge["compact"] = toml_edit::value(compact);
                }
                if percentage || no_percentage {
                    badge["show_percentage"] = toml_edit::value(percentage);
                }
                Ok(())
            })?;
            badge_show(&root)?;
            println!();
            scan_cmd(&root)
        }
        Command::Check { strict } => check(&root, strict),
    }
}

fn badge_show(root: &Path) -> Result<()> {
    let b = config::load(root)?.badge;
    println!("style        {} ({})", b.style.key(), b.style.description());
    println!("compact      {}", if b.compact { "yes" } else { "no" });
    println!(
        "percentage   {}",
        if b.show_percentage { "shown" } else { "hidden" }
    );
    println!("text         {}", b.text);
    println!("file         {}", config::BADGE_FILE);
    Ok(())
}

fn init(root: &Path, style: Option<BadgeStyle>, no_readme: bool) -> Result<()> {
    let path = root.join(CONFIG_FILE);
    if path.symlink_metadata().is_ok() {
        println!("{CONFIG_FILE} already exists; leaving it unchanged.");
        println!("\nNext: wbh mark <file>   declare a whole file handwritten");
        println!("      wbh scan          regenerate the badge and reports");
        return Ok(());
    }
    let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    let style = match style {
        Some(s) => s,
        None if interactive => prompt_style()?,
        None => BadgeStyle::Stamp,
    };
    let text = config::template(&config::default_name(root), style, !no_readme);
    let cfg = config::parse(&text, root)?;
    paths::write_if_changed(&path, text.as_bytes())?;
    println!("Created {CONFIG_FILE} (badge style: {})\n", style.key());

    scan_cmd(root)?;

    if cfg.readme.enabled
        && paths::join(root, &cfg.readme.path)
            .symlink_metadata()
            .is_err()
    {
        println!(
            "\nNo {} found, so none was created. Add this where you want the badge:\n",
            cfg.readme.path
        );
        for line in readme::block(&cfg) {
            println!("{line}");
        }
    }
    println!("\nNext: wbh mark <file>   declare a whole file handwritten");
    println!("      wbh scan          regenerate after adding @handwritten comments");
    Ok(())
}

fn prompt_style() -> Result<BadgeStyle> {
    eprintln!("Pick a badge style:");
    for (i, s) in BadgeStyle::ALL.iter().enumerate() {
        eprintln!("  {}) {:<9} {}", i + 1, s.key(), s.description());
    }
    eprint!("Style [1]: ");
    std::io::stderr().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    let line = line.trim();
    if line.is_empty() {
        return Ok(BadgeStyle::Stamp);
    }
    line.parse::<usize>()
        .ok()
        .and_then(|n| BadgeStyle::ALL.get(n.wrapping_sub(1)).copied())
        .with_context(|| format!("invalid choice {line:?}; enter 1–{}", BadgeStyle::ALL.len()))
}

/// A generated file and its expected bytes.
struct Output {
    rel: String,
    bytes: Vec<u8>,
}

/// Renders every generated file from one scan, in memory. Nothing is written here,
/// so a failure anywhere leaves the previous outputs untouched.
fn render_outputs(root: &Path, cfg: &Config, result: &scan::ScanResult) -> Result<Vec<Output>> {
    let name = &cfg.project.name;
    let mut outputs = vec![
        Output {
            rel: config::REPORT_MD.into(),
            bytes: report::markdown(name, result, config::REPORT_MD).into_bytes(),
        },
        Output {
            rel: config::BADGE_FILE.into(),
            bytes: badge::render(&cfg.badge, &result.stats).into_bytes(),
        },
    ];
    if cfg.report.json {
        outputs.push(Output {
            rel: config::REPORT_JSON.into(),
            bytes: report::json(name, result).into_bytes(),
        });
    }
    if cfg.readme.enabled {
        let path = paths::join(root, &cfg.readme.path);
        match fs::read_to_string(&path) {
            Ok(text) => outputs.push(Output {
                rel: cfg.readme.path.clone(),
                bytes: readme::apply(&text, &readme::block(cfg))
                    .with_context(|| cfg.readme.path.clone())?
                    .into_bytes(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("cannot read {}", cfg.readme.path)),
        }
    }
    Ok(outputs)
}

fn scan_cmd(root: &Path) -> Result<()> {
    let cfg = config::load(root)?;
    let result = scan::scan(root, &cfg)?;
    let outputs = render_outputs(root, &cfg, &result)?;
    for w in &result.warnings {
        eprintln!("warning: {w}");
    }
    let s = &result.stats;
    let plural = if s.files_scanned == 1 { "" } else { "s" };
    println!("Written by Hand ✍");
    println!(
        "  {} source file{plural} scanned",
        report::thousands(s.files_scanned)
    );
    println!(
        "  {} / {} eligible lines marked ({})\n",
        report::thousands(s.marked_lines),
        report::thousands(s.eligible_lines),
        scan::format_pct(scan::tenths(s.marked_lines, s.eligible_lines))
    );
    if s.marked_lines == 0 && s.eligible_lines > 0 {
        println!("  Nothing is marked yet. Add `// @handwritten` at the top of a file you wrote,");
        println!(
            "  wrap a block in `// @handwritten:start` / `// @handwritten:end`, or run `wbh mark <file>`.\n"
        );
    }
    for o in &outputs {
        // Re-check right before writing in case something changed since config load.
        let path = paths::resolve_inside(root, &o.rel).with_context(|| o.rel.clone())?;
        let wrote = paths::write_if_changed(&path, &o.bytes)?;
        println!("  ✓ {}{}", o.rel, if wrote { "" } else { " (unchanged)" });
    }
    println!("\nSelf-reported. No AI detection, no certification.");
    Ok(())
}

/// Always validates config, markers and the README block. Stale outputs only fail
/// with `--strict`: otherwise every code change would break CI for contributors
/// who didn't touch any declarations.
fn check(root: &Path, strict: bool) -> Result<()> {
    let cfg = config::load(root)?;
    let result = scan::scan(root, &cfg)?;
    let outputs = render_outputs(root, &cfg, &result)?;
    for w in &result.warnings {
        eprintln!("warning: {w}");
    }
    // Git may check generated files out with CRLF on Windows; that isn't staleness.
    let strip_cr = |b: &[u8]| {
        b.iter()
            .copied()
            .filter(|&c| c != b'\r')
            .collect::<Vec<_>>()
    };
    let mut problems = Vec::new();
    for o in &outputs {
        match fs::read(paths::join(root, &o.rel)) {
            Err(_) => problems.push(format!("missing: {}", o.rel)),
            Ok(actual) if strip_cr(&actual) != strip_cr(&o.bytes) => {
                problems.push(format!("out of date: {}", o.rel))
            }
            Ok(_) => {}
        }
    }
    if problems.is_empty() {
        println!("✓ markers and config are valid; generated files are up to date");
    } else if strict {
        bail!(
            "generated files don't match the sources\n  {}\n  hint: run `wbh scan` and commit the result",
            problems.join("\n  ")
        );
    } else {
        println!("✓ markers and config are valid");
        println!(
            "  note: generated files are out of date ({}); run `wbh scan` to refresh them",
            problems.join(", ")
        );
    }
    Ok(())
}

fn mark(root: &Path, paths: &[String]) -> Result<()> {
    let cfg = config::load(root)?;
    let filter = filter::Filter::new(&cfg.scan)?;
    let scanned = scan::discover(root, &cfg, &filter, &mut Vec::new());
    // Validate everything before touching the config.
    let mut rels = Vec::new();
    for p in paths {
        let rel = paths::normalize_rel(p).with_context(|| format!("cannot mark {p}"))?;
        paths::existing_file(root, &rel).with_context(|| format!("cannot mark {p}"))?;
        if let Some(why) = filter.skip_reason(&rel) {
            bail!(
                "cannot mark {rel}: file is not scanned ({why})\nhint: adjust [scan] in {CONFIG_FILE}"
            );
        }
        if scanned.binary_search(&rel).is_err() {
            bail!("cannot mark {rel}: file is ignored by .gitignore or .ignore");
        }
        rels.push(rel);
    }
    edit_claims(root, |files| {
        for rel in rels {
            if files.iter().any(|v| same_claim(v, &rel)) {
                println!("already marked {rel}");
            } else {
                files.push(rel.as_str());
                println!("marked {rel}");
            }
        }
    })?;
    println!("\nRun `wbh scan` to update the badge and reports.");
    Ok(())
}

fn unmark(root: &Path, paths: &[String]) -> Result<()> {
    config::load(root)?;
    let rels = paths
        .iter()
        .map(|p| paths::normalize_rel(p).with_context(|| format!("cannot unmark {p}")))
        .collect::<Result<Vec<_>>>()?;
    edit_claims(root, |files| {
        for rel in rels {
            let before = files.len();
            files.retain(|v| !same_claim(v, &rel));
            if files.len() == before {
                eprintln!("warning: {rel} was not marked in {CONFIG_FILE}");
                continue;
            }
            println!("unmarked {rel}");
            let has_marker = paths::existing_file(root, &rel)
                .and_then(|f| Ok(fs::read_to_string(f)?))
                .is_ok_and(|s| s.contains("@handwritten"));
            if has_marker {
                println!("  note: {rel} contains an @handwritten comment, so it may still count;");
                println!(
                    "        remove that comment from the file yourself if you want it unmarked"
                );
            }
        }
    })
}

fn same_claim(v: &toml_edit::Value, rel: &str) -> bool {
    v.as_str()
        .and_then(|s| paths::normalize_rel(s).ok())
        .is_some_and(|s| s == rel)
}

/// Edits `[claims].files` in place, preserving the rest of the file's formatting.
fn edit_claims(root: &Path, edit: impl FnOnce(&mut toml_edit::Array)) -> Result<()> {
    edit_config(root, |doc| {
        let files = doc
            .entry("claims")
            .or_insert(toml_edit::table())
            .as_table_like_mut()
            .context("[claims] must be a table")?
            .entry("files")
            .or_insert(toml_edit::value(toml_edit::Array::new()))
            .as_array_mut()
            .context("claims.files must be an array")?;
        edit(files);
        // One entry per line keeps diffs readable.
        for v in files.iter_mut() {
            v.decor_mut().set_prefix("\n  ");
            v.decor_mut().set_suffix("");
        }
        files.set_trailing_comma(!files.is_empty());
        files.set_trailing(if files.is_empty() { "" } else { "\n" });
        Ok(())
    })
}

/// Edits `.wbh.toml` with formatting and comments preserved, re-validating before writing.
fn edit_config(
    root: &Path,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<()>,
) -> Result<()> {
    let path = root.join(CONFIG_FILE);
    let text = fs::read_to_string(&path).with_context(|| format!("cannot read {CONFIG_FILE}"))?;
    let mut doc: toml_edit::DocumentMut = text.parse()?;
    edit(&mut doc)?;
    let text = doc.to_string();
    config::parse(&text, root)?;
    paths::write_if_changed(&path, text.as_bytes())?;
    Ok(())
}
