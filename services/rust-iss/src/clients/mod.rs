use crate::{config::Config, domain::ApiError};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
pub mod astronomy;
pub mod iss;
pub mod jwst;
pub mod osdr;
#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    slots: Arc<Mutex<HashMap<String, Instant>>>,
    gap: Duration,
    retries: u32,
}
impl HttpClient {
    pub fn new(c: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(c.timeout)
                .connect_timeout(Duration::from_secs(3))
                .user_agent("Cassiopeia-Education/2.0")
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            slots: Default::default(),
            gap: c.request_gap,
            retries: c.retries,
        })
    }
    pub async fn get(
        &self,
        url: &str,
        query: &[(String, String)],
        headers: &[(String, String)],
        auth: Option<(&str, &str)>,
    ) -> Result<Value, ApiError> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|_| ApiError::new("CONFIG_ERROR", "Invalid upstream URL"))?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        for attempt in 0..=self.retries {
            let wait = {
                let mut slots = self.slots.lock().await;
                let now = Instant::now();
                let slot = slots.entry(host.clone()).or_insert(now);
                let scheduled = (*slot).max(now);
                *slot = scheduled + self.gap;
                scheduled.saturating_duration_since(now)
            };
            tokio::time::sleep(wait).await;
            let mut req = self.client.get(url).query(query);
            for (k, v) in headers {
                req = req.header(k.as_str(), v.as_str());
            }
            if let Some((user, pass)) = auth {
                req = req.basic_auth(user, Some(pass));
            }
            match req.send().await {
                Ok(mut response) => {
                    let status = response.status();
                    if status.is_success() {
                        let mut data = Vec::new();
                        while let Some(chunk) = response.chunk().await.map_err(|_| {
                            ApiError::new("UPSTREAM_IO", "Upstream body unavailable")
                        })? {
                            if data.len() + chunk.len() > 8 * 1024 * 1024 {
                                return Err(ApiError::new(
                                    "UPSTREAM_SIZE",
                                    "Upstream response too large",
                                ));
                            }
                            data.extend_from_slice(&chunk);
                        }
                        return serde_json::from_slice(&data)
                            .map_err(|_| ApiError::new("UPSTREAM_JSON", "Invalid upstream JSON"));
                    }
                    if attempt < self.retries
                        && (status.as_u16() == 429 || status.is_server_error())
                    {
                        let delay = response
                            .headers()
                            .get("retry-after")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(1 << attempt)
                            .min(5);
                        tokio::time::sleep(Duration::from_secs(delay)).await;
                        continue;
                    }
                    return Err(ApiError::new(
                        format!("UPSTREAM_{}", status.as_u16()),
                        "External provider rejected request",
                    ));
                }
                Err(error) => {
                    if attempt < self.retries {
                        tokio::time::sleep(Duration::from_millis(200 * (1 << attempt))).await;
                        continue;
                    }
                    return Err(ApiError::new(
                        if error.is_timeout() {
                            "UPSTREAM_TIMEOUT"
                        } else {
                            "UPSTREAM_UNAVAILABLE"
                        },
                        "External provider unavailable",
                    ));
                }
            }
        }
        Err(ApiError::new(
            "UPSTREAM_UNAVAILABLE",
            "External provider unavailable",
        ))
    }
}
