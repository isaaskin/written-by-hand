//! `report.json` and `REPORT.md`, rendered from a `ScanResult`, plus link helpers.

use crate::scan::{ClaimSource, ScanResult, format_pct, tenths};
use serde::Serialize;
use std::fmt::Write;

pub const DISCLOSURE: &str = "These handwritten markings are self-reported by the project maintainer; no independent verification or AI detection is performed.";

#[derive(Serialize)]
struct Report<'a> {
    schema_version: u32,
    project: Project<'a>,
    declaration: &'static str,
    stats: Stats,
    files: Vec<File<'a>>,
}

#[derive(Serialize)]
struct Project<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct Stats {
    files_scanned: usize,
    files_with_marks: usize,
    eligible_lines: usize,
    marked_lines: usize,
    percentage: Option<f64>,
}

#[derive(Serialize)]
struct File<'a> {
    path: &'a str,
    physical_lines: usize,
    eligible_lines: usize,
    marked_lines: usize,
    percentage: Option<f64>,
    claim_sources: &'a [ClaimSource],
    ranges: &'a [(usize, usize)],
}

fn pct(marked: usize, eligible: usize) -> Option<f64> {
    tenths(marked, eligible).map(|t| t as f64 / 10.0)
}

pub fn json(name: &str, r: &ScanResult) -> String {
    let s = &r.stats;
    let report = Report {
        schema_version: 1,
        project: Project { name },
        declaration: "self-reported",
        stats: Stats {
            files_scanned: s.files_scanned,
            files_with_marks: s.files_with_marks,
            eligible_lines: s.eligible_lines,
            marked_lines: s.marked_lines,
            percentage: pct(s.marked_lines, s.eligible_lines),
        },
        files: r
            .files
            .iter()
            .map(|f| File {
                path: &f.path,
                physical_lines: f.physical_lines,
                eligible_lines: f.eligible_lines,
                marked_lines: f.marked_lines,
                percentage: pct(f.marked_lines, f.eligible_lines),
                claim_sources: &f.claim_sources,
                ranges: &f.ranges,
            })
            .collect(),
    };
    let mut out = serde_json::to_string_pretty(&report).expect("report serializes");
    out.push('\n');
    out
}

/// `REPORT.md`, stored at `report_rel` (links are made relative to it).
pub fn markdown(name: &str, r: &ScanResult, report_rel: &str) -> String {
    let s = &r.stats;
    let mut md = String::new();
    let _ = writeln!(md, "# Written by Hand — {}\n", md_escape(name));
    let _ = writeln!(md, "> {DISCLOSURE}\n");
    let _ = writeln!(
        md,
        "**Self-declared handwritten coverage:** {} ({} marked / {} eligible source lines; {} of {} scanned files have marks)\n",
        format_pct(tenths(s.marked_lines, s.eligible_lines)),
        thousands(s.marked_lines),
        thousands(s.eligible_lines),
        thousands(s.files_with_marks),
        thousands(s.files_scanned),
    );

    let marked: Vec<_> = r.files.iter().filter(|f| f.marked_lines > 0).collect();
    if marked.is_empty() {
        md.push_str("_No files are marked yet._\n\n");
    } else {
        md.push_str("| File | Handwritten | Marked ranges |\n|---|---:|---|\n");
        for f in &marked {
            let target = relative_link(report_rel, &f.path);
            let ranges: Vec<String> = f
                .ranges
                .iter()
                .map(|&(a, b)| {
                    if a == b {
                        format!("[L{a}]({target}#L{a})")
                    } else {
                        format!("[L{a}–L{b}]({target}#L{a}-L{b})")
                    }
                })
                .collect();
            let _ = writeln!(
                md,
                "| [{}]({target}) | {} | {} |",
                md_escape(&f.path),
                format_pct(tenths(f.marked_lines, f.eligible_lines)),
                ranges.join(", ")
            );
        }
        let unmarked = r.files.len() - marked.len();
        if unmarked > 0 {
            let _ = writeln!(
                md,
                "\n{} other scanned file{} {} no handwritten markings.",
                thousands(unmarked),
                if unmarked == 1 { "" } else { "s" },
                if unmarked == 1 { "has" } else { "have" }
            );
        }
        md.push('\n');
    }

    md.push_str(
        r#"## How this is measured

- **Eligible lines** are non-blank lines in scanned source files. Comments count; the marker comments themselves don't.
- **Marked lines** are eligible lines the maintainer declared handwritten, counted once even when declarations overlap.
- The percentage is rounded down, so 100.0% means every eligible line is marked.
- Files matching `scan.exclude` in `.wbh.toml`, gitignored files and hidden paths are not scanned.
- Unmarked code is just unmarked. It says nothing about how that code was written.

Declare a whole file with `wbh mark <file>`, or with a comment at the top of the file:

```rust
// @handwritten
```

Declare a block:

```python
# @handwritten:start
def handcrafted():
    return "hello"
# @handwritten:end
```

_Generated by `wbh scan`; run it again to update this report._
"#,
    );
    md
}

/// A link from the file at `from_rel` to `to_rel`, both root-relative.
pub fn relative_link(from_rel: &str, to_rel: &str) -> String {
    let depth = from_rel.matches('/').count();
    format!("{}{}", "../".repeat(depth), percent_encode(to_rel))
}

fn percent_encode(path: &str) -> String {
    let mut out = String::new();
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// Escapes text so Markdown renders it literally (including inside table cells).
pub fn md_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '\\' | '`' | '*' | '_' | '[' | ']' | '(' | ')' | '#' | '!' | '|' | '{' | '}' | '~' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

pub fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{FileResult, Stats};

    fn result() -> ScanResult {
        ScanResult {
            stats: Stats {
                files_scanned: 2,
                files_with_marks: 1,
                eligible_lines: 6,
                marked_lines: 3,
            },
            files: vec![
                FileResult {
                    path: "src/a b|[x].rs".into(),
                    physical_lines: 7,
                    eligible_lines: 4,
                    marked_lines: 3,
                    claim_sources: vec![ClaimSource::BlockMarker],
                    ranges: vec![(2, 4)],
                },
                FileResult {
                    path: "src/z.rs".into(),
                    physical_lines: 2,
                    eligible_lines: 2,
                    marked_lines: 0,
                    claim_sources: vec![],
                    ranges: vec![],
                },
            ],
            warnings: vec![],
        }
    }

    #[test]
    fn json_shape() {
        let j = json("Synth \"</script>\"", &result());
        let v: serde_json::Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["declaration"], "self-reported");
        assert_eq!(v["project"]["name"], "Synth \"</script>\"");
        assert_eq!(v["stats"]["percentage"], 50.0);
        assert_eq!(v["files"][0]["percentage"], 75.0);
        assert_eq!(v["files"][0]["claim_sources"][0], "block-marker");
        assert_eq!(v["files"][0]["ranges"][0][1], 4);
        assert!(j.ends_with("}\n"));
        assert!(j.find("schema_version").unwrap() < j.find("\"files\"").unwrap());

        let empty = ScanResult {
            stats: Stats::default(),
            files: vec![],
            warnings: vec![],
        };
        let v: serde_json::Value = serde_json::from_str(&json("e", &empty)).unwrap();
        assert!(v["stats"]["percentage"].is_null());
    }

    #[test]
    fn markdown_links_and_escaping() {
        let md = markdown("A*B", &result(), ".wbh/REPORT.md");
        assert!(md.starts_with("# Written by Hand — A\\*B\n"));
        assert!(md.contains(DISCLOSURE));
        assert!(md.contains("50.0% (3 marked / 6 eligible"));
        assert!(md.contains(
            "| [src/a b\\|\\[x\\].rs](../src/a%20b%7C%5Bx%5D.rs) | 75.0% | [L2–L4](../src/a%20b%7C%5Bx%5D.rs#L2-L4) |"
        ), "{md}");
        assert!(!md.contains("src/z.rs"));
        assert!(md.contains("1 other scanned file has no handwritten markings."));
        assert_eq!(
            relative_link("README.md", ".wbh/badge.svg"),
            ".wbh/badge.svg"
        );
        assert_eq!(relative_link("docs/a/README.md", "x.rs"), "../../x.rs");
        assert_eq!(thousands(1234567), "1,234,567");
        assert_eq!(thousands(999), "999");
    }
}
