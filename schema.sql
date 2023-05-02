-- DROP INDEX IF EXISTS index_novel_data_on_ncode;
-- DROP TABLE IF EXISTS novel_data;

CREATE TABLE IF NOT EXISTS novel_data (
  id BIGSERIAL PRIMARY KEY,
  ncode VARCHAR NOT NULL,
  data JSON NOT NULL,
  error VARCHAR,
  last_fetched_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS index_novel_data_on_ncode ON novel_data (ncode);
