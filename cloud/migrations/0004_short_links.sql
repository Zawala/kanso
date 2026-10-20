-- Short links for published notes (docs/ACCOUNTS.md › Published notes › Short links).
-- Additive only. When the Worker has a Rerout API key, publishing also creates a Rerout
-- short link whose target is `/p/<id>` without the #fragment, so Rerout never sees a
-- share's key; its link preview shows the note's title over Kanso's own image.

-- The Rerout link of a share: its code (to update and delete it), its URL as Rerout
-- answered it (the device appends #<key>), and the title Rerout shows in previews.
-- NULL while there is none: Rerout isn't configured, or it failed and the next upload
-- tries again.
ALTER TABLE shares ADD COLUMN short_code TEXT
    CHECK (short_code IS NULL OR (length(short_code) BETWEEN 1 AND 64 AND short_code NOT GLOB '*[^A-Za-z0-9_-]*'));
ALTER TABLE shares ADD COLUMN short_url TEXT
    CHECK (short_url IS NULL OR (length(short_url) <= 256 AND short_url GLOB 'https://*' AND short_url NOT GLOB '*[#? ]*'));
ALTER TABLE shares ADD COLUMN short_title TEXT
    CHECK (short_title IS NULL OR length(short_title) BETWEEN 1 AND 90);

-- Rerout links still to delete: of a share that stopped being served, or one a racing
-- upload made and lost. The cron deletes them and drops the row once Rerout has; a link
-- that keeps failing moves behind the others (attempts).
CREATE TABLE short_link_purges (
    code TEXT PRIMARY KEY NOT NULL CHECK (length(code) BETWEEN 1 AND 64 AND code NOT GLOB '*[^A-Za-z0-9_-]*'),
    requested_at INTEGER NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0
);

-- Whatever deletes a share's row (unpublish, expiry, takedown, account deletion, the
-- reconcile) queues its short link, as shares_retire retires its ID.
CREATE TRIGGER shares_short_link_purge AFTER DELETE ON shares
WHEN OLD.short_code IS NOT NULL
BEGIN
    INSERT OR IGNORE INTO short_link_purges (code, requested_at)
    VALUES (OLD.short_code, CAST(strftime('%s', 'now') AS INTEGER) * 1000);
END;
