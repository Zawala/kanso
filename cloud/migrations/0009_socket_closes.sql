-- Accounts whose live event sockets (ACCOUNT_ROOM) must close, for the session ends the
-- Worker doesn't see happen: the admin script's `disable` and `delete` run through
-- `wrangler d1 execute` and can't reach a Durable Object. They insert a row here; the
-- cron closes every socket of the account and deletes the row, so those sockets close
-- within one cron interval (15 minutes). Additive; nothing else reads it.
CREATE TABLE socket_closes (
    account_id TEXT PRIMARY KEY NOT NULL CHECK (length(account_id) BETWEEN 1 AND 64),
    requested_at INTEGER NOT NULL
);
