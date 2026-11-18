use crate::{
    domain::{http_url, ApiError, GalleryQuery},
    services::AppState,
};
use serde_json::{json, Value};
fn image_url(value: &Value) -> Option<&str> {
    let mut stack = vec![value];
    while let Some(node) = stack.pop() {
        match node {
            Value::String(url) if http_url(url) => {
                let clean = url.split('?').next().unwrap_or("").to_lowercase();
                if [".jpg", ".jpeg", ".png"]
                    .iter()
                    .any(|ext| clean.ends_with(ext))
                {
                    return Some(url);
                }
            }
            Value::Array(items) => stack.extend(items.iter().rev()),
            Value::Object(items) => stack.extend(items.values()),
            _ => {}
        }
    }
    None
}
pub async fn feed(st: &AppState, q: GalleryQuery) -> Result<Value, ApiError> {
    let page = q.page.unwrap_or(1);
    let per = q.per_page.unwrap_or(24);
    if page == 0 || page > 10000 || !(1..=60).contains(&per) {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "Invalid gallery pagination",
        ));
    }
    let path = match q.source.as_str() {
        "jpg" => "all/type/jpg".into(),
        "suffix" => {
            let s = q.suffix.unwrap_or_default();
            if s.is_empty()
                || s.len() > 32
                || !s
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            {
                return Err(ApiError::new("VALIDATION_ERROR", "Invalid suffix"));
            }
            format!("all/suffix/{s}")
        }
        "program" => {
            let s = q.program.unwrap_or_default();
            if s.is_empty() || s.len() > 16 || !s.bytes().all(|c| c.is_ascii_digit()) {
                return Err(ApiError::new("VALIDATION_ERROR", "Invalid program"));
            }
            format!("program/id/{s}")
        }
        _ => return Err(ApiError::new("VALIDATION_ERROR", "Invalid gallery source")),
    };
    let instrument = q.instrument.unwrap_or_default().to_uppercase();
    if !["", "NIRCAM", "MIRI", "NIRISS", "NIRSPEC", "FGS"].contains(&instrument.as_str()) {
        return Err(ApiError::new("VALIDATION_ERROR", "Invalid instrument"));
    }
    let mut headers = vec![];
    if !st.config.jwst_key.is_empty() {
        headers.push(("x-api-key".into(), st.config.jwst_key.clone()));
    }
    if !st.config.jwst_email.is_empty() {
        headers.push(("email".into(), st.config.jwst_email.clone()));
    }
    let data = st
        .http
        .get(
            &format!("{}/{}", st.config.urls["jwst"].trim_end_matches('/'), path),
            &[
                ("page".into(), page.to_string()),
                ("perPage".into(), per.to_string()),
            ],
            &headers,
            None,
        )
        .await?;
    let list = data
        .as_array()
        .or_else(|| data.get("body").and_then(Value::as_array))
        .or_else(|| data.get("data").and_then(Value::as_array))
        .ok_or_else(|| ApiError::new("UPSTREAM_SCHEMA", "Gallery list missing"))?;
    let mut items = vec![];
    for it in list {
        let url = ["location", "thumbnail", "url", "image", "thumbnailUrl"]
            .iter()
            .filter_map(|k| it[*k].as_str())
            .find(|u| {
                http_url(u) && {
                    let clean = u.split('?').next().unwrap_or("").to_lowercase();
                    [".jpg", ".jpeg", ".png"].iter().any(|x| clean.ends_with(x))
                }
            });
        let Some(url) = url.or_else(|| image_url(it)) else {
            continue;
        };
        let inst: Vec<_> = it
            .pointer("/details/instruments")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|v| v["instrument"].as_str())
            .map(str::to_uppercase)
            .collect();
        if !instrument.is_empty() && !inst.is_empty() && !inst.contains(&instrument) {
            continue;
        }
        let obs = it["observation_id"]
            .as_str()
            .or_else(|| it["observationId"].as_str())
            .unwrap_or("");
        let program = it["program"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| {
                it["program"]
                    .as_u64()
                    .map(|n| n.to_string())
                    .unwrap_or_default()
            });
        let suffix = it
            .pointer("/details/suffix")
            .and_then(Value::as_str)
            .or_else(|| it["suffix"].as_str())
            .unwrap_or("");
        items.push(json!({"url":url,"link":url,"obs":obs,"program":program,"suffix":suffix,"inst":inst,"caption":format!("{obs} · P{program} · {suffix}")}));
        if items.len() >= per {
            break;
        }
    }
    Ok(json!({"source":path,"count":items.len(),"items":items}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_images_remain_supported() {
        let value =
            json!({"details":{"files":[{"url":"https://example.org/space.png?size=small"}]}});
        assert_eq!(
            image_url(&value),
            Some("https://example.org/space.png?size=small")
        );
        assert_eq!(image_url(&json!({"url":"javascript:alert(1)"})), None);
    }
}
