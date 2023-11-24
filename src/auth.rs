//! Authentication data to be extracted in routes
//!

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use database::{
    models::{Identity, Token},
    Connection,
};
use tracing::info;

use crate::state::AppState;

pub struct Authentication {
    identity: Identity,
    token: Token,
}

impl Authentication {
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    async fn from_token(db: &mut Connection, token: &str) -> Option<Self> {
        let token = Token::from_token(db, token).await.ok()??;
        let identity = token.owner(db).await.ok()?;
        Some(Self { identity, token })
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Authentication {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        info!("Begun looking for authentication data...");
        let mut db = Connection::from_request_parts(parts, state)
            .await
            .map_err(|e| e.into_response())?;

        // Authorisation should be a token presented as a bearer token
        let auth_header = parts.headers.get(AUTHORIZATION).ok_or_else(|| {
            (StatusCode::FORBIDDEN, "You need to provide a token, sorry").into_response()
        })?;
        let token = auth_header.to_str().map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid authorisation header: {e}"),
            )
                .into_response()
        })?;

        Authentication::from_token(&mut db, token)
            .await
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid or unknown token: {token}"),
                )
                    .into_response()
            })
    }
}
