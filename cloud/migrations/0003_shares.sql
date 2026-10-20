-- Published notes (docs/ACCOUNTS.md › Published notes). Additive only. A share is one
-- note, sealed on the Mac with AES-256-GCM under a key that lives only in the link's
-- #fragment; the Worker stores the ciphertext in R2 at `shares/<id>` and this row,
-- which says whose it is, how big, and when it stops being served.
--
-- A share counts as one file towards its account's `files` and `file_bytes` (its size +
-- 1024), so the quota triggers of 0002 refuse a share the account has no room for.

CREATE TABLE shares (
    -- 16 random bytes from the Mac, as unpadded base64url.
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    -- 0x01 || nonce(12) || ciphertext || tag(16): 29 bytes up to 10 MiB of plaintext.
    size INTEGER NOT NULL CHECK (size BETWEEN 29 AND 10485789),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    -- Unix ms after which the share is no longer served; NULL for never.
    expires_at INTEGER CHECK (expires_at IS NULL OR expires_at > 0)
);
CREATE INDEX shares_by_account ON shares (account_id);
CREATE INDEX shares_by_expiry ON shares (expires_at) WHERE expires_at IS NOT NULL;

-- Server-wide upload bytes per UTC day, counted for shares as for files (0002's
-- uploads_global and uploads_count_*), with the same fair share past the cap.
CREATE TRIGGER shares_uploads_global BEFORE INSERT ON shares
WHEN coalesce((SELECT CASE WHEN day >= NEW.updated_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'upload_bytes'), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day FROM defaults), 0) - NEW.size
 AND coalesce((SELECT CASE WHEN window_start = NEW.updated_at / 86400000 * 86400000 THEN count END
               FROM limits WHERE key = 'uploads-account:' || NEW.account_id), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day / max_accounts FROM defaults WHERE max_accounts > 0), -1)
BEGIN SELECT RAISE(ABORT, 'rate_limited:uploads-global'); END;

CREATE TRIGGER shares_uploads_count_insert AFTER INSERT ON shares
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.updated_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.updated_at / 86400000)
    WHERE name = 'upload_bytes';
END;

CREATE TRIGGER shares_uploads_count_update AFTER UPDATE OF updated_at ON shares
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.updated_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.updated_at / 86400000)
    WHERE name = 'upload_bytes';
END;

-- R2 objects `shares/<id>` still to delete: unpublished, expired, or of a deleted
-- account. An ID here can't be published again; the cron deletes the object and, once
-- the row is more than an hour old (so an upload still in flight can't leave an orphan),
-- the row.
CREATE TABLE share_purges (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    requested_at INTEGER NOT NULL
);
