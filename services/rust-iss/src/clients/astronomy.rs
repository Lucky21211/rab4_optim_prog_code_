use crate::{
    domain::{ApiError, AstroQuery},
    services::AppState,
};
use serde_json::Value;
pub async fn events(st: &AppState, q: AstroQuery) -> Result<Value, ApiError> {
    let lat = q.lat.unwrap_or(55.7558);
    let lon = q.lon.unwrap_or(37.6176);
    let days = q.days.unwrap_or(7);
    if !lat.is_finite()
        || !lon.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lon)
        || !(1..=30).contains(&days)
    {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "Invalid coordinates or days",
        ));
    }
    if st.config.astro_id.is_empty() || st.config.astro_secret.is_empty() {
        return Err(ApiError::new(
            "CONFIG_ERROR",
            "AstronomyAPI credentials are not configured",
        ));
    }
    let from = chrono::Utc::now().date_naive();
    let to = from + chrono::Days::new(days as u64);
    let mut all = Vec::new();
    let mut events = Vec::new();
    for body in ["sun", "moon"] {
        let data = st
            .http
            .get(
                &format!("{}/{}", st.config.urls["astro"].trim_end_matches('/'), body),
                &[
                    ("latitude".into(), lat.to_string()),
                    ("longitude".into(), lon.to_string()),
                    ("from_date".into(), from.to_string()),
                    ("to_date".into(), to.to_string()),
                    ("elevation".into(), "0".into()),
                    ("time".into(), "12:00:00".into()),
                    ("output".into(), "rows".into()),
                ],
                &[],
                Some((&st.config.astro_id, &st.config.astro_secret)),
            )
            .await?;
        if let Some(rows) = data.pointer("/data/rows").and_then(Value::as_array) {
            for row in rows {
                if let Some(list) = row["events"].as_array() {
                    for event in list {
                        events.push(serde_json::json!({"name":row.pointer("/body/name").and_then(Value::as_str).unwrap_or(body),"type":event["type"],"time":event.pointer("/eventHighlights/peak/date"),"altitude":event.pointer("/eventHighlights/peak/altitude")}));
                    }
                }
            }
        }
        all.push(serde_json::json!({"body":body,"raw":data}));
    }
    Ok(serde_json::json!({"data":all,"events":events}))
}
