CREATE TABLE IF NOT EXISTS "fingers" (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id   INTEGER,
    finger    BLOB,
    size_data INTEGER
);
