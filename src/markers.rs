//! `@handwritten` directive recognition (spec §6). A directive must fill an entire
//! comment line; there is no string-literal lexing (see docs/MARKERS.md).

use crate::filter::Family;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Directive {
    File,
    Start,
    End,
}

#[derive(Debug, Default, PartialEq)]
pub struct Marks {
    /// One entry per physical line: non-blank and not a directive.
    pub eligible: Vec<bool>,
    pub file_marker: bool,
    /// Inclusive 1-based interiors of `start`/`end` pairs that hold eligible lines.
    pub blocks: Vec<(usize, usize)>,
    pub errors: Vec<(usize, String)>,
    pub warnings: Vec<(usize, String)>,
}

pub fn parse(text: &str, family: Family) -> Marks {
    let (line_comment, block_comment) = syntax(family);
    let mut m = Marks::default();
    let mut file_marker_line = None;
    let mut open: Option<usize> = None;
    let mut header = true;
    let mut in_block_comment = false;

    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let t = raw.trim();
        header = header && is_header_line(t, n, line_comment, block_comment, &mut in_block_comment);

        let inner = comment_inner(t, family);
        let directive = inner.and_then(|s| match s {
            "@handwritten" => Some(Directive::File),
            "@handwritten:start" => Some(Directive::Start),
            "@handwritten:end" => Some(Directive::End),
            _ => None,
        });
        m.eligible.push(directive.is_none() && !t.is_empty());

        match directive {
            None => {
                if inner.is_some_and(|s| s.starts_with("@handwritten")) {
                    m.warnings.push((n, "unrecognized @handwritten directive; expected @handwritten, @handwritten:start or @handwritten:end".into()));
                }
            }
            Some(Directive::File) => {
                if !header {
                    m.errors.push((n, "file marker must be in the header, before any code\n  hint: move it to the top of the file, or use @handwritten:start / @handwritten:end for a block".into()));
                } else if let Some(first) = file_marker_line {
                    m.errors.push((n, format!("duplicate file marker (first one is on line {first})")));
                } else {
                    file_marker_line = Some(n);
                }
            }
            Some(Directive::Start) => match open {
                Some(s) => m.errors.push((n, format!("nested @handwritten:start (block already opened on line {s}); nesting is not supported"))),
                None => open = Some(n),
            },
            Some(Directive::End) => match open.take() {
                None => m.errors.push((n, "@handwritten:end without a matching @handwritten:start".into())),
                Some(s) => {
                    if m.eligible[s..n - 1].contains(&true) {
                        m.blocks.push((s + 1, n - 1));
                    } else {
                        m.warnings.push((s, "empty @handwritten block marks no lines".into()));
                    }
                }
            },
        }
    }
    if let Some(s) = open {
        m.errors.push((s, "unclosed @handwritten:start\n  hint: add a matching @handwritten:end after the block, or remove the start marker".into()));
    }
    m.file_marker = file_marker_line.is_some();
    m
}

type BlockComment = (&'static str, &'static str);

fn syntax(family: Family) -> (Option<&'static str>, Option<BlockComment>) {
    match family {
        Family::CLike => (Some("//"), Some(("/*", "*/"))),
        Family::Hash => (Some("#"), None),
        Family::Html => (None, Some(("<!--", "-->"))),
        Family::Css => (None, Some(("/*", "*/"))),
        Family::Json => (None, None),
    }
}

/// The trimmed text inside a whole-line comment, if `t` is one for `family`.
fn comment_inner(t: &str, family: Family) -> Option<&str> {
    let (line, block) = syntax(family);
    line.and_then(|open| t.strip_prefix(open))
        .or_else(|| block.and_then(|(open, close)| t.strip_prefix(open)?.strip_suffix(close)))
        .map(str::trim)
}

/// Whether line `n` keeps the leading header area going: blank lines, a shebang,
/// and comments (including multi-line block comments) do; code ends it.
fn is_header_line(
    t: &str,
    n: usize,
    line_comment: Option<&str>,
    block_comment: Option<BlockComment>,
    in_block: &mut bool,
) -> bool {
    if let Some((_, close)) = block_comment.filter(|_| *in_block) {
        if t.contains(close) {
            *in_block = false;
            return t.ends_with(close);
        }
        return true;
    }
    if t.is_empty() || (n == 1 && t.starts_with("#!") && !t.starts_with("#![")) {
        return true;
    }
    if line_comment.is_some_and(|c| t.starts_with(c)) {
        return true;
    }
    if let Some((open, close)) = block_comment
        && let Some(rest) = t.strip_prefix(open)
    {
        if rest.contains(close) {
            return t.ends_with(close);
        }
        *in_block = true;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use Family::*;

    fn counts(m: &Marks) -> usize {
        m.eligible.iter().filter(|e| **e).count()
    }

    #[test]
    fn spec_fixture_15_3() {
        let m = parse(
            "// @handwritten:start\nfn main() {\n    println!(\"hello\");\n}\n// @handwritten:end\n\nfn other() {}\n",
            CLike,
        );
        assert_eq!(m.eligible, [false, true, true, true, false, false, true]);
        assert_eq!(m.blocks, [(2, 4)]);
        assert!(m.errors.is_empty() && !m.file_marker);
    }

    #[test]
    fn comment_wrappers_per_family() {
        let ok = [
            (CLike, "// @handwritten:start"),
            (CLike, "/* @handwritten:start */"),
            (CLike, "    //@handwritten:start"),
            (Hash, "#   @handwritten:start"),
            (Html, "<!-- @handwritten:start -->"),
            (Css, "/*@handwritten:start*/"),
        ];
        for (fam, line) in ok {
            let m = parse(
                &format!("{line}\nx\n{}\n", line.replace("start", "end")),
                fam,
            );
            assert_eq!(m.blocks, [(2, 2)], "{fam:?} {line}");
        }
        let not_markers = [
            (Hash, "// @handwritten:start"),
            (Css, "// @handwritten:start"),
            (CLike, "# @handwritten:start"),
            (Json, "// @handwritten:start"),
            (CLike, "/// @handwritten:start"),
            (CLike, "const x = 1; // @handwritten:start"),
            (CLike, "let s = \"// @handwritten:start\";"),
            (Hash, "x = '# @handwritten:start'"),
            (CLike, "// This was @handwritten yesterday"),
            (CLike, "// @Handwritten:start"),
        ];
        for (fam, line) in not_markers {
            let m = parse(&format!("{line}\n"), fam);
            assert_eq!(m.eligible, [true], "{fam:?} {line}");
            assert!(m.errors.is_empty() && m.blocks.is_empty(), "{fam:?} {line}");
        }
    }

    #[test]
    fn file_marker_header_rules() {
        let header_ok = [
            (CLike, "// @handwritten\nint x;\n"),
            (
                CLike,
                "/*\n * MIT License\n */\n\n// @handwritten\nint x;\n",
            ),
            (
                Hash,
                "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n\n# @handwritten\nx = 1\n",
            ),
            (
                Html,
                "<!--\n  license\n-->\n<!-- @handwritten -->\n<div></div>\n",
            ),
        ];
        for (fam, text) in header_ok {
            let m = parse(text, fam);
            assert!(
                m.file_marker && m.errors.is_empty(),
                "{text:?}: {:?}",
                m.errors
            );
        }
        let header_bad = [
            (CLike, "int x;\n// @handwritten\n"),
            (CLike, "#![allow(dead_code)]\n// @handwritten\n"),
            (CLike, "/* a */ int x;\n// @handwritten\n"),
            (Hash, "\"\"\"docstring\"\"\"\n# @handwritten\n"),
        ];
        for (fam, text) in header_bad {
            let m = parse(text, fam);
            assert!(
                !m.file_marker && m.errors[0].1.contains("header"),
                "{text:?}"
            );
        }
        // Long license headers are fine: there is no line limit, only "before any code".
        let long = format!("{}// @handwritten\nint x;\n", "// license\n".repeat(40));
        assert!(parse(&long, CLike).file_marker);

        let m = parse("// @handwritten\n// @handwritten\nx\n", CLike);
        assert_eq!(m.errors.len(), 1);
        assert_eq!(m.errors[0].0, 2);
        assert!(m.errors[0].1.contains("duplicate"));
    }

    #[test]
    fn structural_errors_carry_line_numbers() {
        let m = parse("a\n// @handwritten:start\nb\n", CLike);
        assert_eq!(m.errors[0].0, 2);
        assert!(m.errors[0].1.contains("unclosed"));

        let m = parse("a\n// @handwritten:end\n", CLike);
        assert_eq!(m.errors[0].0, 2);
        assert!(m.errors[0].1.contains("without a matching"));

        let m = parse(
            "// @handwritten:start\n// @handwritten:start\nx\n// @handwritten:end\n",
            CLike,
        );
        assert_eq!(m.errors[0].0, 2);
        assert!(m.errors[0].1.contains("nested"));
    }

    #[test]
    fn empty_blocks_and_lookalikes_warn() {
        let m = parse(
            "// @handwritten:start\n\n   \n// @handwritten:end\nx\n",
            CLike,
        );
        assert!(m.blocks.is_empty() && m.errors.is_empty());
        assert_eq!(m.warnings.len(), 1);
        let m = parse("// @handwritten:begin\n", CLike);
        assert!(m.warnings[0].1.contains("unrecognized"));
    }

    #[test]
    fn line_endings_and_edges() {
        let lf = parse("// @handwritten:start\na\n// @handwritten:end\nb", CLike);
        let crlf = parse(
            "// @handwritten:start\r\na\r\n// @handwritten:end\r\nb",
            CLike,
        );
        assert_eq!(lf, crlf);
        assert_eq!(lf.eligible.len(), 4);
        assert_eq!(parse("", CLike).eligible.len(), 0);
        let m = parse("// @handwritten\n\n  \n", CLike);
        assert_eq!(counts(&m), 0);
        assert!(m.file_marker);
        let m = parse(
            "// @handwritten:start\nlet π = \"✍\";\n// @handwritten:end\n",
            CLike,
        );
        assert_eq!(m.blocks, [(2, 2)]);
    }
}
