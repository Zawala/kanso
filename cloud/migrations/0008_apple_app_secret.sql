-- Sign in with Apple is bound to the app process that opened it. Apple's callback
-- makes a fresh 32-byte secret and stores only its SHA-256 here; the page hands the
-- secret to ASWebAuthenticationSession through kanso://auth/apple?flow=…&s=…, and
-- the app's poll must send it back. Whoever started the flow never learns it, so a
-- phished auth_url can't sign the attacker in to, or link, the victim's Apple ID.
-- `app_claimed_at` is set by the first poll that sends the secret; a ticket or a
-- proof from an Apple flow counts only once the app has claimed it.
ALTER TABLE auth_flows ADD COLUMN app_secret_hash TEXT
    CHECK (app_secret_hash IS NULL OR (length(app_secret_hash) = 64
        AND app_secret_hash NOT GLOB '*[^0-9a-f]*'));
ALTER TABLE auth_flows ADD COLUMN app_claimed_at INTEGER;
