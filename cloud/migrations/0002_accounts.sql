-- Accounts with end-to-end encrypted libraries (docs/ACCOUNTS.md). Additive only: the
-- single-user tables `docs` and `counter` stay as they are until the owner runs
-- `npm run -s admin -- purge-legacy` with a verified manifest. Quotas, caps and the
-- sign-up switch are enforced here, so concurrent requests can't race past them.
-- Every setting fails closed: a missing value refuses (coalesce to 0 or 'closed'), and
-- rows of `settings` and `daily` can't be deleted or renamed.

CREATE TABLE settings (
    name TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    CHECK ((name = 'signup' AND (value IN ('open', 'closed')
                                 OR (value GLOB 'only:*' AND length(value) BETWEEN 8 AND 37)))
           OR (name != 'signup' AND length(value) BETWEEN 1 AND 15 AND value NOT GLOB '*[^0-9]*'))
);
INSERT INTO settings (name, value) VALUES
    ('signup', 'closed'),
    ('max_accounts', '50'),
    ('signups_per_day', '5'),
    ('quota_docs', '50000'),
    ('quota_doc_bytes', '104857600'),
    ('quota_files', '10000'),
    ('quota_file_bytes', '524288000'),
    ('writes_per_day', '20000'),
    ('upload_bytes_per_day', '1073741824'),
    ('global_doc_bytes', '6000000000'),
    ('global_file_bytes', '50000000000'),
    ('global_writes_per_day', '300000'),
    ('global_upload_bytes_per_day', '5000000000');

CREATE TRIGGER settings_keep BEFORE DELETE ON settings
BEGIN SELECT RAISE(ABORT, 'settings_required'); END;

CREATE TRIGGER settings_names_fixed BEFORE UPDATE OF name ON settings
BEGIN SELECT RAISE(ABORT, 'settings_required'); END;

-- The numeric settings as one row, for the triggers. A missing setting reads as NULL,
-- which every trigger turns into a refusal.
CREATE VIEW defaults AS SELECT
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'max_accounts') AS max_accounts,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'signups_per_day') AS signups_per_day,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_docs') AS docs,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_doc_bytes') AS doc_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_files') AS files,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'quota_file_bytes') AS file_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_doc_bytes') AS global_doc_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_file_bytes') AS global_file_bytes,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_writes_per_day') AS global_writes_per_day,
    (SELECT CAST(value AS INTEGER) FROM settings WHERE name = 'global_upload_bytes_per_day')
        AS global_upload_bytes_per_day;

-- Server-wide counters for one UTC day (`day` = floor(ms / 86400000)): accounts created,
-- documents stored and file bytes uploaded. Only triggers write them, and deleting an
-- account doesn't lower them. A write dated before `day` counts towards `day`.
CREATE TABLE daily (
    name TEXT PRIMARY KEY NOT NULL CHECK (name IN ('signups', 'writes', 'upload_bytes')),
    day INTEGER NOT NULL CHECK (day >= 0),
    count INTEGER NOT NULL CHECK (count >= 0)
);
INSERT INTO daily (name, day, count) VALUES ('signups', 0, 0), ('writes', 0, 0), ('upload_bytes', 0, 0);

CREATE TRIGGER daily_keep BEFORE DELETE ON daily
BEGIN SELECT RAISE(ABORT, 'daily_required'); END;

CREATE TRIGGER daily_names_fixed BEFORE UPDATE OF name ON daily
BEGIN SELECT RAISE(ABORT, 'daily_required'); END;

-- `docs` counts live documents; `doc_bytes` is the sum of length(sealed) + 512 over all of
-- the account's rows, tombstones included; `files` and `file_bytes` count account_files rows
-- and their size + 1024. NULL quotas use `defaults`; a NULL `quota_writes_per_day` uses the
-- `writes_per_day` setting (the Worker reads it; no trigger does). `recounted_at` changes
-- only in the admin recount, which the quota triggers exempt.
CREATE TABLE accounts (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (length(id) = 34 AND substr(id, 1, 2) = 'a_' AND substr(id, 3) NOT GLOB '*[^0-9a-f]*'),
    username TEXT NOT NULL UNIQUE
        CHECK (length(username) BETWEEN 3 AND 32 AND username NOT GLOB '*[^a-z0-9_-]*'
               AND username GLOB '[a-z0-9]*' AND username GLOB '*[a-z0-9]'),
    kdf_m_kib INTEGER NOT NULL CHECK (kdf_m_kib BETWEEN 1048576 AND 2097152),
    kdf_t INTEGER NOT NULL CHECK (kdf_t BETWEEN 3 AND 8),
    kdf_p INTEGER NOT NULL CHECK (kdf_p BETWEEN 1 AND 8),
    salt TEXT NOT NULL CHECK (length(salt) = 22 AND salt NOT GLOB '*[^A-Za-z0-9_-]*'),
    auth_hash TEXT NOT NULL CHECK (length(auth_hash) = 64 AND auth_hash NOT GLOB '*[^0-9a-f]*'),
    wrapped_key TEXT NOT NULL CHECK (length(wrapped_key) = 98 AND wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*'),
    recovery_hash TEXT NOT NULL CHECK (length(recovery_hash) = 64 AND recovery_hash NOT GLOB '*[^0-9a-f]*'),
    recovery_wrapped_key TEXT NOT NULL
        CHECK (length(recovery_wrapped_key) = 98 AND recovery_wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*'),
    key_id TEXT NOT NULL CHECK (length(key_id) = 16 AND key_id NOT GLOB '*[^0-9a-f]*'),
    epoch TEXT NOT NULL CHECK (length(epoch) = 16 AND epoch NOT GLOB '*[^0-9a-f]*'),
    last_seq INTEGER NOT NULL DEFAULT 0 CHECK (last_seq >= 0),
    docs INTEGER NOT NULL DEFAULT 0 CHECK (docs >= 0),
    doc_bytes INTEGER NOT NULL DEFAULT 0 CHECK (doc_bytes >= 0),
    files INTEGER NOT NULL DEFAULT 0 CHECK (files >= 0),
    file_bytes INTEGER NOT NULL DEFAULT 0 CHECK (file_bytes >= 0),
    quota_docs INTEGER CHECK (quota_docs > 0),
    quota_doc_bytes INTEGER CHECK (quota_doc_bytes > 0),
    quota_files INTEGER CHECK (quota_files > 0),
    quota_file_bytes INTEGER CHECK (quota_file_bytes > 0),
    quota_writes_per_day INTEGER CHECK (quota_writes_per_day > 0),
    recounted_at INTEGER,
    created_at INTEGER NOT NULL,
    disabled_at INTEGER
);
CREATE INDEX accounts_created ON accounts (created_at);

-- Sign-ups: the switch (or a reservation for one name), then max_accounts, then the daily
-- cap, counted in `daily`. The WHEN clauses exclude each other, so one refusal names one
-- reason.
CREATE TRIGGER signup_closed BEFORE INSERT ON accounts
WHEN coalesce((SELECT value FROM settings WHERE name = 'signup'), 'closed') NOT IN ('open', 'only:' || NEW.username)
BEGIN SELECT RAISE(ABORT, 'signup_closed:closed'); END;

CREATE TRIGGER signup_full BEFORE INSERT ON accounts
WHEN coalesce((SELECT value FROM settings WHERE name = 'signup'), 'closed') IN ('open', 'only:' || NEW.username)
 AND (SELECT count(*) FROM accounts) >= coalesce((SELECT max_accounts FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'signup_closed:full'); END;

CREATE TRIGGER signup_busy BEFORE INSERT ON accounts
WHEN coalesce((SELECT value FROM settings WHERE name = 'signup'), 'closed') IN ('open', 'only:' || NEW.username)
 AND (SELECT count(*) FROM accounts) < coalesce((SELECT max_accounts FROM defaults), 0)
 AND coalesce((SELECT CASE WHEN day >= NEW.created_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'signups'), 9223372036854775807)
     >= coalesce((SELECT signups_per_day FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'signup_closed:busy'); END;

CREATE TRIGGER signup_count AFTER INSERT ON accounts
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.created_at / 86400000 THEN count + 1 ELSE 1 END,
                     day = max(day, NEW.created_at / 86400000)
    WHERE name = 'signups';
END;

-- Quotas only stop growth. One refusal names one quota, in the order docs, doc_bytes,
-- server (global document bytes); files, file_bytes, server (global file bytes).
CREATE TRIGGER quota_docs BEFORE UPDATE OF docs ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.docs > OLD.docs
 AND NEW.docs > coalesce(NEW.quota_docs, (SELECT docs FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:docs'); END;

CREATE TRIGGER quota_doc_bytes BEFORE UPDATE OF doc_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.doc_bytes > OLD.doc_bytes
 AND NOT (NEW.docs > OLD.docs AND NEW.docs > coalesce(NEW.quota_docs, (SELECT docs FROM defaults), 0))
 AND NEW.doc_bytes > coalesce(NEW.quota_doc_bytes, (SELECT doc_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:doc_bytes'); END;

CREATE TRIGGER quota_server_docs BEFORE UPDATE OF doc_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.doc_bytes > OLD.doc_bytes
 AND NOT (NEW.docs > OLD.docs AND NEW.docs > coalesce(NEW.quota_docs, (SELECT docs FROM defaults), 0))
 AND NEW.doc_bytes <= coalesce(NEW.quota_doc_bytes, (SELECT doc_bytes FROM defaults), 0)
 AND (SELECT sum(doc_bytes) FROM accounts) - OLD.doc_bytes + NEW.doc_bytes
     > coalesce((SELECT global_doc_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:server'); END;

CREATE TRIGGER quota_files BEFORE UPDATE OF files ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.files > OLD.files
 AND NEW.files > coalesce(NEW.quota_files, (SELECT files FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:files'); END;

CREATE TRIGGER quota_file_bytes BEFORE UPDATE OF file_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.file_bytes > OLD.file_bytes
 AND NOT (NEW.files > OLD.files AND NEW.files > coalesce(NEW.quota_files, (SELECT files FROM defaults), 0))
 AND NEW.file_bytes > coalesce(NEW.quota_file_bytes, (SELECT file_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:file_bytes'); END;

CREATE TRIGGER quota_server_files BEFORE UPDATE OF file_bytes ON accounts
WHEN NEW.recounted_at IS OLD.recounted_at AND NEW.file_bytes > OLD.file_bytes
 AND NOT (NEW.files > OLD.files AND NEW.files > coalesce(NEW.quota_files, (SELECT files FROM defaults), 0))
 AND NEW.file_bytes <= coalesce(NEW.quota_file_bytes, (SELECT file_bytes FROM defaults), 0)
 AND (SELECT sum(file_bytes) FROM accounts) - OLD.file_bytes + NEW.file_bytes
     > coalesce((SELECT global_file_bytes FROM defaults), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:server'); END;

-- One row per device per account. Revoked rows stay 90 days (to explain a 401 and to keep a
-- returning device's name); active rows expire after 180 days unseen. `proof_hash` is SHA-256
-- of the device's proof, which only that device can derive; only an active row's exempts a
-- sign-in from `login-user`.
CREATE TABLE sessions (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    device_id TEXT NOT NULL
        CHECK (length(device_id) = 34 AND substr(device_id, 1, 2) = 'd_' AND substr(device_id, 3) NOT GLOB '*[^0-9a-f]*'),
    token_hash TEXT NOT NULL UNIQUE CHECK (length(token_hash) = 64 AND token_hash NOT GLOB '*[^0-9a-f]*'),
    proof_hash TEXT NOT NULL CHECK (length(proof_hash) = 64 AND proof_hash NOT GLOB '*[^0-9a-f]*'),
    sealed_name TEXT
        CHECK (sealed_name IS NULL OR (length(sealed_name) BETWEEN 55 AND 400 AND sealed_name NOT GLOB '*[^A-Za-z0-9_-]*')),
    created_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    revoked_at INTEGER,
    revoked_reason TEXT CHECK (revoked_reason IN
        ('signed_out', 'revoked', 'password_changed', 'recovered', 'evicted', 'expired', 'rekeyed')),
    CHECK ((revoked_at IS NULL) = (revoked_reason IS NULL)),
    PRIMARY KEY (account_id, device_id)
);
CREATE INDEX sessions_by_account ON sessions (account_id, revoked_at, last_seen_at);

-- Sealed documents, keyed by (account, wire id). Only base64url can be stored.
CREATE TABLE account_docs (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    id TEXT NOT NULL CHECK (
        ((substr(id, 1, 5) IN ('note:', 'book:', 'file:')) AND length(id) = 27
             AND substr(id, 6) NOT GLOB '*[^A-Za-z0-9_-]*')
        OR (substr(id, 1, 4) = 'tag:' AND length(id) = 26 AND substr(id, 5) NOT GLOB '*[^A-Za-z0-9_-]*')),
    rev TEXT NOT NULL CHECK (length(rev) = 32 AND rev NOT GLOB '*[^0-9a-f]*'),
    seq INTEGER NOT NULL CHECK (seq > 0),
    deleted INTEGER NOT NULL CHECK (deleted IN (0, 1)),
    sealed TEXT NOT NULL CHECK (length(sealed) BETWEEN 55 AND 1900000 AND sealed NOT GLOB '*[^A-Za-z0-9_-]*'),
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, id),
    UNIQUE (account_id, seq)
);

-- Server-wide document writes per UTC day (global_writes_per_day), shared fairly: once the
-- day's count reaches the cap, a write is refused only for an account above its fair share
-- (global_writes_per_day / max_accounts), judged by its own `writes-account` count today, which
-- the Worker takes before the batch (this push included). An account at or below its fair share
-- is never refused here, so a few accounts at their own budgets can't pause everyone; a day's
-- total stays under about twice the cap. Fails closed: a missing setting, max_accounts 0, or no
-- `writes-account` row for today counts as above the fair share. The push's upsert fires BEFORE
-- INSERT on both of its paths, then AFTER INSERT or AFTER UPDATE, so every stored write is
-- checked and counted once.
CREATE TRIGGER writes_global BEFORE INSERT ON account_docs
WHEN coalesce((SELECT CASE WHEN day >= NEW.updated_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'writes'), 9223372036854775807)
     >= coalesce((SELECT global_writes_per_day FROM defaults), 0)
 AND coalesce((SELECT CASE WHEN window_start = NEW.updated_at / 86400000 * 86400000 THEN count END
               FROM limits WHERE key = 'writes-account:' || NEW.account_id), 9223372036854775807)
     > coalesce((SELECT global_writes_per_day / max_accounts FROM defaults WHERE max_accounts > 0), -1)
BEGIN SELECT RAISE(ABORT, 'rate_limited:writes-global'); END;

CREATE TRIGGER writes_count_insert AFTER INSERT ON account_docs
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.updated_at / 86400000 THEN count + 1 ELSE 1 END,
                     day = max(day, NEW.updated_at / 86400000)
    WHERE name = 'writes';
END;

CREATE TRIGGER writes_count_update AFTER UPDATE OF rev ON account_docs
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.updated_at / 86400000 THEN count + 1 ELSE 1 END,
                     day = max(day, NEW.updated_at / 86400000)
    WHERE name = 'writes';
END;

-- Every stored R2 object and its size, for quotas and for deleting without listing.
-- `uploaded_at` is the time of the upload that stored this size; corrections (a failed R2
-- put's compensation, the cron's reconciliation) change `size` but never `uploaded_at`.
CREATE TABLE account_files (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    id TEXT NOT NULL CHECK (substr(id, 1, 5) = 'file:' AND length(id) = 27 AND substr(id, 6) NOT GLOB '*[^A-Za-z0-9_-]*'),
    size INTEGER NOT NULL CHECK (size BETWEEN 41 AND 10485801),
    uploaded_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, id)
);

-- Server-wide upload bytes per UTC day (global_upload_bytes_per_day), weighted by size and
-- counted on every upload, replacements included. Shared fairly as document writes are: past
-- the cap, only an account whose own `uploads-account` count today (this upload included) is
-- above global_upload_bytes_per_day / max_accounts is refused, and the same things fail closed.
CREATE TRIGGER uploads_global BEFORE INSERT ON account_files
WHEN coalesce((SELECT CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count ELSE 0 END
               FROM daily WHERE name = 'upload_bytes'), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day FROM defaults), 0) - NEW.size
 AND coalesce((SELECT CASE WHEN window_start = NEW.uploaded_at / 86400000 * 86400000 THEN count END
               FROM limits WHERE key = 'uploads-account:' || NEW.account_id), 9223372036854775807)
     > coalesce((SELECT global_upload_bytes_per_day / max_accounts FROM defaults WHERE max_accounts > 0), -1)
BEGIN SELECT RAISE(ABORT, 'rate_limited:uploads-global'); END;

CREATE TRIGGER uploads_count_insert AFTER INSERT ON account_files
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.uploaded_at / 86400000)
    WHERE name = 'upload_bytes';
END;

CREATE TRIGGER uploads_count_update AFTER UPDATE OF uploaded_at ON account_files
BEGIN
    UPDATE daily SET count = CASE WHEN day >= NEW.uploaded_at / 86400000 THEN count + NEW.size ELSE NEW.size END,
                     day = max(day, NEW.uploaded_at / 86400000)
    WHERE name = 'upload_bytes';
END;

-- Fixed-window counters. Keys hold HMACs of IPs and usernames, never the values.
CREATE TABLE limits (
    key TEXT PRIMARY KEY NOT NULL CHECK (length(key) <= 80),
    window_start INTEGER NOT NULL,
    count INTEGER NOT NULL CHECK (count >= 0)
);

-- R2 prefixes still to empty (deleted accounts, legacy files/).
CREATE TABLE purges (
    prefix TEXT PRIMARY KEY NOT NULL
        CHECK (prefix = 'files/' OR (prefix GLOB 'accounts/a_*/' AND length(prefix) = 44)),
    requested_at INTEGER NOT NULL
);
