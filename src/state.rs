use axum::extract::FromRef;

use crate::configuration::Configuration;

#[derive(Clone, FromRef)]
pub struct AppState {
    config: Configuration,
    pool: database::Pool,
}

impl AppState {
    pub fn new(config: Configuration, pool: database::Pool) -> Self {
        Self { config, pool }
    }
}
