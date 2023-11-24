use std::path::{Path, PathBuf};

use axum::extract::State;
use axum::response::Response;
use axum::Json;
use axum::{body::Bytes, http::StatusCode, response::IntoResponse, routing::put, Router};
use bytes::Buf;
use database::models::Krate;
use database::Connection;
use metadata::{index, publish};
use serde::Serialize;
use thiserror::Error;
use tracing::info;

use crate::configuration::Configuration;
use crate::{auth::Authentication, state::AppState};

#[derive(Debug, Error)]
enum PublishError {
    #[error("Database error: {0}")]
    Database(#[from] database::DieselError),
    #[error("Invalid body length, Needed {need} but had {had}")]
    InvalidBodyLength { need: usize, had: usize },
    #[error("Bad metadata length: {0}")]
    BadMetadataLength(usize),
    #[error("Failure during deserialisation: {0}")]
    Deserialise(#[from] serde_path_to_error::Error<serde_json::Error>),
    #[error("Some dependencies unmet: {0:?}")]
    UnmetDeps(Vec<String>),
    #[error("IO error storing crate: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Serialize)]
struct GenericError {
    error: String,
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        let code = match &self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidBodyLength { .. } => StatusCode::BAD_REQUEST,
            Self::BadMetadataLength(_) => StatusCode::BAD_REQUEST,
            Self::Deserialise(_) => StatusCode::BAD_REQUEST,
            Self::UnmetDeps(_) => StatusCode::BAD_REQUEST,
            Self::IO(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let msg = self.to_string();
        (code, Json(GenericError { error: msg })).into_response()
    }
}

#[derive(Default, Serialize)]
struct PublishResponse {
    warnings: PublishWarnings,
}

#[derive(Default, Serialize)]
struct PublishWarnings {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

fn make_crate_filename(base: &Path, krate: &str, version: &str) -> std::io::Result<PathBuf> {
    let container = match krate.len() {
        0 => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "empty crate name",
            ))
        }
        1 => "1".to_string(),
        2 => "2".to_string(),
        3 => format!("3/{}", krate.chars().next().unwrap()),
        _ => {
            let chars: Vec<char> = krate.chars().take(4).collect();
            format!("{}{}/{}{}", chars[0], chars[1], chars[2], chars[3])
        }
    };
    let mut dir_to_make = base.join(container);
    std::fs::create_dir_all(&dir_to_make)?;
    dir_to_make.push(format!("{krate}-{version}.crate"));
    Ok(dir_to_make)
}

async fn publish_crate(
    mut db: Connection,
    auth: Authentication,
    State(config): State<Configuration>,
    mut body: Bytes,
) -> Result<Json<PublishResponse>, PublishError> {
    info!("Begin publish flow...");
    // Step one, acquire the metadata
    if body.len() < 4 {
        return Err(PublishError::InvalidBodyLength {
            need: 4,
            had: body.len(),
        });
    }

    let metalen = body.get_u32_le();
    if metalen == 0 || metalen > 1024 * 1024 || (metalen as usize) > body.len() {
        return Err(PublishError::BadMetadataLength(metalen as usize));
    }

    let metaraw = body.split_to(metalen as usize);

    let cratelen = body.get_u32_le();
    if (cratelen as usize) != body.len() {
        return Err(PublishError::InvalidBodyLength {
            need: cratelen as usize,
            had: body.len(),
        });
    }

    // At this point, metaraw is the bytes for the metadata
    // and body is the bytes for the crate

    let mut deser = serde_json::Deserializer::from_reader(metaraw.reader());

    let meta: publish::Metadata = serde_path_to_error::deserialize(&mut deser)?;

    let cksum = sha256::digest(body.as_ref());
    let entry = index::Entry::from_publish(meta, cksum);

    let mut bad_deps = Vec::new();
    // Now that we're ready, let's check deps
    for dep in &entry.deps {
        if matches!(dep.kind, index::Kind::Dev) {
            // Skip dev-depends
            continue;
        }
        let krate_name = dep.package.as_deref().unwrap_or(&dep.name);
        let krate = match Krate::by_name(&mut db, krate_name).await? {
            Some(krate) => krate,
            None => {
                bad_deps.push(krate_name);
                continue;
            }
        };
        if !krate.satisfies(&mut db, &dep.req).await? {
            bad_deps.push(krate_name);
        }
    }

    if !bad_deps.is_empty() {
        return Err(PublishError::UnmetDeps(
            bad_deps.into_iter().map(String::from).collect(),
        ));
    }

    // At this point we can be happy that the upload is good

    let crate_filename = make_crate_filename(config.crate_path(), &entry.name, &entry.vers)?;

    tokio::fs::write(crate_filename, body).await?;

    let krate = Krate::by_name_or_new(&mut db, &entry.name, auth.identity()).await?;

    let _vers = krate.new_version(&mut db, &entry).await?;

    // At some point return any warnings
    Ok(Json(PublishResponse::default()))
}

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().route("/v1/crates/new", put(publish_crate))
}
