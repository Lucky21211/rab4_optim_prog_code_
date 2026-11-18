use crate::{handlers as h, services::AppState};
use axum::{routing::get, Router};
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(h::health))
        .route("/last", get(h::last))
        .route("/fetch", get(h::fetch))
        .route("/iss/trend", get(h::trend))
        .route("/osdr/sync", get(h::osdr_sync))
        .route("/osdr/list", get(h::osdr_list))
        .route("/space/:src/latest", get(h::latest))
        .route("/space/refresh", get(h::refresh))
        .route("/space/summary", get(h::summary))
        .route("/jwst/feed", get(h::jwst))
        .route("/astro/events", get(h::astro))
        .fallback(h::fallback)
        .method_not_allowed_fallback(|| async {
            crate::domain::ApiError::new("METHOD_NOT_ALLOWED", "Unsupported API method")
        })
        .with_state(state)
}
