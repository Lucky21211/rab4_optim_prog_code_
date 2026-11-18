use std::{collections::HashMap, env, time::Duration};
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub urls: HashMap<String, String>,
    pub nasa_key: String,
    pub jwst_key: String,
    pub jwst_email: String,
    pub astro_id: String,
    pub astro_secret: String,
    pub timeout: Duration,
    pub request_gap: Duration,
    pub retries: u32,
    pub intervals: Vec<(&'static str, u64)>,
    pub scheduler: bool,
}
fn number(name: &str, default: u64, low: u64, high: u64) -> anyhow::Result<u64> {
    let v = env::var(name)
        .ok()
        .map(|v| v.parse::<u64>())
        .transpose()?
        .unwrap_or(default);
    anyhow::ensure!((low..=high).contains(&v), "invalid setting {name}");
    Ok(v)
}
fn secret(name: &str) -> anyhow::Result<String> {
    if let Ok(file) = env::var(format!("{name}_FILE")) {
        return Ok(std::fs::read_to_string(file)?.trim().into());
    }
    Ok(env::var(name).unwrap_or_default())
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut urls = HashMap::new();
        for (src, key, default) in [
            (
                "iss",
                "WHERE_ISS_URL",
                "https://api.wheretheiss.at/v1/satellites/25544",
            ),
            (
                "osdr",
                "NASA_API_URL",
                "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json",
            ),
            ("apod", "APOD_URL", "https://api.nasa.gov/planetary/apod"),
            ("neo", "NEO_URL", "https://api.nasa.gov/neo/rest/v1/feed"),
            ("flr", "FLR_URL", "https://api.nasa.gov/DONKI/FLR"),
            ("cme", "CME_URL", "https://api.nasa.gov/DONKI/CME"),
            (
                "spacex",
                "SPACEX_URL",
                "https://api.spacexdata.com/v4/launches/next",
            ),
            ("jwst", "JWST_HOST", "https://api.jwstapi.com"),
            (
                "astro",
                "ASTRO_URL",
                "https://api.astronomyapi.com/api/v2/bodies/events",
            ),
        ] {
            let value = env::var(key)
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(default.into());
            let parsed = reqwest::Url::parse(&value)?;
            anyhow::ensure!(
                matches!(parsed.scheme(), "http" | "https")
                    && parsed.host_str().is_some()
                    && parsed.username().is_empty()
                    && parsed.password().is_none(),
                "invalid URL setting {key}"
            );
            urls.insert(src.into(), value);
        }
        let database_url = secret("DATABASE_URL")?;
        anyhow::ensure!(!database_url.is_empty(), "DATABASE_URL is required");
        Ok(Self {
            database_url,
            urls,
            nasa_key: secret("NASA_API_KEY")?,
            jwst_key: secret("JWST_API_KEY")?,
            jwst_email: secret("JWST_EMAIL")?,
            astro_id: secret("ASTRO_APP_ID")?,
            astro_secret: secret("ASTRO_APP_SECRET")?,
            timeout: Duration::from_secs(number("HTTP_TIMEOUT_SECONDS", 10, 1, 60)?),
            request_gap: Duration::from_millis(number("RATE_LIMIT_MS", 1000, 0, 60000)?),
            retries: number("HTTP_RETRIES", 2, 0, 3)? as u32,
            scheduler: env::var("SCHEDULER_ENABLED").unwrap_or("true".into()) == "true",
            intervals: vec![
                ("iss", number("ISS_EVERY_SECONDS", 120, 1, 604800)?),
                ("osdr", number("FETCH_EVERY_SECONDS", 600, 1, 604800)?),
                ("apod", number("APOD_EVERY_SECONDS", 43200, 1, 604800)?),
                ("neo", number("NEO_EVERY_SECONDS", 7200, 1, 604800)?),
                ("flr", number("DONKI_EVERY_SECONDS", 3600, 1, 604800)?),
                ("cme", number("DONKI_EVERY_SECONDS", 3600, 1, 604800)?),
                ("spacex", number("SPACEX_EVERY_SECONDS", 3600, 1, 604800)?),
            ],
        })
    }
}
