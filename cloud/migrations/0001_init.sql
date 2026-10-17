-- The current revision of every document Kanso syncs, tombstones included.
-- `seq` is the change sequence of the document's last accepted write.
CREATE TABLE docs (
    id TEXT PRIMARY KEY NOT NULL,
    rev TEXT NOT NULL,
    seq INTEGER NOT NULL UNIQUE,
    deleted INTEGER NOT NULL,
    fields TEXT NOT NULL,
    updated_at INTEGER
);

-- The change-sequence counter: the seq of the last accepted write. It only
-- grows, across all documents, so a client cursor never skips a change.
CREATE TABLE counter (
    name TEXT PRIMARY KEY NOT NULL,
    value INTEGER NOT NULL
);
INSERT INTO counter (name, value) VALUES ('seq', 0);
