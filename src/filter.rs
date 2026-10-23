use crate::config::ScanConfig;
use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// Comment syntax family, chosen by file extension (spec §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    CLike,
    Hash,
    Html,
    Css,
    Json,
}

pub fn family(rel: &str) -> Option<Family> {
    let name = rel.rsplit('/').next()?;
    let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
    Some(match ext.as_str() {
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "rs" | "swift" | "js" | "jsx" | "ts"
        | "tsx" | "go" | "java" | "kt" | "kts" => Family::CLike,
        "py" | "sh" | "bash" | "zsh" | "yml" | "yaml" | "rb" | "toml" => Family::Hash,
        "html" | "htm" | "xml" | "md" => Family::Html,
        "css" | "scss" => Family::Css,
        "json" | "jsonc" => Family::Json,
        _ => return None,
    })
}

/// Decides whether a normalized root-relative path is part of the scan.
pub struct Filter {
    include: GlobSet,
    exclude: GlobSet,
}

impl Filter {
    pub fn new(scan: &ScanConfig) -> Result<Self> {
        Ok(Self {
            include: globs("scan.include", &scan.include)?,
            exclude: globs("scan.exclude", &scan.exclude)?,
        })
    }

    /// Why `rel` is not scanned, or `None` if it is.
    pub fn skip_reason(&self, rel: &str) -> Option<&'static str> {
        if rel.split('/').any(|seg| seg.starts_with('.')) {
            Some("hidden paths, .git/ and .wbh/ are never scanned")
        } else if family(rel).is_none() {
            Some("unsupported file extension")
        } else if self.exclude.is_match(rel) {
            Some("matched by scan.exclude")
        } else if !self.include.is_match(rel) {
            Some("not matched by scan.include")
        } else {
            None
        }
    }
}

fn globs(key: &str, patterns: &[String]) -> Result<GlobSet> {
    let mut set = GlobSetBuilder::new();
    for p in patterns {
        let glob = GlobBuilder::new(p)
            .literal_separator(true)
            .build()
            .with_context(|| format!("invalid glob in {key}: {p:?}"))?;
        set.add(glob);
    }
    Ok(set.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter() {
        let f = Filter::new(&ScanConfig::default()).unwrap();
        assert_eq!(f.skip_reason("src/main.rs"), None);
        assert_eq!(f.skip_reason("main.CPP"), None);
        assert!(f.skip_reason("target/debug/x.rs").is_some());
        assert!(f.skip_reason("a/node_modules/b.js").is_some());
        assert!(f.skip_reason("package-lock.json").is_some());
        assert!(f.skip_reason("README.md").is_some());
        assert!(f.skip_reason(".github/x.yml").is_some());
        assert!(f.skip_reason(".wbh/report.json").is_some());
        assert!(f.skip_reason("src/image.png").is_some());
        assert!(f.skip_reason("Makefile").is_some());
    }
}
