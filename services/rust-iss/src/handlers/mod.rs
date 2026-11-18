use crate::{
    clients,
    domain::{ApiError, AstroQuery, GalleryQuery, ListQuery},
    repo::{CacheRepo, IssRepo, OsdrRepo},
    services::{collection, telemetry, AppState},
};
use axum::{
    extract::{rejection::QueryRejection, Path, Query, State},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
fn query<T: DeserializeOwned>(q: Result<Query<T>, QueryRejection>) -> Result<T, ApiError> {
    q.map(|Query(v)| v)
        .map_err(|_| ApiError::new("VALIDATION_ERROR", "Invalid query parameters"))
}
pub async fn health() -> Json<Value> {
    Json(json!({"status":"ok","now":chrono::Utc::now()}))
}
pub async fn last(State(st): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(match IssRepo(&st.pool).latest().await? {
        Some(row) => serde_json::to_value(row).unwrap_or_default(),
        None => json!({"message":"no data"}),
    }))
}
pub async fn fetch(State(st): State<AppState>) -> Result<Json<Value>, ApiError> {
    collection::refresh(&st, "iss").await?;
    last(State(st)).await
}
pub async fn trend(
    State(st): State<AppState>,
    q: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        telemetry::trend(&st, query(q)?.limit.unwrap_or(240)).await?,
    ))
}
pub async fn osdr_sync(State(st): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        json!({"written":collection::refresh(&st,"osdr").await?}),
    ))
}
pub async fn osdr_list(
    State(st): State<AppState>,
    q: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Json<Value>, ApiError> {
    let limit = query(q)?.limit.unwrap_or(20);
    if !(1..=100).contains(&limit) {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "limit must be 1 through 100",
        ));
    }
    Ok(Json(
        json!({"items":OsdrRepo(&st.pool).list(limit as i64).await?}),
    ))
}
pub async fn latest(
    State(st): State<AppState>,
    Path(src): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if !collection::SOURCES.contains(&src.as_str()) {
        return Err(ApiError::new("VALIDATION_ERROR", "Unknown source"));
    }
    Ok(Json(CacheRepo(&st.pool).latest(&src).await?))
}
pub async fn refresh(
    State(st): State<AppState>,
    q: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        collection::refresh_many(
            &st,
            query(q)?
                .src
                .as_deref()
                .unwrap_or("apod,neo,flr,cme,spacex"),
        )
        .await?,
    ))
}
pub async fn summary(State(st): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(CacheRepo(&st.pool).summary().await?))
}
pub async fn jwst(
    State(st): State<AppState>,
    q: Result<Query<GalleryQuery>, QueryRejection>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(clients::jwst::feed(&st, query(q)?).await?))
}
pub async fn astro(
    State(st): State<AppState>,
    q: Result<Query<AstroQuery>, QueryRejection>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(clients::astronomy::events(&st, query(q)?).await?))
}
pub async fn fallback() -> ApiError {
    ApiError::new("NOT_FOUND", "Unknown API route")
}
