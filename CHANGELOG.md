# Changelog

## Unreleased

First public version.

- `wbh init`, `mark`, `unmark`, `scan`, `check`, and `badge` / `badge list` / `badge set <style> [--compact] [--no-percentage]`.
- `@handwritten` file and block markers for C-like, hash-comment, HTML/XML and CSS languages; whole-file claims via `.wbh.toml`.
- Self-declared coverage, rounded down so 100.0% only means every eligible line is marked. Empty projects report `N/A`, never 100%.
- Deterministic `.wbh/REPORT.md`, plus an opt-in `.wbh/report.json` (schema version 1).
- Zero-install badge gallery (`site/`, published with GitHub Pages): pick a style and label, then copy the Markdown.
- Six badge styles, each with compact and percentage variants: `stamp`, `classic`, `terminal`, `pixel`, `minimal`, `coffee`. The percentage is off by default.
- Managed README badge block that preserves the BOM, line endings and every byte outside the block.
- `wbh check` for CI validates markers and config; `--strict` also fails on stale outputs. CRLF differences from Windows checkouts are ignored.
- Installable with `cargo install` / `cargo binstall written-by-hand`.
- Release binaries for macOS (arm64, x86_64), Linux (x86_64, arm64; static musl) and Windows (x86_64), with SHA-256 checksums.
