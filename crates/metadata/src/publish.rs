//! Structs for publish metadata

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::index::Kind;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// The name of the package
    pub name: String,
    /// Semantic version of the package
    pub vers: String,
    /// The declared dependencies of the package
    pub deps: Vec<Dep>,
    /// The features defined by this package
    pub features: BTreeMap<String, Vec<String>>,
    /// The package links field
    pub links: Option<String>,
    /// The minimum rust version this package needs
    pub rust_version: Option<String>,
    // In theory there's lots of other stuff we could get here
    // but we won't declare them for now
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dep {
    /// The name of the dependency.  If the dependency is renamed
    /// in the package then this is the original crate name and
    /// the rename is present in the `explicit_name_in_toml` field
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
    /// If the dependency was renamed, this field contains the rename used in Cargo.toml
    pub explicit_name_in_toml: Option<String>,
}
