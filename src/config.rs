use crate::filter::Filter;
use crate::paths::{normalize_rel, resolve_inside};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub const CONFIG_FILE: &str = ".wbh.toml";
pub const BADGE_FILE: &str = ".wbh/badge.svg";
pub const REPORT_MD: &str = ".wbh/REPORT.md";
pub const REPORT_JSON: &str = ".wbh/report.json";

const DEFAULT_EXCLUDE: &[&str] = &[
    "**/node_modules/**",
    "**/target/**",
    "**/vendor/**",
    "**/third_party/**",
    "**/build/**",
    "**/dist/**",
    "**/generated/**",
    "**/docs/**",
    "**/*.json",
    "**/*.jsonc",
    "**/*.yml",
    "**/*.yaml",
    "**/*.toml",
    "**/*.md",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum BadgeStyle {
    Stamp,
    Classic,
    Terminal,
    Pixel,
    Minimal,
    Coffee,
}

impl BadgeStyle {
    pub const ALL: [BadgeStyle; 6] = [
        Self::Stamp,
        Self::Classic,
        Self::Terminal,
        Self::Pixel,
        Self::Minimal,
        Self::Coffee,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Stamp => "stamp",
            Self::Classic => "classic",
            Self::Terminal => "terminal",
            Self::Pixel => "pixel",
            Self::Minimal => "minimal",
            Self::Coffee => "coffee",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Stamp => "Rubber Stamp: warm paper and ink, handmade feeling",
            Self::Classic => "Classic: compact two-tone GitHub-style badge",
            Self::Terminal => "Terminal: green-on-dark monospace",
            Self::Pixel => "Pixel: 8-bit block lettering",
            Self::Minimal => "Minimal: quiet type with a pencil",
            Self::Coffee => "Coffee Edition: a little coffee-shop seal",
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub project: ProjectConfig,
    pub badge: BadgeConfig,
    pub scan: ScanConfig,
    pub claims: ClaimsConfig,
    pub report: ReportConfig,
    pub readme: ReadmeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            project: ProjectConfig::default(),
            badge: BadgeConfig::default(),
            scan: ScanConfig::default(),
            claims: ClaimsConfig::default(),
            report: ReportConfig::default(),
            readme: ReadmeConfig::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectConfig {
    /// Empty means "use the root directory name".
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BadgeConfig {
    pub style: BadgeStyle,
    pub show_percentage: bool,
    pub compact: bool,
    pub text: String,
}

impl Default for BadgeConfig {
    fn default() -> Self {
        Self {
            style: BadgeStyle::Stamp,
            show_percentage: false,
            compact: false,
            text: "Written by Hand".into(),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScanConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub respect_gitignore: bool,
    pub max_file_bytes: u64,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include: vec!["**/*".into()],
            exclude: DEFAULT_EXCLUDE.iter().map(|s| s.to_string()).collect(),
            respect_gitignore: true,
            max_file_bytes: 2 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClaimsConfig {
    pub files: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReportConfig {
    /// Also write `.wbh/report.json`.
    pub json: bool,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReadmeConfig {
    pub enabled: bool,
    pub path: String,
}

impl Default for ReadmeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "README.md".into(),
        }
    }
}

/// Reads and validates `<root>/.wbh.toml`. `root` must be canonical.
pub fn load(root: &Path) -> Result<Config> {
    let path = root.join(CONFIG_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!(
                "no {CONFIG_FILE} in {}\nhint: run `wbh init` first",
                root.display()
            )
        }
        Err(e) => return Err(e).with_context(|| format!("cannot read {}", path.display())),
    };
    parse(&text, root)
}

pub fn parse(text: &str, root: &Path) -> Result<Config> {
    let mut cfg: Config =
        toml_edit::de::from_str(text).with_context(|| format!("invalid {CONFIG_FILE}"))?;
    if cfg.project.name.is_empty() {
        cfg.project.name = default_name(root);
    }
    cfg.validate(root)
        .with_context(|| format!("invalid {CONFIG_FILE}"))?;
    Ok(cfg)
}

impl Config {
    fn validate(&mut self, root: &Path) -> Result<()> {
        if self.version != 1 {
            bail!(
                "unsupported version {}; this wbh supports version 1",
                self.version
            );
        }
        check_text("project.name", &self.project.name, 100)?;
        check_text("badge.text", &self.badge.text, 60)?;
        if self.scan.max_file_bytes == 0 {
            bail!("scan.max_file_bytes must be greater than 0");
        }
        Filter::new(&self.scan)?;
        for f in &self.claims.files {
            normalize_rel(f).with_context(|| format!("claims.files entry {f:?}"))?;
        }
        // Stored in `a/b` form so link math downstream can rely on it.
        self.readme.path = normalize_rel(&self.readme.path)
            .with_context(|| format!("readme.path = {:?}", self.readme.path))?;
        if self.readme.path.starts_with(".wbh/") {
            bail!("readme.path must not be inside .wbh/");
        }
        if self.readme.enabled {
            resolve_inside(root, &self.readme.path)
                .with_context(|| format!("readme.path = {:?}", self.readme.path))?;
        }
        Ok(())
    }
}

fn check_text(key: &str, value: &str, max: usize) -> Result<()> {
    let len = value.chars().count();
    if len == 0 || len > max {
        bail!("{key} must be 1–{max} characters (got {len})");
    }
    if value.chars().any(char::is_control) {
        bail!("{key} must not contain control characters or newlines");
    }
    Ok(())
}

/// Root directory name, cleaned up to pass `project.name` validation.
pub fn default_name(root: &Path) -> String {
    let name: String = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .take(100)
        .collect();
    if name.is_empty() {
        "project".into()
    } else {
        name
    }
}

/// The `.wbh.toml` written by `wbh init`, with every default spelled out.
pub fn template(name: &str, style: BadgeStyle, readme_enabled: bool) -> String {
    let name = toml_edit::Value::from(name).to_string();
    let exclude: String = DEFAULT_EXCLUDE
        .iter()
        .map(|p| format!("  \"{p}\",\n"))
        .collect();
    let style = style.key();
    format!(
        r#"# Written by Hand configuration.
# Handwritten status is self-declared; wbh performs no verification or AI detection.
version = 1

[project]
name = {name}

[badge]
# One of: stamp, classic, terminal, pixel, minimal, coffee
style = "{style}"
# Show the handwritten percentage on the badge. It changes whenever code changes.
show_percentage = false
compact = false
text = "Written by Hand"

[scan]
include = ["**/*"]
# Exclude wins over include. Hidden paths, .git/ and .wbh/ are always skipped.
# Data and docs formats are excluded so lockfiles don't swamp the numbers;
# remove a line to opt that format in.
exclude = [
{exclude}]
respect_gitignore = true
max_file_bytes = 2097152

[claims]
# Whole files declared handwritten. Manage with `wbh mark` / `wbh unmark`.
files = []

[report]
# Also write .wbh/report.json, a machine-readable list of every scanned file.
json = false

[readme]
enabled = {readme_enabled}
path = "README.md"
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        fs::canonicalize(std::env::temp_dir()).unwrap()
    }

    fn err(text: &str) -> String {
        format!("{:#}", parse(text, &root()).unwrap_err())
    }

    #[test]
    fn template_matches_defaults() {
        let cfg = parse(
            &template("My \"Synth\"", BadgeStyle::Terminal, false),
            &root(),
        )
        .unwrap();
        let mut expected = Config::default();
        expected.project.name = "My \"Synth\"".into();
        expected.badge.style = BadgeStyle::Terminal;
        expected.readme.enabled = false;
        assert_eq!(cfg, expected);
    }

    #[test]
    fn empty_file_uses_defaults() {
        let cfg = parse("", &root()).unwrap();
        assert_eq!(cfg.badge, BadgeConfig::default());
        assert!(!cfg.badge.show_percentage && !cfg.report.json);
        assert_eq!(cfg.project.name, default_name(&root()));
    }

    #[test]
    fn rejects_bad_configs() {
        assert!(err("version = 2").contains("unsupported version"));
        assert!(err("colour = 1").contains("unknown field"));
        assert!(err("[badge]\nstlye = \"stamp\"").contains("unknown field"));
        assert!(err("[badge]\nstyle = \"comic\"").contains("unknown variant"));
        assert!(err("[badge]\ntext = \"\"").contains("badge.text"));
        assert!(err("[badge]\ntext = \"a\\nb\"").contains("control"));
        assert!(err(&format!("[badge]\ntext = \"{}\"", "x".repeat(61))).contains("badge.text"));
        assert!(err("[scan]\nexclude = [\"a/[\"]").contains("invalid glob"));
        // Output paths are fixed, and the Code Map isn't part of wbh.
        assert!(err("[badge]\noutput = \"x.svg\"").contains("unknown field"));
        assert!(err("[site]\nenabled = true").contains("unknown field"));
        assert!(err("[readme]\npath = \"../README.md\"").contains("outside"));
        assert!(err("[readme]\npath = \"/tmp/README.md\"").contains("absolute"));
        assert!(err("[readme]\npath = \".wbh/README.md\"").contains("inside .wbh/"));
        assert!(err("[claims]\nfiles = [\"../x.rs\"]").contains("outside"));
        assert!(err("[scan]\nmax_file_bytes = 0").contains("max_file_bytes"));
    }
}
