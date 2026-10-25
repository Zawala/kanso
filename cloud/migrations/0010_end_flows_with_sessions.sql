-- A session's sensitive ceremonies end with it (security review of PR #67, finding 1). A link,
-- a proof or a passkey registration a session started must not outlive that session: when the
-- session is revoked (signed out, revoked, evicted, expired, or ended by a password change, a
-- first password or a recovery), its device's unconsumed `link` and `proof` flows (provider
-- flows by intent; passkey proofs are kind `proof`) and `passkey_register` flows are consumed in
-- the same statement's transaction, and when an account is disabled (the admin script), all of
-- the account's. Sign-in and sign-up flows aren't touched: they belong to no session. Additive.
CREATE TRIGGER sessions_end_flows AFTER UPDATE OF revoked_at ON sessions
WHEN OLD.revoked_at IS NULL AND NEW.revoked_at IS NOT NULL
BEGIN
    UPDATE auth_flows SET consumed_at = NEW.revoked_at
    WHERE account_id = NEW.account_id AND device_id = NEW.device_id AND consumed_at IS NULL
        AND (kind IN ('proof', 'passkey_register') OR intent IN ('link', 'proof'));
END;

CREATE TRIGGER accounts_disabled_end_flows AFTER UPDATE OF disabled_at ON accounts
WHEN OLD.disabled_at IS NULL AND NEW.disabled_at IS NOT NULL
BEGIN
    UPDATE auth_flows SET consumed_at = NEW.disabled_at
    WHERE account_id = NEW.id AND consumed_at IS NULL
        AND (kind IN ('proof', 'passkey_register') OR intent IN ('link', 'proof'));
END;
