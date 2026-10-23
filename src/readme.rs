//! The managed badge block in README.md. Only the block is ever touched: other bytes,
//! line endings and a BOM are preserved.

use crate::config::Config;
use crate::report::{md_escape, relative_link};
use anyhow::{Result, bail};

const START: &str = "<!-- wbh:badge:start -->";
const END: &str = "<!-- wbh:badge:end -->";

/// The block's lines, with links relative to the README's location.
pub fn block(cfg: &Config) -> [String; 3] {
    let from = &cfg.readme.path;
    [
        START.to_string(),
        format!(
            "[![{}]({})]({})",
            md_escape(&cfg.badge.text),
            relative_link(from, crate::config::BADGE_FILE),
            relative_link(from, crate::config::REPORT_MD)
        ),
        END.to_string(),
    ]
}

/// Returns `readme` with the managed block inserted or replaced.
pub fn apply(readme: &str, block: &[String]) -> Result<String> {
    let (bom, body) = match readme.strip_prefix('\u{feff}') {
        Some(rest) => ("\u{feff}", rest),
        None => ("", readme),
    };
    let nl = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = body.split_inclusive('\n').map(String::from).collect();
    let block: Vec<String> = block.iter().map(|l| format!("{l}{nl}")).collect();

    let mut starts = Vec::new();
    let mut ends = Vec::new();
    let mut h1 = None;
    let mut in_fence = false;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        match t {
            START => starts.push(i),
            END => ends.push(i),
            _ => {}
        }
        if h1.is_none() {
            let indent = line.len() - line.trim_start().len();
            if indent <= 3 && (t == "#" || t.starts_with("# ")) {
                h1 = Some(i);
            } else if i > 0
                && !t.is_empty()
                && t.chars().all(|c| c == '=')
                && !lines[i - 1].trim().is_empty()
            {
                h1 = Some(i); // setext: "Title\n====="
            }
        }
    }

    match (starts.as_slice(), ends.as_slice()) {
        ([s], [e]) if s < e => {
            lines.splice(s..=e, block);
        }
        ([], []) => {
            let at = match h1 {
                Some(i) => {
                    if !lines[i].ends_with('\n') {
                        lines[i].push_str(nl);
                    }
                    lines.insert(i + 1, nl.to_string());
                    i + 2
                }
                None => 0,
            };
            let next_is_blank = lines.get(at).is_none_or(|l| l.trim().is_empty());
            let n = block.len();
            lines.splice(at..at, block);
            if !next_is_blank {
                lines.insert(at + n, nl.to_string());
            }
        }
        _ => bail!(
            "README has malformed wbh badge markers (found {} start and {} end markers outside code blocks)\n  hint: fix or remove the lines {START} … {END} by hand",
            starts.len(),
            ends.len()
        ),
    }
    Ok(format!("{bom}{}", lines.concat()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block() -> Vec<String> {
        let mut cfg = Config::default();
        cfg.badge.text = "Hand [made]".into();
        super::block(&cfg).to_vec()
    }

    #[test]
    fn block_content() {
        assert_eq!(
            block()[1],
            "[![Hand \\[made\\]](.wbh/badge.svg)](.wbh/REPORT.md)"
        );
        let mut cfg = Config::default();
        cfg.readme.path = "docs/README.md".into();
        assert_eq!(
            super::block(&cfg)[1],
            "[![Written by Hand](../.wbh/badge.svg)](../.wbh/REPORT.md)"
        );
    }

    #[test]
    fn inserts_after_h1_and_is_idempotent() {
        let out = apply("# Title\nIntro\n", &block()).unwrap();
        assert_eq!(
            out,
            "# Title\n\n<!-- wbh:badge:start -->\n[![Hand \\[made\\]](.wbh/badge.svg)](.wbh/REPORT.md)\n<!-- wbh:badge:end -->\n\nIntro\n"
        );
        assert_eq!(apply(&out, &block()).unwrap(), out);

        // Replacing only touches the block.
        let changed = apply(
            &out,
            &[
                "<!-- wbh:badge:start -->".into(),
                "new".into(),
                "<!-- wbh:badge:end -->".into(),
            ],
        )
        .unwrap();
        assert_eq!(
            changed,
            "# Title\n\n<!-- wbh:badge:start -->\nnew\n<!-- wbh:badge:end -->\n\nIntro\n"
        );
    }

    #[test]
    fn insertion_positions() {
        let b = block();
        // No H1: top of file.
        assert!(
            apply("Some text\n", &b)
                .unwrap()
                .starts_with("<!-- wbh:badge:start -->\n")
        );
        assert!(
            apply("Some text\n", &b)
                .unwrap()
                .ends_with("end -->\n\nSome text\n")
        );
        // H1 at EOF without newline; empty file.
        assert!(apply("# T", &b).unwrap().starts_with("# T\n\n<!-- wbh"));
        assert!(apply("", &b).unwrap().starts_with("<!-- wbh"));
        // H1 after blank line: no doubled blank lines.
        assert_eq!(
            apply("# T\n\nBody\n", &b)
                .unwrap()
                .matches("\n\n\n")
                .count(),
            0
        );
        // Setext H1; ## is not an H1; # inside a fence is ignored.
        assert!(
            apply("Title\n=====\nx\n", &b)
                .unwrap()
                .starts_with("Title\n=====\n\n<!-- wbh")
        );
        assert!(
            apply("## Sub\n# Main\n", &b)
                .unwrap()
                .starts_with("## Sub\n# Main\n\n<!--")
        );
        assert!(
            apply("```\n# not\n```\n# Real\n", &b)
                .unwrap()
                .contains("# Real\n\n<!--")
        );
    }

    #[test]
    fn preserves_crlf_and_bom() {
        let out = apply("\u{feff}# T\r\nBody\r\n", &block()).unwrap();
        assert!(out.starts_with("\u{feff}# T\r\n\r\n<!-- wbh:badge:start -->\r\n"));
        assert!(out.ends_with("end -->\r\n\r\nBody\r\n"));
        assert!(!out.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn markers_in_code_fences_are_documentation() {
        let readme = "# T\n```markdown\n<!-- wbh:badge:start -->\n```\n";
        let out = apply(readme, &block()).unwrap();
        assert!(out.ends_with("```markdown\n<!-- wbh:badge:start -->\n```\n"));
        assert_eq!(out.matches(START).count(), 2);
    }

    #[test]
    fn malformed_blocks_are_errors() {
        for bad in [
            "<!-- wbh:badge:start -->\n",
            "<!-- wbh:badge:end -->\n<!-- wbh:badge:start -->\n",
            "<!-- wbh:badge:start -->\n<!-- wbh:badge:end -->\n<!-- wbh:badge:start -->\n<!-- wbh:badge:end -->\n",
        ] {
            assert!(apply(bad, &block()).is_err(), "{bad}");
        }
    }
}
