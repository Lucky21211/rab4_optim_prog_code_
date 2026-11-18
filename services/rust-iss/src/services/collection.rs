use crate::{
    clients,
    domain::ApiError,
    repo::{self, CacheRepo, IssRepo, OsdrRepo},
    services::AppState,
};
use serde_json::{json, Value};
pub const SOURCES: &[&str] = &["apod", "neo", "flr", "cme", "spacex"];
pub async fn refresh(st: &AppState, source: &str) -> Result<usize, ApiError> {
    let _permit = st
        .jobs
        .acquire()
        .await
        .map_err(|_| ApiError::new("JOB_BUSY", "Collector stopping"))?;
    let Some(lock) = repo::lock(&st.pool, source).await? else {
        return Err(ApiError::new("JOB_BUSY", "Collection already running"));
    };
    let result = async {
        match source {
            "iss" => {
                let data = clients::iss::fetch(st).await?;
                IssRepo(&st.pool)
                    .insert(&st.config.urls["iss"], data)
                    .await?;
                Ok(1)
            }
            "osdr" => {
                let data = st.http.get(&st.config.urls["osdr"], &[], &[], None).await?;
                OsdrRepo(&st.pool)
                    .upsert(clients::osdr::normalize(data))
                    .await
            }
            src if SOURCES.contains(&src) => {
                let mut query = vec![];
                let today = chrono::Utc::now().date_naive();
                if src != "spacex" && !st.config.nasa_key.is_empty() {
                    query.push(("api_key".into(), st.config.nasa_key.clone()));
                }
                match src {
                    "apod" => query.push(("thumbs".into(), "true".into())),
                    "neo" => {
                        query.push((
                            "start_date".into(),
                            (today - chrono::Days::new(2)).to_string(),
                        ));
                        query.push(("end_date".into(), today.to_string()));
                    }
                    "flr" | "cme" => {
                        query.push((
                            "startDate".into(),
                            (today - chrono::Days::new(5)).to_string(),
                        ));
                        query.push(("endDate".into(), today.to_string()));
                    }
                    _ => {}
                }
                let data = st.http.get(&st.config.urls[src], &query, &[], None).await?;
                if data.get("error").is_some() {
                    return Err(ApiError::new(
                        "UPSTREAM_SCHEMA",
                        "Provider returned an error document",
                    ));
                }
                CacheRepo(&st.pool).put(src, data).await?;
                Ok(1)
            }
            _ => Err(ApiError::new("VALIDATION_ERROR", "Unknown source")),
        }
    }
    .await;
    lock.commit().await?;
    match &result {
        Ok(n) => tracing::info!(event = "collection_complete", source, written = n),
        Err(e) => {
            tracing::warn!(event="collection_failed",source,code=%e.code,trace_id=%e.trace_id)
        }
    }
    result
}
pub async fn refresh_many(st: &AppState, list: &str) -> Result<Value, ApiError> {
    let sources: Vec<_> = list.split(',').map(str::trim).collect();
    if sources.is_empty() || sources.len() > 5 || sources.iter().any(|s| !SOURCES.contains(s)) {
        return Err(ApiError::new("VALIDATION_ERROR", "Invalid source list"));
    }
    let mut done = vec![];
    for src in sources {
        refresh(st, src).await?;
        done.push(src);
    }
    Ok(json!({"refreshed":done}))
}
