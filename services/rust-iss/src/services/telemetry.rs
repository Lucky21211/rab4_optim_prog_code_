use crate::{
    domain::{haversine_km, number, ApiError},
    repo::IssRepo,
    services::AppState,
};
use serde_json::{json, Value};
pub async fn trend(st: &AppState, limit: usize) -> Result<Value, ApiError> {
    if !(2..=500).contains(&limit) {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "limit must be 2 through 500",
        ));
    }
    let rows = IssRepo(&st.pool).history(limit as i64).await?;
    let mut out = json!({"movement":false,"delta_km":0.0,"dt_sec":0.0,"velocity_kmh":null,"from_time":null,"to_time":null,"from_lat":null,"from_lon":null,"to_lat":null,"to_lon":null});
    if rows.len() >= 2 {
        let (new, old) = (&rows[0], &rows[1]);
        let a = number(&old.payload["latitude"]);
        let o = number(&old.payload["longitude"]);
        let b = number(&new.payload["latitude"]);
        let p = number(&new.payload["longitude"]);
        let distance = match (a, o, b, p) {
            (Some(a), Some(o), Some(b), Some(p)) => haversine_km(a, o, b, p),
            _ => 0.0,
        };
        out = json!({"movement":distance>0.1,"delta_km":distance,"dt_sec":(new.fetched_at-old.fetched_at).num_milliseconds() as f64/1000.0,"velocity_kmh":number(&new.payload["velocity"]),"from_time":old.fetched_at,"to_time":new.fetched_at,"from_lat":a,"from_lon":o,"to_lat":b,"to_lon":p});
    }
    out["points"]=Value::Array(rows.iter().rev().filter_map(|r|Some(json!({"at":r.fetched_at,"lat":number(&r.payload["latitude"])? ,"lon":number(&r.payload["longitude"])? ,"velocity":number(&r.payload["velocity"]),"altitude":number(&r.payload["altitude"])}))).collect());
    Ok(out)
}
