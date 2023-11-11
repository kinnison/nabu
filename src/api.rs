use axum::Router;

use crate::state::AppState;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
}
