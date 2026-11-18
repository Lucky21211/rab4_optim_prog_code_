use axum::{
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub trace_id: String,
}
impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(_: sqlx::Error) -> Self {
        Self::new("DATABASE_ERROR", "Database temporarily unavailable")
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::warn!(event="api_error",code=%self.code,trace_id=%self.trace_id);
        Json(json!({"ok":false,"error":{"code":self.code,"message":self.message,"trace_id":self.trace_id}})).into_response()
    }
}
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct IssSample {
    pub id: i64,
    pub fetched_at: DateTime<Utc>,
    pub source_url: String,
    pub payload: Value,
}
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Dataset {
    pub id: i64,
    pub dataset_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub inserted_at: DateTime<Utc>,
    pub raw: Value,
}
#[derive(Default, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub src: Option<String>,
}
#[derive(Deserialize)]
pub struct GalleryQuery {
    #[serde(default = "default_source")]
    pub source: String,
    pub suffix: Option<String>,
    pub program: Option<String>,
    pub instrument: Option<String>,
    pub page: Option<usize>,
    #[serde(rename = "perPage")]
    pub per_page: Option<usize>,
}
fn default_source() -> String {
    "jpg".into()
}
#[derive(Deserialize)]
pub struct AstroQuery {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub days: Option<u32>,
}
pub fn number(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str()?.parse().ok())
        .filter(|n| n.is_finite())
}
pub fn haversine_km(a: f64, o: f64, b: f64, p: f64) -> f64 {
    let h = ((b - a).to_radians() / 2.0).sin().powi(2)
        + a.to_radians().cos() * b.to_radians().cos() * ((p - o).to_radians() / 2.0).sin().powi(2);
    let h = h.clamp(0.0, 1.0);
    6371.0 * 2.0 * h.sqrt().atan2((1.0 - h).sqrt())
}
pub fn http_url(s: &str) -> bool {
    reqwest::Url::parse(s)
        .is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distance_is_stable() {
        assert_eq!(haversine_km(0., 0., 0., 0.), 0.);
        assert!((haversine_km(0., 0., 0., 180.) - 20015.087).abs() < 0.01);
    }
    #[test]
    fn safe_links() {
        assert!(http_url("https://example.org/image.jpg"));
        assert!(!http_url("javascript:alert(1)"));
    }
    #[test]
    fn numeric_strings() {
        assert_eq!(number(&json!("12.5")), Some(12.5));
        assert_eq!(number(&json!("NaN")), None);
    }
}
