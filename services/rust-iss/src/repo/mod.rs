use crate::{
    clients::osdr,
    domain::{ApiError, Dataset, IssSample},
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
pub struct IssRepo<'a>(pub &'a PgPool);
impl IssRepo<'_> {
    pub async fn latest(&self) -> Result<Option<IssSample>, ApiError> {
        Ok(sqlx::query_as(
            "SELECT id,fetched_at,source_url,payload FROM iss_fetch_log ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(self.0)
        .await?)
    }
    pub async fn history(&self, limit: i64) -> Result<Vec<IssSample>, ApiError> {
        Ok(sqlx::query_as(
            "SELECT id,fetched_at,source_url,payload FROM iss_fetch_log ORDER BY id DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(self.0)
        .await?)
    }
    pub async fn insert(&self, url: &str, payload: Value) -> Result<(), ApiError> {
        sqlx::query("INSERT INTO iss_fetch_log(source_url,payload) VALUES($1,$2)")
            .bind(url)
            .bind(payload)
            .execute(self.0)
            .await?;
        Ok(())
    }
}
pub struct OsdrRepo<'a>(pub &'a PgPool);
impl OsdrRepo<'_> {
    pub async fn list(&self, limit: i64) -> Result<Vec<Dataset>, ApiError> {
        Ok(sqlx::query_as("SELECT id,dataset_id,title,status,updated_at,inserted_at,raw FROM osdr_items ORDER BY inserted_at DESC,id DESC LIMIT $1").bind(limit).fetch_all(self.0).await?)
    }
    pub async fn upsert(&self, items: Vec<Value>) -> Result<usize, ApiError> {
        let mut tx = self.0.begin().await?;
        let mut written = 0;
        for item in items {
            let Some(id) = osdr::pick(
                &item,
                &[
                    "dataset_id",
                    "id",
                    "uuid",
                    "studyId",
                    "accession",
                    "osdr_id",
                ],
            ) else {
                continue;
            };
            let title = osdr::title(&item);
            let status = osdr::pick(&item, &["status", "state", "lifecycle"]);
            let updated = [
                "updated",
                "updated_at",
                "modified",
                "lastUpdated",
                "timestamp",
            ]
            .iter()
            .find_map(|k| osdr::timestamp(&item[*k]));
            sqlx::query("INSERT INTO osdr_items(dataset_id,title,status,updated_at,raw) VALUES($1,$2,$3,$4,$5) ON CONFLICT(dataset_id) WHERE dataset_id IS NOT NULL DO UPDATE SET title=EXCLUDED.title,status=EXCLUDED.status,updated_at=EXCLUDED.updated_at,raw=EXCLUDED.raw")
    .bind(id).bind(title).bind(status).bind(updated).bind(item).execute(&mut *tx).await?;
            written += 1;
        }
        tx.commit().await?;
        Ok(written)
    }
}
pub struct CacheRepo<'a>(pub &'a PgPool);
impl CacheRepo<'_> {
    pub async fn put(&self, source: &str, payload: Value) -> Result<(), ApiError> {
        sqlx::query("INSERT INTO space_cache(source,payload) VALUES($1,$2) ON CONFLICT(source) DO UPDATE SET fetched_at=now(),payload=EXCLUDED.payload").bind(source).bind(payload).execute(self.0).await?;
        Ok(())
    }
    pub async fn latest(&self, source: &str) -> Result<Value, ApiError> {
        let row = sqlx::query("SELECT fetched_at,payload FROM space_cache WHERE source=$1")
            .bind(source)
            .fetch_optional(self.0)
            .await?;
        Ok(match row {
            Some(r) => {
                json!({"source":source,"fetched_at":r.get::<DateTime<Utc>,_>("fetched_at"),"payload":r.get::<Value,_>("payload")})
            }
            None => json!({"source":source,"message":"no data"}),
        })
    }
    pub async fn summary(&self) -> Result<Value, ApiError> {
        let rows = sqlx::query("SELECT source,fetched_at,payload FROM space_cache")
            .fetch_all(self.0)
            .await?;
        let mut out =
            json!({"apod":{},"neo":{},"flr":{},"cme":{},"spacex":{},"iss":{},"osdr_count":0});
        for r in rows {
            let src = r.get::<String, _>("source");
            out[&src] = json!({"at":r.get::<DateTime<Utc>,_>("fetched_at"),"payload":r.get::<Value,_>("payload")});
        }
        if let Some(r) = IssRepo(self.0).latest().await? {
            out["iss"] = json!({"at":r.fetched_at,"payload":r.payload});
        }
        out["osdr_count"] = json!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM osdr_items")
                .fetch_one(self.0)
                .await?
        );
        Ok(out)
    }
}
pub async fn lock(
    pool: &PgPool,
    src: &str,
) -> Result<Option<sqlx::Transaction<'static, sqlx::Postgres>>, ApiError> {
    let mut tx = pool.begin().await?;
    let taken: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(format!("cassiopeia:{src}"))
        .fetch_one(&mut *tx)
        .await?;
    if taken {
        Ok(Some(tx))
    } else {
        tx.rollback().await?;
        Ok(None)
    }
}
