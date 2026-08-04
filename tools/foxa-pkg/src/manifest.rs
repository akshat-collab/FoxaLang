//! Foxa.toml manifest parsing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Root manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package metadata.
    pub package: Package,
    /// Dependencies: name → version req or path table.
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}

/// `[package]` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package name.
    pub name: String,
    /// Semver version.
    pub version: String,
    /// Edition year string.
    #[serde(default = "default_edition")]
    pub edition: String,
}

fn default_edition() -> String {
    "2026".into()
}

/// A dependency specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version requirement string, e.g. `"1.0"`.
    Version(String),
    /// Detailed dependency.
    Detailed {
        /// Version requirement.
        #[serde(default)]
        version: Option<String>,
        /// Local path.
        #[serde(default)]
        path: Option<String>,
    },
}

impl Dependency {
    /// Returns the version requirement if present.
    #[must_use]
    pub fn version_req(&self) -> Option<&str> {
        match self {
            Self::Version(v) => Some(v),
            Self::Detailed { version, .. } => version.as_deref(),
        }
    }

    /// Returns a local path if present.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Detailed { path, .. } => path.as_deref(),
            Self::Version(_) => None,
        }
    }
}

/// Loads a manifest from disk.
pub fn load(path: &Path) -> anyhow::Result<Manifest> {
    let text = fs::read_to_string(path)?;
    let manifest: Manifest = toml::from_str(&text)?;
    Ok(manifest)
}

/// Writes a default manifest.
pub fn write_default(path: &Path, name: &str) -> anyhow::Result<()> {
    let m = Manifest {
        package: Package {
            name: name.to_string(),
            version: "0.1.0".into(),
            edition: "2026".into(),
        },
        dependencies: BTreeMap::new(),
    };
    fs::write(path, toml::to_string_pretty(&m)?)?;
    Ok(())
}

/// Adds a dependency `name` or `name@version` to the manifest.
pub fn add_dependency(path: &Path, spec: &str) -> anyhow::Result<()> {
    let mut m = if path.exists() {
        load(path)?
    } else {
        Manifest {
            package: Package {
                name: "app".into(),
                version: "0.1.0".into(),
                edition: "2026".into(),
            },
            dependencies: BTreeMap::new(),
        }
    };
    let (name, version) = if let Some((n, v)) = spec.split_once('@') {
        (n.to_string(), v.to_string())
    } else {
        (spec.to_string(), "*".to_string())
    };
    m.dependencies.insert(name, Dependency::Version(version));
    fs::write(path, toml::to_string_pretty(&m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn roundtrip_manifest() {
        let dir = std::env::temp_dir().join(format!("foxa-pkg-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Foxa.toml");
        write_default(&path, "demo").unwrap();
        add_dependency(&path, "http@0.1").unwrap();
        let m = load(&path).unwrap();
        assert_eq!(m.package.name, "demo");
        assert!(m.dependencies.contains_key("http"));
        let _ = fs::remove_dir_all(&dir);
        let _ = PathBuf::from(".");
    }
}
