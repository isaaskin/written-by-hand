# Written by Hand — Implementation Specification

> **Agent-ready engineering plan** · Working project name: **Written by Hand** · CLI: **`wbh`** · Primary language: **Rust** · License: **MIT**
>
> **Product tagline:** *A tiny badge for code made the old-fashioned way.*
>
> **Status:** Implementation specification, not an implementation. Follow the decisions in this document unless a clearly documented technical blocker requires a change.

---

## 0. Instructions for the coding agent

You are building a complete, maintainable, cross-platform open-source developer tool called **Written by Hand**. **Begin implementing Phase 1 immediately**. Do not begin by creating a complex planning framework or asking open-ended product questions. This document defines the intended behavior, architecture, acceptance tests, and rollout sequence.

1. Read the entire specification before editing code.
2. Inspect the existing repository, if any, and preserve useful existing work. If it is empty, initialize a Rust project as described below.
3. Implement milestones in order; **fully complete and test the current milestone before advancing**.
4. Prioritize a small, reliable single binary over an elaborate framework. Do not add a backend, account system, AI detection, or runtime dependency on Node/Python.
5. Commit deterministic, human-readable generated assets and provide runnable examples.
6. Use idiomatic Rust, readable modules, strong error messages, integration tests, and secure handling of untrusted paths/content.
7. If a library or tooling decision becomes obsolete or unsupported, choose the closest maintained alternative and record the difference in `docs/DECISIONS.md`.
8. Do not claim successful compilation or tests without running them. If the environment cannot run them, report the exact blocker.
9. When finished with a milestone, report: implemented commands, files changed, tests executed, failures, and next remaining work.

### Minimum first action

```sh
cargo init --bin --name written-by-hand .
```

If a Cargo project already exists, **do not overwrite it**. Inspect it first. The final executable must be named **`wbh`** (see Cargo configuration).

### Non-negotiable product constraints

- No AI-generated-code detection. **Handwritten status is a self-declaration**, not a verified provenance claim.
- No user accounts, hosted API, database, telemetry, or network calls in normal `wbh` operations.
- Build and run as a native executable on macOS, Linux, and Windows.
- Do not require users to install Node.js, Python, Docker, or Rust to run released binaries.
- Never modify user source code during `init`, `scan`, `check`, `badge`, `report`, or `site build`.
- `mark` / `unmark` modify only `.wbh.toml`, **not source files**. Developers insert or remove in-source comment markers themselves.
- Generated SVGs must be self-contained and suitable for GitHub README rendering.
- Do not classify unmarked code as AI-written. Use **marked** vs **unmarked** terminology.
- Respect `.gitignore` and explicit exclusions, and never scan outside the selected repository root.
- All generated artifacts must be stable for identical input (no unstable ordering or default timestamps).

---

## 1. Product overview

### 1.1 What it does

Written by Hand lets developers mark files or code blocks that they personally wrote, choose from several attractive badge designs, and add a clickable badge to their GitHub README. The badge opens a report showing which source files and line ranges they declared handwritten. An optional static HTML **Code Map** provides an interactive visual exploration of those declarations.

It is intentionally a small, humorous developer-culture utility, not a serious certification service.

Example copy:

- **Written by Hand**
- *Proudly written by hand.*
- *Some things are still better made by hand.*
- *Made with fingers and questionable decisions.*
- *Artisanal bugs since 2026.*
- *Certified by Me™* — optional joke style only; never describe it as independent certification.

### 1.2 Intended users

- Maintainers of open-source projects who want a playful README badge.
- Developers who write some modules manually and want to distinguish them from **unmarked** modules.
- Creators of hobby projects who want a craft/handmade aesthetic.

### 1.3 Core user journey

1. Download a precompiled `wbh` binary.
2. Open a Git repository and run `wbh init`.
3. Choose a badge design, or use the noninteractive default.
4. Mark files with `wbh mark src/file.rs`, or add `@handwritten` comments to file headers / code blocks.
5. Run `wbh scan`.
6. Commit `.wbh.toml`, `.wbh/badge.svg`, `.wbh/report.json`, `.wbh/REPORT.md`, and the README change.
7. Visitors see the badge and click it to view the report.
8. Optionally generate and publish an interactive, static Code Map.

### 1.4 Product philosophy

**Small, fun, opt-in, transparent, offline, local-first, zero-account, single-binary.**

This project should feel like a delightful GitHub easter egg, not compliance software.

---

## 2. Scope, releases, and priorities

| Priority | Feature | Target |
|---|---|---|
| P0 | Rust single-binary CLI | v0.1 |
| P0 | TOML config and defaults | v0.1 |
| P0 | File-level claims in config | v0.1 |
| P0 | File-level and block-level comment markers | v0.1 |
| P0 | Source scanning, ignored files, accurate line coverage | v0.1 |
| P0 | JSON report and Markdown report | v0.1 |
| P0 | At least three SVG badge styles | v0.1 |
| P0 | Safe, idempotent README injection | v0.1 |
| P0 | `init`, `mark`, `unmark`, `scan`, `badge`, `report`, `check` | v0.1 |
| P0 | Unit/integration tests and sample repos | v0.1 |
| P1 | All six badge designs and compact variants | v0.5 |
| P1 | Cross-platform release binaries and installers | v0.5 |
| P1 | GitHub Actions validation example | v0.5 |
| P1 | Interactive static Code Map generator | v1.0 |
| P1 | Optional GitHub Pages publishing workflow | v1.0 |
| P2 | More languages and extensive customization | post-v1 |
| P2 | Watch mode, shell completion, richer previews | post-v1 |

**Explicitly out of scope:** AI detectors, server-side verification, provenance attestation, user accounts, GitHub OAuth, database, hosted dashboard, VS Code extension, AST-based parsing, payments, cloud API, browser extension, and a GUI badge editor.

---

## 3. Technology and architecture

### 3.1 Language and build

- Stable Rust, edition 2024 if supported by the selected toolchain, otherwise 2021.
- One Cargo package and one primary binary named `wbh`.
- Commit `Cargo.lock` for reproducible builds.
- Default release profile should optimize executable size reasonably, but do not sacrifice straightforward debugging during development.
- Keep the project backend-free and without JavaScript runtime requirements for the CLI.
- The **optional viewer** uses vanilla HTML/CSS/JavaScript generated as static assets; its JS executes only in the viewer browser, not in the CLI.

### 3.2 Suggested Rust crates

| Purpose | Preferred crate | Notes |
|---|---|---|
| CLI parser | `clap` (derive) | Subcommands, documented flags |
| Structured errors | `anyhow`, `thiserror` | Context for IO and validation |
| Serialization | `serde`, `serde_json` | Stable report format |
| Config | `toml`, `toml_edit` | Preserve config formatting/comments when marking |
| File traversal | `ignore` | Honor `.gitignore` by default |
| Pattern matching | `globset` | Explicit include/exclude globs |
| Terminal prompts | `dialoguer` or a minimal equivalent | Only if interactive TTY |
| Path normalization | Standard library, plus small helper if needed | Never accept escaping repo root |
| Tests | `tempfile`, `assert_cmd`, `predicates`, snapshots optional | Integration fixtures |

Prefer a conservative dependency graph. **Do not introduce a template engine** for half a dozen SVG templates unless string-based rendering becomes genuinely awkward. Escape XML properly and test it. Bundle SVG/HTML assets with `include_str!` or `include_bytes!`.

### 3.3 Module structure

```text
written-by-hand/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
├── CHANGELOG.md
├── docs/
│   ├── DECISIONS.md
│   ├── MARKERS.md
│   ├── CONFIG.md
│   ├── SECURITY.md
│   └── ROADMAP.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── error.rs
│   ├── config.rs
│   ├── model.rs
│   ├── paths.rs
│   ├── scanner/
│   │   ├── mod.rs
│   │   ├── discover.rs
│   │   ├── languages.rs
│   │   ├── markers.rs
│   │   └── coverage.rs
│   ├── badges/
│   │   ├── mod.rs
│   │   └── renderer.rs
│   ├── reports/
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   └── markdown.rs
│   ├── github/
│   │   ├── mod.rs
│   │   ├── readme.rs
│   │   └── links.rs
│   └── site/
│       ├── mod.rs
│       └── generator.rs
├── assets/
│   ├── badges/
│   │   ├── stamp.svg
│   │   ├── classic.svg
│   │   ├── terminal.svg
│   │   ├── pixel.svg
│   │   ├── minimal.svg
│   │   └── coffee.svg
│   └── site/
│       ├── index.html
│       ├── style.css
│       └── app.js
├── tests/
│   ├── cli_init.rs
│   ├── cli_mark.rs
│   ├── scan_fixtures.rs
│   ├── readme.rs
│   ├── security.rs
│   └── check.rs
└── examples/
    ├── cpp-project/
    ├── rust-project/
    └── python-project/
```

**Do not create empty placeholder modules or docs just to reproduce the tree.** Create them when their actual functionality is implemented.

### 3.4 Data flow

```text
User config + source tree
          │
          ▼
  File discovery + exclusions
          │
          ▼
  Comment marker parser
          │
          ▼
  Line eligibility + interval union
          │
          ▼
   In-memory ScanResult
          │
          ├────────► report.json
          ├────────► REPORT.md
          ├────────► badge.svg
          └────────► optional static HTML Code Map
```

**`ScanResult` is the single source of truth** for all outputs. Renderers must not independently re-scan or compute alternative percentages.

---

## 4. CLI contract

All commands support `--help` and `--version` where appropriate. A global `--root <path>` selects repository root and defaults to the current working directory; canonicalize it before scanning. Use a root-independent internal path representation with `/` separators even on Windows.

### 4.1 Commands

| Command | Behavior | Source modifications? |
|---|---|---|
| `wbh init` | Create config, initial outputs, insert README badge | No |
| `wbh mark <path>...` | Add explicit file claim(s) in config | No |
| `wbh unmark <path>...` | Remove explicit file claim(s) in config | No |
| `wbh scan` | Validate, scan source, regenerate configured outputs | No |
| `wbh badge --style <style>` | Change style in config and regenerate outputs | No |
| `wbh badge --list` | List styles, descriptions, variants | No |
| `wbh report` | Regenerate report via same scanner pipeline; also keep badge synchronized | No |
| `wbh check` | Recompute expectations and verify generated outputs without writing | No |
| `wbh site build` | Generate static Code Map from a fresh scan | No |

`report` is intentionally a user-friendly alias of `scan`, reporting the Markdown/JSON output locations; do not develop divergent scan behavior. `badge --style` writes `.wbh.toml`, then invokes the same regeneration pipeline.

### 4.2 `wbh init`

```sh
wbh init
wbh init --style stamp
wbh init --style classic --no-readme
wbh init --style terminal --yes
wbh --root /path/to/repo init
```

Behavior:

1. Detect whether `.wbh.toml` already exists. If yes, do not overwrite it. Explain next commands. Optional `--force` is **not** required for v0.1.
2. Detect whether `stdin` and `stderr` are interactive TTYs. When interactive and `--style` is absent, present numbered badge choices. In CI/noninteractive contexts, default to `stamp`.
3. Create `.wbh.toml` using conservative defaults.
4. Scan files; create `.wbh/` outputs even if no file is marked. If there are zero eligible lines, display `N/A` rather than dividing by zero.
5. When `README.md` exists, add one managed Markdown badge section immediately following the first H1 heading if present, otherwise near the beginning. If the file does not exist, **do not create one silently**: print a notice and show the exact Markdown snippet to insert later.
6. `--no-readme` skips README editing entirely. `--readme <path>` can opt into an alternate Markdown file under the root (optional for v0.1, recommended).
7. `--yes` disables prompts, using specified options or defaults.
8. Print created/updated paths and suggested next steps.

### 4.3 `wbh mark` / `wbh unmark`

```sh
wbh mark src/dsp/Oscillator.cpp
wbh mark src/dsp/Oscillator.cpp src/dsp/Chorus.cpp
wbh unmark src/dsp/Oscillator.cpp
```

- Only **repository-relative regular files** inside the root are supported.
- Normalize separators to `/` in `.wbh.toml`.
- Reject missing paths, directories, paths outside root, symlinks resolving outside root, and duplicate entries (duplicates should be harmless and idempotent, not repeated).
- Preserve unrelated TOML values and comments with `toml_edit`.
- `unmark` removes only config claims. If a file contains an in-source `@handwritten` marker, explain why it is still counted and instruct the user to remove the comment manually.
- Do not silently mark whole directory trees. A `mark-dir` convenience command, if ever introduced, must preview its expansion and be opt-in.
- A path excluded by scanner configuration should be rejected with an actionable error rather than silently accepted as a claim that has no effect.
- `mark` and `unmark` do not automatically commit or run Git commands.

### 4.4 `wbh scan`

```sh
wbh scan
wbh scan --json              # Optional: print concise JSON summary to stdout
wbh scan --quiet             # Optional: suppress normal output
```

Steps:

1. Load and validate config.
2. Discover eligible source files.
3. Parse markers and file claims; fail on structural marker errors.
4. Compute line-level coverage.
5. Assemble deterministic `ScanResult`.
6. Render JSON, Markdown, SVG and, **only if enabled**, optional viewer assets.
7. Stage rendered contents and write only when different; do not rewrite identical files (preserve modification times).
8. Print a short summary (total files, eligible lines, marked lines, percentage or N/A, artifact locations).

Failure rules: failed config validation, unsafe output paths, malformed markers, unreadable required source files, or render errors return a nonzero exit status. Avoid truncating any previous valid output if scan fails. Use temporary files and atomic replacement for each artifact where platform behavior permits; there is no cross-file transaction guarantee.

### 4.5 `wbh check`

Recompute the complete expected output set in memory and compare against existing files, **without writing**.

- Exit `0` if markers/config are valid and all tracked generated artifacts match.
- Exit `1` for invalid config/markers or stale/missing generated outputs, with actionable diagnostics.
- Use exit `2` for invalid invocation if that is practical, but document actual implemented codes.
- Designed for CI, e.g. `wbh check` on pull requests.
- Do not make any network requests or edit README in `check` mode.
- Validate that the existing README managed badge block (if init configured it) points at the expected generated badge/report paths; do not require a README in repositories that opted out.

### 4.6 `wbh site build`

- Available in Phase 3 (v1.0).
- Performs a **fresh scan** (never trust a potentially stale JSON report) and renders the static site.
- Default site contains **only metadata** (file paths, line spans, counts, GitHub links), not source code contents.
- Explicit `--embed-source` or `[site].embed_source = true` opts in to embedding source snippets (see security section).
- Output directory defaults to `.wbh/site/` and must remain inside the repository root.
- Never publish/deploy automatically; generating the site is a local action. Provide documentation and an **optional sample** GitHub Pages workflow.

---

## 5. Configuration schema

Config path: `<root>/.wbh.toml`. Format: UTF-8 TOML. Unknown top-level sections/fields should produce a useful warning or validation error according to consistent policy (prefer errors for mistyped supported keys). `version = 1` permits future migration.

### 5.1 Full example

```toml
version = 1

[project]
name = "Little Angel Synth"

[badge]
style = "stamp"
show_percentage = true
compact = false
text = "Written by Hand"
output = ".wbh/badge.svg"

[scan]
include = ["**/*"]
exclude = [
  "**/generated/**",
  "**/vendor/**",
  "**/third_party/**",
  "**/build/**",
  "**/dist/**"
]
respect_gitignore = true
max_file_bytes = 2097152

[claims]
files = [
  "src/dsp/Oscillator.cpp",
  "src/dsp/Chorus.cpp"
]

[report]
json_output = ".wbh/report.json"
markdown_output = ".wbh/REPORT.md"

[readme]
enabled = true
path = "README.md"

[site]
enabled = false
output_dir = ".wbh/site"
embed_source = false
```

### 5.2 Defaults

| Field | Default |
|---|---|
| `version` | `1` |
| `project.name` | Directory basename, optionally detected from Git remote |
| `badge.style` | `stamp` |
| `badge.show_percentage` | `true` |
| `badge.compact` | `false` |
| `badge.text` | `Written by Hand` |
| `badge.output` | `.wbh/badge.svg` |
| `scan.include` | `["**/*"]` filtered by known extensions |
| `scan.exclude` | Build/deps/generated defaults plus `.wbh/**`, `.git/**` |
| `scan.respect_gitignore` | `true` |
| `scan.max_file_bytes` | `2097152` (2 MiB) |
| `claims.files` | `[]` |
| `report.json_output` | `.wbh/report.json` |
| `report.markdown_output` | `.wbh/REPORT.md` |
| `readme.enabled` | `true` |
| `readme.path` | `README.md` |
| `site.enabled` | `false` |
| `site.output_dir` | `.wbh/site` |
| `site.embed_source` | `false` |

### 5.3 Config validation

- All output paths and input paths must remain under canonical root; reject `..` escape, absolute output paths, and symlink escapes.
- Avoid duplicate output destinations, e.g. JSON and badge must never resolve to the same file.
- Prevent scanning `.wbh/`, `.git/`, common VCS/build/vendor directories and any configured output directory even if include globs match them.
- Reject unknown badge style and malformed glob patterns before touching outputs.
- Do not expand environment variables or execute shell expressions from config.
- Scan extensions are selected from the supported language table below. Unknown binary-looking files are excluded by default. To explicitly claim a non-source text file, a future `scan.extensions` override may be added; not needed for v0.1.

---

## 6. Marker specification (normative)

The **marker syntax is the product's core contract**. Use `@handwritten` as the recognizable token; only parse it inside allowed whole-line comments for each language family.

### 6.1 File-level declaration

```cpp
// @handwritten
#include <cmath>

float oscillator(float x) { return std::sin(x); }
```

```python
# @handwritten

def run():
    return 42
```

```html
<!-- @handwritten -->
<div>Handcoded</div>
```

A whole-file marker must occur in the **leading comment/header area**, no later than the first 20 physical lines, and **before the first non-comment code line**. Blank lines, shebang lines, and contiguous header comments do not end the header area. Non-header `@handwritten` with no suffix is invalid to avoid surprising whole-file declarations in the middle of a file. When present, all eligible lines in that file are marked, independently of any block markers or config claims.

A whole-file config claim (`[claims].files`) has identical coverage semantics without adding comments to the file.

### 6.2 Block-level declaration

```cpp
class SynthEngine {
public:
    // @handwritten:start
    float process(float phase) {
        return sinf(phase * 6.283185f);
    }
    // @handwritten:end
};
```

```python
# @handwritten:start
def handcrafted():
    return "hello"
# @handwritten:end
```

Definitions:

- `@handwritten:start` opens a block. `@handwritten:end` closes it.
- Marker comment lines themselves never count toward eligible/marked totals.
- Every eligible line strictly **between** the open and close markers is marked.
- Multiple, separate block spans per file are allowed.
- **Nested blocks are invalid** for v0.1, with clear path and physical line number.
- Orphan `end`, unclosed `start`, multiple file-level markers (duplicate whole-file declarations), and marker mismatches are errors.
- A whole-file claim plus valid block spans is permitted, but the file is counted only once; do not double count lines.
- Empty spans are permitted with zero marked lines but may produce a warning.
- A file-level config claim does not excuse malformed block markers; parsing must still validate the file.

### 6.3 Supported comment formats

| Family | Extensions | Allowed whole-line marker comment |
|---|---|---|
| C-like | `.c`, `.h`, `.cc`, `.cpp`, `.cxx`, `.hpp`, `.hxx`, `.rs`, `.swift`, `.js`, `.jsx`, `.ts`, `.tsx`, `.go`, `.java`, `.kt`, `.kts` | `// @handwritten:start` or `/* @handwritten:start */` |
| Hash | `.py`, `.sh`, `.bash`, `.zsh`, `.yml`, `.yaml`, `.rb`, `.toml` | `# @handwritten:start` |
| HTML/XML-like | `.html`, `.htm`, `.xml`, `.md` | `<!-- @handwritten:start -->` |
| CSS | `.css`, `.scss` | `/* @handwritten:start */` |
| JSON | `.json`, `.jsonc` | Config claims only for `.json`; optional `//` markers in `.jsonc` if explicitly implemented |

Replace `:start` with `:end` or remove suffix for whole-file declaration. Whitespace before/after marker within the enclosing comment is allowed. A marker must occupy the entire logical comment line; for example `const x = 1; // @handwritten:start` **must not count**.

**Parser limitations and safety:** Recognizing comment-looking lines inside multi-line string literals is a risk if using simple regexes. Do not silently make an inaccurate claim. Implement a lightweight lexical state machine for common strings/raw strings/templates where feasible for v0.1, and document exactly which constructs are handled. At minimum, never match when a marker is part of an ordinary quoted string on the same line. Add regression tests for Rust raw strings, Python triple-quoted strings, C/C++ raw strings and JavaScript template strings. If full lexical coverage is deferred, explicitly surface documented limitations and provide an option to use **config-only claims** to avoid ambiguous markers.

**Avoid claiming a full language parser:** This is a language-aware comment directive scanner, not a compiler.

### 6.4 Whitespace, encoding, and line endings

- Read UTF-8 source files. Support UTF-8 BOM; preserve original source untouched.
- Support LF and CRLF. A final line without newline is a valid physical line.
- Reject invalid UTF-8 source files in the eligible extension set with a useful message, or explicitly skip with a diagnostic under a documented policy. Prefer failing for files with explicit markers/claims.
- Case-sensitive markers: `@handwritten` exactly. Do not recognize natural-language uses like `// This was @handwritten yesterday`.
- Directives may be indented in the file.
- Marker recognition uses exact token and supported comment wrapper; whitespace is normalized but not spelling/case.

### 6.5 Example behavior table

| Situation | Result |
|---|---|
| `// @handwritten` in top header | File marked |
| Same file listed in config | No double count |
| Valid `start` + `end` | Interior eligible lines marked |
| `start` without `end` | Error with path:line |
| Orphan `end` | Error with path:line |
| Nested starts | Error with path:line |
| `// @handwritten` after real code | Error: file marker must be in header |
| Marker-looking text inside normal string literal | Not a marker |
| Unsupported comment form for extension | Not a marker |
| File only containing marker comments / blank lines | 0 eligible lines, coverage N/A |

---

## 7. File discovery, eligibility, and coverage

### 7.1 Which files count?

Default behavior:

1. Start from the canonical root.
2. Respect `.gitignore` / `.ignore` via `ignore` crate when configured; do not traverse `.git` or generated output directories.
3. Apply include/exclude globs. **Exclude wins**.
4. Restrict the set to explicitly supported source/text extensions (JSON can be included but has config-only claims).
5. Skip non-regular files, binary files, symlink traversal, and files over `max_file_bytes` unless user explicitly changes the size policy.
6. Sort all root-relative normalized paths lexicographically before computing/rendering.

**Do not include README.md and project documentation as source coverage by default**, even though Markdown marker comments can be parsed when a user explicitly opts a Markdown source file into scanning. Avoid recursive self-measurement when README contains the badge. Recommended default exclusions include `README.md`, `CHANGELOG.md`, `LICENSE*`, `docs/**`, `.github/**`, `.wbh/**`, `node_modules/**`, `target/**`, `vendor/**`, `third_party/**`, `build/**`, `dist/**`. An explicitly configured include pattern may opt ordinary docs back in, but may **never** override exclusions for `.git`, `.wbh`, output paths, or unsafe files.

### 7.2 Eligible line definition

A line is **eligible** if and only if:

1. It is in a discovered eligible file;
2. It contains at least one non-whitespace character; and
3. It is **not** an exact recognized marker directive line.

Normal source comments are eligible (this tool measures marked source text lines, not AST/executable LOC). This definition is intentionally simple, language-agnostic after marker recognition, and easy to test. A line with only braces is eligible. Shebangs and regular comments are eligible. Blank lines and marker lines are not.

### 7.3 Marked line definition

An eligible line is **marked** if it belongs to the union of any of these declarations:

- Its file is in `[claims].files`;
- Its file has a valid header `@handwritten`;
- Its physical line number lies between a valid block marker pair.

Compute a set/union of marked physical line numbers, then intersect it with eligible lines; never sum overlapping ranges blindly.

### 7.4 Percentage

```text
eligible_lines = count(all eligible lines across eligible files)
marked_lines   = count(all eligible lines in the union of declarations)
percentage     = round(marked_lines * 100 / eligible_lines, 1 decimal)
```

Example: `1800 / 2400 = 75.0%`.

When `eligible_lines == 0`, JSON percentage must be `null`, terminal shows `N/A`, and badge shows `—` or omits the percentage. **Never interpret empty project as 100% handwritten.**

A more accurate label for UI explanations is **“Self-declared handwritten coverage.”** The shorthand `handwritten` can be used on badge designs, but the report must explain the metric.

### 7.5 Ignore and exclusion behavior

- Always skip `.wbh/` and output directories to prevent feedback loops.
- Apply `.gitignore` even outside a Git repo where the crate supports it.
- Hidden files are not automatically included; document a consistent choice. Ensure source under common dot-directories does not accidentally enter coverage unless requested.
- A config claim targeting a missing, ignored, excluded, binary, or oversized source must produce a diagnostic rather than inflating coverage invisibly.
- Extremely large/unreadable files should not crash or hang the scanner.
- Never read files outside root via symlink, path traversal, or malformed config.

### 7.6 Report precision

Store physical line ranges in reports. Counting is performed on eligible lines, so range boundaries may include intervening blank lines; report marked line counts must still exclude them. If merging adjacent/overlapping intervals for display, preserve correct physical boundaries.

---

## 8. Data model and JSON schema contract

The `ScanResult` and `FileResult` should be explicit Rust structs. Use `serde` and structured validation. The report format is versioned.

### 8.1 Example `.wbh/report.json`

```json
{
  "schema_version": 1,
  "project": {
    "name": "Little Angel Synth"
  },
  "declaration": "self-reported",
  "stats": {
    "files_scanned": 2,
    "files_with_marks": 2,
    "eligible_lines": 360,
    "marked_lines": 320,
    "percentage": 88.9
  },
  "files": [
    {
      "path": "src/dsp/Oscillator.cpp",
      "physical_lines": 320,
      "eligible_lines": 280,
      "marked_lines": 280,
      "percentage": 100.0,
      "claim_sources": ["config"],
      "ranges": [[1, 320]]
    },
    {
      "path": "src/ui/MainView.swift",
      "physical_lines": 100,
      "eligible_lines": 80,
      "marked_lines": 40,
      "percentage": 50.0,
      "claim_sources": ["block-marker"],
      "ranges": [[12, 62]]
    }
  ]
}
```

The values above are **illustrative**, not computed by the product. The two example files sum to 360 eligible lines and 320 marked lines (88.9% rounded to one decimal). In real output, every figure must be internally consistent. If an example uses entire-file range `[1, 320]`, this means declared coverage spans physical lines 1–320, while only 280 eligible lines count. A block span should use exact interior physical boundaries, not the marker lines. (For example, if `start` is on line 11 and `end` is on line 63, the interior range is `[12, 62]`.)

### 8.2 Required JSON design choices

- `schema_version`: integer `1`.
- `declaration`: exact literal `"self-reported"`.
- `percentage`: number rounded to one decimal, or `null` if denominator zero.
- `files`: sorted by normalized path, includes **all discovered files**, not just marked ones.
- `ranges`: `[start_line, end_line]`, inclusive, sorted, merged, and non-overlapping. These represent declared physical line intervals; eligible/marked counts still follow Section 7.
- `claim_sources`: sorted, de-duplicated values chosen from `config`, `file-marker`, `block-marker`.
- No machine-specific absolute paths.
- No timestamps or random identifiers in default output.
- End JSON with a newline; serialize with deterministic field and collection ordering.
- The scanner should compute a fully consistent report from its data model, rather than trusting manually authored numbers.

### 8.3 Empty result

Valid, predictable empty scan:

```json
{
  "schema_version": 1,
  "project": { "name": "empty-project" },
  "declaration": "self-reported",
  "stats": {
    "files_scanned": 0,
    "files_with_marks": 0,
    "eligible_lines": 0,
    "marked_lines": 0,
    "percentage": null
  },
  "files": []
}
```

---

## 9. Badge system and design requirements

### 9.1 Output

- Default: `.wbh/badge.svg`.
- Valid static SVG without remote fonts, images, CSS, scripts, or fetches.
- File must be embeddable in a GitHub README using ordinary Markdown image syntax.
- Escape all custom text before inserting it into XML, including `&`, `<`, `>`, `"` where relevant.
- Include accessible SVG `<title>` and `<desc>` with clear wording and self-reported semantics.
- Calculate text width enough to avoid clipping common custom labels; constrain text length, and add tests for long strings and special characters.
- Prefer system font stacks and/or SVG-drawn simple icons. **Do not package copyrighted font files**.
- Avoid fragile CSS filters, external gradients, or browser-only effects that may not render in GitHub SVG sanitization.
- Badge remains legible in GitHub light and dark modes.
- SVG output stable for identical inputs.

### 9.2 Six styles

| Key | Display name | Direction | Priority |
|---|---|---|---|
| `stamp` | Rubber Stamp | Warm paper/ink vintage stamp, handmade feeling | P0 |
| `classic` | Classic | Compact two-tone GitHub-style badge | P0 |
| `terminal` | Terminal | Monospace, green-on-dark terminal | P0 |
| `pixel` | Pixel | Retro computer/game aesthetic | P1 |
| `minimal` | Minimal | Restrained typography, pencil/hand icon | P1 |
| `coffee` | Coffee Edition | Playful coffee/handmade seal | P1 |

### 9.3 Style variants

For each style, support:

- `show_percentage = true/false`.
- `compact = true/false`.
- Custom label text (sane length limit, e.g. 1–60 Unicode scalar values or display columns; document exact rule).
- Percentage rendering: if no eligible lines, show `N/A` or omit the number; never show `100%`.

Example outputs (visual direction only):

```text
[ ✍  WRITTEN BY HAND ]
[ written by hand | 75% ]
[ ~/ human_written: true ]
[ ▦  HUMAN CODED ]
[ ✎  Handcrafted code. ]
[ ☕  NO PROMPTS ]
```

**Copy constraints:** `NO PROMPTS` and `100% HUMAN MADE` are humorous optional style labels, not proof or automatically inferred facts. The generic default should use the neutral label `Written by Hand`.

### 9.4 Badge generator interface

Suggested internal interface:

```rust
pub fn render_badge(style: BadgeStyle, options: &BadgeOptions, stats: &Stats) -> Result<String>;
```

Tests:

- All six styles (when introduced) generate well-formed SVG.
- Valid output for `0`, `25.5`, `100`, and `null` percent.
- Text escapes cannot inject markup or scripts.
- Output dimensions are positive and text is not clipped in reference render checks where rendering tests are available.
- No `http://` or `https://` links to external SVG resources.

---

## 10. README badge integration

### 10.1 Generated Markdown

For the default relative paths, the README contains:

```markdown
<!-- wbh:badge:start -->
[![Written by Hand](.wbh/badge.svg)](.wbh/REPORT.md)
<!-- wbh:badge:end -->
```

Use ordinary **relative links** so the repo badge works with no service, and can be viewed on GitHub without a hosted site. The clickable destination defaults to the Markdown report; a later config choice may point it at the GitHub Pages site.

### 10.2 Safe editing rules

- Preserve unrelated README content byte-for-byte where practical.
- Preserve its existing LF/CRLF style and UTF-8 BOM if present.
- If a managed block already exists, replace only that block.
- If multiple or malformed managed blocks exist, stop with a descriptive error; do not guess.
- If no block exists, insert after the first top-level H1 if found; otherwise insert near the beginning.
- Keep exactly one managed block after repeated `init` or badge changes.
- Do not overwrite a missing README without explicit user request.
- Do not delete a README or rewrite its entire formatting.
- Perform atomic replacement when feasible.
- Do not embed arbitrary unescaped user-entered Markdown into alt text or URLs; escape appropriate Markdown characters and reject control characters/newlines.

### 10.3 GitHub line links

For `REPORT.md` stored under `.wbh/`, construct file-relative links correctly, for example:

```markdown
[src/dsp/Oscillator.cpp](../src/dsp/Oscillator.cpp#L12-L62)
```

For a site viewer, Github remote URLs may be used if parseable. Prefer an exact commit SHA for link stability when available. Do not assume every repo uses `main`; support `master`, arbitrary branches, and detached HEAD without crashing. For a local/non-Git folder, fall back to file-relative report links and omit unsupported remote links.

To avoid URL/path injection, percent-encode GitHub path segments as needed and validate hosts before constructing external links. Never concatenate attacker-controlled HTML unescaped.

---

## 11. Markdown report

Generated path: `.wbh/REPORT.md`.

Minimum sections:

1. Project name and heading `Written by Hand`.
2. One-sentence disclosure: **“These handwritten markings are self-reported by the project maintainer; no independent verification or AI detection is performed.”**
3. Overall stats: eligible lines, marked lines, percentage or N/A, scanned files, marked files.
4. Per-file Markdown table, sorted by path, with physical ranges and percentages.
5. Links to source files, and for each marked range, a clickable line-range link where supported.
6. Explanation of the methodology and exclusions, with `@handwritten` examples.
7. Small note saying `wbh scan` updates this report.

Example:

```markdown
# Written by Hand — Little Angel Synth

> Self-reported handwritten markings. Not AI detection or certification.

**Coverage:** 75.0% (1,800 marked / 2,400 eligible source lines)

| File | Handwritten | Marked ranges |
|---|---:|---|
| [src/dsp/Oscillator.cpp](../src/dsp/Oscillator.cpp) | 100.0% | [L1–L320](../src/dsp/Oscillator.cpp#L1-L320) |
| [src/ui/MainView.swift](../src/ui/MainView.swift) | 50.0% | [L12–L62](../src/ui/MainView.swift#L12-L62) |
```

Any shown counts/percentages in fixtures must derive from the actual fixture scanner results, not copied from illustrations.

---

## 12. Static Code Map (Phase 3)

### 12.1 Purpose

An optional, **offline-capable static viewer** that visually answers “Which portions of this repository did the maintainer mark as handwritten?” It is a more delightful experience than a Markdown table but should not require a hosted server.

### 12.2 Features

- Header with project name, playful branding, percent, and disclosure.
- Coverage bar: **marked vs unmarked** (never marked vs AI).
- Searchable file tree/list, sorted by path.
- Filter: all files / marked files / unmarked files.
- Per-file coverage and marked line ranges.
- Clicking a file opens a details panel with ranges and source links.
- Clicking ranges navigates to the matching repository file/line span.
- Responsive layout usable on mobile and desktop.
- Works locally by opening generated `index.html` directly where practical; avoid requiring `fetch()` from `file://`. Embed the report data into HTML as properly escaped JSON, or bundle a JS module whose import works under local browser restrictions. A static server may be documented as a fallback, **not required for ordinary report usage**.
- Accessible keyboard controls, readable contrast, no essential animation.

### 12.3 Default privacy policy

By default, the site contains **only report metadata, not actual source code contents**. This protects against inadvertently publishing source via GitHub Pages. The viewer can navigate to lines on GitHub where accessible. A private repo may be inaccessible to public viewers; do not misrepresent this as a bug in coverage.

A documented explicit flag or config opt-in enables embedding source snippets. If enabled, the CLI **must warn** that site output will contain source text that could become public if published. Escape all code into text nodes; never allow code text to become executable HTML.

### 12.4 Output

```text
.wbh/site/
├── index.html
├── style.css
└── app.js
```

Use standalone static files, no CDN, no frameworks, no fonts from remote services, no analytics, and no external API.

### 12.5 Deployment

Offer an example GitHub Pages workflow in `docs/GITHUB_PAGES.md` at Phase 3:

- User explicitly enables GitHub Pages and adds the workflow.
- Workflow builds/downloads `wbh`, runs `wbh site build`, and publishes `.wbh/site/` using supported GitHub Actions Pages steps.
- Publishing is opt-in and occurs from the intended branch only.
- Document that GitHub Pages can make content publicly accessible and that private repositories require special care.
- The `wbh` CLI itself does not authenticate to GitHub or mutate repo settings.

---

## 13. Security and resilience requirements

This tool reads arbitrary repositories and outputs HTML/SVG/Markdown; security matters despite its playful nature.

### 13.1 Path security

- Canonicalize root and candidate input paths; reject traversal outside root.
- Do not follow symlinks outside root; simplest v0.1 policy is to skip symlinks entirely.
- Output destinations may not be absolute or escape root; reject targets that resolve through symlinks to outside root.
- Disallow writing over user source files or README from misconfigured badge/report paths.
- Never execute code, hooks, arbitrary commands, build scripts, or git subcommands without clear necessity. Reading Git metadata is acceptable; if calling Git, treat arguments safely and avoid shell interpolation.

### 13.2 Injection prevention

- Escape file paths, project names and custom badge labels for XML, HTML and Markdown destinations.
- Never evaluate rendered source snippets as HTML or JavaScript.
- When embedding JSON into HTML, avoid literal `</script>` injection by JSON-safe escaping or by using an inert safely encoded payload.
- Do not generate unsafe external links from malformed remotes.
- Prevent control characters and newline injection in user-configured text fields.

### 13.3 Reliability

- Do not overwrite valid outputs after a failed scan.
- Write only when bytes differ to avoid dirtying the Git working tree on every run.
- Use tempfile + rename where possible, with Windows-aware fallback.
- Normalize JSON path separators to `/` across platforms.
- Deterministic ordering and stable numeric rounding.
- If marker parsing finds inconsistent structure, report all useful errors up to a sensible cap, then exit nonzero.
- Respect scanning size limits and avoid loading huge binary files into memory.
- All normal commands operate offline and never collect or send developer source code.

### 13.4 Public claims

Never present this tool as a forensic proof of human authorship. The README/report/web viewer should always offer an unambiguous **self-declared** disclosure. No “verified” shield or authoritative certification language.

---

## 14. Development milestones and exact order

Suggested total implementation effort is about **10–17 focused developer-days**, highly dependent on test thoroughness. These are planning estimates, not deadlines. Do not rush correctness to match them.

### Phase 1A — Rust skeleton and config

**Goal:** A compiling CLI with config IO and validation.

Tasks:

1. Initialize Cargo project, executable named `wbh`.
2. Set up `clap` commands and global `--root`.
3. Define `Config`, `BadgeConfig`, `ScanConfig`, `ClaimsConfig`, `ReportConfig`, `ReadmeConfig`, `SiteConfig` structs.
4. Implement config parsing, defaults, schema version checking, path and glob validation.
5. Add `wbh init --style stamp --yes` and noninteractive fallback.
6. Add `mark` / `unmark` that safely modify `.wbh.toml` with preserved formatting.
7. Add unit/integration tests for config, path traversal, and idempotency.

**Done when:** A fresh repository can run `wbh init`, `wbh mark`, and `wbh unmark`, and all tests pass. Stub scan outputs should not be presented as real coverage; if scanner is not yet ready, explicitly show “scan not implemented” rather than fake percentages.

### Phase 1B — Scanner engine

**Goal:** Deterministically discover files, parse declarations, and calculate coverage.

Tasks:

1. Build safe source-file walker with include/exclude and Git ignore behavior.
2. Implement language-to-comment-syntax mapping.
3. Implement marker recognition with line numbers and header rules.
4. Implement structural validation and informative parse errors.
5. Implement eligible line calculation and merged declaration intervals.
6. Implement per-file and project-wide stats.
7. Add fixtures for C++, Rust, Swift, Python, JS, HTML and JSON config claims.
8. Add tests for CRLF, BOM, Unicode, no trailing newline, zero lines, overlap, stale claims and symlinks.

**Done when:** Sample fixture projects produce exact expected eligible and marked line counts with no double counting and no unintended source modifications.

### Phase 1C — JSON / Markdown reports + badge + README

**Goal:** End-to-end useful v0.1.

Tasks:

1. Serialize deterministic versioned JSON.
2. Generate Markdown report and source line links.
3. Implement three starter badge styles: `stamp`, `classic`, `terminal`.
4. Escape text safely in all outputs.
5. Implement managed README badge insertion and replacement.
6. Complete `scan`, `badge --style`, `badge --list`, `report`, and `check`.
7. Ensure no-change runs leave files unmodified.
8. Create realistic demo projects and full CLI tests.
9. Write user-facing README with install/use examples and the self-reported disclaimer.

**Done when:** Every acceptance test in Section 16 for v0.1 passes on macOS/Linux (and Windows CI where feasible). Ship version `0.1.0`.

### Phase 2 — Polish and distribution (v0.5)

Tasks:

1. Add `pixel`, `minimal`, and `coffee` badge styles.
2. Complete compact and percentage variants for all styles.
3. Improve terminal onboarding and helpful errors.
4. Add a Markdown or generated SVG badge gallery to project README.
5. Add `wbh check` CI workflow example.
6. Add GitHub Actions builds/tests for macOS, Linux, Windows.
7. Publish release artifacts for at least macOS arm64/x86_64, Linux x86_64, Windows x86_64.
8. Add checksum files and installation instructions; optional Homebrew later.
9. Add a changelog and contribution guide.

**Done when:** A user can download a precompiled binary, install it without any language runtime, and reproduce the sample demo in a new repo.

### Phase 3 — Interactive Code Map (v1.0)

Tasks:

1. Generate a static viewer from fresh `ScanResult`.
2. Design file tree/list, coverage charts, search/filter controls and file details.
3. Add safe GitHub line links and non-Git fallback.
4. Keep source contents excluded by default; implement explicit optional embedding.
5. Add accessible, responsive vanilla JS/CSS UI.
6. Test viewer with generated fixtures and untrusted file names/custom labels.
7. Add optional GitHub Pages publish workflow and privacy documentation.
8. Improve docs, examples, release QA, and CI.

**Done when:** A user can generate a Code Map and inspect marked files/ranges in a browser with no backend or account, and can publish it only by opting into deployment.

---

## 15. Test strategy and fixture matrix

### 15.1 Unit tests

- Config defaults, validation, parse/serialize roundtrips.
- Normalization and path/root protection.
- Marker matching and line numbers for every supported syntax family.
- File-level header recognition and invalid placement.
- Unmatched, nested, duplicated and overlapping marker scenarios.
- Eligible and marked counts with comments, blanks, braces and directives.
- Percent rounding, denominator zero, source claim union.
- XML/HTML/Markdown escaping.
- Deterministic JSON and SVG snapshots.
- README parsing and managed section replacement.

### 15.2 Integration tests

Build temporary Git repositories or fixture folder trees and execute compiled `wbh`:

1. **Fresh setup:** `wbh init --style stamp --yes` creates config and three artifacts and inserts a README badge.
2. **Repeat init:** no duplicate badge, no config reset.
3. **Mark file:** `wbh mark src/main.rs` creates exactly one config claim and never changes source.
4. **Unmark file:** removes config claim, preserves other TOML keys and comments.
5. **Inline full-file marker:** 100% of the file's eligible lines marked, excluding marker line.
6. **Inline block marker:** only eligible interior lines marked.
7. **Mixed claims:** file claim plus blocks is counted once.
8. **Malformed markers:** no outputs overwritten; error includes location.
9. **Ignored generated files:** `target`, `build`, `.wbh`, `vendor`, and `.git` do not affect percentages.
10. **Staleness:** modify source after scan; `wbh check` fails, `wbh scan` repairs, then `check` passes.
11. **No-change:** run scan twice and confirm artifact content and modification times do not change on second run.
12. **Different line endings:** preserve source and README endings, calculate lines accurately.
13. **Unicode/special chars:** labels and file names do not break SVG/HTML/Markdown.
14. **Empty repo:** zero eligible gives `null` and `N/A`, not 100%.
15. **Paths:** `../outside.rs`, absolute output paths and symlink escapes are rejected.
16. **Windows paths:** normalized JSON paths use `/` and no scanner panic.
17. **Noninteractive init:** does not block for input and selects default style.
18. **Multiple badges:** `wbh badge --style terminal` updates SVG and config without duplicating README block.
19. **README absent:** init succeeds and prints manual snippet instead of writing a README silently.
20. **Private/source site safety:** default static viewer does not copy source content.

### 15.3 Suggested exact scanner fixture

`src/main.rs`:

```rust
// @handwritten:start
fn main() {
    println!("hello");
}
// @handwritten:end

fn other() {}
```

Expected physical line count: **7**. Eligible lines: **4** (`fn main`, `println`, `}`, `fn other`). Marked lines: **3**. Percentage: **75.0%**. The two marker lines and one blank line are excluded.

For clarity, here is the final expected breakdown:

| Physical line | Text | Eligible? | Marked? |
|---|---|---|---|
| 1 | `// @handwritten:start` | No | No |
| 2 | `fn main() {` | Yes | Yes |
| 3 | `    println!("hello");` | Yes | Yes |
| 4 | `}` | Yes | Yes |
| 5 | `// @handwritten:end` | No | No |
| 6 | blank | No | No |
| 7 | `fn other() {}` | Yes | No |

Thus `3 / 4 = 75.0%`.

### 15.4 Test quality bar

Run and fix:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

Where feasible, CI should exercise each supported OS. Add golden fixtures for JSON, Markdown and SVG. Revisit snapshots deliberately when schemas or badge rendering change.

---

## 16. Acceptance criteria

### 16.1 MVP smoke test

From a new repository with `README.md` and at least one source file:

```sh
wbh init --style stamp --yes
wbh mark src/engine.cpp
wbh scan
wbh check
```

All of these must be true:

- `.wbh.toml` exists and can be edited by a human.
- `.wbh/badge.svg` is valid and readable.
- `.wbh/report.json` has a coherent versioned schema and correct counts.
- `.wbh/REPORT.md` lists marked files and correct source/range links.
- README contains exactly one clickable managed badge pointing at the local report.
- `check` exits `0` after `scan`.
- No source file bytes are changed by CLI operations.
- Repeating `scan` produces byte-identical artifacts and no timestamp-only changes.
- A missing/malformed marker gives a clear error, no corrupt artifact.
- The core workflow works without internet and without Node/Python.

### 16.2 Project-ready release criteria

- Fresh-install instructions verified against release archives.
- CI green on supported platforms.
- `--help` examples accurate and short.
- Untrusted file names and labels cannot inject HTML/SVG.
- Documentation repeatedly clarifies self-declared, non-verified nature.
- An actual demo repository demonstrates partial and complete file declarations.
- All released artifacts have checksums.

### 16.3 v1.0 Code Map criteria

- File tree, coverage bar, filter/search and ranges work without server dependency.
- GitHub source URLs are correct for repository metadata when available.
- Non-Git projects still produce valid reports and viewer.
- Site does not embed source contents by default.
- Source-embedding opt-in has a clear public-disclosure warning.
- Accessible keyboard interactions and mobile layout are tested.

---

## 17. Distribution and repository automation

### 17.1 Release targets

| OS | Architecture | Priority |
|---|---|---|
| macOS | arm64 (Apple Silicon) | P0 |
| macOS | x86_64 | P0 |
| Linux | x86_64 | P0 |
| Windows | x86_64 | P0 |
| Linux | aarch64 | P1 |
| Windows | arm64 | P2 |

Use GitHub Releases for archives of the binary, checksums, and a concise changelog. `cargo-dist`/`dist` is an acceptable release option if currently supported; alternatively use explicit GitHub Actions matrix builds. Avoid elaborate installer work in v0.1.

Possible later distribution channels: Homebrew tap, Winget/Scoop. **Do not use npm as the primary or required installation route.**

### 17.2 CI

Add workflows to:

- Run `fmt`, `clippy`, unit and integration tests on pushes and pull requests.
- Build on major platforms.
- Publish tagged release artifacts only on deliberate release tags.
- Include an example downstream workflow that runs `wbh check` after checkout.
- Pin or maintain GitHub Actions dependencies responsibly; avoid exposing elevated write permissions to untrusted PR workflows.

### 17.3 Documentation and demo assets

The public repository README should show:

1. **A visual badge gallery** (static SVGs built from the actual renderer).
2. A one-sentence funny pitch.
3. Install commands for supported platforms.
4. Three-step quickstart.
5. One full-file and one block marker example.
6. Screenshot or small animation of the Code Map after v1.0.
7. Disclosure: self-reported, no certification.
8. Link to marker/config/reference docs.
9. License and contribution instructions.

The public-facing documentation should be in English; internal code comments and error strings should also default to English.

---

## 18. UX, copy and design direction

### 18.1 Personality

Lighthearted, nostalgic, hand-crafted, developer-friendly. Avoid aggressive anti-AI framing; the point is to celebrate craft, not shame anyone who uses tools.

### 18.2 Badge styling

- `stamp`: cream paper, terracotta/brown ink, imperfect feeling achieved with SVG path/stroke styling; do not sacrifice legibility.
- `classic`: compact neutral gray/dark with a warm accent, two-piece shield.
- `terminal`: near-black background, muted green monospace.
- `pixel`: restrained 8-bit retro styling, block lettering.
- `minimal`: whitespace, small pencil/hand pictogram, elegant simple typography.
- `coffee`: circular/oval vintage coffee seal, playful but readable in small size.

Ensure designs are visually distinct and readable at a common README display height (approximately 20–32 px for compact badges; larger for decorative stamp variants). Offer examples rather than a full graphical editor.

### 18.3 Error copy examples

```text
error: unclosed @handwritten:start in src/audio/Engine.cpp:42
hint: add // @handwritten:end after the block, or remove the start marker
```

```text
error: cannot mark ../outside.cpp: path resolves outside project root
```

```text
warning: this project contains no eligible source lines; coverage is N/A
```

### 18.4 Terminal output example

```text
$ wbh scan

Written by Hand ✍
  12 source files scanned
  1,800 / 2,400 eligible lines marked (75.0%)

  ✓ .wbh/report.json
  ✓ .wbh/REPORT.md
  ✓ .wbh/badge.svg

Self-reported. No AI detection, no certification.
```

The values are sample UX copy, not hardcoded defaults.

---

## 19. Key implementation decisions and tradeoffs

| Question | Decision | Rationale |
|---|---|---|
| Rust or C++? | **Rust** | Single native binary, good CLI/file tooling, cross-platform. |
| npm package? | **No** | Zero runtime/install burden for release binaries. |
| Verification vs honor system? | **Honor system** | Fun, minimal, no false claims. |
| Badge hosting? | **Local SVG in repository** | No backend, robust README integration. |
| Click destination? | **Markdown report by default** | Works without Pages. |
| Code Map? | **Optional static site** | Better experience without hosting dependency. |
| Scanner precision? | **Language-aware comment directives, no AST** | Small implementation surface. |
| Coverage basis? | **Nonblank, non-directive lines** | Simple, transparent, deterministic. |
| File marking? | **Config or header comment** | Source cleanliness or inline visibility. |
| Block marking? | **Paired comments** | Familiar and explicit. |
| Generated outputs tracked? | **Yes, by default** | Badge works directly from GitHub with no automation. |
| Network access? | **None in normal operation** | Privacy and reliability. |
| GitHub Pages? | **Manual opt-in** | No surprise publication. |
| In-viewer source content? | **Off by default** | Prevent unintentional source disclosure. |
| Repo root? | **Current directory or `--root`** | Predictable, works outside Git. |
| Noninteractive setup? | **Use default stamp** | CI-friendly. |

---

## 20. Example complete workflow

```sh
# Download/install a native release of wbh, then:
cd little-angel-synth

# Create .wbh.toml, badge, reports, and README insertion.
wbh init --style stamp --yes

# Mark whole files without editing their source.
wbh mark src/dsp/Oscillator.cpp src/dsp/Chorus.cpp

# Or manually add paired comment directives in any supported source file:
# // @handwritten:start
# ...your own code...
# // @handwritten:end

# Re-scan and regenerate badge + reports.
wbh scan

# See other styles.
wbh badge --list

# Change badge style.
wbh badge --style terminal

# Validate checked-in outputs in CI.
wbh check

# After Phase 3, optionally generate static viewer.
wbh site build

# Commit the self-contained assets.
git add .wbh.toml .wbh README.md
git commit -m "Add Written by Hand badge"
```

The default README badge points to `.wbh/REPORT.md`:

```markdown
<!-- wbh:badge:start -->
[![Written by Hand](.wbh/badge.svg)](.wbh/REPORT.md)
<!-- wbh:badge:end -->
```

A report may include GitHub line links when remote information is known, but the ordinary Markdown links remain meaningful within the repository without relying on any external badge service.

---

## 21. Backlog — deliberately not part of v1.0

Only consider these after users actually adopt the tool:

- `wbh watch` to rescan on file changes.
- `wbh badge preview` to generate a local style gallery.
- `wbh doctor` diagnostics.
- Git `blame` visual overlays (must not be presented as proof of handwritten code).
- Custom SVG themes or badge marketplace.
- Optional source span claims stored outside code using stable anchors/hashes.
- Editor integrations or LSP hints for markers.
- More language-aware parsers for tricky raw-string/comment syntax.
- Directory-level opt-in claims with explicit warnings.
- Automated gallery of community repositories, only with opt-in.
- PNG export and social cards.
- Better code excerpt highlighting with syntax-aware rendering.
- Package-manager distribution beyond direct binaries.

Do **not** preemptively implement these in the initial build.

---

## 22. Agent execution checklist

Use this as an implementation tracker. Mark items done only after verifying them.

### Foundation

- [ ] Inspect repository and initialize Cargo package if needed.
- [ ] Set binary target name `wbh`.
- [ ] Implement CLI flags/subcommands and help.
- [ ] Implement typed `.wbh.toml` config and path/glob validation.
- [ ] Implement safe `init`, `mark`, and `unmark`.

### Scanner

- [ ] Safe file discovery and ignore rules.
- [ ] Language/comment marker mapping.
- [ ] Whole-file header marker support.
- [ ] Block `start`/`end` support and structural validation.
- [ ] Correct eligible/marked line counting.
- [ ] Deterministic ScanResult and schema.
- [ ] Unicode/CRLF/BOM/symlink/size/empty-file edge cases.

### Outputs

- [ ] Deterministic `report.json`.
- [ ] Markdown report with correct links.
- [ ] Stamp, classic, terminal SVG badge templates.
- [ ] Safe/accessible SVG rendering.
- [ ] Idempotent README insertion.
- [ ] `scan`, `report`, `badge --style`, `badge --list`, `check`.
- [ ] Do not rewrite identical outputs.

### Quality and release

- [ ] Unit and integration tests across realistic fixtures.
- [ ] Run fmt, clippy, tests, release build.
- [ ] Document assumptions, limitations, examples.
- [ ] v0.1 sample repo and README gallery.
- [ ] Three additional styles: pixel, minimal, coffee.
- [ ] Cross-platform CI and native release binaries.

### Viewer

- [ ] `site build` from fresh scan.
- [ ] Interactive static Code Map (search/filter/coverage/ranges).
- [ ] Safe source link generation.
- [ ] Source contents excluded by default.
- [ ] Optional explicit source embedding and warning.
- [ ] Offline/local operation where feasible.
- [ ] Optional GitHub Pages instructions and workflow.

---

## 23. Completion handoff format

At the end of each milestone, provide the project owner with a concise engineering update:

```text
Implemented:
- [feature/commands]

Files changed:
- [important files]

Verification:
- cargo fmt --check: PASS/FAIL/NOT RUN
- cargo clippy ...: PASS/FAIL/NOT RUN
- cargo test --all-targets: PASS/FAIL/NOT RUN
- cargo build --release: PASS/FAIL/NOT RUN
- manual smoke test: PASS/FAIL/NOT RUN

Known limitations:
- [specific technical limitations, not vague claims]

Next milestone:
- [specific work]
```

### Final guiding principle

> **Written by Hand should be the simplest possible way to put a beautiful, clickable, self-declared handmade-code badge on a GitHub repository.**
>
> The charm is its simplicity. Ship a polished small tool before expanding the scope.
