-- Passkeys and third-party sign-in (design: Kanso accounts spec, revision 3). Additive:
-- 0002's rows, triggers and quotas are untouched. `has_password` defaults to 1, so every
-- existing account is a password account. A provider row holds the stable subject only;
-- emails, names, profile data and provider tokens are never stored.

ALTER TABLE accounts ADD COLUMN has_password INTEGER NOT NULL DEFAULT 1;

CREATE TABLE account_identities (
    account_id TEXT NOT NULL REFERENCES accounts (id),
    provider TEXT NOT NULL
        CHECK (length(provider) BETWEEN 2 AND 32 AND provider GLOB '[a-z]*'
               AND provider NOT GLOB '*[^a-z0-9_]*'),
    subject TEXT NOT NULL
        CHECK (length(subject) BETWEEN 1 AND 255 AND subject NOT GLOB '*[^ -~]*'),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (provider, subject)
);
CREATE INDEX account_identities_by_account ON account_identities (account_id, provider);

-- One row per passkey. `wrapped_key` is the data key under the PRF-derived KEK (98 base64url
-- characters), or NULL when the credential registered without PRF. `aaguid` is for display.
CREATE TABLE account_passkeys (
    credential_id TEXT PRIMARY KEY NOT NULL
        CHECK (length(credential_id) BETWEEN 22 AND 1364
               AND credential_id NOT GLOB '*[^A-Za-z0-9_-]*'),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    public_key TEXT NOT NULL
        CHECK (length(public_key) BETWEEN 20 AND 2048 AND public_key NOT GLOB '*[^A-Za-z0-9_-]*'),
    alg INTEGER NOT NULL CHECK (alg IN (-8, -7, -257)),
    sign_count INTEGER NOT NULL CHECK (sign_count >= 0),
    transports TEXT NOT NULL DEFAULT ''
        CHECK (length(transports) <= 64 AND transports NOT GLOB '*[^a-z ]*'),
    wrapped_key TEXT CHECK (wrapped_key IS NULL
        OR (length(wrapped_key) = 98 AND wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*')),
    aaguid TEXT CHECK (aaguid IS NULL
        OR (length(aaguid) = 32 AND aaguid NOT GLOB '*[^0-9a-f]*')),
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    CHECK (last_used_at IS NULL OR last_used_at >= created_at)
);
CREATE INDEX account_passkeys_by_account ON account_passkeys (account_id, created_at);

-- One row per provider or passkey ceremony. `device_code` is GitHub's (sealed under the pepper's
-- flow key, because it can mint a token); `nonce_hash` and `state_hash` are SHA-256 hex. A row is
-- consumed once and deleted by the cron.
CREATE TABLE auth_flows (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (length(id) = 43 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    kind TEXT NOT NULL CHECK (length(kind) BETWEEN 2 AND 32 AND kind GLOB '[a-z]*'
        AND kind NOT GLOB '*[^a-z0-9_]*'),
    intent TEXT CHECK (intent IS NULL OR intent IN ('signin', 'signup', 'link', 'proof')),
    -- The account, or for a sign-up flow the client's proposed account ID, which has no row yet.
    account_id TEXT,
    device_id TEXT,
    device_proof_hash TEXT CHECK (device_proof_hash IS NULL OR (length(device_proof_hash) = 64
        AND device_proof_hash NOT GLOB '*[^0-9a-f]*')),
    challenge TEXT CHECK (challenge IS NULL OR (length(challenge) = 43
        AND challenge NOT GLOB '*[^A-Za-z0-9_-]*')),
    rp_id TEXT,
    nonce_hash TEXT CHECK (nonce_hash IS NULL OR (length(nonce_hash) = 64
        AND nonce_hash NOT GLOB '*[^0-9a-f]*')),
    state_hash TEXT CHECK (state_hash IS NULL OR (length(state_hash) = 64
        AND state_hash NOT GLOB '*[^0-9a-f]*')),
    pkce_hash TEXT CHECK (pkce_hash IS NULL OR (length(pkce_hash) = 64
        AND pkce_hash NOT GLOB '*[^0-9a-f]*')),
    redirect_uri TEXT CHECK (redirect_uri IS NULL OR length(redirect_uri) <= 255),
    provider TEXT,
    subject TEXT CHECK (subject IS NULL
        OR (length(subject) BETWEEN 1 AND 255 AND subject NOT GLOB '*[^ -~]*')),
    device_code TEXT CHECK (device_code IS NULL OR length(device_code) <= 512),
    next_poll_at INTEGER,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    consumed_at INTEGER,
    CHECK (expires_at > created_at),
    -- Provider flows name their provider; passkey and proof flows don't. A new provider's kind is
    -- just its name, so it needs no migration.
    CHECK ((kind IN ('passkey_signin', 'passkey_signup', 'passkey_register', 'proof'))
           = (provider IS NULL))
);
CREATE INDEX auth_flows_expiry ON auth_flows (expires_at);

-- One row per device-approval request. The account, requesting device and requesting ephemeral
-- key never change; the approver's key, the sealed name and the sealed data key are filled once.
CREATE TABLE device_approvals (
    id TEXT PRIMARY KEY NOT NULL
        CHECK (length(id) = 43 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    device_id TEXT NOT NULL CHECK (length(device_id) = 34 AND substr(device_id, 1, 2) = 'd_'
        AND substr(device_id, 3) NOT GLOB '*[^0-9a-f]*'),
    commit_hash TEXT NOT NULL
        CHECK (length(commit_hash) = 43 AND commit_hash NOT GLOB '*[^A-Za-z0-9_-]*'),
    -- Revealed with the sealed name, after the challenge; checked against commit_hash.
    epk TEXT CHECK (epk IS NULL OR (length(epk) = 43 AND epk NOT GLOB '*[^A-Za-z0-9_-]*')),
    approver_device_id TEXT,
    approver_epk TEXT CHECK (approver_epk IS NULL
        OR (length(approver_epk) = 43 AND approver_epk NOT GLOB '*[^A-Za-z0-9_-]*')),
    sealed_name TEXT CHECK (sealed_name IS NULL
        OR (length(sealed_name) BETWEEN 55 AND 400 AND sealed_name NOT GLOB '*[^A-Za-z0-9_-]*')),
    wrapped_key TEXT CHECK (wrapped_key IS NULL
        OR (length(wrapped_key) = 98 AND wrapped_key NOT GLOB '*[^A-Za-z0-9_-]*')),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    completed_at INTEGER,
    cancelled_at INTEGER,
    CHECK (expires_at > created_at),
    CHECK (cancelled_at IS NULL OR completed_at IS NULL),
    -- A revealed requester key only exists once an approver has challenged: it is
    -- posted with the sealed name, after that key is known.
    CHECK (epk IS NULL OR approver_epk IS NOT NULL)
);
CREATE INDEX device_approvals_by_account ON device_approvals (account_id, expires_at);

CREATE TRIGGER device_approvals_immutable
BEFORE UPDATE OF account_id, device_id, commit_hash, created_at, expires_at ON device_approvals
BEGIN SELECT RAISE(ABORT, 'approval_immutable'); END;

-- epk, approver_epk and sealed_name are each set once.
CREATE TRIGGER device_approvals_set_once
BEFORE UPDATE OF epk, approver_epk, sealed_name ON device_approvals
WHEN (OLD.epk IS NOT NULL AND NEW.epk IS NOT OLD.epk)
  OR (OLD.approver_epk IS NOT NULL AND NEW.approver_epk IS NOT OLD.approver_epk)
  OR (OLD.sealed_name IS NOT NULL AND NEW.sealed_name IS NOT OLD.sealed_name)
BEGIN SELECT RAISE(ABORT, 'approval_immutable'); END;
