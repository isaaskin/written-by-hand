# Markers

`wbh` recognizes `@handwritten` directives in comments. It is a comment-directive scanner, not a language parser. Handwritten status is **self-declared**: nothing is verified, and no AI detection is performed.

## Whole file

```rust
// @handwritten
fn main() {}
```

The marker must be in the file's **header**, with nothing before it except blank lines, a shebang (`#!/...`) and comments, including multi-line `/* ... */` or `<!-- ... -->` blocks such as license headers. Anything else ends the header, including Rust `#![...]` attributes and Python docstrings. A whole-file marker after that point is an error.

You can declare a whole file without editing it by running `wbh mark <file>`, which adds the file to `[claims].files` in `.wbh.toml`.

## Blocks

```python
# @handwritten:start
def handcrafted():
    return "hello"
# @handwritten:end
```

Every non-blank line strictly between `start` and `end` is marked. A file may contain any number of blocks.

The following are errors, reported as `path:line`, and the scan writes nothing:

- a `start` with no `end`
- an `end` with no `start`
- a `start` inside an open block (nesting is not supported)
- a second whole-file marker in the same file

A block with no non-blank lines inside it produces a warning.

## Comment forms

| Files | Marker line |
|---|---|
| `.c .h .cc .cpp .cxx .hpp .hxx .rs .swift .js .jsx .ts .tsx .go .java .kt .kts` | `// @handwritten:start` or `/* @handwritten:start */` |
| `.py .sh .bash .zsh .yml .yaml .rb .toml` | `# @handwritten:start` |
| `.html .htm .xml .md` | `<!-- @handwritten:start -->` |
| `.css .scss` | `/* @handwritten:start */` |
| `.json .jsonc` | no markers; use `wbh mark` |

The marker must fill the **entire line** apart from indentation and the comment wrapper. Whitespace inside the wrapper is optional. Markers are case-sensitive.

These lines are *not* markers:

```rust
const x = 1; // @handwritten:start     (code before the comment)
/// @handwritten:start                 (doc comment)
// This was @handwritten yesterday     (not the whole comment)
let s = "// @handwritten:start";       (inside a string on the same line)
```

A comment that starts with `@handwritten` but isn't one of the three exact forms (for example `// @handwritten:begin`) produces a warning.

## What counts

- **Eligible line:** a non-blank line in a scanned file that isn't a marker line. Comments, braces and shebangs count.
- **Marked line:** an eligible line in a claimed file, in a file with a whole-file marker, or inside a block. Overlapping declarations are counted once.
- **Coverage:** `marked / eligible`, rounded **down** to one decimal place, so `100.0%` only appears when every eligible line is marked. If a project has no eligible lines, coverage is `N/A`, never `100%`.

UTF-8 files are read with or without a BOM, with LF or CRLF line endings, and with or without a final newline. Files that are binary, not valid UTF-8, or larger than `scan.max_file_bytes` are skipped with a warning. If such a file is listed in `claims.files`, the warning says its claim has no effect.

## Known limitation: multi-line strings

There is no string lexing. A line *inside a multi-line string literal* that consists exactly of a marker comment is treated as a marker. For example:

```rust
let s = r#"
// @handwritten:start
"#;
```

This affects Rust raw strings, Python triple-quoted strings, C++ raw strings and JavaScript template literals. If a file contains such text, declare it with `wbh mark` instead of markers.
