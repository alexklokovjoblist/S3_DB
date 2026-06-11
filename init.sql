-- Если таблицы уже есть — можно закомментировать DROP
DROP TABLE IF EXISTS file_deliveries CASCADE;
DROP TABLE IF EXISTS files CASCADE;

CREATE TABLE files (
    id          BIGSERIAL PRIMARY KEY,
    file_key    TEXT UNIQUE NOT NULL,
    bucket      TEXT NOT NULL,
    size_bytes  BIGINT NOT NULL DEFAULT 0,
    content_type TEXT,
    is_used     BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ON files (is_used);
CREATE INDEX ON files (bucket);

CREATE TABLE file_deliveries (
    id          BIGSERIAL PRIMARY KEY,
    payment_id  TEXT NOT NULL,
    email       TEXT NOT NULL,
    file_id     BIGINT NOT NULL REFERENCES files(id) ON DELETE RESTRICT,
    status      TEXT NOT NULL DEFAULT 'pending',
    sent_at     TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ON file_deliveries (file_id);
CREATE INDEX ON file_deliveries (status);
CREATE INDEX ON file_deliveries (payment_id);