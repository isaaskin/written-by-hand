# Running `wbh check` in CI

`wbh check` rescans your sources without writing anything or touching the network. It fails when the config, the `@handwritten` markers or the README badge block are invalid.

Ordinary code changes make `.wbh/` slightly out of date, because the number of eligible lines moves. Plain `check` only *notes* that, so contributors who never touched a declaration don't get a red build. Use `wbh check --strict` if you want CI to insist that `wbh scan` was run.

Exit codes:

- `0`: config and markers are valid (and, with `--strict`, outputs are up to date).
- `1`: invalid config or markers, or, with `--strict`, stale or missing outputs.
- `2`: invalid command-line usage.

## GitHub Actions

Add `.github/workflows/wbh.yml` to your project:

```yaml
name: Written by Hand

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  check:
    runs-on: ubuntu-latest
    env:
      WBH_REPO: isaaskin/written-by-hand   # where wbh releases are published
    steps:
      - uses: actions/checkout@v4
      - name: Install wbh
        run: |
          base="https://github.com/$WBH_REPO/releases/latest/download"
          curl -fsSLO "$base/wbh-x86_64-unknown-linux-musl.tar.gz"
          curl -fsSLO "$base/wbh-x86_64-unknown-linux-musl.tar.gz.sha256"
          sha256sum -c wbh-x86_64-unknown-linux-musl.tar.gz.sha256
          tar xzf wbh-x86_64-unknown-linux-musl.tar.gz
          echo "$PWD/wbh-x86_64-unknown-linux-musl" >> "$GITHUB_PATH"
      - run: wbh check
```

To pin a version instead of following the latest release, replace `latest/download` with `download/v0.5.0`.

## Tips

- Run `wbh scan` on a clean working tree before committing. Untracked files that aren't gitignored are scanned locally but don't exist in CI.
- On Windows runners, git may check files out with CRLF line endings. `check` ignores that difference.
