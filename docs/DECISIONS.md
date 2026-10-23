# Decisions

Deviations from and clarifications of `WRITTEN_BY_HAND_IMPLEMENTATION_PLAN.md`. Treat the spec as a set of proposals: where this file disagrees, this file wins.

## Spec review: product changes

These come from questioning the spec as a whole after v0.1 was built.

1. **`check` validates; `check --strict` also enforces freshness.** Every code change shifts the eligible-line count, so with strict-by-default freshness, contributors who never touched a declaration would get a red CI until someone ran `wbh scan`. Plain `check` fails only on invalid config, markers or README block, and *notes* stale outputs. `--strict` fails on them too.
2. **No percentage on the badge by default** (`show_percentage = false`). "Written by Hand · 3.2%" reads as a confession, and `init` would put 0.0% in a README. A label-only badge celebrates and rarely changes. The numbers stay in `REPORT.md`, and `wbh badge set --percentage` turns them on.
3. **`report.json` is opt-in** (`[report] json = true`). Nothing consumed it, and because it lists every scanned file it changed in nearly every PR.
4. **No Code Map (spec Phase 3).** `REPORT.md` already answers "what is declared?" with clickable line links on GitHub. Rebuild it only if users ask.
5. **No `[site]` config.** Shipping keys for a feature that doesn't exist only creates configs that break if it's removed. Unknown keys are rejected, so they can be added later safely.
6. **Output paths are fixed:** `.wbh/badge.svg`, `.wbh/REPORT.md` and `.wbh/report.json`. Configurable paths that had to stay inside `.wbh/` were nearly meaningless and needed validation code. Only `readme.path` stays configurable, for monorepos.
7. **No line limit for whole-file markers.** "Before any code" already prevents surprise declarations, and the spec's 20-line cap only rejected long license headers.
8. **Claim problems are warnings.** A claimed file that's deleted, renamed, excluded or unreadable is skipped, and the warning names the `wbh unmark` command. Renaming a file shouldn't break every scan.
9. **Also published to crates.io.** `cargo install written-by-hand` works for Rust users, and `cargo binstall written-by-hand` fetches the GitHub release binaries.
10. **No `init --yes`.** `--style` skips the prompt, and non-interactive runs never prompt.

## CLI shape

The spec's `badge --list` / `badge --style S` flags and the `report` alias of `scan` were replaced. Actions are subcommands and flags only modify them:

- `wbh badge` shows the current settings.
- `wbh badge list` lists the styles.
- `wbh badge set [STYLE] [--compact|--no-compact] [--percentage|--no-percentage]` changes them and regenerates. It needs at least one change, and contradictory flags are a usage error (exit 2).
- `report` was dropped. A second name for `scan` adds something to learn and does nothing extra.

## Reproducible across machines

1. **No commit SHAs in tracked outputs.** Committing a report that contains HEAD's SHA changes HEAD, so it would be stale forever. All links are relative.
2. **`project.name` is always written to `.wbh.toml` by `init`.** The directory-basename default applies only when the key is missing, because CI clones into differently named directories.
3. **`check` ignores `\r` when comparing.** Git `autocrlf` on Windows rewrites checked-out files to CRLF, and that isn't staleness.
4. **Untracked files can make local and CI scans differ.** This is documented, so users run `wbh scan` on a clean tree. A later option could restrict scanning to the files git tracks.
5. **Percentages round down, to one decimal place, using integer math.** `99.96%` displays as `99.9%`; `100.0%` means every eligible line is marked.

## Spec contradictions resolved

- `mark` on an already-claimed path is a no-op, not an error.
- `unmark` accepts any path currently claimed, even one that no longer exists on disk. Unclaimed paths print a warning and exit 0.
- The default excludes live in one place, `.wbh.toml`, which `init` writes with every default spelled out. `.git/`, `.wbh/` and hidden paths (any component starting with `.`) are always skipped, and config cannot override this.
- `init`, `badge set` and `scan` all render the README block with the same function. `check` verifies that block.
- README links are computed relative to the README's location (`readme.path`).

## Defaults

- `.json`, `.jsonc`, `.yml`, `.yaml`, `.toml` and `.md` files are excluded by default (via `scan.exclude`), so lockfiles and docs don't swamp the numbers. Remove the exclude line to opt in.
- `REPORT.md` lists only marked files plus a count of unmarked ones. The optional `report.json` lists every scanned file.
- `badge.text` is 1–60 Unicode scalar values and `project.name` 1–100, with no control characters.

## Scanning

- **Only ignore files inside the root are read:** `.gitignore` and `.ignore`, applied even outside a git repo. The global gitignore, `.git/info/exclude` and ignore files in parent directories are not read, because they differ between machines.
- `.json` and `.jsonc` files never contain markers; they can only be claimed through config.
- `mark` rejects gitignored files, using the same discovery code as the scan.
- On a marker error, all errors (up to 50) are reported and nothing is written.

## Outputs

- **One render function for every command.** `scan`, `badge set`, `init` and `check` render every output, including the README, in memory from a single `ScanResult`. Nothing is written until everything has rendered, and then only files whose bytes changed.
- **Nothing is written through a symlink.** Outputs and the README must not pass through any symlink, even one pointing inside the root. A `.wbh → src` link would otherwise redirect writes into source files.
- **README block:**
  - Placement: inserted after the first ATX (`# `) or setext (`===`) H1 outside code fences, otherwise at the top. Only the block is ever touched.
  - Preserved: the BOM and the file's line-ending style.
  - Documentation: marker lines inside code fences are treated as documentation, not as the managed block.
  - Malformed blocks: any shape other than exactly one start followed by one end is an error.
- **Badge widths are estimated from per-character tables**, without font files. Each style was checked in a browser against real system fonts, with long and non-ASCII labels, and nothing clipped.
- **What `compact` means:** a smaller height and font, and the stamp and coffee styles drop their inner border.
- **Missing percentage:** shown as `N/A` in the badge, terminal and Markdown, and as `null` in JSON.
- **The `pixel` style draws its lettering from a built-in 5×7 bitmap font** (A–Z, 0–9 and common punctuation) as SVG paths, so it looks identical everywhere without shipping a font. Labels with other characters fall back to bold monospace text.
- **The README gallery (`docs/badges/`) is generated by the renderer.** A unit test fails if it's stale, and `WBH_BLESS=1 cargo test gallery` regenerates it.

## Distribution

- **Paths are joined component by component (`paths::join`).** On Windows, canonical roots are `\\?\C:\…` paths, in which `/` is not a separator.
- **Releases use an explicit GitHub Actions matrix, not cargo-dist.** It's one short workflow with no extra tooling.
  - Linux builds are static musl binaries, so they run on any distro.
  - Asset names carry no version (`wbh-<target>.tar.gz`), so `releases/latest/download/…` links stay stable.
  - The workflow refuses tags that don't match `Cargo.toml` and creates a *draft* release for a human to publish.
- **Workflows use only first-party actions** (`actions/checkout`, `upload-artifact`, `download-artifact`), plus `rustup` and `gh`, which are preinstalled on GitHub runners. Actions are pinned by major version, not commit SHA. Only the release job gets `contents: write`.
- **`cargo publish` is a manual release step**, not part of the workflow, so no registry token lives in CI.
- **Not done:** Homebrew, Winget, Scoop and an install script.

## Scope cuts

- **No lexical string state machine.** A marker counts only when it fills an entire whole-line comment, so the only false positive is a multi-line string literal containing a line that is exactly a marker comment. This is documented in `docs/MARKERS.md`. Use `wbh mark` to avoid it.
- Dependencies: `anyhow` only (no `thiserror`). A plain `read_line` prompt replaces `dialoguer`. `toml_edit` handles both serde parsing and format-preserving edits. Tests use `std::process::Command` with `CARGO_BIN_EXE_wbh` (no `assert_cmd`/`predicates`).
- Not built: `scan --json`, `scan --quiet`, `init --readme`, and project-name detection from the git remote.
- Atomic writes use a sibling temp file plus `fs::rename`, which replaces existing files on Windows too.
- **Symlinks are never followed.** `mark` rejects any path that canonicalizes differently from its lexical form.
- Paths given to `mark`/`unmark` are relative to the project root. Absolute paths are rejected.
