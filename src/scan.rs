//! File discovery and coverage. `ScanResult` is the single source of truth for every output.

use crate::config::Config;
use crate::filter::{Filter, family};
use crate::markers;
use crate::paths::{join, normalize_rel};
use anyhow::{Result, bail};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const MAX_ERRORS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimSource {
    // Declared in alphabetical order of their JSON names (block-marker, config,
    // file-marker), so sorting the enum sorts the report.
    BlockMarker,
    Config,
    FileMarker,
}

#[derive(Debug, PartialEq)]
pub struct FileResult {
    pub path: String,
    pub physical_lines: usize,
    pub eligible_lines: usize,
    pub marked_lines: usize,
    pub claim_sources: Vec<ClaimSource>,
    /// Inclusive, sorted, non-overlapping physical line ranges.
    pub ranges: Vec<(usize, usize)>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub files_scanned: usize,
    pub files_with_marks: usize,
    pub eligible_lines: usize,
    pub marked_lines: usize,
}

#[derive(Debug)]
pub struct ScanResult {
    pub stats: Stats,
    pub files: Vec<FileResult>,
    pub warnings: Vec<String>,
}

/// Percentage in tenths, rounded down so 100.0 only ever means "every line".
/// `None` when there is nothing to measure.
pub fn tenths(marked: usize, eligible: usize) -> Option<u64> {
    (eligible > 0).then(|| marked as u64 * 1000 / eligible as u64)
}

pub fn format_pct(tenths: Option<u64>) -> String {
    match tenths {
        Some(t) => format!("{}.{}%", t / 10, t % 10),
        None => "N/A".into(),
    }
}

/// Sorted root-relative paths of files that pass ignore rules and the scan filter.
/// Symlinks are never followed and gitignore rules from outside the root are not read.
pub fn discover(
    root: &Path,
    cfg: &Config,
    filter: &Filter,
    warnings: &mut Vec<String>,
) -> Vec<String> {
    let respect = cfg.scan.respect_gitignore;
    let walk = ignore::WalkBuilder::new(root)
        .hidden(true)
        .parents(false)
        .follow_links(false)
        .require_git(false)
        .git_ignore(respect)
        .ignore(respect)
        // Machine-local ignore sources would make CI and local scans disagree.
        .git_global(false)
        .git_exclude(false)
        .build();
    // ponytail: walks into excluded dirs (e.g. an un-gitignored node_modules) before
    // filtering their files; prune with filter_entry if that gets slow.
    let mut paths = Vec::new();
    for entry in walk {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warnings.push(format!("skipped unreadable path: {e}"));
                continue;
            }
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Some(rel) = rel_path(root, entry.path()) else {
            warnings.push(format!("skipped non-UTF-8 path {}", entry.path().display()));
            continue;
        };
        if filter.skip_reason(&rel).is_none() {
            paths.push(rel);
        }
    }
    paths.sort();
    paths
}

fn rel_path(root: &Path, path: &Path) -> Option<String> {
    let parts = path
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|c| c.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?;
    Some(parts.join("/"))
}

pub fn scan(root: &Path, cfg: &Config) -> Result<ScanResult> {
    let filter = Filter::new(&cfg.scan)?;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let paths = discover(root, cfg, &filter, &mut warnings);

    let claims: BTreeSet<String> = cfg
        .claims
        .files
        .iter()
        .filter_map(|f| normalize_rel(f).ok()) // already validated by config
        .collect();
    for c in &claims {
        if paths.binary_search(c).is_err() {
            let why = if !join(root, c).is_file() {
                "no such file"
            } else {
                filter
                    .skip_reason(c)
                    .unwrap_or("ignored by .gitignore or .ignore")
            };
            // A warning, not an error: renaming or deleting a claimed file
            // shouldn't break every scan until the claim is cleaned up.
            warnings.push(format!(
                "claims.files: {c} is not scanned ({why}); run `wbh unmark {c}`"
            ));
        }
    }

    let mut files = Vec::new();
    for rel in paths {
        let claimed = claims.contains(&rel);
        let mut skip = |why: String| {
            if claimed {
                warnings.push(format!(
                    "skipped {rel}: {why}, so its claim in claims.files has no effect"
                ));
            } else {
                warnings.push(format!("skipped {rel}: {why}"));
            }
        };
        let full = join(root, &rel);
        let size = match fs::symlink_metadata(&full) {
            Ok(m) => m.len(),
            Err(e) => {
                errors.push(format!("{rel}: cannot read: {e}"));
                continue;
            }
        };
        if size > cfg.scan.max_file_bytes {
            skip(format!(
                "larger than scan.max_file_bytes ({size} > {})",
                cfg.scan.max_file_bytes
            ));
            continue;
        }
        let bytes = match fs::read(&full) {
            Ok(b) => b,
            Err(e) => {
                errors.push(format!("{rel}: cannot read: {e}"));
                continue;
            }
        };
        if bytes.contains(&0) {
            skip("it looks binary".into());
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes) else {
            skip("it is not valid UTF-8".into());
            continue;
        };
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let family = family(&rel).expect("discover only returns supported extensions");
        let marks = markers::parse(text, family);

        errors.extend(marks.errors.iter().map(|(n, e)| format!("{rel}:{n}: {e}")));
        warnings.extend(
            marks
                .warnings
                .iter()
                .map(|(n, w)| format!("{rel}:{n}: {w}")),
        );

        let physical_lines = marks.eligible.len();
        let ranges = if claimed || marks.file_marker {
            if physical_lines > 0 {
                vec![(1, physical_lines)]
            } else {
                vec![]
            }
        } else {
            marks.blocks.clone()
        };
        let count = |a: usize, b: usize| marks.eligible[a - 1..b].iter().filter(|e| **e).count();
        let mut claim_sources = Vec::new();
        if claimed {
            claim_sources.push(ClaimSource::Config);
        }
        if marks.file_marker {
            claim_sources.push(ClaimSource::FileMarker);
        }
        if !marks.blocks.is_empty() {
            claim_sources.push(ClaimSource::BlockMarker);
        }
        claim_sources.sort();
        files.push(FileResult {
            eligible_lines: count(1, physical_lines),
            marked_lines: ranges.iter().map(|&(a, b)| count(a, b)).sum(),
            path: rel,
            physical_lines,
            claim_sources,
            ranges,
        });
    }

    if !errors.is_empty() {
        let total = errors.len();
        errors.truncate(MAX_ERRORS);
        let mut msg = format!("found {total} problem(s); nothing was written\n");
        for e in &errors {
            msg.push_str(&format!("  {e}\n"));
        }
        if total > MAX_ERRORS {
            msg.push_str(&format!("  ... and {} more\n", total - MAX_ERRORS));
        }
        bail!(msg.trim_end().to_string());
    }

    let stats = Stats {
        files_scanned: files.len(),
        files_with_marks: files.iter().filter(|f| f.marked_lines > 0).count(),
        eligible_lines: files.iter().map(|f| f.eligible_lines).sum(),
        marked_lines: files.iter().map(|f| f.marked_lines).sum(),
    };
    if stats.eligible_lines == 0 {
        warnings.push("this project contains no eligible source lines; coverage is N/A".into());
    }
    Ok(ScanResult {
        stats,
        files,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct Project {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    fn project(files: &[(&str, &[u8])], config: &str) -> Project {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        for (path, content) in files {
            let p = join(&root, path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, content).unwrap();
        }
        fs::write(root.join(".wbh.toml"), config).unwrap();
        Project { _dir: dir, root }
    }

    fn run(p: &Project) -> Result<ScanResult> {
        scan(&p.root, &crate::config::load(&p.root)?)
    }

    fn file<'a>(r: &'a ScanResult, path: &str) -> &'a FileResult {
        r.files.iter().find(|f| f.path == path).unwrap()
    }

    use ClaimSource::*;

    #[test]
    fn rounding_never_reaches_100_early() {
        assert_eq!(format_pct(tenths(3, 4)), "75.0%");
        assert_eq!(format_pct(tenths(9996, 10000)), "99.9%");
        assert_eq!(format_pct(tenths(1, 3)), "33.3%");
        assert_eq!(format_pct(tenths(2, 3)), "66.6%");
        assert_eq!(format_pct(tenths(5, 5)), "100.0%");
        assert_eq!(format_pct(tenths(0, 0)), "N/A");
    }

    #[test]
    fn mixed_language_fixture() {
        let p = project(
            &[
                // Spec §15.3: 4 eligible, 3 marked.
                ("src/main.rs", b"// @handwritten:start\nfn main() {\n    println!(\"hello\");\n}\n// @handwritten:end\n\nfn other() {}\n"),
                // Whole-file header marker after a license block: 5 eligible, all marked.
                ("src/dsp/Osc.cpp", b"/*\n * MIT\n */\n// @handwritten\n#include <cmath>\n\nfloat osc(float x) { return std::sin(x); }\n"),
                // Two blocks: 5 eligible, 2 marked.
                ("ui/View.swift", b"import UI\n// @handwritten:start\nlet a = 1\n// @handwritten:end\nlet b = 2\n/* @handwritten:start */\nlet c = 3\n\n/* @handwritten:end */\nlet d = 4\n"),
                // Shebang header then file marker: 3 eligible, all marked.
                ("tools/run.py", b"#!/usr/bin/env python3\n# @handwritten\n\ndef run():\n    return 42\n"),
                // Not marked at all: 2 eligible.
                ("web/app.js", b"const x = `\n// not a marker`;\n"),
                // HTML block: 3 eligible, 1 marked.
                ("web/index.html", b"<html>\n<!-- @handwritten:start -->\n<div>Hand</div>\n<!-- @handwritten:end -->\n</html>\n"),
                // JSON is opt-in and claims-only: 3 eligible, all marked via config.
                ("data/settings.json", b"{\n  \"a\": 1\n}\n"),
                // Ignored/excluded/hidden files never count.
                ("target/gen.rs", b"fn x() {}\n"),
                ("vendor/lib.c", b"int y;\n"),
                (".hidden/z.rs", b"fn z() {}\n"),
                ("README.md", b"# Hi\n"),
                ("logs/out.rs", b"fn log() {}\n"),
                (".gitignore", b"logs/\n"),
            ],
            "[scan]\nexclude = [\"**/target/**\", \"**/vendor/**\", \"**/*.md\"]\n[claims]\nfiles = [\"data/settings.json\", \"src/main.rs\"]\n",
        );
        let r = run(&p).unwrap();
        let paths: Vec<_> = r.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "data/settings.json",
                "src/dsp/Osc.cpp",
                "src/main.rs",
                "tools/run.py",
                "ui/View.swift",
                "web/app.js",
                "web/index.html"
            ]
        );
        let swift = file(&r, "ui/View.swift");
        assert_eq!((swift.eligible_lines, swift.marked_lines), (5, 2));
        assert_eq!(swift.ranges, [(3, 3), (7, 8)]);
        assert_eq!(swift.claim_sources, [BlockMarker]);

        // Config claim + block markers: whole file, counted once.
        let main = file(&r, "src/main.rs");
        assert_eq!(
            (main.physical_lines, main.eligible_lines, main.marked_lines),
            (7, 4, 4)
        );
        assert_eq!(main.ranges, [(1, 7)]);
        assert_eq!(main.claim_sources, [BlockMarker, Config]);

        let cpp = file(&r, "src/dsp/Osc.cpp");
        assert_eq!((cpp.eligible_lines, cpp.marked_lines), (5, 5));
        assert_eq!(cpp.claim_sources, [FileMarker]);

        assert_eq!(file(&r, "tools/run.py").marked_lines, 3);
        assert_eq!(file(&r, "web/app.js").marked_lines, 0);
        assert_eq!(file(&r, "web/index.html").ranges, [(3, 3)]);
        assert_eq!(file(&r, "data/settings.json").marked_lines, 3);

        assert_eq!(
            r.stats,
            Stats {
                files_scanned: 7,
                files_with_marks: 6,
                eligible_lines: 3 + 5 + 4 + 3 + 5 + 2 + 3,
                marked_lines: 3 + 5 + 4 + 3 + 2 + 1,
            }
        );
    }

    #[test]
    fn spec_fixture_without_claim_is_75_percent() {
        let p = project(
            &[("src/main.rs", b"// @handwritten:start\nfn main() {\n    println!(\"hello\");\n}\n// @handwritten:end\n\nfn other() {}\n")],
            "",
        );
        let r = run(&p).unwrap();
        assert_eq!((r.stats.eligible_lines, r.stats.marked_lines), (4, 3));
        assert_eq!(format_pct(tenths(3, 4)), "75.0%");
        assert_eq!(r.files[0].ranges, [(2, 4)]);
    }

    #[test]
    fn encodings_and_line_endings() {
        let p = project(
            &[
                (
                    "a.rs",
                    b"\xEF\xBB\xBF// @handwritten\r\nfn a() {}\r\n\r\nfn b() {}",
                ),
                ("b.rs", b"\xff\xfe not utf8"),
                ("c.rs", b"bin\0ary"),
                (
                    "ünï/ç.rs",
                    "// @handwritten:start\nlet ✍ = 1;\n// @handwritten:end\n".as_bytes(),
                ),
                ("empty.rs", b""),
                ("big.rs", &[b'x'; 64]),
            ],
            "[scan]\nmax_file_bytes = 63\n",
        );
        let r = run(&p).unwrap();
        let a = file(&r, "a.rs");
        assert_eq!(
            (a.physical_lines, a.eligible_lines, a.marked_lines),
            (4, 2, 2)
        );
        assert_eq!(file(&r, "ünï/ç.rs").marked_lines, 1);
        assert_eq!(file(&r, "empty.rs").physical_lines, 0);
        assert!(
            r.files
                .iter()
                .all(|f| !["b.rs", "c.rs", "big.rs"].contains(&f.path.as_str()))
        );
        assert_eq!(r.warnings.len(), 3, "{:?}", r.warnings);
    }

    #[test]
    fn empty_project_is_not_100_percent() {
        let p = project(&[("blank.py", b"\n\n# @handwritten\n")], "");
        let r = run(&p).unwrap();
        assert_eq!(r.stats.eligible_lines, 0);
        assert_eq!(
            format_pct(tenths(r.stats.marked_lines, r.stats.eligible_lines)),
            "N/A"
        );
        assert!(r.warnings.iter().any(|w| w.contains("N/A")));
    }

    #[test]
    fn stale_and_unscannable_claims_warn() {
        let p = project(
            &[
                ("bin.rs", b"\0"),
                ("x.txt", b"x"),
                ("ok.rs", b"fn ok() {}\n"),
            ],
            "[claims]\nfiles = [\"gone.rs\", \"bin.rs\", \"x.txt\", \"ok.rs\"]\n",
        );
        let r = run(&p).unwrap();
        let w = r.warnings.join("\n");
        assert!(
            w.contains("gone.rs is not scanned (no such file); run `wbh unmark gone.rs`"),
            "{w}"
        );
        assert!(
            w.contains("x.txt is not scanned (unsupported file extension)"),
            "{w}"
        );
        assert!(
            w.contains("skipped bin.rs: it looks binary, so its claim"),
            "{w}"
        );
        // Only the real file counts.
        assert_eq!((r.stats.files_scanned, r.stats.marked_lines), (1, 1));
    }

    #[test]
    fn marker_errors_report_all_files() {
        let p = project(
            &[
                ("a.rs", b"x\n// @handwritten:start\n"),
                ("b.py", b"x = 1\n# @handwritten\n"),
            ],
            "",
        );
        let err = format!("{:#}", run(&p).unwrap_err());
        assert!(err.contains("a.rs:2: unclosed @handwritten:start"), "{err}");
        assert!(
            err.contains("b.py:2: file marker must be in the header"),
            "{err}"
        );
    }

    #[test]
    fn gitignore_respected_unless_disabled() {
        let files: &[(&str, &[u8])] = &[
            (".gitignore", b"gen.rs\n"),
            ("gen.rs", b"x\n"),
            ("a.rs", b"y\n"),
        ];
        let r = run(&project(files, "")).unwrap();
        assert_eq!(r.files.len(), 1);
        let r = run(&project(files, "[scan]\nrespect_gitignore = false\n")).unwrap();
        assert_eq!(r.files.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_followed() {
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("o.rs"), "fn o() {}\n").unwrap();
        let p = project(&[("a.rs", b"fn a() {}\n")], "");
        std::os::unix::fs::symlink(outside.path(), p.root.join("linkdir")).unwrap();
        std::os::unix::fs::symlink(p.root.join("a.rs"), p.root.join("alias.rs")).unwrap();
        let r = run(&p).unwrap();
        assert_eq!(r.files.len(), 1);
    }
}
