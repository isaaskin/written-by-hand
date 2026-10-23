//! End-to-end tests for generated outputs: reports, badge, README and `check`.

use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn wbh(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wbh"))
        .arg("--root")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn fail(out: &Output) -> String {
    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap()
}

const MAIN_RS: &str = "// @handwritten:start\nfn main() {\n    println!(\"hello\");\n}\n// @handwritten:end\n\nfn other() {}\n";

fn project(readme: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), MAIN_RS).unwrap();
    if let Some(r) = readme {
        fs::write(root.join("README.md"), r).unwrap();
    }
    dir
}

const ARTIFACTS: [&str; 2] = [".wbh/REPORT.md", ".wbh/badge.svg"];

/// `report.json` is opt-in; switch it on and regenerate.
fn enable_json(root: &Path) {
    let cfg = read(root, ".wbh.toml").replace("json = false", "json = true");
    fs::write(root.join(".wbh.toml"), cfg).unwrap();
    ok(&wbh(root, &["scan"]));
}

#[test]
fn mvp_smoke_test() {
    let dir = project(Some("# Synth\n\nMakes noise.\n"));
    let root = dir.path();
    fs::write(
        root.join("src/engine.cpp"),
        "int engine() {\n  return 1;\n}\n",
    )
    .unwrap();

    ok(&wbh(root, &["init", "--style", "stamp"]));
    ok(&wbh(root, &["mark", "src/engine.cpp"]));
    ok(&wbh(root, &["scan"]));
    let out = ok(&wbh(root, &["check"]));
    assert!(out.contains("generated files are up to date"), "{out}");
    assert!(!root.join(".wbh/report.json").exists(), "JSON is opt-in");

    for a in ARTIFACTS {
        assert!(root.join(a).is_file(), "{a}");
    }
    assert_eq!(read(root, "src/main.rs"), MAIN_RS);
    let readme = read(root, "README.md");
    assert_eq!(
        readme,
        "# Synth\n\n<!-- wbh:badge:start -->\n[![Written by Hand](.wbh/badge.svg)](.wbh/REPORT.md)\n<!-- wbh:badge:end -->\n\nMakes noise.\n"
    );
    let md = read(root, ".wbh/REPORT.md");
    assert!(
        md.contains(
            "| [src/engine.cpp](../src/engine.cpp) | 100.0% | [L1–L3](../src/engine.cpp#L1-L3) |"
        ),
        "{md}"
    );
    assert!(
        md.contains("| [src/main.rs](../src/main.rs) | 75.0% | [L2–L4](../src/main.rs#L2-L4) |"),
        "{md}"
    );
    assert!(
        md.contains("**Self-declared handwritten coverage:** 85.7%"),
        "{md}"
    );
    assert!(
        !read(root, ".wbh/badge.svg").contains('%'),
        "no percentage by default"
    );
}

#[test]
fn report_json_golden() {
    let dir = project(None);
    ok(&wbh(dir.path(), &["init"]));
    enable_json(dir.path());
    let name = dir.path().canonicalize().unwrap();
    let name = name.file_name().unwrap().to_str().unwrap();
    assert_eq!(
        read(dir.path(), ".wbh/report.json"),
        format!(
            r#"{{
  "schema_version": 1,
  "project": {{
    "name": "{name}"
  }},
  "declaration": "self-reported",
  "stats": {{
    "files_scanned": 1,
    "files_with_marks": 1,
    "eligible_lines": 4,
    "marked_lines": 3,
    "percentage": 75.0
  }},
  "files": [
    {{
      "path": "src/main.rs",
      "physical_lines": 7,
      "eligible_lines": 4,
      "marked_lines": 3,
      "percentage": 75.0,
      "claim_sources": [
        "block-marker"
      ],
      "ranges": [
        [
          2,
          4
        ]
      ]
    }}
  ]
}}
"#
        )
    );
}

#[test]
fn repeat_runs_change_nothing() {
    let dir = project(Some("# T\n"));
    let root = dir.path();
    ok(&wbh(root, &["init"]));
    let files = [".wbh.toml", "README.md", ARTIFACTS[0], ARTIFACTS[1]];
    let snapshot = || {
        files
            .iter()
            .map(|f| {
                (
                    read(root, f),
                    fs::metadata(root.join(f)).unwrap().modified().unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    let before = snapshot();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let out = ok(&wbh(root, &["scan"]));
    assert_eq!(out.matches("(unchanged)").count(), 3, "{out}");
    ok(&wbh(root, &["init"]));
    assert_eq!(snapshot(), before);
    assert_eq!(
        read(root, "README.md").matches("wbh:badge:start").count(),
        1
    );
}

#[test]
fn check_is_lenient_about_staleness_unless_strict() {
    let dir = project(Some("# T\n"));
    let root = dir.path();
    ok(&wbh(root, &["init"]));

    // Ordinary code changes make outputs stale, but don't fail plain `check`.
    fs::write(root.join("src/new.rs"), "fn new() {}\n").unwrap();
    let out = ok(&wbh(root, &["check"]));
    assert!(out.contains("markers and config are valid"), "{out}");
    assert!(out.contains("out of date: .wbh/REPORT.md"), "{out}");
    let err = fail(&wbh(root, &["check", "--strict"]));
    assert!(err.contains("out of date: .wbh/REPORT.md"), "{err}");
    assert!(
        !read(root, ".wbh/REPORT.md").contains("other scanned file"),
        "check must not write"
    );

    ok(&wbh(root, &["scan"]));
    ok(&wbh(root, &["check", "--strict"]));

    fs::remove_file(root.join(".wbh/badge.svg")).unwrap();
    assert!(fail(&wbh(root, &["check", "--strict"])).contains("missing: .wbh/badge.svg"));
    assert!(!root.join(".wbh/badge.svg").exists());

    // A README edited by hand to drop the badge is stale too.
    ok(&wbh(root, &["scan"]));
    fs::write(root.join("README.md"), "# T\n").unwrap();
    assert!(fail(&wbh(root, &["check", "--strict"])).contains("out of date: README.md"));

    // Broken markers always fail, strict or not.
    fs::write(root.join("src/bad.rs"), "// @handwritten:end\n").unwrap();
    assert!(fail(&wbh(root, &["check"])).contains("src/bad.rs:1"));
}

#[test]
fn check_tolerates_crlf_checkouts() {
    let dir = project(Some("# T\n"));
    let root = dir.path();
    ok(&wbh(root, &["init"]));
    for f in ARTIFACTS {
        let crlf = read(root, f).replace('\n', "\r\n");
        fs::write(root.join(f), crlf).unwrap();
    }
    ok(&wbh(root, &["check", "--strict"]));
}

#[test]
fn readme_line_endings_and_bom_preserved() {
    let dir = project(Some("\u{feff}# T\r\n\r\nBody\r\n"));
    let root = dir.path();
    ok(&wbh(root, &["init"]));
    let readme = read(root, "README.md");
    assert!(readme.starts_with("\u{feff}# T\r\n\r\n<!-- wbh:badge:start -->\r\n"));
    assert!(readme.ends_with("<!-- wbh:badge:end -->\r\n\r\nBody\r\n"));
    ok(&wbh(root, &["check"]));
}

#[test]
fn badge_commands_update_config_svg_and_keep_one_readme_block() {
    let dir = project(Some("# T\n"));
    let root = dir.path();
    ok(&wbh(root, &["init"]));
    let stamp = read(root, ".wbh/badge.svg");
    ok(&wbh(root, &["badge", "set", "terminal"]));
    assert!(read(root, ".wbh.toml").contains("style = \"terminal\""));
    assert_ne!(read(root, ".wbh/badge.svg"), stamp);
    assert!(read(root, ".wbh/badge.svg").contains("~$"));
    assert_eq!(
        read(root, "README.md").matches("wbh:badge:start").count(),
        1
    );
    ok(&wbh(root, &["check"]));

    let list = ok(&wbh(root, &["badge", "list"]));
    assert!(list.contains("stamp") && list.contains("pixel") && list.contains("coffee"));

    let out = ok(&wbh(
        root,
        &["badge", "set", "--compact", "--no-percentage"],
    ));
    assert!(out.contains("style        terminal"), "{out}");
    assert!(
        out.contains("compact      yes") && out.contains("percentage   hidden"),
        "{out}"
    );
    let cfg = read(root, ".wbh.toml");
    assert!(cfg.contains("compact = true") && cfg.contains("show_percentage = false"));
    assert!(!read(root, ".wbh/badge.svg").contains('%'));
    ok(&wbh(root, &["check"]));

    let show = ok(&wbh(root, &["badge"]));
    assert!(show.contains("style        terminal"), "{show}");
    // Nothing to change, or contradictory flags, is a usage error.
    assert_eq!(wbh(root, &["badge", "set"]).status.code(), Some(2));
    assert_eq!(
        wbh(root, &["badge", "set", "--compact", "--no-compact"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(wbh(root, &["badge", "set", "comic"]).status.code(), Some(2));
}

#[test]
fn failed_scan_leaves_outputs_untouched() {
    let dir = project(Some("# T\n"));
    let root = dir.path();
    ok(&wbh(root, &["init"]));
    let before: Vec<_> = ARTIFACTS.iter().map(|a| read(root, a)).collect();
    let readme = read(root, "README.md");

    fs::write(root.join("src/bad.rs"), "// @handwritten:end\n").unwrap();
    assert!(fail(&wbh(root, &["scan"])).contains("src/bad.rs:1"));
    fs::remove_file(root.join("src/bad.rs")).unwrap();

    // Malformed README block: stop, don't guess, write nothing.
    fs::write(root.join("src/extra.rs"), "fn e() {}\n").unwrap();
    fs::write(root.join("README.md"), "# T\n<!-- wbh:badge:start -->\n").unwrap();
    assert!(fail(&wbh(root, &["scan"])).contains("malformed wbh badge markers"));

    let after: Vec<_> = ARTIFACTS.iter().map(|a| read(root, a)).collect();
    assert_eq!(after, before);
    assert_ne!(read(root, "README.md"), readme);
}

#[test]
fn ignored_dirs_do_not_count() {
    let dir = project(None);
    let root = dir.path();
    for d in ["target", "build", "vendor", "node_modules", ".git", ".wbh"] {
        fs::create_dir_all(root.join(d)).unwrap();
        fs::write(root.join(d).join("x.rs"), "fn x() {}\nfn y() {}\n").unwrap();
    }
    ok(&wbh(root, &["init", "--no-readme"]));
    enable_json(root);
    let json = read(root, ".wbh/report.json");
    assert!(json.contains("\"files_scanned\": 1"), "{json}");
    assert!(json.contains("\"eligible_lines\": 4"));
}

#[test]
fn untrusted_names_and_labels_are_escaped() {
    let dir = project(None);
    let root = dir.path();
    let name = "src/a&b [x](y) ü.rs";
    fs::write(root.join(name), "// @handwritten\nfn a() {}\n").unwrap();
    ok(&wbh(root, &["init", "--no-readme"]));
    let cfg = read(root, ".wbh.toml")
        .replace("json = false", "json = true")
        .replace(
            "text = \"Written by Hand\"",
            "text = \"<script>x</script> & \\\"q\\\"\"",
        );
    fs::write(root.join(".wbh.toml"), cfg).unwrap();
    ok(&wbh(root, &["scan"]));

    let svg = read(root, ".wbh/badge.svg");
    assert!(!svg.contains("<script"), "{svg}");
    assert!(svg.contains("&lt;script&gt;"));
    let md = read(root, ".wbh/REPORT.md");
    assert!(
        md.contains("[src/a&amp;b \\[x\\]\\(y\\) ü.rs](../src/a%26b%20%5Bx%5D%28y%29%20%C3%BC.rs)"),
        "{md}"
    );
    assert!(read(root, ".wbh/report.json").contains("\"path\": \"src/a&b [x](y) ü.rs\""));
}

#[test]
fn empty_project_badge_is_not_100() {
    let dir = tempfile::tempdir().unwrap();
    ok(&wbh(dir.path(), &["init"]));
    enable_json(dir.path());
    assert!(read(dir.path(), ".wbh/report.json").contains("\"percentage\": null"));
    ok(&wbh(dir.path(), &["badge", "set", "--percentage"]));
    let svg = read(dir.path(), ".wbh/badge.svg");
    assert!(svg.contains("N/A") && !svg.contains("100.0%"));
    assert!(read(dir.path(), ".wbh/REPORT.md").contains("N/A"));
}

#[test]
fn readme_opt_out_is_respected() {
    let dir = project(Some("# T\n"));
    let root = dir.path();
    ok(&wbh(root, &["init", "--no-readme"]));
    assert_eq!(read(root, "README.md"), "# T\n");
    ok(&wbh(root, &["check"]));
}

#[cfg(unix)]
#[test]
fn outputs_never_written_through_symlinks() {
    let dir = project(None);
    let root = dir.path();
    ok(&wbh(root, &["init", "--no-readme"]));
    fs::remove_dir_all(root.join(".wbh")).unwrap();
    std::os::unix::fs::symlink(root.join("src"), root.join(".wbh")).unwrap();
    assert!(fail(&wbh(root, &["scan"])).contains("symlink"));
    assert!(!root.join("src/badge.svg").exists());
    assert_eq!(read(root, "src/main.rs"), MAIN_RS);
}

#[test]
fn committed_examples_are_up_to_date() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut n = 0;
    for entry in fs::read_dir(examples).unwrap() {
        let out = wbh(&entry.unwrap().path(), &["check", "--strict"]);
        ok(&out);
        n += 1;
    }
    assert_eq!(n, 3);
}
