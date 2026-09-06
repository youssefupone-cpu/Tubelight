-- 0002_downloads.sql — download job persistence (Phase 3 exit gate).
--
-- The live queue stays in-memory (`yoube-yt-dlp::Downloader`) for speed, but
-- every enqueue/cancel/done transition is mirrored here so jobs survive an
-- app restart. Partial `*.part` files in the destination dir are resumed with
-- `yt-dlp --continue` on next launch (the UI re-enqueues `interrupted` rows).
CREATE TABLE IF NOT EXISTS downloads (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id      INTEGER NOT NULL UNIQUE,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL DEFAULT '',
  format_id   TEXT    NOT NULL,
  dest_dir    TEXT    NOT NULL,
  state       TEXT    NOT NULL DEFAULT 'queued'
              CHECK (state IN ('queued','running','paused','done','failed','cancelled','interrupted')),
  progress_bytes INTEGER NOT NULL DEFAULT 0,
  total_bytes INTEGER,
  eta_s       INTEGER,
  error       TEXT,
  created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_downloads_state ON downloads(state, updated_at DESC);
