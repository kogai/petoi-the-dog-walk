//! Repository checks that the compiler cannot express. Run with `cargo xtask <check>`.
//!
//! - `deps`: every workspace crate may depend only on the crates listed in
//!   docs/design/02-architecture.md §2 ("依存してよいもの"). The table below mirrors that section;
//!   change both together.
//!   Normal and build dependencies are checked; dev-dependencies only for the rules in
//!   `FORBIDDEN_DEV_DEPS`. External crates are not checked (review covers them).
//! - `profiles`: no Cargo profile may set `panic = "abort"`, neither in a manifest nor in
//!   `.cargo/config.toml` (profiles or `-C panic=abort` in rustflags). Environment variables such as
//!   `CARGO_PROFILE_*_PANIC` are not checked (docs/rules/coding.md §3).
//! - `lints`: every workspace crate sets `[lints] workspace = true`, so the lint policy applies.
#![expect(
    clippy::print_stderr,
    reason = "a command-line checker reports on stderr"
)]

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde::Deserialize;

/// Which workspace crates each workspace crate may use as a normal (non-dev) dependency.
/// `None` means "no normal dependencies at all, not even external ones".
const ALLOWED_WORKSPACE_DEPS: &[(&str, Option<&[&str]>)] = &[
    ("walk-core", None),
    ("walk-jev", Some(&["walk-core"])),
    ("walk-bittle", Some(&["walk-core"])),
    ("walk-flybrain", Some(&["walk-core"])),
    (
        "walk-app",
        Some(&["walk-core", "walk-jev", "walk-bittle", "walk-flybrain"]),
    ),
    ("walk-testing", Some(&["walk-core"])),
    ("walk-experiments", Some(&[])),
    ("xtask", Some(&[])),
];

/// Crates that may appear only under `[dev-dependencies]`.
const DEV_ONLY: &[&str] = &["walk-testing"];

/// Dev-dependencies that are forbidden: walk-testing depends on walk-core, so walk-core's own
/// tests using it would create a cycle (docs/design/02-architecture.md §2).
const FORBIDDEN_DEV_DEPS: &[(&str, &str)] = &[("walk-core", "walk-testing")];

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
    workspace_root: PathBuf,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    name: String,
    kind: Option<String>,
    rename: Option<String>,
    target: Option<String>,
}

impl Dependency {
    fn is_dev(&self) -> bool {
        self.kind.as_deref() == Some("dev")
    }

    /// Where the dependency is declared, for error messages.
    fn describe(&self) -> String {
        let section = match self.kind.as_deref() {
            Some("dev") => "dev-dependencies",
            Some("build") => "build-dependencies",
            Some(_) | None => "dependencies",
        };
        let target = self
            .target
            .as_deref()
            .map_or(String::new(), |t| format!(" (target {t})"));
        let rename = self
            .rename
            .as_deref()
            .map_or(String::new(), |r| format!(" as {r}"));
        format!("{}{rename} in [{section}]{target}", self.name)
    }
}

enum Error {
    Cargo(String),
    Io(PathBuf, std::io::Error),
    Parse(String),
    Violations(Vec<String>),
    Usage,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(msg) => write!(f, "cargo metadata failed: {msg}"),
            Self::Io(path, err) => write!(f, "cannot read {}: {err}", path.display()),
            Self::Parse(msg) => write!(f, "cannot parse: {msg}"),
            Self::Violations(list) => {
                writeln!(f, "{} violation(s):", list.len())?;
                list.iter().try_for_each(|v| writeln!(f, "  - {v}"))
            }
            Self::Usage => write!(f, "usage: cargo xtask <deps|profiles|lints>"),
        }
    }
}

fn metadata() -> Result<Metadata, Error> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .output()
        .map_err(|e| Error::Cargo(e.to_string()))?;
    if !output.status.success() {
        return Err(Error::Cargo(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| Error::Parse(e.to_string()))
}

fn members(meta: &Metadata) -> Vec<&Package> {
    let ids: BTreeSet<&str> = meta.workspace_members.iter().map(String::as_str).collect();
    meta.packages
        .iter()
        .filter(|p| ids.contains(p.id.as_str()))
        .collect()
}

/// Pure check of the dependency table; returns human-readable violations.
fn dependency_violations(packages: &[&Package]) -> Vec<String> {
    let workspace: BTreeSet<&str> = packages.iter().map(|p| p.name.as_str()).collect();
    let table = packages.iter().flat_map(|pkg| {
        let linked: Vec<&Dependency> = pkg.dependencies.iter().filter(|d| !d.is_dev()).collect();
        let rule = ALLOWED_WORKSPACE_DEPS
            .iter()
            .find(|(name, _)| *name == pkg.name);
        match rule {
            None => vec![format!(
                "{} is not in the table in xtask/src/main.rs; add it together with \
                 docs/design/02-architecture.md §2",
                pkg.name
            )],
            Some((_, None)) => linked
                .iter()
                .map(|d| {
                    format!(
                        "{} must have no dependencies but uses {}",
                        pkg.name,
                        d.describe()
                    )
                })
                .collect(),
            Some((_, Some(allowed))) => linked
                .iter()
                .filter(|d| workspace.contains(d.name.as_str()))
                .filter_map(|d| {
                    if DEV_ONLY.contains(&d.name.as_str()) {
                        Some(format!(
                            "{} uses {}; it is dev-only",
                            pkg.name,
                            d.describe()
                        ))
                    } else if allowed.contains(&d.name.as_str()) {
                        None
                    } else {
                        Some(format!("{} may not use {}", pkg.name, d.describe()))
                    }
                })
                .collect(),
        }
    });
    let dev = packages.iter().flat_map(|pkg| {
        pkg.dependencies
            .iter()
            .filter(|d| d.is_dev())
            .filter(|d| FORBIDDEN_DEV_DEPS.contains(&(pkg.name.as_str(), d.name.as_str())))
            .map(|d| format!("{} may not use {} (cycle)", pkg.name, d.describe()))
            .collect::<Vec<_>>()
    });
    table.chain(dev).collect()
}

fn check_deps() -> Result<(), Error> {
    let meta = metadata()?;
    let violations = dependency_violations(&members(&meta));
    if violations.is_empty() {
        Ok(())
    } else {
        Err(Error::Violations(violations))
    }
}

/// Returns the profile names in a manifest that set `panic = "abort"`.
fn abort_profiles(manifest: &toml::Table) -> Vec<String> {
    manifest
        .get("profile")
        .and_then(toml::Value::as_table)
        .map(|profiles| {
            profiles
                .iter()
                .filter(|(_, p)| p.get("panic").and_then(toml::Value::as_str) == Some("abort"))
                .map(|(name, _)| name.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn read_manifest(path: &Path) -> Result<toml::Table, Error> {
    let text = std::fs::read_to_string(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
    text.parse::<toml::Table>()
        .map_err(|e| Error::Parse(format!("{}: {e}", path.display())))
}

/// Returns `true` if a `.cargo/config.toml` rustflags list asks for `panic=abort`.
fn rustflags_abort(config: &toml::Table) -> bool {
    let flags = |table: Option<&toml::Value>| -> Vec<String> {
        table
            .and_then(|t| t.get("rustflags"))
            .and_then(toml::Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let build = flags(config.get("build"));
    let targets: Vec<String> = config
        .get("target")
        .and_then(toml::Value::as_table)
        .map(|t| t.values().flat_map(|v| flags(Some(v))).collect())
        .unwrap_or_default();
    let all: Vec<String> = build.into_iter().chain(targets).collect();
    all.iter()
        .any(|f| f.replace(' ', "").contains("panic=abort"))
}

fn check_profiles() -> Result<(), Error> {
    let meta = metadata()?;
    let manifests: BTreeSet<PathBuf> = members(&meta)
        .iter()
        .map(|p| p.manifest_path.clone())
        .chain(std::iter::once(meta.workspace_root.join("Cargo.toml")))
        .collect();
    let config_path = meta.workspace_root.join(".cargo").join("config.toml");
    let config = if config_path.exists() {
        Some(read_manifest(&config_path)?)
    } else {
        None
    };
    let files = manifests
        .iter()
        .map(|path| read_manifest(path).map(|m| (path.clone(), m)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .chain(config.clone().map(|c| (config_path.clone(), c)));
    let mut violations: Vec<String> = files
        .flat_map(|(path, table)| {
            // Relative path: output may be pasted into this public repository.
            let shown = path
                .strip_prefix(&meta.workspace_root)
                .unwrap_or(&path)
                .to_path_buf();
            abort_profiles(&table)
                .into_iter()
                .map(move |name| {
                    format!("{}: profile.{name} sets panic = \"abort\"", shown.display())
                })
                .collect::<Vec<_>>()
        })
        .collect();
    if config.as_ref().is_some_and(rustflags_abort) {
        violations.push(".cargo/config.toml: rustflags contain panic=abort".to_owned());
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(Error::Violations(violations))
    }
}

/// Returns `true` if the manifest opts into the workspace lint policy.
fn uses_workspace_lints(manifest: &toml::Table) -> bool {
    manifest
        .get("lints")
        .and_then(|l| l.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn check_lints() -> Result<(), Error> {
    let meta = metadata()?;
    let violations = members(&meta)
        .iter()
        .map(|p| read_manifest(&p.manifest_path).map(|m| (p.name.clone(), m)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(_, m)| !uses_workspace_lints(m))
        .map(|(name, _)| format!("{name} does not set [lints] workspace = true"))
        .collect::<Vec<_>>();
    if violations.is_empty() {
        Ok(())
    } else {
        Err(Error::Violations(violations))
    }
}

fn main() -> ExitCode {
    let result = match std::env::args().nth(1).as_deref() {
        Some("deps") => check_deps(),
        Some("profiles") => check_profiles(),
        Some("lints") => check_lints(),
        Some(_) | None => Err(Error::Usage),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, deps: &[(&str, Option<&str>)]) -> Package {
        Package {
            id: name.to_owned(),
            name: name.to_owned(),
            manifest_path: PathBuf::new(),
            dependencies: deps
                .iter()
                .map(|(n, k)| Dependency {
                    name: (*n).to_owned(),
                    kind: k.map(str::to_owned),
                    rename: None,
                    target: None,
                })
                .collect(),
        }
    }

    #[test]
    fn allowed_layout_has_no_violations() {
        let core = pkg("walk-core", &[("proptest", Some("dev"))]);
        let jev = pkg("walk-jev", &[("walk-core", None), ("serde", None)]);

        let v = dependency_violations(&[&core, &jev]);

        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn core_with_any_normal_dependency_is_rejected() {
        let core = pkg("walk-core", &[("serde", None)]);

        let v = dependency_violations(&[&core]);

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn upward_dependency_is_rejected() {
        let jev = pkg("walk-jev", &[("walk-bittle", None)]);
        let bittle = pkg("walk-bittle", &[]);

        let v = dependency_violations(&[&jev, &bittle]);

        assert_eq!(
            v,
            vec!["walk-jev may not use walk-bittle in [dependencies]".to_owned()]
        );
    }

    #[test]
    fn testing_crate_as_normal_dependency_is_rejected() {
        let app = pkg("walk-app", &[("walk-testing", None)]);
        let testing = pkg("walk-testing", &[]);

        let v = dependency_violations(&[&app, &testing]);

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn unknown_crate_is_rejected() {
        let other = pkg("walk-other", &[]);

        let v = dependency_violations(&[&other]);

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn build_dependency_counts_like_a_normal_one() {
        let core = pkg("walk-core", &[("cc", Some("build"))]);
        let jev = pkg("walk-jev", &[("walk-testing", Some("build"))]);
        let testing = pkg("walk-testing", &[]);

        let v = dependency_violations(&[&core, &jev, &testing]);

        assert_eq!(v.len(), 2, "{v:?}");
    }

    #[test]
    fn core_using_testing_as_dev_dependency_is_rejected() {
        let core = pkg("walk-core", &[("walk-testing", Some("dev"))]);
        let testing = pkg("walk-testing", &[]);

        let v = dependency_violations(&[&core, &testing]);

        assert_eq!(
            v,
            vec!["walk-core may not use walk-testing in [dev-dependencies] (cycle)"]
        );
    }

    #[test]
    fn rustflags_abort_is_detected() {
        let config: toml::Table = "[build]\nrustflags = [\"-C\", \"panic=abort\"]\n"
            .parse()
            .unwrap();
        let clean: toml::Table = "[alias]\nxtask = \"run\"\n".parse().unwrap();

        assert!(rustflags_abort(&config));
        assert!(!rustflags_abort(&clean));
    }

    #[test]
    fn missing_workspace_lints_is_detected() {
        let with: toml::Table = "[lints]\nworkspace = true\n".parse().unwrap();
        let without: toml::Table = "[package]\nname = \"x\"\n".parse().unwrap();

        assert!(uses_workspace_lints(&with));
        assert!(!uses_workspace_lints(&without));
    }

    #[test]
    fn abort_profile_is_detected() {
        let manifest: toml::Table =
            "[profile.release]\npanic = \"abort\"\n[profile.dev]\npanic = \"unwind\"\n"
                .parse()
                .unwrap();

        assert_eq!(abort_profiles(&manifest), vec!["release".to_owned()]);
    }
}
