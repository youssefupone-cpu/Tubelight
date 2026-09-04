-- 0001_init.sql — per-user SQLite schema (spec §9)
CREATE TABLE users (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT NOT NULL,
  avatar_path  TEXT,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S.%f', 'now')),
  last_used_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S.%f', 'now'))
);
CREATE TABLE subscriptions (
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  channel_id  TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  thumb_url   TEXT,
  added_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (user_id, channel_id)
);
CREATE TABLE playlists (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title       TEXT    NOT NULL,
  description TEXT,
  is_watch_later INTEGER NOT NULL DEFAULT 0,
  is_liked     INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE playlist_items (
  playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
  position    INTEGER NOT NULL,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  channel_id  TEXT,
  channel_title TEXT,
  duration_s  INTEGER,
  thumb_url   TEXT,
  added_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (playlist_id, position)
);
CREATE TABLE history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  channel_id  TEXT,
  channel_title TEXT,
  duration_s  INTEGER,
  thumb_url   TEXT,
  watched_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  position_s  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_history_user_time ON history(user_id, watched_at DESC);
CREATE TABLE likes (
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  video_id    TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  channel_id  TEXT,
  channel_title TEXT,
  duration_s  INTEGER,
  thumb_url   TEXT,
  liked_at    TEXT    NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (user_id, video_id)
);
CREATE VIRTUAL TABLE search_index USING fts5(
  video_id, title, channel_title, kind UNINDEXED,
  tokenize = 'unicode61 remove_diacritics 1'
);
