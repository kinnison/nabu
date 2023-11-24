//! The crate index and such

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::{configuration::Configuration, state::AppState};

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
            dl.set_path(&format!("{}crates/dl", dl.path()))
        } else {
            dl.set_path(&format!("{}/crates/dl", dl.path()))
        }
        dl
    };
    let base_url = base_url.to_string();
    let base_url = base_url
        .strip_suffix('/')
        .map(String::from)
        .unwrap_or(base_url);
    Json(ConfigJson {
        api: base_url,
        dl: dl_url.to_string(),
    })
}

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().route("/config.json", get(config_json))
}
