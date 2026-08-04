//! Dependency resolution.

use crate::manifest::{Dependency, Manifest};
use semver::{Version, VersionReq};
use std::fs;
use std::path::PathBuf;

/// A resolved package node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPackage {
    /// Package name.
    pub name: String,
    /// Chosen version.
    pub version: Version,
    /// Local path if path dependency.
    pub path: Option<PathBuf>,
}

/// Resolves dependencies for a root manifest.
///
/// Current strategy:
/// - Root package is always included
/// - Version deps are resolved to the minimum satisfying version `0.1.0` when
///   the req matches, or the exact version if the req is concrete — without a
///   registry we synthesize a satisfying version for offline resolve
/// - Path deps load nested Foxa.toml and recurse
pub fn resolve(root: &Manifest) -> anyhow::Result<Vec<ResolvedPackage>> {
    let mut out = Vec::new();
    let root_ver = Version::parse(&root.package.version)?;
    out.push(ResolvedPackage {
        name: root.package.name.clone(),
        version: root_ver,
        path: None,
    });
    resolve_deps(&root.package.name, &root.dependencies, &mut out)?;
    Ok(out)
}

fn resolve_deps(
    _parent: &str,
    deps: &std::collections::BTreeMap<String, Dependency>,
    out: &mut Vec<ResolvedPackage>,
) -> anyhow::Result<()> {
    for (name, dep) in deps {
        if out.iter().any(|p| p.name == *name) {
            continue;
        }
        if let Some(path) = dep.path() {
            let manifest_path = PathBuf::from(path).join("Foxa.toml");
            let text = fs::read_to_string(&manifest_path)?;
            let nested: Manifest = toml::from_str(&text)?;
            let ver = Version::parse(&nested.package.version)?;
            out.push(ResolvedPackage {
                name: name.clone(),
                version: ver,
                path: Some(PathBuf::from(path)),
            });
            resolve_deps(name, &nested.dependencies, out)?;
        } else if let Some(req_str) = dep.version_req() {
            let ver = pick_version(req_str)?;
            out.push(ResolvedPackage {
                name: name.clone(),
                version: ver,
                path: None,
            });
        }
    }
    Ok(())
}

fn pick_version(req_str: &str) -> anyhow::Result<Version> {
    if req_str == "*" {
        return Ok(Version::new(0, 1, 0));
    }
    if let Ok(exact) = Version::parse(req_str) {
        return Ok(exact);
    }
    let normalized = if req_str.starts_with('^')
        || req_str.starts_with('~')
        || req_str.starts_with('=')
        || req_str.starts_with('>')
        || req_str.starts_with('<')
    {
        req_str.to_string()
    } else {
        format!("^{req_str}")
    };
    let req = VersionReq::parse(&normalized)?;
    for candidate in ["0.1.0", "1.0.0", "0.0.1", "2.0.0", "0.1.1", "1.0.1"] {
        let v = Version::parse(candidate)?;
        if req.matches(&v) {
            return Ok(v);
        }
    }
    anyhow::bail!("no offline candidate satisfies `{req_str}` (registry fetch not yet available)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Dependency, Package};
    use std::collections::BTreeMap;

    #[test]
    fn resolves_version_dep() {
        let mut deps = BTreeMap::new();
        deps.insert("http".into(), Dependency::Version("0.1".into()));
        let m = Manifest {
            package: Package {
                name: "app".into(),
                version: "0.1.0".into(),
                edition: "2026".into(),
            },
            dependencies: deps,
        };
        let g = resolve(&m).unwrap();
        assert!(g.iter().any(|p| p.name == "http"));
        assert!(g.iter().any(|p| p.name == "app"));
    }
}
