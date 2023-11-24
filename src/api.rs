use axum::response::Response;
use axum::routing::any;
use axum::Json;
use axum::{body::Bytes, http::StatusCode, response::IntoResponse, routing::put, Router};
use bytes::Buf;
use database::models::Krate;
use database::Connection;
use metadata::{index, publish};
use serde::Serialize;
use thiserror::Error;
use tracing::info;

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

async fn publish_crate(
    mut db: Connection,
    auth: Authentication,
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
    // TODO: Store the .crate into some backing store

    let krate = Krate::by_name_or_new(&mut db, &entry.name, auth.identity()).await?;

    let vers = krate.new_version(&mut db, &entry).await?;

    // At some point return any warnings
    Ok(Json(PublishResponse::default()))
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new().route("/v1/crates/new", put(publish_crate))
}
