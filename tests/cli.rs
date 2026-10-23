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

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        dir.path().join("src/lib.rs"),
        "// @handwritten\npub fn f() {}\n",
    )
    .unwrap();
    dir
}

fn config(dir: &Path) -> String {
    fs::read_to_string(dir.join(".wbh.toml")).unwrap()
}

#[test]
fn noninteractive_init_defaults_to_stamp_and_never_resets() {
    let dir = project();
    let out = ok(&wbh(dir.path(), &["init"]));
    assert!(out.contains("No README.md found"), "{out}");
    assert!(out.contains("[![Written by Hand](.wbh/badge.svg)](.wbh/REPORT.md)"));
    assert!(!dir.path().join("README.md").exists());
    let cfg = config(dir.path());
    assert!(cfg.contains("style = \"stamp\""));
    assert!(cfg.contains("enabled = true"));

    fs::write(dir.path().join(".wbh.toml"), format!("# mine\n{cfg}")).unwrap();
    ok(&wbh(dir.path(), &["init", "--style", "terminal"]));
    assert!(config(dir.path()).starts_with("# mine\n"));
}

#[test]
fn init_with_options() {
    let dir = project();
    ok(&wbh(
        dir.path(),
        &["init", "--style", "terminal", "--no-readme"],
    ));
    let cfg = config(dir.path());
    assert!(cfg.contains("style = \"terminal\""));
    assert!(cfg.contains("[readme]\nenabled = false"));
}

#[test]
fn mark_and_unmark_edit_only_the_config() {
    let dir = project();
    ok(&wbh(dir.path(), &["init"]));
    let cfg = config(dir.path()).replace("[report]", "# keep me\n[report]");
    fs::write(dir.path().join(".wbh.toml"), &cfg).unwrap();

    ok(&wbh(dir.path(), &["mark", "src/main.rs", "./src//lib.rs"]));
    let out = ok(&wbh(dir.path(), &["mark", "src/main.rs"]));
    assert!(out.contains("already marked"));
    let cfg = config(dir.path());
    assert!(
        cfg.contains("files = [\n  \"src/main.rs\",\n  \"src/lib.rs\",\n]"),
        "{cfg}"
    );
    assert!(cfg.contains("# keep me"));
    assert_eq!(
        fs::read_to_string(dir.path().join("src/main.rs")).unwrap(),
        "fn main() {}\n"
    );

    let out = ok(&wbh(dir.path(), &["unmark", "src/lib.rs"]));
    assert!(out.contains("@handwritten comment"), "{out}");
    let cfg = config(dir.path());
    assert!(cfg.contains("files = [\n  \"src/main.rs\",\n]"), "{cfg}");
    assert!(cfg.contains("# keep me"));

    // A deleted file can still be unmarked.
    fs::remove_file(dir.path().join("src/main.rs")).unwrap();
    ok(&wbh(dir.path(), &["unmark", "src/main.rs"]));
    assert!(config(dir.path()).contains("files = []"));

    let out = wbh(dir.path(), &["unmark", "src/main.rs"]);
    ok(&out);
    assert!(String::from_utf8_lossy(&out.stderr).contains("was not marked"));
}

#[test]
fn mark_rejects_unsafe_and_unscanned_paths() {
    let dir = project();
    ok(&wbh(dir.path(), &["init"]));
    fs::write(dir.path().join("package.json"), "{}").unwrap();
    fs::write(dir.path().join("notes.txt"), "hi").unwrap();
    let before = config(dir.path());

    let cases = [
        ("../outside.cpp", "path resolves outside project root"),
        ("/etc/hosts", "absolute"),
        ("src/missing.rs", "no such file"),
        ("src", "not a regular file"),
        ("package.json", "scan.exclude"),
        ("notes.txt", "unsupported file extension"),
        (".wbh.toml", "never scanned"),
    ];
    for (path, msg) in cases {
        let err = fail(&wbh(dir.path(), &["mark", path]));
        assert!(err.contains(msg), "{path}: {err}");
    }
    // One bad path aborts the whole command.
    fail(&wbh(dir.path(), &["mark", "src/main.rs", "../x.rs"]));
    assert_eq!(config(dir.path()), before);
}

#[cfg(unix)]
#[test]
fn mark_rejects_symlinks() {
    let dir = project();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("evil.rs"), "").unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("evil.rs"),
        dir.path().join("src/evil.rs"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        dir.path().join("src/main.rs"),
        dir.path().join("src/alias.rs"),
    )
    .unwrap();
    ok(&wbh(dir.path(), &["init"]));
    assert!(fail(&wbh(dir.path(), &["mark", "src/evil.rs"])).contains("outside project root"));
    assert!(fail(&wbh(dir.path(), &["mark", "src/alias.rs"])).contains("symlink"));
}

#[test]
fn commands_need_init_and_a_valid_config() {
    let dir = project();
    assert!(fail(&wbh(dir.path(), &["mark", "src/main.rs"])).contains("wbh init"));
    fs::write(dir.path().join(".wbh.toml"), "[badge]\nstyle = \"comic\"\n").unwrap();
    assert!(fail(&wbh(dir.path(), &["mark", "src/main.rs"])).contains("comic"));
}

#[test]
fn usage_errors_exit_2() {
    let dir = project();
    assert_eq!(wbh(dir.path(), &["mark"]).status.code(), Some(2));
    assert_eq!(
        wbh(dir.path(), &["init", "--style", "comic"]).status.code(),
        Some(2)
    );
}

#[test]
fn scan_prints_summary_and_fails_on_bad_markers() {
    let dir = project();
    let out = ok(&wbh(dir.path(), &["init"]));
    assert!(out.contains("2 source files scanned"), "{out}");
    assert!(out.contains("1 / 2 eligible lines marked (50.0%)"), "{out}");

    assert_eq!(
        wbh(dir.path(), &["report"]).status.code(),
        Some(2),
        "alias was removed"
    );

    fs::write(
        dir.path().join("src/bad.rs"),
        "fn x() {}\n// @handwritten:start\n",
    )
    .unwrap();
    let err = fail(&wbh(dir.path(), &["scan"]));
    assert!(
        err.contains("src/bad.rs:2: unclosed @handwritten:start"),
        "{err}"
    );
}

#[test]
fn empty_project_reports_na() {
    let dir = tempfile::tempdir().unwrap();
    let out = wbh(dir.path(), &["init"]);
    assert!(ok(&out).contains("0 / 0 eligible lines marked (N/A)"));
    assert!(String::from_utf8_lossy(&out.stderr).contains("coverage is N/A"));
}

#[test]
fn mark_rejects_gitignored_files() {
    let dir = project();
    fs::write(dir.path().join(".gitignore"), "src/main.rs\n").unwrap();
    ok(&wbh(dir.path(), &["init"]));
    let err = fail(&wbh(dir.path(), &["mark", "src/main.rs"]));
    assert!(err.contains("ignored by .gitignore"), "{err}");
}
