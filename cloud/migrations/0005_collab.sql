-- Shared notes (docs/ACCOUNTS.md › Collaboration). Additive only: new tables, indexes and
-- triggers, nothing existing changes. The note itself never reaches D1: each shared note's
-- log lives in its own Durable Object (NoteRoom, src/room.ts). D1 holds who may do what:
-- each account's public keys, every share's owner and epoch, every epoch's signed member
-- list, the members and their wrapped note keys, invite links, requests to join, and notifications.
-- Every value the devices send is a base64url string of a fixed shape, checked here.
--
-- The caps (members, owned shares, live links, pending requests) are triggers, so
-- concurrent requests can't race past them; they refuse with `quota_exceeded:<kind>`,
-- which the Worker answers with 413. Notifications keep the newest 500 per recipient.

-- One identity per account, set once by its first device that needs one and never replaced:
-- the Ed25519 and X25519 public keys, the binding signature over them, and both private keys
-- sealed under a key only the account's devices can derive.
CREATE TABLE identities (
    account_id TEXT PRIMARY KEY NOT NULL REFERENCES accounts (id),
    sign_pub TEXT NOT NULL CHECK (length(sign_pub) = 43 AND sign_pub NOT GLOB '*[^A-Za-z0-9_-]*'),
    box_pub TEXT NOT NULL CHECK (length(box_pub) = 43 AND box_pub NOT GLOB '*[^A-Za-z0-9_-]*'),
    binding TEXT NOT NULL CHECK (length(binding) = 86 AND binding NOT GLOB '*[^A-Za-z0-9_-]*'),
    sealed TEXT NOT NULL CHECK (length(sealed) = 140 AND sealed NOT GLOB '*[^A-Za-z0-9_-]*'),
    created_at INTEGER NOT NULL
);

-- One row per shared note. `id` is 16 random bytes from the owner's device. `list` is the
-- owner's signed member list of the current epoch, as base64url of its JSON bytes, and
-- `list_sig` its Ed25519 signature. `bytes` is what the note's room stores (its snapshot and
-- log), as the room last reported it; it counts towards the owner's document bytes when they
-- share another note.
CREATE TABLE collab_shares (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    owner_id TEXT NOT NULL REFERENCES accounts (id),
    epoch INTEGER NOT NULL CHECK (epoch >= 1),
    list TEXT NOT NULL CHECK (length(list) BETWEEN 1 AND 16384 AND list NOT GLOB '*[^A-Za-z0-9_-]*'),
    list_sig TEXT NOT NULL CHECK (length(list_sig) = 86 AND list_sig NOT GLOB '*[^A-Za-z0-9_-]*'),
    bytes INTEGER NOT NULL DEFAULT 0 CHECK (bytes >= 0),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX collab_shares_by_owner ON collab_shares (owner_id);

-- At most 200 shared notes per owner, and none once the owner's documents and shared notes
-- together are past their document-byte quota (fails closed like 0002's quota triggers).
CREATE TRIGGER collab_shares_cap BEFORE INSERT ON collab_shares
WHEN (SELECT count(*) FROM collab_shares WHERE owner_id = NEW.owner_id) >= 200
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:shares'); END;

CREATE TRIGGER collab_shares_doc_bytes BEFORE INSERT ON collab_shares
WHEN (SELECT count(*) FROM collab_shares WHERE owner_id = NEW.owner_id) < 200
 AND coalesce((SELECT doc_bytes FROM accounts WHERE id = NEW.owner_id), 0)
     + coalesce((SELECT sum(bytes) FROM collab_shares WHERE owner_id = NEW.owner_id), 0) + NEW.bytes
     > coalesce((SELECT coalesce(a.quota_doc_bytes, d.doc_bytes) FROM accounts a, defaults d WHERE a.id = NEW.owner_id), 0)
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:doc_bytes'); END;

-- Who is in each note, with the role the signed list gives them: `invited` until they accept
-- (or, when they asked through a link, `member` at once), `member` after. The owner is a
-- member with the role `owner`.
CREATE TABLE collab_members (
    share_id TEXT NOT NULL REFERENCES collab_shares (id),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    role TEXT NOT NULL CHECK (role IN ('owner', 'edit', 'comment', 'view')),
    status TEXT NOT NULL CHECK (status IN ('invited', 'member')),
    invited_at INTEGER NOT NULL,
    joined_at INTEGER,
    PRIMARY KEY (share_id, account_id)
);
CREATE INDEX collab_members_by_account ON collab_members (account_id);

-- At most 20 people per note, the owner included. An upsert of someone already in the note
-- fires this too, so they don't count against themselves.
CREATE TRIGGER collab_members_cap BEFORE INSERT ON collab_members
WHEN (SELECT count(*) FROM collab_members WHERE share_id = NEW.share_id AND account_id != NEW.account_id) >= 20
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:members'); END;

-- Each member's note key for each epoch they were in, wrapped to their box key: 105 bytes.
-- A member removed from the note loses every wrap.
CREATE TABLE collab_wraps (
    share_id TEXT NOT NULL REFERENCES collab_shares (id),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    epoch INTEGER NOT NULL CHECK (epoch >= 1),
    wrapped TEXT NOT NULL CHECK (length(wrapped) = 140 AND wrapped NOT GLOB '*[^A-Za-z0-9_-]*'),
    PRIMARY KEY (share_id, account_id, epoch)
);
CREATE INDEX collab_wraps_by_account ON collab_wraps (account_id);

-- Every epoch's signed member list, kept for the life of the share: a member checks each
-- envelope's author, and each epoch key's commitment, against the list of that envelope's
-- own epoch. A member is given the lists of the epochs it holds a key for, and the current one.
CREATE TABLE collab_lists (
    share_id TEXT NOT NULL REFERENCES collab_shares (id),
    epoch INTEGER NOT NULL CHECK (epoch >= 1),
    list TEXT NOT NULL CHECK (length(list) BETWEEN 1 AND 16384 AND list NOT GLOB '*[^A-Za-z0-9_-]*'),
    list_sig TEXT NOT NULL CHECK (length(list_sig) = 86 AND list_sig NOT GLOB '*[^A-Za-z0-9_-]*'),
    PRIMARY KEY (share_id, epoch)
);

-- Invite links: `<server>/j/<id>#<secret>`. The server keeps SHA-256 of the proof derived
-- from the secret, never the secret, and the role an approved request is offered.
CREATE TABLE collab_links (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    share_id TEXT NOT NULL REFERENCES collab_shares (id),
    proof_hash TEXT NOT NULL CHECK (length(proof_hash) = 64 AND proof_hash NOT GLOB '*[^0-9a-f]*'),
    role TEXT NOT NULL CHECK (role IN ('edit', 'comment', 'view')),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL CHECK (expires_at > created_at)
);
CREATE INDEX collab_links_by_share ON collab_links (share_id, expires_at);

-- At most 10 live (unexpired) links per note.
CREATE TRIGGER collab_links_cap BEFORE INSERT ON collab_links
WHEN (SELECT count(*) FROM collab_links WHERE share_id = NEW.share_id AND expires_at > NEW.created_at) >= 10
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:links'); END;

-- Requests to join through a link, until the owner approves (a new epoch with them in it)
-- or declines.
CREATE TABLE collab_requests (
    share_id TEXT NOT NULL REFERENCES collab_shares (id),
    account_id TEXT NOT NULL REFERENCES accounts (id),
    link_id TEXT NOT NULL CHECK (length(link_id) = 22 AND link_id NOT GLOB '*[^A-Za-z0-9_-]*'),
    role TEXT NOT NULL CHECK (role IN ('edit', 'comment', 'view')),
    requested_at INTEGER NOT NULL,
    PRIMARY KEY (share_id, account_id)
);
CREATE INDEX collab_requests_by_account ON collab_requests (account_id);

-- At most 50 pending requests per note; asking again replaces one's own.
CREATE TRIGGER collab_requests_cap BEFORE INSERT ON collab_requests
WHEN (SELECT count(*) FROM collab_requests WHERE share_id = NEW.share_id AND account_id != NEW.account_id) >= 50
BEGIN SELECT RAISE(ABORT, 'quota_exceeded:requests'); END;

-- Notifications (@mentions), sealed to the recipient's box key: the server sees who wrote to
-- whom about which note, and when, never what.
CREATE TABLE collab_notifications (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    recipient_id TEXT NOT NULL REFERENCES accounts (id),
    share_id TEXT NOT NULL CHECK (length(share_id) = 22 AND share_id NOT GLOB '*[^A-Za-z0-9_-]*'),
    sender_id TEXT NOT NULL CHECK (length(sender_id) = 34 AND sender_id GLOB 'a_*'),
    sealed TEXT NOT NULL CHECK (length(sealed) BETWEEN 98 AND 5462 AND sealed NOT GLOB '*[^A-Za-z0-9_-]*'),
    at INTEGER NOT NULL
);
CREATE INDEX collab_notifications_by_recipient ON collab_notifications (recipient_id, at);

-- The newest 500 per recipient are kept: an insert past that drops the oldest.
CREATE TRIGGER collab_notifications_keep AFTER INSERT ON collab_notifications
BEGIN
    DELETE FROM collab_notifications WHERE recipient_id = NEW.recipient_id AND id NOT IN (
        SELECT id FROM collab_notifications WHERE recipient_id = NEW.recipient_id ORDER BY at DESC, id DESC LIMIT 500);
END;

-- Every share ID that stopped being shared (stopped by its owner, or its owner's account
-- deleted), kept for good: no share may use it again.
CREATE TABLE collab_retired (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    retired_at INTEGER NOT NULL
);

-- Whatever deletes a share's row retires its ID, so no path can forget to.
CREATE TRIGGER collab_shares_retire AFTER DELETE ON collab_shares
BEGIN
    INSERT OR IGNORE INTO collab_retired (id, retired_at) VALUES (OLD.id, CAST(strftime('%s', 'now') AS INTEGER) * 1000);
END;

-- Rooms still to purge: the cron tells each one to delete its log and close its sockets,
-- then drops the row.
CREATE TABLE collab_purges (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 22 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
    requested_at INTEGER NOT NULL
);
