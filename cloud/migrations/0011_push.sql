-- Push notifications (docs/ACCOUNTS.md › Push notifications): a device's push token lives on its
-- session row. It is a silent wake-up address, not a secret of the account: a push carries nothing
-- of the library, and the device pulls over HTTPS as it does for any change.
--
-- push_platform, push_token and push_environment are set and cleared together; the token's shape
-- is checked here as well as in the Worker (APNs: lowercase hex, an even 32–200 characters; FCM:
-- 100–4096 of A-Z a-z 0-9 _ : -), and FCM has no sandbox. push_updated_at is when the device last
-- registered. push_last_sent_at and push_due_at coalesce wake-ups to one per device per 30 s: a
-- change marks the device due, and the account's AccountRoom alarm sends and records the time.
-- push_due_at only exists with a token. Columns are added one at a time, each CHECK naming only
-- columns that already exist.
ALTER TABLE sessions ADD COLUMN push_platform TEXT
    CHECK (push_platform IS NULL OR push_platform IN ('apns', 'fcm'));
ALTER TABLE sessions ADD COLUMN push_token TEXT
    CHECK ((push_token IS NULL) = (push_platform IS NULL) AND (push_token IS NULL
        OR (push_platform = 'apns' AND length(push_token) BETWEEN 32 AND 200 AND length(push_token) % 2 = 0
            AND push_token NOT GLOB '*[^0-9a-f]*')
        OR (push_platform = 'fcm' AND length(push_token) BETWEEN 100 AND 4096
            AND push_token NOT GLOB '*[^A-Za-z0-9_:-]*')));
ALTER TABLE sessions ADD COLUMN push_environment TEXT
    CHECK ((push_environment IS NULL) = (push_token IS NULL)
        AND (push_environment IS NULL OR push_environment IN ('production', 'development'))
        AND (push_platform IS NOT 'fcm' OR push_environment IS NULL OR push_environment = 'production'));
ALTER TABLE sessions ADD COLUMN push_updated_at INTEGER
    CHECK ((push_updated_at IS NULL) = (push_token IS NULL));
-- Kept when the token is cleared: a device that signs in again is still owed its 30 s.
ALTER TABLE sessions ADD COLUMN push_last_sent_at INTEGER;
ALTER TABLE sessions ADD COLUMN push_due_at INTEGER
    CHECK (push_due_at IS NULL OR push_token IS NOT NULL);

-- A token lives on one session at most; registering moves it (the Worker clears the old row in the
-- same batch).
CREATE UNIQUE INDEX sessions_push_token ON sessions (push_token) WHERE push_token IS NOT NULL;
-- The rows an account's alarm has to send.
CREATE INDEX sessions_push_due ON sessions (account_id, push_due_at) WHERE push_due_at IS NOT NULL;

-- A session that ends, or is replaced by a new sign-in of its device (a new token hash), no longer
-- receives pushes: sign-out, revoke, expiry, eviction, a password change, a recovery and a rekey
-- all set revoked_at; the cron and the Worker's expiry do too. push_last_sent_at stays.
CREATE TRIGGER sessions_push_clear AFTER UPDATE OF revoked_at, token_hash ON sessions
WHEN (OLD.revoked_at IS NULL AND NEW.revoked_at IS NOT NULL) OR NEW.token_hash != OLD.token_hash
BEGIN
    UPDATE sessions SET push_platform = NULL, push_token = NULL, push_environment = NULL,
        push_updated_at = NULL, push_due_at = NULL
    WHERE account_id = NEW.account_id AND device_id = NEW.device_id;
END;
