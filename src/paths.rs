use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Normalizes a user-supplied, root-relative path to `a/b/c` form.
/// Rejects absolute paths and anything that climbs out of the root.
pub fn normalize_rel(p: &str) -> Result<String> {
    if p.chars().any(char::is_control) {
        bail!("path contains control characters");
    }
    let p = if cfg!(windows) {
        p.replace('\\', "/")
    } else {
        p.to_string()
    };
    if Path::new(&p)
        .components()
        .any(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
    {
        bail!("absolute paths are not allowed; use a path relative to the project root");
    }
    let mut parts = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => bail!("path resolves outside project root"),
            s => parts.push(s),
        }
    }
    if parts.is_empty() {
        bail!("path is empty");
    }
    Ok(parts.join("/"))
}

/// Joins a normalized `a/b/c` path onto `root` component by component. A plain
/// `root.join("a/b")` breaks on Windows, where canonical roots are `\\?\C:\...`
/// paths in which `/` is not a separator.
pub fn join(root: &Path, rel: &str) -> PathBuf {
    rel.split('/')
        .fold(root.to_path_buf(), |p, seg| p.join(seg))
}

/// Joins `rel` (already normalized) onto the canonical `root` and checks that no
/// existing part of the path is a symlink, so writes can't be redirected.
pub fn resolve_inside(root: &Path, rel: &str) -> Result<PathBuf> {
    let full = join(root, rel);
    let existing = full
        .ancestors()
        .find(|a| a.symlink_metadata().is_ok())
        .unwrap_or(root);
    let canon = fs::canonicalize(existing)
        .with_context(|| format!("cannot resolve {}", existing.display()))?;
    if !canon.starts_with(root) {
        bail!("path resolves outside project root");
    }
    if canon != existing {
        bail!("path goes through a symlink; wbh won't write through symlinks");
    }
    Ok(full)
}

/// Resolves `rel` to a regular file inside `root` reached without any symlinks.
pub fn existing_file(root: &Path, rel: &str) -> Result<PathBuf> {
    let full = join(root, rel);
    let canon = fs::canonicalize(&full).map_err(|_| anyhow::anyhow!("no such file"))?;
    if !canon.starts_with(root) {
        bail!("path resolves outside project root");
    }
    if canon != full {
        bail!("path goes through a symlink; symlinks are not followed");
    }
    if !canon.is_file() {
        bail!("not a regular file");
    }
    Ok(full)
}

/// Writes `bytes` via temp file + rename, but only when the content differs,
/// so unchanged files keep their modification time. Returns whether it wrote.
pub fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if fs::read(path).is_ok_and(|old| old == bytes) {
        return Ok(false);
    }
    let parent = path.parent().context("output path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("cannot create {}", parent.display()))?;
    let name = path.file_name().context("output path has no file name")?;
    let tmp = parent.join(format!(".{}.tmp", name.to_string_lossy()));
    fs::write(&tmp, bytes).with_context(|| format!("cannot write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("cannot replace {}", path.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_relative_paths() {
        assert_eq!(normalize_rel("src/main.rs").unwrap(), "src/main.rs");
        assert_eq!(normalize_rel("./src//main.rs").unwrap(), "src/main.rs");
        assert!(normalize_rel("../outside.rs").is_err());
        assert!(normalize_rel("src/../../x.rs").is_err());
        assert!(normalize_rel("/etc/passwd").is_err());
        assert!(normalize_rel("").is_err());
        assert!(normalize_rel(".").is_err());
        assert!(normalize_rel("a\nb").is_err());
    }
}
