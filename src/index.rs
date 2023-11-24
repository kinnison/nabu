//! The crate index and such

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use database::{
    models::{Krate, KrateVer},
    Connection,
};
use serde::Serialize;
use thiserror::Error;

use crate::{auth::Authentication, configuration::Configuration, state::AppState};

#[derive(Serialize)]
struct ConfigJson {
    dl: String,
    api: String,
}

async fn config_json(State(config): State<Configuration>) -> Json<ConfigJson> {
    let base_url = config.base_url();
    let dl_url = {
        let mut dl = base_url.clone();
        if dl.path().ends_with('/') {
            dl.set_path(&format!("{}download", dl.path()))
        } else {
            dl.set_path(&format!("{}/download", dl.path()))
        }
        dl
    };
    let dl_url = format!("{dl_url}/{{prefix}}/{{crate}}-{{version}}.crate");
    let base_url = base_url.to_string();
    let base_url = base_url
        .strip_suffix('/')
        .map(String::from)
        .unwrap_or(base_url);
    Json(ConfigJson {
        api: base_url,
        dl: dl_url,
    })
}

#[derive(Debug, Error)]
enum KrateIndexError {
    #[error("Bad crate name: {0}")]
    BadCrateName(String),
    #[error("Unknown crate name: {0}")]
    UnknownCrate(String),
    #[error("Database error: {0}")]
    Database(#[from] database::DieselError),
}

impl IntoResponse for KrateIndexError {
    fn into_response(self) -> axum::response::Response {
        let code = match &self {
            Self::BadCrateName(_) => StatusCode::BAD_REQUEST,
            Self::UnknownCrate(_) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, self.to_string()).into_response()
    }
}

async fn krate_index(
    mut db: Connection,
    _auth: Option<Authentication>,
    Path(krate): Path<String>,
) -> Result<String, KrateIndexError> {
    let slashpos = krate
        .rfind('/')
        .ok_or_else(|| KrateIndexError::BadCrateName(krate.clone()))?;
    let krate_name = &krate[slashpos + 1..];
    if krate_name.is_empty() {
        return Err(KrateIndexError::BadCrateName(krate));
    }
    let dbkrate = Krate::by_name(&mut db, krate_name)
        .await?
        .ok_or_else(|| KrateIndexError::UnknownCrate(krate.clone()))?;
    let versions = dbkrate.versions(&mut db).await?;

    let versions: Vec<String> = versions.iter().map(KrateVer::index_line).collect();

    Ok(versions.join("\n"))
}

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/config.json", get(config_json))
        .route("/*krate", get(krate_index))
}
