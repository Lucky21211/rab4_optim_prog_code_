use crate::{
    domain::{number, ApiError},
    services::AppState,
};
pub async fn fetch(st: &AppState) -> Result<serde_json::Value, ApiError> {
    let json = st.http.get(&st.config.urls["iss"], &[], &[], None).await?;
    let lat = number(&json["latitude"]);
    let lon = number(&json["longitude"]);
    if !lat.is_some_and(|v| (-90.0..=90.0).contains(&v))
        || !lon.is_some_and(|v| (-180.0..=180.0).contains(&v))
    {
        return Err(ApiError::new(
            "UPSTREAM_SCHEMA",
            "ISS coordinates missing or invalid",
        ));
    }
    Ok(json)
}
