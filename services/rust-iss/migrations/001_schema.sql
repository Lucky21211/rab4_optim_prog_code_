CREATE TABLE IF NOT EXISTS iss_fetch_log (
 id BIGSERIAL PRIMARY KEY, fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(), source_url TEXT NOT NULL, payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS osdr_items (
 id BIGSERIAL PRIMARY KEY, dataset_id TEXT, title TEXT, status TEXT, updated_at TIMESTAMPTZ,
 inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(), raw JSONB NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS ix_osdr_inserted ON osdr_items(inserted_at DESC, id DESC);
CREATE TABLE IF NOT EXISTS space_cache (
 id BIGSERIAL PRIMARY KEY, source TEXT NOT NULL, fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(), payload JSONB NOT NULL
);
DELETE FROM space_cache a USING space_cache b WHERE a.source=b.source AND a.id<b.id;
CREATE UNIQUE INDEX IF NOT EXISTS ux_space_source ON space_cache(source);
