//! Structs for Index metadata

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    /// The name of the package
    pub name: String,
    /// Semantic version of the package
    pub vers: String,
    /// The declared dependencies of the package
    pub deps: Vec<Dep>,
    /// The SHA256 Checksum of the package file itself
    pub cksum: String,
    /// The features defined by this package
    pub features: BTreeMap<String, Vec<String>>,
    /// Whether or not this version is yanked from the index
    pub yanked: bool,
    /// The package links field
    pub links: Option<String>,
    /// The entry version (should be 2 to remind older cargo to ignore us)
    pub v: u32,
    /// Newer features
    pub features2: Option<BTreeMap<String, Vec<String>>>,
    /// The minimum rust version this package needs
    pub rust_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dep {
    /// The name of the dependency.  If the dependency is renamed
    /// in the package then this is the "unique" name and the
    /// `package` field contains the original crate name.
    pub name: String,
    /// The Semver requirement for this package
    pub req: String,
    /// The features enabled for this package
    pub features: Vec<String>,
    /// Whether or not this is an optional dependency
    pub optional: bool,
    /// Whether or not default features are enabled for this dependency
    pub default_features: bool,
    /// The target for this dependency if needed
    pub target: Option<String>,
    /// The kind of this dependency
    pub kind: Kind,
    /// The registry that this dependency comes from.  If None then
    /// it comes from this registry
    pub registry: Option<String>,
    /// If the dependency was renamed, this field contains the original crate name
    pub package: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Dev,
    Build,
    Normal,
}

impl From<crate::publish::Dep> for Dep {
    fn from(value: crate::publish::Dep) -> Self {
        Self {
            name: value
                .explicit_name_in_toml
                .clone()
                .unwrap_or_else(|| value.name.clone()),
            req: value.req,
            features: value.features,
            optional: value.optional,
            default_features: value.default_features,
            target: value.target,
            kind: value.kind,
            registry: value.registry,
            package: value.explicit_name_in_toml.and(Some(value.name)),
        }
    }
}

impl Entry {
    /// Create an index entry from a publish entry and a checksum
    pub fn from_publish(value: crate::publish::Metadata, cksum: String) -> Self {
        Self {
            name: value.name,
            vers: value.vers,
            deps: value.deps.into_iter().map(Dep::from).collect(),
            cksum,
            features: value.features,
            yanked: false,
            links: value.links,
            v: 2,
            features2: Default::default(),
            rust_version: value.rust_version,
        }
    }
}
