use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use config::{Config, ConfigError, Environment};
use git_testament::git_testament;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigurationInner {
    #[serde(default = "default_port")]
    port: u16,
    database_url: Url,
    #[serde(default = "String::new")]
    version: String,
    base_url: Url,
    crate_path: PathBuf,
}

fn default_port() -> u16 {
    1537
}

git_testament!(VERSION);

#[derive(Clone)]
pub struct Configuration {
    inner: Arc<ConfigurationInner>,
}

impl std::ops::Deref for Configuration {
    type Target = ConfigurationInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl ConfigurationInner {
    /// Database URL
    pub fn database_url(&self) -> &Url {
        &self.database_url
    }

    /// The port to bind to
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The version of this program
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The base URL of the registry
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// The crates path
    pub fn crate_path(&self) -> &Path {
        &self.crate_path
    }
}

impl Configuration {
    /// Load a configuration from the environment
    pub fn load() -> Result<Configuration, ConfigError> {
        let config = Config::builder().add_source(Environment::default().try_parsing(true));
        let mut inner: ConfigurationInner = config.build()?.try_deserialize()?;
        inner.version = format!("{VERSION}");
        inner.crate_path = std::fs::canonicalize(inner.crate_path)
            .map_err(|e| ConfigError::Foreign(Box::new(e)))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}
