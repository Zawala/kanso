# Accounts and end-to-end encryption

Status: implementation spec, revision 2 (2026-10-18), after one plan review and one security
review. It freezes when step 1 of the build order merges. From then on, changing a frozen file needs
approval from every builder that consumes it (see "Build order"). It replaces the single
shared-token sync in [PLAN.md](PLAN.md) › Sync. Step 1 applied a security re-check while freezing the
contract; its decisions are listed under "Build order" › "Contract notes".

Kanso's sync moves from one library behind one shared secret to accounts. Anyone with the app can
create an account on a Kanso sync Worker, by default the owner's. Each account's library is
private, and everything in it is encrypted on the Mac before it leaves. The Worker and its owner
hold ciphertext, sizes and timestamps. They can't read notes, notebooks, tags, file names, file
bytes or device names.

Out of scope: sharing between accounts, email, two-factor sign-in and a web client.

## Changes since review

Owner's answers, final:

- **Recovery key.** "New recovery key" is a route (`POST /v2/account/recovery-key`), and recovery rotates the key it used.
- **Plan.** The account is on Workers Paid and R2 Paid. Every default and capacity sum is re-sized for a 10 GB D1 database, with the arithmetic shown.
- **Password change** keeps signing out the other Macs.
- **Sessions** expire after 180 days unseen. A sign-out made offline stays pending until the server accepts it.
- **Distribution.** Friends run ad-hoc-signed builds the owner makes. The threat model and the rollout now say what that means, including Gatekeeper.

Plan review (all applied):

1. **Rollback lost edits.** The old app would have inherited v2's `synced_rev` and cursor. Fix:
   - v2 keeps its own meta keys (`sync.v2.*`).
   - `kanso-sync unbind` resets the bookkeeping for v1.
   - The config keeps `endpoint`.
   - The rollback steps say deleted notes may come back.
   - A test proves no edit is lost.

   Merged with security H5.
2. **Re-binding during a sync could mix accounts.** Every re-bind first cancels and joins the running sync. The store refuses sync writes whose account doesn't match the library's binding, checked in the same transaction. Merged with M6.
3. **Replace is one transaction** (`Store::replace_library`). The recovery key goes into the Keychain right after sign-up succeeds, before the library is bound.
4. **Step 1 freezes everything the other builders compile against:** every type, stubs, `MemoryServer` controls, Store APIs, and fixtures with named owners. A Rust milestone, 2a, unblocks the app's flow tests. `crypto-v1.json` moves out of step 1.
5. **Problem model.** It has one label per variant. Problems found in a successful sync are reported separately (`report_problem`). Each problem says whether the schedule keeps running. Unreadable conflicts no longer land in `rejected`.
6. **Wire details.** Status codes for every limit; which fields are required; unknown keys ignored; a mandatory SHA-256 on file uploads; exact username trimming and its fixture cases; the order quota errors are reported in; and codes for an unsupported KDF, a mismatched username and a disabled logout. All key and ID inputs are pinned, and auth conformance cases are added.
7. **No `test-kdf` feature.** The KDF floor comes from the backend, so tests use `MemoryServer`'s low floor and no build can lower the production one.
8. **Capacity re-sized for Workers Paid.** The numbers are under "Quotas and capacity".
9. **WrongKey has one trigger.** It comes from a failed unwrap or a key-id mismatch, never from document failures. An unknown format byte means "Update Kanso". Merged with M3.
10. **Merging is explained.** The question says notes deleted elsewhere may come back. Sign-up into an empty account doesn't push tombstones. The rollout syncs every Mac first and adds a step per other Mac.
11. **Device identity.** A random library secret makes a device ID per account. A hash of the hardware UUID notices a cloned data folder. There is one device-name rule.
12. **Open questions are resolved.** Only explicit deferrals remain, under "Deferred".
13. **Tests.** New tests are added, the existing tests that must change are listed, and a decrypted test view replaces raw server access.
14. **Minor points.** Second 413 → stop; storage counts all item kinds; MiB flag names; eviction text; `enabled`; where `Session.server` comes from; how the CLI is run; exact purge checks; the Keychain recovery-key item; the CPU bench.

Security review (all applied):

- **H1. Races.** Limits, caps and quotas are enforced inside D1 (triggers, counting upserts, one-batch device eviction), and a parallel-request test proves it.
- **H2. Storage.** Sign-up starts closed. Global caps, a file-count quota and charging real row and file costs are added. Only base64url can be stored. The admin script refuses to overcommit. Global and IPv6-/48 sign-up limits are added.
- **H3. Recovery key.** It rotates on use and is replaceable while signed in. 401s carry a reason. The clipboard copy is host-only, concealed and cleared after 60 s. Saving to iCloud warns. Sign-out deletes an unconfirmed key.
- **H4. Legacy purge.** It requires a manifest from the old server and a client report proving everything in it is on the Mac and on the encrypted server. Backups of the old tables and files are kept.
- **M1. Account rollback** is detectable through `head_seq` and `epoch` on every page, after which the client re-merges. A document the server lost is pushed again.
- **M2. Revision generations only grow.** Replayed older revisions are refused as stale.
- **M3. Undecryptable documents** are repaired from the Mac's copy.
- **M4. Rekey** is specified as a later route, without a format change. The threat model is corrected: a password change doesn't replace the data key.
- **M5. The Argon2 salt input** binds the server's origin and the username, and so do the recovery derivations.
- **M7. Browser requests are refused:** the wrong content type gets 415, and any `Origin` header gets 403.
- **M8.** Sign-ins from known devices don't count against the username's limit. `recover-user` is gone. A Workers rate-limit binding sits in front of D1. IPv6 is grouped by /56. A custom domain with a WAF rule is needed to survive a flood.
- **L1. Auth hashes** are SHA-256; the pepper only keys fake salts and rate-limit keys.
- **L2. Device IDs** are per account, and device names are sealed.
- **L3. The client** caps responses per route and follows no redirects. KDF ceilings are lower.
- **L4. Session expiry** after 180 days unseen, and pending logouts.
- **L5. Usernames:** a strict ASCII check comes first, with fixture cases.
- **L6. Time Travel retention** is documented. A purge row is kept an hour after its prefix empties.
- **L7.** No test feature to misuse. Device revocation is scoped to the account.
- **L8. Invocation logs off** and no D1 read replicas.

Security review 2 of the Worker (2026-10-18, all applied; the migration was edited in place, as
nothing was deployed yet):

1. **A password change replaces the recovery key** in the same compare-and-swap update, so a
   recovery key planted by someone who once knew the password can't outlive the owner's password
   change. `POST /v2/account/password` gains `recovery_auth_key` and `recovery_wrapped_key`.
2. **Expensive session reads are bounded.** Lengths are read with `octet_length(sealed)`, which
   doesn't read the text. A second write-free binding, `SESSION_LIMITER`, allows about 120
   requests a minute per account on the session routes (429 `session-account`). A push refused as
   malformed or too large counts one write.
3. **Fair shares of the server-wide daily caps.** Past `global_writes_per_day` or
   `global_upload_bytes_per_day`, the triggers refuse only accounts above their fair share (the cap
   ÷ `max_accounts`), so five accounts at their own budgets can't pause a sixth. A day's total
   stays under about twice each cap. The admin script prints the shares.
4. **Only signed-in devices are exempt from `login-user`.** A device signed out for any reason
   counts like a stranger. `DELETE /v2/account/devices/<id>` on a row that is already signed out
   marks it `revoked` and answers 204 instead of 404.
5. **Logs and traces pinned.** `wrangler.jsonc` sets `observability.logs.enabled: true` with
   `invocation_logs: false`, and `observability.traces.enabled: false`, so a changed default
   can't start recording requests. `cloud/DEPLOYED.md` says what `wrangler tail` still shows.
6. **`signups only` right before the sign-up.** The deploy steps, the README and the admin help
   say to run it immediately before the owner's sign-up, since anyone who guesses the name can
   take it while it is reserved, and what to do if they did: `delete`, then `signups only` again.
7. **Accuracy.** The Decisions row on deletion names what Time Travel keeps (ciphertext, the
   username and the password-protected key, for up to 30 days). The migration's comment on
   `recounted_at` names both corrections that set it, and the cron's reconciliation is described
   as it is: it doesn't set `recounted_at`.
8. **Day rollover.** Tests now send requests at different times of one UTC day and on both sides
   of midnight, for the server-wide writes cap and `signups_per_day`. They found one fault: a
   request stamped just before a window's boundary but counted just after it reset that row of
   `limits` to the old window, losing the new day's count. It now counts against the newer window,
   as the `daily` counters do.
9. **IPv6 per /48 everywhere.** `login-ip` and `recover-ip` count IPv6 per /48, as `signup-ip`
   and the binding already did, so one subscriber's 256 /56s are one group.
10. **`admin restore`.** D1 is restored only through `restore <time | bookmark>`, which runs the
    Time Travel restore and then, at once, the epoch rotation of `restored --all`. Without
    `--confirm` it prints the commands; the tests never run wrangler.

Rejected: no finding. Where the result differs from a review's suggestion:

- **The global document cap is 6 GB, not the suggested 5 GB.** 50 accounts × 100 MiB is 5.24 GB,
  so a 5 GB cap would overcommit the defaults. 6 GB still leaves 36% of the database free.
- **No `test-kdf` feature, so L7's `compile_error!` guard has nothing to guard.** The floor comes
  from the backend instead (fix 7).
- **Auth hashes are SHA-256, not the brief's `HMAC(pepper, auth_key)`** (L1).
- **File uploads have no "drop the header" escape hatch.** The Worker hashes every body itself;
  CPU is no concern on Paid.

Two places go beyond the reviews:

- **Client-chosen account IDs and an `account_id` in prelogin answers.** A per-account device ID
  (L2) has to be computed before the first request, so the client must know the account ID first.
- **A daily write budget per account.** On Paid, abuse costs money rather than causing an outage,
  so the budget bounds it.

## Decisions

| Topic | Decision |
|---|---|
| Sign-up | Open to anyone with the app, once the owner opens it; the migration seeds it closed. A `signup` setting (`open`, `closed`, or `only:<username>`), `max_accounts` and `signups_per_day` are enforced by D1 triggers. No CAPTCHA. |
| Abuse controls | A write-free Workers rate-limit binding in front, then fixed-window counters in D1 (sign-up per IP; sign-ins per account and per IP; recovery per IP; password checks per account; writes per account per day). Per-account quotas on live documents, document bytes, files and file bytes, plus global caps and server-wide daily caps on sign-ups, document writes and upload bytes, all enforced by triggers that fail closed. The daily write and upload caps are shared fairly: an account within its fair share (the cap ÷ `max_accounts`) is never refused by them. |
| Identifier | A username: ASCII, case-folded, 3–32 characters. There is no email anywhere. |
| Keys | Argon2id(password, salt bound to the server's origin and the username) → HKDF splits it into an auth key (sent) and a key-encryption key (never leaves). A random data key encrypts everything. It is wrapped by the password KEK and, separately, by a recovery key that rotates on use. |
| Server secrets | The server keeps SHA-256(auth key) and compares in constant time. It hands out a wrapped data key only after the matching proof. |
| Content | Documents, files and device names are sealed with XChaCha20-Poly1305. The associated data binds the account, document, revision and deleted flag. Wire IDs and revisions are keyed hashes, and revision generations only grow. |
| Sessions | One per device; device IDs are per account. Tokens are stored hashed and expire after 180 days unseen. At most 10 active devices; signing in on an 11th signs out the least recently seen. Revoked sessions are kept 90 days to explain 401s and to keep a returning device's name. Only a signed-in device, proving itself with a per-account device proof, is exempt from the per-account sign-in limit. |
| Mac storage | The Keychain holds the session token, the data key, any pending logouts, and the recovery key until the user confirms they saved it. The Mac never stores the password, master key or KEKs. |
| Server choice | The owner's Worker is compiled in as the default. "Use a different server…" points at another deployment. Accounts are bound to their server's origin. |
| Signing out | Keychain items go (a logout is retried until the server accepts it). Local notes and the library's link to the account stay. |
| Deleting the account | Removes the account's D1 rows, R2 objects and sessions. Local notes stay. Cloudflare D1 Time Travel still holds what the deleted rows held: ciphertext, your username and your password-protected key, for up to 30 days. |
| Protocol | `/v2/`. The new Worker drops `/v1/` and the shared token; the client has one mode. |
| Legacy data | The migration only adds tables. Legacy plaintext is removed only by an explicit admin command, which requires a verified manifest. |
| Server CPU | On Workers Paid (30 s of CPU per request), the Worker still never runs Argon2 and never decodes document or file payloads. It does hash file bodies (about 25 ms for 10 MiB). |

## Starting point

Today the Worker (`cloud/src/index.ts`) serves `/v1/push`, `/v1/changes` and `/v1/files/<id>` for
one library behind `SYNC_TOKEN`. D1 holds plaintext `docs(id, rev, seq, deleted, fields,
updated_at)` and `counter`, and R2 holds bytes under `files/<id>`. The client (`kanso-sync`) runs
`sync_once`: it pulls after the cursor `sync.last_seq` in `meta`, then pushes every doc whose `rev
!= synced_rev` with `base_rev = synced_rev`. Conflicts resolve by `updatedAt`, with the loser kept
in `revisions`. `MemoryRemote` mirrors the Worker, and `cloud/test/fixtures/conformance.json` (renamed
`conformance-v1.json` in step 1) holds cases both pass.

Branch `claude/vigorous-stonebraker-0ea41a` (commit `b4e2733`) added per-user isolation. It is not
merged: its migration drops the plaintext tables. The builders copy parts of it with `git checkout
b4e2733 -- <path>` and adapt them:

| Reused | Replaced |
|---|---|
| Per-user scoping of every statement: `ACCEPT` with the user ID, primary key `(user, id)`, `UNIQUE (user, seq)`, and a per-user `last_seq` counter. | Migration `0002_users.sql`. The new `0002_accounts.sql` only adds tables. |
| Per-user R2 prefixes, without the owner special case at `files/`. | The `users` and `tokens` tables and admin-minted tokens. Sign-up and sign-in create sessions instead. |
| `newToken`, `hashToken`, `isTokenShape` (`kanso_` + 43 base64url characters, stored as SHA-256 hex). | `SYNC_TOKEN` signing in as the owner. `isLabel` (device names are sealed now). |
| The admin runner `scripts/admin.ts` (`wrangler d1 execute`, SQL quoting, `UsageError`, report tables), `scripts/tsconfig.json`, and the "no admin HTTP route" rule. | The admin commands themselves (new list below). |
| Tests: `isolation.test.ts` (model-based plus non-interference); the helpers `client(token)`, `r2Keys`, `admin()`, `execute()`; the injection properties; the `MIGRATION_DB` pattern. | `migration.test.ts`'s expectations. The new migration leaves legacy rows byte-identical. |

## What the server can and can't see

The server sees:

- **Accounts:** usernames, account IDs, creation times, disabled flags, and an epoch (random, changed only by the admin after a restore).
- **Key material:** each account's KDF parameters and salt, SHA-256 hashes of its auth key and
  recovery auth key, its two wrapped data keys (ciphertext), and a 64-bit key ID (an HKDF output
  of the data key that only says whether two parties hold the same key).
- **Documents:** for each one, its kind (`note`, `book`, `tag` or `file`), an opaque ID that lasts
  for the document's life, and an opaque revision token that changes on every edit. Also whether it
  is deleted, its ciphertext length (the plaintext JSON length plus 41 bytes, ×4/3 for base64), the
  server time of every write, and the order of writes.
- **Activity:** counts per kind and usage totals, which documents are written together, and when the account is active.
- **Files:** each one's opaque ID, its ciphertext size (the file size plus 41 bytes) and upload times.
- **Devices:** per-account device IDs, which can't be linked across accounts, and a hash of each
  device's proof. Sign-in and last-seen
  times, the latter to the hour. How many devices there are, and why each was signed out. Not their
  names: those are sealed.
- **Network:** IP addresses, times and the `kanso-sync/<version>` user agent, in Cloudflare's own
  request handling. Workers invocation logs and traces are pinned off
  (`observability.logs.invocation_logs: false`, `observability.traces.enabled: false`), and the
  Worker logs only route names and error codes. `wrangler tail` (or the dashboard's live logs)
  still shows each request's URL, headers and Cloudflare metadata live to whoever runs it, with
  some values redacted by Cloudflare's heuristics; nothing from it is stored unless they save it. D1 stores IPs and usernames only as keyed
  hashes, inside rate-limit rows kept at most 2 days.
- **Backups:** Cloudflare D1 Time Travel keeps 30 days of history. Deleted accounts' rows and the
  purged legacy plaintext stay restorable by the owner for that window.

The server can't see note titles or bodies; notebook names or hierarchy; tag names or colors; which
notes are in which notebook or carry which tag; statuses or pins; documents' own created and updated
times; links between notes; file names, content types or bytes; device names; local document IDs or
revisions; the password; the recovery key; or the data key.

## Constructions

### Encodings

- Binary values in JSON and headers are unpadded base64url (RFC 4648 §5), except hashes, which are lowercase hex.
- The Worker decodes base64url only for 32-byte proofs (auth keys, recovery auth keys, device
  proofs). It rejects
  input of the wrong length, or non-canonical input (where re-encoding doesn't reproduce it), with
  400 `bad_request`.
- Fixed lengths: 16 bytes → 22 characters, 32 → 43, 73 → 98.
- Integers are JSON numbers. Times are Unix milliseconds.
- `lp(x)` means a 2-byte big-endian byte length of `x`, then the UTF-8 bytes of `x`.
- Format version 1 is the byte `0x01`.
- **Origins.** `origin` is `url::Url::parse(server)?.origin().ascii_serialization()` of the URL the
  client actually connects to, for example `https://kanso-sync.imngonii.workers.dev`: lowercase,
  punycode, default port omitted, path dropped. The server never supplies it.

### Password and KDF

- **Password.** Normalized to Unicode NFC (`unicode-normalization`), then used as UTF-8. Kanso
  requires 12–256 characters (Unicode scalar values) and refuses a password equal to the username,
  ignoring case. Only the client checks this.
- **Salt input.** `salt_input = "kanso/v1/salt" || lp(origin) || lp(username) || salt16`. Here
  `salt16` is 16 random bytes per account from the server's prelogin answer, and `username` is
  normalized. A second server that echoes the first server's salt still gets an auth key bound to
  its own origin, which is useless on the first server. Every password change and recovery replaces
  `salt16`.
- **Master key.** `master = Argon2id(password, salt_input, m = 1,048,576 KiB, t = 3, p = 4, version
  0x13, 32 bytes)`.
- **Timing.** 0.36 s warm and 0.45–0.56 s cold on the owner's M4 Pro, with `argon2 = 0.6` and its
  `parallel` feature (rayon, already in `Cargo.lock`). Without `parallel` it takes about 3.6 times
  as long. Expect about 1–1.5 s on an 8 GB M1.
- **Floor and ceilings.** Each account stores its parameters. The production floor is `m_kib ≥
  1,048,576`, `t ≥ 3`, `1 ≤ p`. The ceilings are `m_kib ≤ 2,097,152`, `t ≤ 8`, `p ≤ 8`, which caps
  a hostile server's demand at about 2 s and 2 GiB on the M4 Pro.
- **Where they come from.** The client takes both from its backend: `Backend::kdf_floor()` and
  `kdf_default()`. `HttpBackend` returns the production constants. `MemoryServer`, the test server,
  returns `m = 64 KiB, t = 1, p = 1`. There is no cargo feature that lowers the floor, so no build
  of the app can. The Worker enforces the production floor and ceilings regardless.

### Keys

All derived keys are 32 bytes unless stated. "HKDF" means HKDF-SHA256; without a stated salt it
uses the default all-zero salt (RFC 5869).

| Key | Derivation | Use |
|---|---|---|
| `master` | Argon2id as above | Input only; zeroized right after HKDF |
| `auth_key` | HKDF(`master`, info `"kanso/v1/auth"`) | Sent at sign-up, sign-in, password change, new recovery key and deletion |
| `kek` | HKDF(`master`, info `"kanso/v1/kek"`) | Wraps the data key; never leaves the Mac |
| `recovery_secret` (16) | Random | Shown as the recovery key; replaced on every use and every password change |
| `recovery_auth_key` | HKDF(ikm `recovery_secret`, salt `"kanso/v1/recovery" \|\| lp(origin) \|\| lp(username)`, info `"kanso/v1/recovery-auth"`) | Sent at sign-up, recovery, new recovery key and password change |
| `recovery_kek` | HKDF(same ikm and salt, info `"kanso/v1/recovery-kek"`) | Wraps the data key |
| `data_key` | Random at sign-up (rekey, deferred, would replace it) | Root of the content keys; kept in the Keychain |
| `doc_key` | HKDF(`data_key`, info `"kanso/v1/doc"`) | Seals documents |
| `file_key` | HKDF(`data_key`, info `"kanso/v1/file"`) | Seals file bytes |
| `id_key` | HKDF(`data_key`, info `"kanso/v1/wire-id"`) | HMAC key for wire IDs |
| `rev_key` | HKDF(`data_key`, info `"kanso/v1/wire-rev"`) | HMAC key for wire revisions |
| `device_name_key` | HKDF(`data_key`, info `"kanso/v1/device-name"`) | Seals device names |
| `key_id` (8) | HKDF(`data_key`, info `"kanso/v1/key-id"`, 8 bytes), as 16 hex digits | Told to the server; the client compares it on sign-in and on every changes page |

Every secret type wraps `Zeroizing` and has a redacted `Debug`.

### Wrapped data key

`wrapped = 0x01 || nonce(24) || XChaCha20-Poly1305(key, nonce, aad, data_key)`. That is 73 bytes,
or 98 base64url characters.

- **AAD.** `"kanso-wrap" || 0x01 || purpose || lp(account_id)`. The purpose byte is `0x50` (`P`) for
  the password wrap and `0x52` (`R`) for the recovery wrap. Each wrap opens only under its own KEK
  and for its own account, so a server can't pass one account's wrap off as another's (security
  re-check 3).
- **Rewrapping.** A password change rewraps the same data key, so notes aren't re-encrypted.
  Recovery unwraps with `recovery_kek`, then wraps under the new password's KEK and under a new
  recovery key.
- **Checks after unwrapping.** A failed unwrap is `KeyMismatch`. The client also compares
  `key_id(data_key)` with the server's `key_id`; a difference is `KeyMismatch` too.

### Recovery key

The recovery key is 16 random bytes (128 bits), shown as 28 Crockford base32 characters in 7 groups
of 4, for example `0K7M-QX9P-3J2H-ZD8W-5TNB-1RGV-E6CA` (layout only; its checksum isn't real).

- **Encoding.** Read the 16 bytes as a big-endian integer `V`. The first 26 characters are `V` in
  base32, most significant digit first. 130 bits hold 128, so the first character is `0`–`7`.
- **Checksum.** The last 2 characters are `V mod 1021` (1021 is prime and below 32²), as two base32
  digits, most significant first. Every single-character substitution, and every adjacent
  transposition within the 26 data characters, changes `V mod 1021`, so the checksum always catches
  them. A script check over 20,000 random keys found no exception. Other typos get through with a
  probability of about 1 in 1021, and are then refused by the server and the unwrap.
- **Alphabet.** `0123456789ABCDEFGHJKMNPQRSTVWXYZ`.
- **Parsing.** Drop spaces and hyphens and uppercase. Read `O` as `0` and `I`/`L` as `1`. Reject
  `U` and every other character. Then require exactly 28 characters, a first character below `8`, a
  check value below 1021, and a matching checksum.
- **Rotation.** Recovery replaces the key: the used key stops working and the new one is shown. "New
  recovery key…" replaces it while signed in, and so does every password change.

### Sealed documents

A document travels as `sealed = base64url(0x01 || nonce(24) || XChaCha20-Poly1305(doc_key, nonce,
aad, plaintext))`. The nonce is 24 random bytes from `getrandom`, new for every seal.

- **Plaintext.** UTF-8 JSON `{"id": <local id>, "rev": <local rev>, "fields": <object>}`.
  Tombstones have `"fields": {}`. Unknown top-level keys are ignored, for forward compatibility.
- **AAD.** `"kanso-doc" || 0x01 || lp(account_id) || lp(wire_id) || lp(wire_rev) || deleted`, where
  `deleted` is the byte `0x00` or `0x01`.
- **Opening** gives one of three results:
  1. *Too new*: the first byte is greater than `0x01`. Pulling stops at this document and the Mac
     says "Update Kanso". It never overwrites such a document.
  2. *Undecryptable*: the first byte is `0x01` but one of these fails: the AEAD; the JSON shape;
     `wire_id(id)` and `wire_rev(id, rev)` equalling the wire values; the local ID's kind equalling
     the wire kind; a tombstone having empty fields.
  3. *Opened.*
- **Size.** Sealed strings are 55 to 1,900,000 characters, so a D1 row stays under 2 MB. The client
  refuses to push a document whose `fields` JSON is over 1,400,000 bytes; it lands in the report's
  `too_large`. The largest real note is 253 KB.

### Opaque IDs and revisions

Local revisions are `{generation}-{MD5 of canonical JSON}`, and API callers may choose local IDs.
Neither goes on the wire:

- `wire_id(id) = kind + ":" + base64url(HMAC-SHA256(id_key, id)[0..16])`. The HMAC input is the
  full local ID including its `kind:` prefix, for example `note:Q2hhbmdlTWVQbGVhc2UxMg` from
  `note:07HmFZwo`.
- `wire_rev(id, rev) = hex(HMAC-SHA256(rev_key, lp(id) || rev)[0..16])`, 32 lowercase hex digits.
- `base_rev` on the wire is `wire_rev(id, synced_rev)`.

### Revision generations only grow (M2)

This is the invariant: every revision the server accepts for a document has a higher generation
than the one it replaces. Local writes already use `Rev::next(current)`. The Inkdrop importer only
writes when the incoming generation is higher (`Store::import_doc`).

Two store rules keep it, in `apply_remote`:

- **When the local version wins a conflict,** it gets a new revision: generation `max(local,
  remote) + 1`, with the same content hash. That revision is pushed with `base_rev =` the remote
  revision.
- **A remote revision is refused as `Stale`** when its generation is at or below the document's
  replay floor, `synced_gen`, and it isn't the revision the Mac holds or last synced. `synced_gen`
  is the highest generation this Mac has seen on this account for the document. A server-reported
  loss and an epoch change never lower it (see "Sync with encryption › What a Mac remembers per
  document"). A replayed older version or tombstone therefore can't revert a Mac that has seen
  something newer, including a note re-created under the same ID. That holds even after the server
  claims it lost the document or rotates its epoch. The one exception is a live version this Mac
  never held, arriving over a local deletion: an edit always beats a deletion, so it wins (rule 4
  in "Sync with encryption › In order" has the trade-off).

The report lists stale documents; the cursor moves past them.

### Sealed files

A file travels as `sealed = 0x01 || nonce(24) || XChaCha20-Poly1305(file_key, nonce, aad, bytes)`,
sent raw.

- **AAD.** `"kanso-file" || 0x01 || lp(account_id) || lp(wire_id)`.
- **Size.** The plaintext limit is 10 MiB (`MAX_ATTACHMENT_SIZE`), so a sealed file is 41 to 10,485,801 bytes.
- **Content type.** It stays inside the sealed file document. R2 objects are always `application/octet-stream`.
- **Matching bytes to a version.** The opened bytes must match the file document's `md5digest`. A
  mismatch counts as missing, and a later ordinary sync fetches them again ("Sync with encryption ›
  Files").

### Sealed device names

The name is the Mac's ComputerName (`scutil --get ComputerName`) with control characters removed,
trimmed, and cut to 64 Unicode scalar values. It is "Mac" when that leaves nothing.

`sealed_name = base64url(0x01 || nonce(24) || XChaCha20-Poly1305(device_name_key, nonce,
"kanso-device" || 0x01 || lp(account_id) || lp(device_id), name))`. That is 55–400 characters.

The server stores and returns it opaquely. A device that has no name yet (just signed in) shows as
"A Mac" until it sets one.

### Device identity (per library)

- **Creating it.** The first account action on a library creates, in its `meta`:
  - `sync.v2.library_id`: `l_` + 32 lowercase hex characters from `getrandom`; it names the Keychain items.
  - `sync.v2.device_secret`: 32 random bytes as hex.
  - `sync.v2.hardware`: `hex(SHA-256("kanso-hw" || IOPlatformUUID))[0..32]`. The UUID comes from
    `ioreg -rd1 -c IOPlatformExpertDevice`. If `ioreg` gives no UUID, the value is empty and the check below
    is skipped.
- **The device ID for an account** is `d_ + hex(HMAC-SHA256(device_secret, account_id)[0..16])`, so the server can't link one Mac's sessions across accounts.
- **The device proof for an account** is `HKDF(ikm device_secret, info "kanso/v1/device-proof" ||
  lp(origin) || lp(account_id))`, 32 bytes, sent as 43 base64url characters as `proof` in every
  `device` object (sign-up, sign-in, recovery). The server keeps only its SHA-256 in the session row
  (`proof_hash`). A sign-in skips `login-user` only when the proof matches the active session row of
  that device, so knowing a device ID (every Mac of the account sees them) isn't enough (security
  re-check 5). A signed-out row, whatever signed it out, exempts nothing: an old copy of the library
  holds the device secret, and the device list doesn't show the row (security review 2, finding 4).
- **A copied library.** When the stored `sync.v2.hardware` differs from this Mac's, the library was
  copied (Migration Assistant, restoring a backup to new hardware). The library ID and device secret
  are regenerated, which leaves the copy signed out: its Keychain item names change. Its binding and
  cursor stay, so signing in resumes as a new device instead of fighting the original Mac over one
  session.
- **Sign-out and unbind keep all three values.**

### Server-side hashes and the pepper

- `auth_hash = hex(SHA-256(auth_key))` and `recovery_hash = hex(SHA-256(recovery_auth_key))`. The
  inputs are 256-bit derived keys, so a plain hash can't be inverted. A pepper would add nothing:
  the wrapped data key in the same row allows the same offline password guessing, and the pepper's
  only effect would be locking everyone out if it were lost. The Worker compares the 32-byte
  digests with `crypto.subtle.timingSafeEqual`. Unknown usernames still cost one hash.
- **Session tokens** are 256-bit random values, stored as SHA-256 hex. **Device proofs** are stored
  the same way, as `proof_hash`.
- **`AUTH_PEPPER`** is a 32-byte base64url secret, used only through HKDF subkeys (WebCrypto `deriveBits`):
  - **Fake prelogin answers.** `fake_key = HKDF(pepper, info "kanso/v1/fake")`. The fake salt is
    `HMAC(fake_key, "salt" || 0x00 || username)[0..16]`. The fake account ID is `a_` +
    `hex(HMAC(fake_key, "account" || 0x00 || username)[0..16])`.
  - **Rate-limit keys.** `limit_key = HKDF(pepper, info "kanso/v1/limit")`. For IP subjects the key
    is `rule + ":" + hex(HMAC(limit_key, rule || 0x00 || subject)[0..16])`. For account subjects it
    is `rule + ":" + account_id`, which isn't secret, so the admin can read it. `login-user` uses the
    account ID of an existing username, and one shared key, `login-user:unknown`, for every name
    without an account.
- **A missing or malformed pepper** answers 503 `misconfigured` on the routes that use it (health,
  prelogin, sign-up, login, recovery). Session routes keep working.

### Rust crates

Workspace dependencies, all on the `digest 0.11` generation already in `Cargo.lock`:

- `argon2 = { version = "0.6", default-features = false, features = ["alloc", "parallel", "zeroize"] }`
- `chacha20poly1305 = { version = "0.11", default-features = false, features = ["alloc", "zeroize"] }`
- `hkdf = "0.13"`, `hmac = "0.13"`, `sha2 = "0.11"`, `zeroize = "1.9"`, `getrandom = "0.4"`, `unicode-normalization = "0.1"`

The existing `base64` (0.22, `URL_SAFE_NO_PAD`) and `url` are reused. There are no custom
primitives.

## Usernames

- **Normalizing,** on every route that takes a username and on the client. All three steps happen
  before any lookup, rate-limit key or fake answer:
  1. Strip leading and trailing U+0009, U+000A, U+000C, U+000D and U+0020. These five only: not JS
     `trim()`, which strips more.
  2. Reject (`invalid_username`) any remaining character outside U+0021–U+007E. This comes before
     lowercasing, because JS `toLowerCase` maps U+212A KELVIN SIGN to `k`.
  3. ASCII-lowercase.
- **Valid.** `^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$`: 3–32 characters, starting and ending with a
  letter or digit. A `CHECK` repeats this in D1.
- **Unique** on the normalized form.
- **Reserved,** at sign-up only (`reserved_username`): `abuse admin administrator anonymous api
  everyone guest help hostmaster kanso me null owner postmaster root security server support sync
  system undefined webmaster www`.
- **Shared fixture.** `cloud/test/fixtures/usernames.json` (owner: the Worker builder) lists
  `{input, normalized, error}` cases. `normalized` is filled for reserved names too, and `error` is
  `invalid_username`, `reserved_username` or null. It includes U+00A0, U+2028, U+000B, U+212A,
  U+FEFF and each of the five trimmed characters. `cloud/src/usernames.ts` and
  `crates/kanso-sync/src/username.rs` both run it.
- **Enumeration.** Sign-up must say `username_taken`, so names can be enumerated there. That route
  is rate-limited per IP and capped per day. Prelogin answers an unknown name with a deterministic
  fake salt, fake account ID and default parameters, and sign-in fails the same way for an unknown
  name as for a wrong password.

## HTTP protocol (v2)

### Conventions

- **Base URL.** The server URL (default `https://kanso-sync.imngonii.workers.dev`), checked by
  `check_endpoint`: https, or http to loopback only. A host ending in a dot is refused, since it would
  give the same server a second origin.
- **Browser requests (M7).** Any request with an `Origin` header gets 403 `forbidden_origin`,
  checked first; the native client never sends one. Requests with a body must have the right
  `Content-Type`, or they get 415 `unsupported_media_type`: `application/json` (with an optional
  `charset=utf-8`) for JSON, and `application/octet-stream` for file uploads.
- **JSON bodies.** Required fields must be present with the right type. Unknown keys are ignored in
  requests and responses. Auth and account request bodies are limited to 8 KiB.
- **Sessions.** Session routes take `Authorization: Bearer kanso_<43 base64url>`. A header of any
  other shape gets 401 without touching D1.
- **Errors** are `{"error": "<code>", "message": "<English, for logs>", ...extras}`. The client maps
  codes to its own words and never shows `message`.
- **Unknown paths.** `/v1/*` and every other unknown path answer 404 `not_found`.
- **Client.** Blocking `ureq`, with `max_redirects(0)` so a redirect can never re-send a proof.
  Response size caps:

  | Response | Cap |
  |---|---|
  | Health, auth and account routes | 64 KiB |
  | Push and changes | 12 MiB |
  | Files | 10,485,801 bytes |

| Status | Code | Extras | When |
|---|---|---|---|
| 400 | `bad_request` | | Malformed JSON, wrong types, bad ID, revision, length, encoding or device ID, sealed under 55 characters, file body under 41 bytes |
| 400 | `invalid_username` | | Fails the rules |
| 400 | `reserved_username` | | Sign-up only |
| 400 | `unsupported_kdf` | | `kdf.alg` isn't `argon2id` or `kdf.version` isn't 19 |
| 400 | `weak_kdf` | | Parameters outside the floor or ceilings |
| 400 | `username_mismatch` | | `account/delete` with a username that isn't the account's |
| 400 | `checksum_mismatch` | | File body doesn't match `x-kanso-sha256` |
| 401 | `unauthorized` | `reason`: `unknown`, `signed_out`, `revoked`, `password_changed`, `recovered`, `evicted`, `expired` or `rekeyed` | Session missing, malformed, unknown, revoked or expired |
| 401 | `invalid_credentials` | | Wrong proof, or an unknown username |
| 403 | `forbidden_origin` | | An `Origin` header |
| 403 | `account_disabled` | | Disabled by the owner. Checked after credentials. Logout still answers 204. |
| 403 | `signup_closed` | `reason`: `closed`, `full` or `busy` | Switched off, `max_accounts` reached, or `signups_per_day` reached |
| 404 | `not_found` | | Unknown route, file, or well-formed but unknown device |
| 405 | `method_not_allowed` | `Allow` header | |
| 409 | `username_taken` | | |
| 409 | `account_id_taken` | | Sign-up with an existing account ID (client retries with a new one) |
| 413 | `too_large` | | More than 200 documents, a push body over 4 MiB, sealed over 1,900,000 characters, a file over 10,485,801 bytes, an auth body over 8 KiB |
| 413 | `quota_exceeded` | `quota`: `docs`, `doc_bytes`, `files`, `file_bytes` or `server`; `used` (usage before this request); `limit` | When several apply, the first in that order |
| 415 | `unsupported_media_type` | | Wrong `Content-Type` |
| 429 | `rate_limited` | `retry_after` (seconds to the end of the window, rounded up), `Retry-After` header, `limit`: the rule name, `binding` or `session-account` (the bindings), or `writes-global` or `uploads-global` | A rate-limit rule or binding, or a server-wide daily cap |
| 500 | `internal` | | |
| 503 | `misconfigured` | | `AUTH_PEPPER` missing or malformed |

### Rate limits

- **First layer, the bindings.** Two Workers rate-limit bindings, which write nothing to D1. Over
  either, the answer is 429 with `retry_after: 60`. If a binding is missing (unit tests), its layer
  is skipped. Like every Workers rate limit they count per Cloudflare location and settle
  eventually, so they are a filter in front of D1, not an exact cap.
  - `AUTH_LIMITER` (namespace `"1001"`, 20 requests per 60 s) is keyed by the caller's IP group and
    covers every `/v2/auth/*` and `/v2/health` request (`limit` `binding`).
  - `SESSION_LIMITER` (namespace `"1002"`, 120 requests per 60 s) is keyed by the account ID and
    covers every session route except logout, right after the session lookup (`limit`
    `session-account`). One signed-in account can't keep the shared database busy, from however
    many IPs it uses (security review 2, finding 2). Logout stays behind `AUTH_LIMITER` only.
- **Second layer, D1.** D1 is the authority, in fixed windows, and counts first (H1):

  ```sql
  INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, ?3)
  ON CONFLICT (key) DO UPDATE SET
    count = CASE WHEN window_start >= excluded.window_start THEN count + excluded.count ELSE excluded.count END,
    window_start = max(window_start, excluded.window_start)
  RETURNING count
  ```

  Here `?2 = floor(now / window) * window`, and `?3` is the amount (1, or the number of documents in
  a push). The upsert runs before credentials are checked or any work is done. If the returned
  count is over the maximum, the request is refused with 429, so parallel requests can't pass
  together. A request stamped with an earlier window than the row's (begun just before the
  boundary, or from a Worker whose clock is a little behind) counts against the newer window
  rather than resetting it, as the `daily` counters do (security review 2, finding 8).
- **IP groups.** IPv4 counts per address and IPv6 per /48, in D1 and in the binding alike. A /48
  is the smallest block a home or small office is commonly given, so per /56 one subscriber could
  rotate through 256 groups (security review 2, finding 9). The value comes from `CF-Connecting-IP`;
  `unknown` stands in when it's absent (local dev).
- **Expiry.** A rule's rows are deleted once their window has ended (by the cron; the counting
  upsert also overwrites a stale window).

| Rule | Subject | Window | Max | Counts |
|---|---|---|---|---|
| `signup-ip` | IP group (/48) | 1 h | 10 | Every sign-up request |
| `login-user` | account ID; one shared row for names without an account | 15 min | 10 | Every sign-in, unless its device ID has an active session row (`revoked_at IS NULL`) for that account and SHA-256 of the request's device proof equals that row's `proof_hash` (M8, security re-check 5, security review 2) |
| `login-ip` | IP group (/48) | 15 min | 30 | Every sign-in |
| `recover-ip` | IP group (/48) | 1 h | 10 | Every `recover/start` and `recover/finish` |
| `password-account` | account ID | 15 min | 5 | Every current-password check (password change, new recovery key, delete) |
| `writes-account` | account ID | 1 day | the account's `quota_writes_per_day`, else `settings.writes_per_day` | Documents in each push, plus 1 per file upload, plus 1 per push refused as malformed or too large (so failed pushes aren't free) |
| `uploads-account` | account ID | 1 day | `settings.upload_bytes_per_day` | Each upload's `Content-Length` (capped at the largest file plus one byte; the largest file when it's missing), counted before the body is read, after `writes-account` passes; a body longer than it declared counts the rest |

There is no per-username recovery rule, so nobody can lock someone else out of recovery (M8). A
sign-up past `signups_per_day` (server-wide per UTC day, counted by a trigger in `daily`; deleting
an account doesn't give one back) gets 403 `signup_closed` with reason `busy`. Two more server-wide
daily caps are counted by triggers in `daily`, not here: stored document writes
(`global_writes_per_day`) and uploaded file bytes (`global_upload_bytes_per_day`). Past either, a
push or upload gets 429 with `limit` `writes-global` or `uploads-global` and `retry_after` up to the
next UTC midnight (security re-check 2), but only for an account above its fair share.

**Fair shares** (security review 2, finding 3). First come, first served, five accounts at their
own upload budgets, or fifteen at their write budgets, could use a cap up by noon and pause every
other Mac until midnight. So each cap has a fair share per account: the cap ÷ `max_accounts`, in
SQLite's integer division (6,000 writes and 100,000,000 upload bytes by default). Past the cap, the
trigger refuses a write only when the account's own count today is above its fair share:
`writes-account` for documents, `uploads-account` for upload bytes, both counted by the Worker
before the batch, this request included. An account at or below its fair share is never refused
by a server-wide cap. Fail closed: a missing setting, `max_accounts` 0, or no row for today in
`limits` counts as above the fair share. The price is the bound: once the cap is reached, every
account may still use its fair share, so a day's total stays under about twice the cap (the cap,
plus `max_accounts` fair shares, plus the request that crossed it, plus a fair share for each
account created that day).

Login keys `login-user` by account, so attempts on many unknown names share one row. That tells a
prober whether a name exists, which sign-up's `username_taken` already does.

**Floods.** A `workers.dev` address can't have WAF rules. Surviving a real flood needs a custom
domain with a Cloudflare WAF rate-limiting rule in front of the Worker. Expensive requests from a
valid session, which an IP-keyed WAF rule can't stop, are bounded by `SESSION_LIMITER` per account. Accounts are bound to their
server's origin (M5), so the owner must choose the final URL before the first account is created.

### Routes

| Method | Path | Session | Purpose |
|---|---|---|---|
| GET | `/v2/health` | no | Liveness, protocol version, whether sign-ups are open |
| POST | `/v2/auth/prelogin` | no | KDF parameters, salt and account ID for a username |
| POST | `/v2/auth/signup` | no | Create an account and this device's session |
| POST | `/v2/auth/login` | no | Sign in a device |
| POST | `/v2/auth/recover/start` | no | Prove the recovery key; get the recovery-wrapped data key |
| POST | `/v2/auth/recover/finish` | no | Set a new password and a new recovery key |
| POST | `/v2/auth/logout` | yes | Sign out this device |
| GET | `/v2/account` | yes | Account, usage and quotas |
| POST | `/v2/account/password` | yes | Change the password |
| POST | `/v2/account/recovery-key` | yes | Replace the recovery key |
| POST | `/v2/account/delete` | yes | Delete the account and its server data |
| PUT | `/v2/account/device` | yes | Set this device's sealed name |
| GET | `/v2/account/devices` | yes | List active devices |
| DELETE | `/v2/account/devices/<device_id>` | yes | Sign out one device |
| POST | `/v2/push` | yes | Write sealed documents |
| GET | `/v2/changes?since=&limit=` | yes | The account's change feed |
| PUT | `/v2/files/<wire_id>` | yes | Store sealed file bytes |
| GET | `/v2/files/<wire_id>` | yes | Fetch sealed file bytes |
| POST | `/v2/account/rekey` | yes | Deferred: specified under "Deferred", not built in v1. It answers 404 until then. |

Shared shapes:

```json
"kdf":    {"alg": "argon2id", "version": 19, "m_kib": 1048576, "t": 3, "p": 4}
"device": {"id": "d_<32 hex>", "sealed_name": "<55–400 base64url>" | null, "proof": "<43>"}
```

#### GET /v2/health

200 `{"ok": true, "protocol": 2, "signup": "open" | "closed" | "full"}`. A setting of
`only:<name>` reports `closed`; `full` means the switch is open and `max_accounts` is reached. The
daily cap isn't reported. 503 `misconfigured` without a pepper.

#### POST /v2/auth/prelogin

Request `{"username"}`. 200 `{"kdf", "salt": "<22>", "account_id": "a_<32 hex>"}`. Unknown names
get the fake salt, the fake account ID and the default parameters. 400 `invalid_username`. Only the
binding limits it: it reads, and writes nothing.

#### POST /v2/auth/signup

```json
{"username", "account_id": "a_<32 hex, client-generated>", "kdf", "salt": "<22, client-generated>",
 "auth_key": "<43>", "wrapped_key": "<98>", "recovery_auth_key": "<43>", "recovery_wrapped_key": "<98>",
 "key_id": "<16 hex>", "device": {...}}
```

201 `{"account_id", "username", "token": "kanso_<43>", "epoch": "<16 hex>"}`.

Steps:

1. Count `signup-ip`.
2. Validate the body.
3. If an account with this username exists and this is the same request that created it (the same
   account ID, KDF parameters, salt, wrapped keys and key ID, its `auth_hash` equal to SHA-256 of
   `auth_key` and its `recovery_hash` to SHA-256 of `recovery_auth_key`, both hashes always
   compared), this sign-up already happened and its answer was lost: upsert this device's session
   and answer 201 with that account's ID, username and epoch and the new token, whatever the
   sign-up settings say. It doesn't count towards `signups_per_day`. A disabled account gets 403
   `account_disabled`. The auth key alone isn't enough: that would make sign-up a password oracle
   outside the sign-in limits.
4. Otherwise insert the account and the session in one batch. Triggers refuse it when sign-ups are
   closed, the server is full, or the day's limit is reached (403 `signup_closed` with `closed`,
   `full` or `busy`), before any `UNIQUE` check. `UNIQUE` failures give 409 `account_id_taken`
   (checked first) or `username_taken`. The Worker generates the epoch.

Retrying (security re-check 7): after a lost response the client sends the same request again (the
same account ID, salt and keys) and gets 201 for the account it created; it checks that the
answer's `account_id` is its own. A 409 `username_taken` means someone else holds the name.

#### POST /v2/auth/login

Request `{"username", "auth_key", "device": {"id", "proof"}}` (no name yet: the client doesn't hold
the data key). 200:

```json
{"account_id", "username", "token", "wrapped_key", "key_id", "epoch",
 "evicted_device": null | {"id", "sealed_name"}}
```

Steps:

1. Count `login-ip`, and `login-user` unless the device is known: its ID has a session row for the
   account that is active (`revoked_at IS NULL`) and whose `proof_hash` is SHA-256 of `device.proof`.
2. Check the credentials: 401 `invalid_credentials` if wrong.
3. Check that the account is enabled: 403 if disabled.
4. One batch:
   1. If 10 other sessions are active, revoke the one with the oldest `last_seen_at` (`evicted`).
   2. Upsert this device's session as active, with a new token hash, the proof's hash, and
      `sealed_name` kept from its earlier row, if it has one.
   3. Delete the account's revoked rows beyond the 90 most recently revoked, so sign-ins can't
      grow `sessions` without bound.

Because it's one batch, concurrent sign-ins can't exceed 10 active devices. After unwrapping, the
client sends `PUT /v2/account/device` with its sealed name.

#### POST /v2/auth/recover/start

Request `{"username", "recovery_auth_key"}`. 200 `{"account_id", "recovery_wrapped_key",
"key_id"}`. Counts `recover-ip`. 401 `invalid_credentials`; 403 if disabled (after the proof).

#### POST /v2/auth/recover/finish

```json
{"username", "recovery_auth_key", "kdf", "salt", "auth_key", "wrapped_key",
 "recovery_auth_key_new", "recovery_wrapped_key_new", "device": {...}}
```

200 `{"account_id", "username", "token", "epoch"}`.

Counts `recover-ip` and checks `recovery_hash`. One batch then:

1. Replaces the KDF parameters, salt, `auth_hash` and `wrapped_key`.
2. Replaces `recovery_hash` and `recovery_wrapped_key` with the new ones, so the used recovery key
   stops working (H3).
3. Revokes every session (`recovered`).
4. Upserts this device's session.

Retrying: if the old recovery hash no longer matches, but the hash of `recovery_auth_key_new` equals
the stored recovery hash and the hash of `auth_key` equals `auth_hash`, the change is already in
place, and the Worker only upserts the session.

Credential updates are compare-and-swap: here, in password changes and in new recovery keys, the
`UPDATE accounts` carries `WHERE` the hash the Worker just checked (`recovery_hash` here,
`auth_hash` in the others), and counts as done only if it changed the row. Of two concurrent
changes one wins; the other gets 401 `invalid_credentials`, or the retry answer when it asked for
the same values.

#### POST /v2/auth/logout

Revokes this session (`signed_out`) and answers 204, for disabled accounts too. The client treats
401 as done.

#### GET /v2/account

200:

```json
{"account_id", "username", "created_at",
 "usage": {"docs", "doc_bytes", "files", "file_bytes", "writes_today"},
 "quota": {"docs", "doc_bytes", "files", "file_bytes", "writes_per_day"}}
```

#### POST /v2/account/password

Request `{"current_auth_key", "kdf", "salt", "auth_key", "wrapped_key", "recovery_auth_key",
"recovery_wrapped_key"}`. 200 `{"signed_out_devices": n}`.

The two recovery fields are a new recovery key, in the formats `POST /v2/account/recovery-key`
takes and checked the same way (400 `bad_request` when missing or malformed). The client makes a
fresh recovery secret, derives them as it does for "New recovery key…", and shows the new key once
the change succeeds (security review 2, finding 1).

Counts `password-account`. 401 `invalid_credentials` on a wrong current key. On success one
compare-and-swap `UPDATE accounts … WHERE auth_hash = <the checked hash>` replaces the password
material, `recovery_hash` (SHA-256 of `recovery_auth_key`) and `recovery_wrapped_key` together. A
password change therefore always ends every older recovery key, including one set by someone who
once knew the password. Every other active session is revoked (`password_changed`).

Retrying: if the current key fails but `auth_hash`, `wrapped_key`, `recovery_hash` and
`recovery_wrapped_key` already equal the request's new values, the answer is 200 with 0.

#### POST /v2/account/recovery-key

Request `{"current_auth_key", "recovery_auth_key", "recovery_wrapped_key"}`. 200 `{}`. Counts
`password-account`, then replaces the recovery hash and wrap. Sessions stay.

Retrying: if the current key is right and the stored values already equal the new ones, the answer
is 200.

#### POST /v2/account/delete

Request `{"auth_key", "username"}`. 200 `{"deleted": true}`.

Counts `password-account` and checks the key. Then `username_mismatch` unless the username
normalizes to the account's. One batch then:

1. Delete the account's rows from `account_docs`.
2. Delete its rows from `account_files`.
3. Delete its rows from `sessions`.
4. Delete its account-keyed `limits` rows.
5. Delete the `accounts` row.
6. Insert `purges('accounts/<id>/')`.

Then up to 20 purge rounds inline. Retrying: a retry after a lost response gets 401 `unknown` (the
session row is gone), which the client treats as success. Any other 401 reason means the session
ended before the deletion, so the client reports that instead.

#### PUT /v2/account/device, GET /v2/account/devices, DELETE /v2/account/devices/<device_id>

- **PUT** `{"sealed_name"}` sets the current session's name and answers 204.
- **GET** `{"devices": [{"id", "sealed_name", "created_at", "last_seen_at", "current"}]}` lists
  active sessions only, most recently seen first.
- **DELETE** runs `UPDATE sessions SET revoked_at = coalesce(revoked_at, ?3), revoked_reason =
  'revoked' WHERE account_id = ?1 AND device_id = ?2`. A row already signed out, for any reason, is
  marked `revoked` too and keeps its time, so the owner can always remove a Mac they no longer
  trust. It answers 204, or 404 when the account has no row for that device (an unknown ID, or
  another account's device), or 400 for a malformed ID. Deleting the current device is a logout
  (reason `signed_out`).

#### POST /v2/push

Request `{"docs": [{"id", "rev", "deleted", "sealed", "base_rev"}]}`. All five fields are required:
`deleted` is a boolean, and `base_rev` is a string or null. Patterns:

- `id`: `^(note|book|tag|file):[A-Za-z0-9_-]{22}$`
- `rev`, `base_rev`: `^[0-9a-f]{32}$`
- `sealed`: 55–1,900,000 characters of base64url. A `CHECK` enforces the alphabet, so the Worker
  doesn't scan it.

200 `{"results": [...]}`, one per document, in order:

| Result | Meaning |
|---|---|
| `{"id", "status": "ok", "seq"}` | Stored under `seq`. |
| `{"id", "status": "conflict", "current": {"id", "rev", "deleted", "sealed"}}` | `base_rev` isn't the current revision; here is the server's version. |
| `{"id", "status": "conflict", "current": null}` | The client sent a `base_rev`, but the server has no such document: it lost it. |
| `{"id", "status": "behind"}` | A conflict whose current version didn't fit the response's 8 MiB budget of sealed text. The client pulls before retrying. |

Semantics are v1's, scoped to the account: a document is accepted iff its `base_rev` is the
account's current revision of it, or the account has none and `base_rev` is null. Documents apply
in order. The client sends at most 200 documents and 4 MiB of body per push. Steps:

1. Validate the body (400, or 413 for size). A refused body counts 1 against `writes-account`
   first, and the answer is 429 instead if that is over the day's budget. An empty `docs` answers
   200 `{"results": []}` and counts nothing. Otherwise count `writes-account` by the number of
   documents. Over the day's budget → 429. The sealed alphabet is checked only by the `CHECK`,
   after counting.
2. One batch, three statements per document, as in v1 and b4e2733, with `ROW_COST = 512`:

```sql
-- ?1 account, ?2 wire id, ?3 base_rev, ?4 rev, ?5 deleted (0/1), ?6 sealed, ?7 length(sealed), ?8 now
-- Stored lengths are read with octet_length(), from the record header; length() on TEXT reads
-- the whole value. Sealed text is base64url, so bytes equal characters.
ACCEPT  = ((?3 IS NULL AND NOT EXISTS (SELECT 1 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2))
           OR EXISTS (SELECT 1 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2 AND d.rev = ?3))
BUMP    = UPDATE accounts SET
            last_seq = last_seq + 1,
            docs = docs + (1 - ?5)
                 - coalesce((SELECT 1 - d.deleted FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2), 0),
            doc_bytes = doc_bytes + ?7 + 512
                 - coalesce((SELECT octet_length(d.sealed) + 512 FROM account_docs d WHERE d.account_id = ?1 AND d.id = ?2), 0)
          WHERE accounts.id = ?1 AND ACCEPT RETURNING last_seq AS value
CURRENT = SELECT id, rev, deleted, sealed FROM account_docs WHERE account_id = ?1 AND id = ?2 AND NOT ACCEPT
WRITE   = INSERT INTO account_docs (account_id, id, rev, seq, deleted, sealed, updated_at)
            SELECT ?1, ?2, ?4, last_seq, ?5, ?6, ?8 FROM accounts WHERE accounts.id = ?1 AND ACCEPT
          ON CONFLICT (account_id, id) DO UPDATE SET rev = excluded.rev, seq = excluded.seq,
            deleted = excluded.deleted, sealed = excluded.sealed, updated_at = excluded.updated_at
```

   The quota triggers on `accounts` refuse any `BUMP` that grows usage past a quota or cap, and the
   whole batch rolls back. The Worker recognises the refusal by the `quota_exceeded:<kind>`
   (or, at sign-up, `signup_closed:<reason>`) text in D1's error message. It then reads the
   account's usage and answers 413
   `quota_exceeded` with `used` and `limit`. Shrinking writes always pass, so an account over quota
   can still delete. There is no check-then-act anywhere (H1). Every stored write (new or updated)
   also counts towards `global_writes_per_day`; past it, for an account above its fair share (see
   "Rate limits"), a trigger raises `rate_limited:writes-global`, the batch rolls back, and the
   answer is 429 (security re-check 2).
3. **File tombstones.** For accepted file tombstones, delete their R2 objects (one call). Then one
   batch: subtract their `account_files` sizes (plus 1,024 each) and count from the account, and
   delete those rows.

Retrying: a lost response is recovered through the feed. Right before each push the client records
each document's `sent_rev` in a guarded write. The next sync pulls the document back. If the echo is
the Mac's current revision, the store marks it synced. If the note was edited since, the echo equals
`sent_rev`: it becomes `synced_rev`, the newer edit stays pending on top of it, and nothing counts as a
conflict, whatever `updatedAt` says.

#### GET /v2/changes?since=&limit=

200:

```json
{"docs": [{"id", "rev", "deleted", "sealed", "seq"}], "last_seq": n, "more": bool,
 "head_seq": n, "epoch": "<16 hex>", "key_id": "<16 hex>"}
```

- **Parameters.** `limit` defaults to 100 and must be 1–500; `since` defaults to 0 and must not be
  negative. Anything else, a non-integer included, is 400.
- **Two reads keep the page small.** First the sequence numbers and lengths, as
  `octet_length(sealed)`, which SQLite answers without reading the text (`length()` would read
  every candidate row in full, overflow pages included; security review 2, finding 2). Then the
  rows that fit in 8 MiB of sealed text, always at least one.
- **`more`** is true when that prefix was cut, or when the first read returned `limit` rows.
- **`last_seq`** is the last document's seq, or `since` when the page is empty.
- **`head_seq`** is the account's `last_seq`.
- **Splicing.** Each string goes into the response through `JSON.stringify`.

#### PUT and GET /v2/files/<wire_id>

PUT takes the sealed bytes with `Content-Type: application/octet-stream`, `Content-Length` and
`x-kanso-sha256: <64 lowercase hex>`. The client always sends the hash; anything but 64 lowercase
hex is 400. Steps:

1. Check the path's wire ID and the headers (400 or 415). Then count `writes-account` by 1.
2. Read the body: under 41 bytes is 400; over 10,485,801 bytes is 413.
3. Hash it with `crypto.subtle.digest("SHA-256")` and compare: 400 `checksum_mismatch` if different.
4. One batch:
   1. `SELECT size` of the row this replaces.
   2. Update `accounts`: `files` grows by `NOT EXISTS(row)`, and `file_bytes` by `size + 1024 −
      coalesce(old size + 1024, 0)`, computed in SQL. The statement must affect the account row;
      if it doesn't, the account is gone, and the answer is 401.
   3. Upsert `account_files` with `uploaded_at` = now. A trigger counts the size towards
      `global_upload_bytes_per_day`, replacements included, and refuses past it, for an account
      above its fair share, with `rate_limited:uploads-global` (429; security re-check 2).

   Triggers refuse growth past quotas with 413.
5. `FILES.put(key, body, {sha256})`. If R2 fails, a compensating batch restores the old row and
   counters, if the row still holds this upload's size, and the answer is 500. Restoring a larger
   old size sets `recounted_at`, so the quota triggers let the correction through.
6. 200 `{"ok": true, "size": n}`.

Two concurrent uploads of the same ID can leave the counter off by their size difference. `recount`
repairs it. The engine never uploads one file twice at once.

GET answers 200 with the bytes (`application/octet-stream`, `Content-Length`) only when the account
has an `account_files` row for the ID, and 404 otherwise, even if R2 still holds an object (security
re-check 8). The cron reconciles the two (below).

## Storage

### Migration `cloud/migrations/0002_accounts.sql`

This migration only adds things. It never reads or changes `docs` or `counter`. Quotas, caps and
sign-up rules live in triggers, so D1 enforces them atomically. Wrangler's statement splitter tracks
`CREATE TRIGGER … END` the way SQLite does (checked in `wrangler 4.151.1`, `splitSqlQuery`), and
step 1 applies this file with both `wrangler d1 migrations apply --local` and vitest's
`applyD1Migrations`. D1 enforces foreign keys, so deletions run children first.

```sql
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
-- `writes_per_day` setting (the Worker reads it; no trigger does). The quota triggers exempt
-- an update that changes `recounted_at`, which only two corrections do: the admin recount, and
-- an upload's compensation when R2 fails and the row's old, larger size comes back (sync.ts).
-- The cron's size reconciliation doesn't, so a growing correction meets the quotas.
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
```

Before applying it remotely, the deployed database must have only `0001_init` applied. b4e2733's
`0002_users` must never have run there.

### Sessions in the Worker

Every session route authenticates with one read:

```sql
SELECT s.device_id, s.last_seen_at, s.revoked_reason, a.id, a.username, a.disabled_at
FROM sessions s JOIN accounts a ON a.id = s.account_id WHERE s.token_hash = ?1
```

- No row: 401 `unknown`.
- A revoked row: 401 with its reason.
- `last_seen_at` more than 180 days ago: revoke it (`expired`), then 401.
- Otherwise `last_seen_at` is updated when it is more than an hour old.

Auth and rate-limit reads always go to the primary. The Worker never uses D1's Sessions API or read
replicas (L8).

### Quotas and capacity

These are the D1 settings defaults, sized for Workers Paid. D1 there allows 10 GB per database; the
arithmetic uses 10,000,000,000 bytes. Paid includes about 50 million rows written a month, and R2
Paid has no size cap. Every trigger reads its setting with `coalesce(…, 0)` (`'closed'` for
`signup`), and rows of `settings` and `daily` can't be deleted or renamed, so a lost setting
refuses rather than allows (security re-check 1).

| Setting | Default | Meaning |
|---|---|---|
| `max_accounts` | 50 | |
| `signups_per_day` | 5 | Server-wide, per UTC day; deleting an account doesn't give one back |
| `quota_docs` | 50,000 | Live documents of every kind; deleting frees space |
| `quota_doc_bytes` | 100 MiB (104,857,600) | `length(sealed) + 512` per row, tombstones included. Real cost. |
| `quota_files` | 10,000 | Stored file objects |
| `quota_file_bytes` | 500 MiB (524,288,000) | `size + 1,024` per file |
| `writes_per_day` | 20,000 | Pushed documents plus file uploads (and refused push bodies), per account, unless the account's `quota_writes_per_day` says otherwise (`quota <user> --writes-per-day`). A fifteenth of `global_writes_per_day`, so no one account can spend the server's day. A library that needs more in one day, like the owner's first upload or a re-merge of a large library, pauses at the cap and finishes after UTC midnight; the owner raises their own account instead. |
| `upload_bytes_per_day` | 1 GiB (1,073,741,824) | Uploaded file bytes per account per UTC day, refused uploads included. Twice the default file bytes quota, under a fifth of `global_upload_bytes_per_day`. |
| `global_doc_bytes` | 6,000,000,000 | Sum over all accounts |
| `global_file_bytes` | 50,000,000,000 | Sum over all accounts |
| `global_writes_per_day` | 300,000 | Stored document writes (new and updated) per UTC day, all accounts together. Past it, only accounts above their fair share (÷ `max_accounts`: 6,000 writes) are refused, so a day's total stays under about 600,000. |
| `global_upload_bytes_per_day` | 5,000,000,000 | File bytes uploaded per UTC day, all accounts, replacements included. Past it, only accounts above their fair share (÷ `max_accounts`: 100,000,000 bytes) are refused, so a day's total stays under about 10,000,000,000. |

**The 512-byte row cost** covers the row's other columns and its entries in both indexes. A
tombstone measures about 300 B with indexes. `quota.test.ts` pins the figure: it inserts 10,000
random rows into a fresh database and checks that `PRAGMA page_count × page_size` grew by no more
than `Σ(length(sealed) + 512)`.

**D1 budget** (10,000 MB):

| Item | Arithmetic | Bytes |
|---|---|---|
| Documents | global cap; 50 × 104,857,600 = 5,242,880,000 fits inside, leaving 757,120,000 for owner overrides | 6,000,000,000 |
| `account_files` rows | 50 × 10,000 × 200 B | 100,000,000 |
| Sessions | 50 accounts × at most 100 rows (10 active plus 90 days of revoked) × 400 B, rounded up | 10,000,000 |
| `limits` | the binding admits at most 20 auth requests per minute per IP group; rows go when their window ends; budgeted | 200,000,000 |
| Legacy `docs`/`counter` until the purge | 823 notes; `legacy-status` prints the real size | 100,000,000 |
| Total | | 6,410,000,000 |

That leaves 3.59 GB, or 36%, of headroom. The bytes quota is the real limit on documents: 50,000
live documents only reach 100 MiB at an average of about 1,585 bytes sealed (104,857,600 ÷ 50,000 − 512).

**R2:** 50 × 524,288,000 = 26,214,400,000 B of charged file bytes. That is under the 50 GB global
cap, which only overrides can approach, and costs at most 50 GB × $0.015 = $0.75 a month.

**Rows written:** an accepted document writes about 5 rows (the account row, the document row, its
two index entries and the `daily` counter). One account at the daily maximum writes 20,000 × 5 =
100,000 rows a day, which is 3,100,000 in a 31-day month, inside the included 50 million. It takes
seventeen accounts at the maximum all month to exceed it. With fair shares, `global_writes_per_day`
bounds the whole server at about twice 300,000 × 5, 3,000,000 rows a day, about 93 million in a
31-day month in the worst case (every account at its fair share after the heavy ones have used
the cap up, every day). Beyond the included rows it costs $1 per million, so that worst case is
about $43 a month; and the `accounts` command shows `writes_today` for each account. The owner's
first upload is about 909 documents, roughly 3,600 rows.

**Requests:** 50 accounts × 3 Macs × about 400 requests a day is about 1.8 million a month, against
10 million included.

**Admin overcommit rule:** `max-accounts`, `quota` and `default-quota` refuse, unless given
`--force`, any change after which `Σ effective quotas of existing accounts + (max_accounts −
accounts) × default` exceeds a global cap. The daily budgets have a per-account rule:
`writes-per-day`, `upload-bytes-per-day`, `global-writes-per-day`, `global-upload-bytes-per-day`
and `quota --writes-per-day` refuse, unless given `--force`, any change after which one account's
daily budget (the default, an override, or the upload budget) is over its server-wide cap. Each,
and `max-accounts`, prints each cap's fair share (cap ÷ `max_accounts`), how many accounts at their
whole budget reach the cap (after which only accounts above their fair share wait for midnight),
and the day's bound of about twice the cap. `accounts` shows the fair shares beside today's
counts.

### R2 layout, purges and the cron

- Sealed files: `accounts/<account_id>/<wire_file_id>`. The legacy plaintext sits at `files/<id>` until its purge.
- **A purge round** is `list({prefix, limit: 1000})` then `delete(keys)`. A purge row is deleted
  only when a list returns empty and the row is more than an hour old, so an upload still in flight
  can't leave an orphan (L6). Account deletion runs up to 20 rounds inline.
- **The cron** (`*/15 * * * *`, `scheduled()`) does five things:
  - runs up to 50 purge rounds;
  - deletes `limits` rows whose window has ended;
  - reconciles R2 with `account_files` (security re-check 8): it lists a bounded number of
    accounts' prefixes per run, taking accounts in turn, deletes objects that have no row and were
    uploaded more than an hour ago, and sets a row's `size` to its object's size where they differ,
    adjusting `file_bytes` in the same batch. It doesn't set `recounted_at`: a correction that grows
    `file_bytes` meets the quota triggers like an upload, because two racing uploads of one ID can
    leave a small row over a large object. If a quota refuses it, the row goes (refunded) and then
    the object;
  - revokes active sessions unseen for 180 days (`expired`);
  - deletes revoked sessions more than 90 days old.

## Worker configuration

Changes to `cloud/wrangler.jsonc`:

```jsonc
"observability": {
  "enabled": true,
  "logs": { "enabled": true, "invocation_logs": false },
  "traces": { "enabled": false }
},
"triggers": { "crons": ["*/15 * * * *"] },
"ratelimits": [
  { "name": "AUTH_LIMITER", "namespace_id": "1001", "simple": { "limit": 20, "period": 60 } },
  { "name": "SESSION_LIMITER", "namespace_id": "1002", "simple": { "limit": 120, "period": 60 } }
]
// AUTH_PEPPER (required): wrangler secret put AUTH_PEPPER. SYNC_TOKEN is no longer read.
```

The Worker's environment type is `Env { DB: D1Database; FILES: R2Bucket; AUTH_PEPPER?: string;
AUTH_LIMITER?: RateLimit; SESSION_LIMITER?: RateLimit }`. Commands run in `cloud/`, and the deploy token is only referenced by
name:

```bash
export CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN"
pepper=$(openssl rand 32 | base64 | tr '+/' '-_' | tr -d '=\n')
printf %s "$pepper" | npx wrangler secret put AUTH_PEPPER
# Optional backup in a password manager: losing it only changes fake prelogin answers and
# resets rate-limit counters. Then:
unset pepper
```

Tests and local dev use a fixed test pepper: in `vitest.config.ts` bindings, and as `--var
AUTH_PEPPER:<value>` on `wrangler dev`. `.dev.vars` and `cloud/.wrangler/` go in `.gitignore`.

## Client: kanso-sync

### Modules

| Module | Contents |
|---|---|
| `crypto.rs` | `KdfParams` (`FLOOR`, `DEFAULT`, `TEST`, `CEILING`); derivation; HKDF; wrap and unwrap; `seal_doc`/`open_doc` (→ opened, undecryptable or too new); `seal_file`/`open_file`; `seal_name`/`open_name`; `wire_id`, `wire_rev`, `key_id`; `RecoveryKey`, `Password`, zeroizing key types (`AuthKey`, `Kek`, `DataKey`, `DeviceProof`). Pure. |
| `username.rs` | `normalize_username`, reserved list. Pure. |
| `wire.rs` | `WireDoc`, `WirePushResult` (`Ok`, `Conflict`, `Lost`, `Behind`), `WirePage` (with `more`, `head_seq`, `epoch`, `key_id`), patterns, limits, the `WireRemote` trait, `ApiError` and its codes. |
| `remote.rs` | The engine's plaintext `Remote` trait. `PushResult` gains `Lost`, `Behind` and `Unreadable { newer_format }`. `ChangesPage` gains `more`, `head_seq`, `epoch: Option`, `key_id: Option`, `skipped`, `repairable` and `too_new`. `put_file(id, bytes)` and `get_file(id) -> Option<Vec<u8>>`. |
| `sealed.rs` | `Sealed<W: WireRemote>`: seals outgoing documents and opens incoming ones, maps local IDs and revisions to wire ones and back, and keeps the repair bases for documents that were undecryptable. `PlainView`, the decrypted test view (below), lives here. |
| `api.rs` | The `Api` trait (auth and account routes) and `Backend: Send + Sync` (`api(server)`, `remote(session)`, `kdf_floor()`, `kdf_default()`). |
| `http.rs` | `HttpBackend`: every v2 route over blocking `ureq`, with no redirects and the response caps. |
| `memory.rs` | `MemoryServer`: implements `Backend` with the Worker's sync, auth, session, quota and limit rules, run against the shared fixtures. Its auth hashes are SHA-256, as in the Worker. |
| `account.rs` | The public account functions, `Session`, `Step`. |
| `binding.rs` | Device identity (ID and proof), `meta_keys`, the guard pairs, `local_library`, `plan`, `bind`, `unbind`. |
| `keychain.rs` | Replaces `credentials.rs`: `SecretStore`, `Keychain`, `MemorySecrets`, the items below. |
| `engine.rs` | Pull and push as now, with cancellation, the guard, epoch and head checks, re-merge, repair, stale handling, quota and pause handling, and file MD5 binding. |
| `verify.rs` | `verify` (with an optional legacy manifest) and `estimate`. |
| `bin/kanso-sync.rs` | `status`, `sync`, `verify`, `estimate`, `unbind`. |

### Public API the app calls

```rust
pub const DEFAULT_SERVER: &str = "https://kanso-sync.imngonii.workers.dev";

pub trait Backend: Send + Sync { /* api, remote, kdf_floor, kdf_default, retry_pause (1 s, then 4 s) */ }
pub struct HttpBackend;                 // the app
pub struct MemoryServer { /* … */ }     // tests and snapshots only; never constructed by main.rs

pub struct DeviceIdentity { pub library_id: String, pub name: String, /* device secret, private */ }
impl DeviceIdentity {
    pub fn device_id(&self, account_id: &str) -> String;
    pub fn device_proof(&self, origin: &str, account_id: &str) -> DeviceProof;
}
pub enum Step { StoppingSync, DerivingKeys, Contacting, Saving }
pub fn origin(server: &str) -> Result<String, EndpointError>;   // the origin the salt input binds
pub struct Password(/* NFC, Zeroizing */);
impl Password { pub fn new(text: &str, username: &str) -> Result<Password, PasswordError>; }
pub struct RecoveryKey(/* 16 bytes, Zeroizing */);
impl RecoveryKey { pub fn parse(text: &str) -> Result<RecoveryKey, RecoveryKeyError>; pub fn display(&self) -> String; }
pub fn normalize_username(input: &str) -> Result<String, UsernameError>;

/// A signed-in library: Send + Sync, shared as Arc<Session>. The token and data key are
/// private and zeroized on drop. `server` and `origin` come from the Keychain item.
pub struct Session { pub server: String, pub origin: String, pub account_id: String,
                     pub username: String, pub device_id: String, /* … */ }
pub struct DeviceInfo { pub id: String, pub name: Option<String>, pub created_at: i64, pub last_seen_at: i64, pub current: bool }
pub struct Usage { pub docs: u64, pub doc_bytes: u64, pub files: u64, pub file_bytes: u64, pub writes_today: u64, pub quota: Quota }
pub enum SignupState { Open, Closed, Full }
pub struct SignedUp { pub session: Session, pub recovery_key: RecoveryKey }
pub struct SignedIn { pub session: Session, pub evicted: Option<DeviceInfo> }
pub struct Recovered { pub session: Session, pub recovery_key: RecoveryKey }   // the new key
pub struct PasswordChanged { pub signed_out_devices: usize, pub recovery_key: RecoveryKey }
/// A call that sets a new recovery key failed. `recovery_key` is the unsaved key left in
/// `sync-recovery-key:<library>` whenever the server may hold it: show it, never drop it.
pub struct RotationError { pub error: AccountError, pub recovery_key: Option<Box<PendingRecoveryKey>> }

pub fn server_info(b: &dyn Backend, server: &str) -> Result<SignupState, AccountError>;
pub fn sign_up(b: &dyn Backend, secrets: &dyn SecretStore, server: &str, username: &str, password: &Password, device: &DeviceIdentity, progress: &dyn Fn(Step)) -> Result<SignedUp, RotationError>;
pub fn sign_in(b: &dyn Backend, server: &str, username: &str, password: &Password, device: &DeviceIdentity, progress: &dyn Fn(Step)) -> Result<SignedIn, AccountError>;
pub fn recover(b: &dyn Backend, secrets: &dyn SecretStore, server: &str, username: &str, key: &RecoveryKey, new_password: &Password, device: &DeviceIdentity, progress: &dyn Fn(Step)) -> Result<Recovered, RotationError>;
pub fn sign_out(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session) -> Result<(), AccountError>; // always tries the logout; queues it on failure
pub fn discard_session(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session) -> Result<(), AccountError>; // logout only: the library's Keychain items stay
pub fn flush_pending_logouts(b: &dyn Backend, secrets: &dyn SecretStore, library_id: &str);
pub fn change_password(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session, current: &Password, new: &Password, progress: &dyn Fn(Step)) -> Result<PasswordChanged, RotationError>;
pub fn new_recovery_key(b: &dyn Backend, secrets: &dyn SecretStore, session: &Session, current: &Password, progress: &dyn Fn(Step)) -> Result<RecoveryKey, RotationError>;
pub fn delete_account(b: &dyn Backend, session: &Session, password: &Password, typed_username: &str, progress: &dyn Fn(Step)) -> Result<(), AccountError>;
pub fn devices(b: &dyn Backend, session: &Session) -> Result<Vec<DeviceInfo>, AccountError>;
pub fn revoke_device(b: &dyn Backend, session: &Session, device_id: &str) -> Result<(), AccountError>;
pub fn usage(b: &dyn Backend, session: &Session) -> Result<Usage, AccountError>;

pub fn sync(store: &Store, b: &dyn Backend, session: &Session, options: &SyncOptions) -> Result<SyncReport, SyncError>;
pub fn verify(store: &Store, b: &dyn Backend, session: &Session, legacy: Option<&Path>, repair: bool) -> Result<VerifyReport, SyncError>;
pub fn estimate(store: &Store) -> Result<Estimate, StoreError>;   // charged bytes the library needs
pub fn problem(err: &SyncError) -> Option<SyncProblem>;             // None for Cancelled; BackOff for SyncError::BackOff
pub fn report_problem(report: &SyncReport) -> Option<SyncProblem>;  // problems in a sync that finished
```

`SyncOptions` gains two fields:

- `cancel: Option<Arc<AtomicBool>>`. The engine checks it between pages, batches and files, and
  returns `SyncError::Cancelled`.
- `cursor_key: &'static str`, default `"sync.v2.last_seq"`. Only the rollback test sets the legacy
  `"sync.last_seq"`, to run the old rules against `tests/common/legacy.rs`.
- `refetch_files: usize`, default 8: the most live files without bytes one sync fetches again
  (Sync with encryption › Files).

`SyncReport` gains `paused_limit: Option<String>` and `uploads_held_until: Option<i64>` (Sync with
encryption › Pauses). `SyncError` gains `BackOff { until: i64 }`, and `SyncProblem` gains
`ServerBusy { until }` and `BackOff { until }`, right after `Paused`.

In `remote.rs`, `ChangesPage.epoch` and `key_id` are `Option`. `Sealed` always fills them, because
`WirePage` requires both, so a server can't skip the checks by leaving them out. Only the legacy
test remote leaves them `None`.

`AccountError` has one variant per error code, plus:

- `Transport`
- `ServerOutdated`: 404 from `/v2/health`.
- `UnsafeKdf`: prelogin outside the backend's floor or ceilings.
- `KeyMismatch`: a failed unwrap or a different key ID.
- `Unauthorized { reason }`
- `Password`, `Username` and `RecoveryKey` validation.
- `Protocol` (an answer of the wrong shape), `Endpoint` (`check_endpoint`) and `Keychain`.
- `UnsavedRecoveryKey { username }`: the library's Keychain still holds another account's unsaved
  recovery key, which a new one would replace. Nothing was sent.

### Account flows

- **Sign up.**
  1. Normalize the username and check the password, offline.
  2. `server_info`, which must say `Open`.
  3. Generate the account ID, salt, data key and recovery secret.
  4. Derive the keys (`Step::DerivingKeys`) and wrap twice. Seal the device name.
  5. Save `sync-recovery-key:<library>`, unconfirmed (below), then `POST signup` (`Contacting`).
  6. Save the session's Keychain item, then bind (`Saving`).

  The salt input includes the username, so trying another name after `username_taken` derives again.
- **Sign in.**
  1. Prelogin, refusing parameters outside the floor or ceilings, to get the salt and account ID.
  2. Compute the device ID and proof, and derive.
  3. Log in, unwrap, and compare the key ID: a failure is `KeyMismatch` (the session is logged out,
     nothing is saved).
  4. Ask the binding question if needed, then bind and save.
  5. `PUT /v2/account/device` with the sealed name.
  6. If a device was evicted, notify "Signed out “*name*” to make room: an account can be signed
     in on 10 Macs at a time."
- **Recover.**
  1. Parse the key, then prelogin.
  2. `recover/start`; unwrap and compare the key ID.
  3. Make a new salt and a new recovery secret, derive, wrap, save the new key unconfirmed, and
     `recover/finish`.
  4. Show the new recovery key screen.
- **Change password.** Prelogin for the salt, two derivations, and a new recovery secret: a
  recovery key that outlived the password would let whoever holds it take the account back. The
  new key is saved unconfirmed, then `account/password` carries `recovery_auth_key` and
  `recovery_wrapped_key` with the new password's values. Other Macs get 401 `password_changed`;
  this one shows the key screen over "Your password changed, and so did your recovery key: the one
  you saved no longer works."
- **New recovery key.** Derive the current `auth_key`, make a new recovery secret, wrap, save it
  unconfirmed, then `account/recovery-key`. Show the key screen.
- **Delete account.** Derive `auth_key`, then `account/delete`, then `unbind`.
- **Sign out.** First cancel and join the running sync (fix 2). Then delete
  `sync-account:<library>` and any `sync-recovery-key:<library>` (H3). Then try `logout` with a 5 s
  timeout, even when the Keychain refused a deletion (A14; that refusal is reported after). On
  failure, add `{server, token}` to `sync-logout:<library>`. `flush_pending_logouts` retries those
  at launch and before each sync until the server answers 204 or 401. Every read-modify-write of
  the queue holds one lock in the process, never across a request: a flush reads the queue, sends
  the logouts, then takes out only those that went, so a token queued meanwhile stays (sync 9).

**A new recovery key on its way** (sec-client 1). Sign-up, recovery, a password change and "New
recovery key" each choose the key before building the request and save it in
`sync-recovery-key:<library>` marked `unconfirmed` before the request leaves; nothing is sent if
the Keychain refuses. Then:

- The server says yes: the item is marked confirmed.
- Every attempt was refused (a 4xx) or never left the Mac: the item goes back to what it was.
- Any attempt's answer was lost, garbled or a 5xx: the item stays, and `RotationError.recovery_key`
  carries it, so the app shows it with a note that the server may not have taken it. This is
  sticky across the retries (review-verify 1): when the first attempt's answer is lost and a retry then
  never leaves the Mac (offline) or gets a 429, the first one may still have been carried out, so
  the key is kept, not dropped.

While the item holds an unsaved key for the same account, the next rotation sets that same key
again (except the key a recovery proves), so a retried or repeated change can't leave the server
with a key the Mac no longer has. Another account's unsaved key refuses with `UnsavedRecoveryKey`.

**Requests.** A request whose answer is lost is sent again with the identical body, only on routes
the server answers idempotently, after about 1 s and then 4 s (`Backend::retry_pause`). The HTTP
backend gives each request, connecting included, 30 s on the auth and account routes, 5 s on
`logout`, and 300 s on pushes, changes and files (A7). Request bodies and answers that hold keys,
proofs or tokens are zeroized when dropped, and their `Debug` shows `<redacted>`.

### Sync with encryption

The engine works on plaintext `SyncDoc`s through `Remote`, with `Sealed<W>` in between.

#### What a Mac remembers per document

The data-integrity review found one root problem. `synced_rev` was meant to mean "a version this
account's server actually had", but three paths broke that: binding with Add, a server-reported loss,
and an epoch re-merge. The M2 stale rule then turned real changes into silent overwrites or
deletions. Or the other way, a hostile server could clear the floor and bring deleted notes back. So
each document row now keeps separate facts, one per question, and no rule reads a fact meant for
another question:

| Column | Question it answers | Set by | Never changed by |
|---|---|---|---|
| `synced_rev` | Which revision does this account's server hold, as far as this Mac knows? It is the next push's `base_rev`; NULL means "push with no base". | A guarded write that learned it from the server: an accepted push (`mark_synced`), or a server version that was applied, unchanged, stale, an echo or a conflict (`apply_remote`). Cleared by `Lost` (`mark_unsynced`), the re-merge's missing pass (`mark_missing_unsynced`) and binding (`reset_sync`). | Local writes |
| `synced_gen` | The replay floor: the highest generation this Mac has seen on this account for the document. | Raised, never lowered, by every write that sets `synced_rev` from the server. Reset to 0 only when binding to an account (`reset_sync`); `replace_library` removes the rows. | `Lost`, `mark_missing_unsynced`, an epoch change, local writes |
| `sent_rev` | Which revision is in a push whose answer this Mac hasn't processed yet? | `mark_sent`, in one guarded write right before each push request. Cleared by every write that sets or clears `synced_rev`. | Local writes |

A document is pending when `synced_rev IS NULL OR synced_rev != rev`, as before.

A tombstone that `reset_sync` leaves behind (Add, a fresh bind, unbind) gets `synced_rev = rev` and
`synced_gen = 0`. That pair means "never on this account". The tombstone is never pushed, the floor
doesn't protect it from the account's versions, a re-merge doesn't count it as missing, `verify`
skips it, and a note re-created over it is pushed with no base.

`apply_remote` decides in this order:

1. No local row: apply. Same revision: mark it synced (and keep bytes that arrived with it).
   `synced_rev` already equals the remote revision: unchanged.
2. **Our own echo.** The remote revision equals `sent_rev`: the server took our push and the answer
   was lost. `synced_rev` becomes that revision, the newer local revision stays pending on top, and
   it is `Unchanged`. It is never a conflict, so an API merge that kept `updatedAt` isn't reverted by
   its own echo, and an editor save isn't falsely flagged.
3. **The floor (M2).** A remote revision whose generation is at or below `synced_gen` never reverts
   the Mac. It is `Stale`: the local version is rebased above it and pushed again. `Lost`, the
   re-merge's missing pass and an epoch change keep `synced_gen`. So a server that claims it lost a
   document, or rotates its epoch, and then answers with an old sealed version still gets `Stale`. It
   can't bring back a deleted note or roll back an edit, however many such tricks it plays in one sync.
4. **Nothing is discarded silently.** A stale remote version can be live, differ from the local
   content, and have a content hash this Mac doesn't hold (neither current nor in the document's
   history). Then it is kept as a snapshot and the document is listed in `conflicted`
   (`Stale { conflict: true }`, counted in `conflicts_local`). After a restore, generations from
   different Macs no longer compare: another Mac's newer offline edit can fall below this Mac's floor.
   It loses to the local version, as any conflict loser does, and is kept and flagged. A replayed
   version this Mac already holds adds nothing and flags nothing.

   **Except over a deletion** (review-verify 2). When the local side is a tombstone, a live stale version
   whose content this Mac never held isn't stale. It takes the normal "edit beats deletion" path:
   it is stored (a file's bytes with it), `synced_rev` becomes its revision, and it is `RemoteWon`,
   counted in `conflicts_remote` and flagged. So after a restore, a Mac's deletion never pushes a
   tombstone over another Mac's edit it never saw, and never deletes that file's bytes on the
   server or on the other Mac. The outcome no longer depends on which Mac syncs first, or on how
   many edits each made, and the flag always names a note that is there to open. A stale conflict
   is only ever raised over live local content.

   A deletion always puts the version it deletes into history, even from the UI right after an
   edit, whose snapshots are otherwise at most one per 5 minutes. That history is how a replay of
   content this Mac held, the version it deleted or any earlier one still in history, is told from
   another Mac's edit: a replay stays `Stale`, and the deletion stands.

   The trade-off: a hostile server can now bring a deleted note back on a Mac by replaying a version
   another Mac wrote that this Mac never pulled (for instance an intermediate edit it skipped). It
   is real content of the account, flagged where it lands, and of the same class as the existing
   "a Mac that never saw a newer version" risk ("Threat model › Residual risks", stale reads). The
   user-facing rule, "an edit always beats a deletion", wins over that.

   A residual stays on the other side, in the re-merge: a Mac whose clean edit the restore dropped
   takes a deletion that another Mac pushed after the restore without ever seeing that edit,
   because it can't tell that deletion from one made after seeing it. The edit is kept in the
   document's history and the document is flagged, but nothing in the app opens a deleted note's
   history; a backup has it. The Account page names and opens only a conflicted note that is still
   there.
5. Otherwise the conflict rules apply as before. A clean local copy takes the remote version. A
   changed one keeps the newer `updatedAt`, and an edit beats a deletion. The loser is kept as a
   snapshot and flagged.

**Add** (review-sync 1, sec-client 5). The library's live documents get `synced_rev = NULL` and
`synced_gen = 0`, and are pushed with no base. Its tombstones are never on this account. The first
sync pulls first:

- The account's live copy of a note deleted on this Mac comes back: an edit always beats a deletion,
  however many edits either side made. For a file, its bytes come back with it, on the server and on
  every Mac.
- A note deleted in the account but live on this Mac goes back to the account.
- A note changed on both sides keeps both versions and is flagged.

#### In order

- **Start of a sync** (first page):
  - If `key_id` differs from the Mac's: `WrongKey`. Stop and write nothing.
  - If `epoch` differs from `sync.v2.epoch`, or `head_seq` is below the cursor, the server's copy
    was restored or rewound (M1). The Mac re-merges without forgetting what it synced (security
    re-check 4):
    1. `synced_rev` and `synced_gen` stay as they are. The cursor goes to 0, and the epoch becomes
       `remerging` until the re-merge finishes (`set_meta_if`, guarded).
    2. The engine pulls the whole feed through `apply_remote`, so the rules above still hold. A
       revision below the floor comes back `Stale`, and is rebased and pushed. A remote version that
       replaces a different live local version during the re-merge is also flagged in `conflicted`:
       after a restore the Mac can't tell whether it descends from what the Mac synced. The local one
       is kept as a snapshot.
    3. After the last page, `mark_missing_unsynced(present)` takes every document with a
       `synced_rev` (tombstones included, except those never on this account) that the feed didn't
       carry. The server lost it, so it is pushed again with `base_rev = null`, and its floor stays.
       Deletions are never lost.
    4. A normal push follows. Conflicts keep both versions.

    The report sets `remerged`, and the app says "The server's copy was restored from a backup; this
    Mac sent its changes again." The cursor is never just reset, and `reset_sync` is only for
    binding to a different account. `head_seq` below the cursor is a best-effort check: it misses a
    rewind once any Mac has pushed past this Mac's cursor again. A restore must rotate the epoch
    (`admin restore`).
- **Paging.** Continue while `page.more`; after applying a page, save `page.last_seq` with
  `set_meta_if` (guarded).
- **Undecryptable feed documents** (format `0x01` that fails to open) never stop the pull; the
  cursor moves past them. They are repaired automatically only when the Mac has a local document
  with that wire ID and the broken copy's wire revision equals `wire_rev(id, synced_rev)`: the
  server holds a damaged copy of exactly the version this Mac last synced. Then the local version
  gets a new revision one generation higher and is pushed with the broken revision as its base,
  which replaces the server copy (M3); `report.repaired` lists it. Any other undecryptable document
  is only skipped and listed in `report.skipped`; `kanso-sync verify --repair` can replace it from
  this Mac.
- **Too-new documents** (format above `0x01`) stop the pull before them, keeping the cursor. The
  report sets `too_new`, which gives the problem `UpdateKanso`. Pushing continues, and is never a
  repair.
- **Stale documents** (M2): `apply_remote` returns `Stale { new_rev, conflict }`. The remote version
  never reverts the Mac. The local version is rebased onto it like a local win: generation
  `max(local, remote) + 1`, with `synced_rev` set to the remote revision. The remote version is kept
  as a snapshot and flagged when rule 4 says so. The next push sends it. `report.stale` lists the
  stale documents and the cursor moves on.
- **Files.** Opened bytes must match the document's `md5digest`, or they count in `files_missing`.
  After the pull, the engine fetches again up to `SyncOptions::refetch_files` (default 8) live file
  documents whose bytes this Mac lacks, meaning no blob matches the document's `md5digest`:
  - It goes in ID order from where the last sync stopped (`sync.v2.refetch_after`), skipping files
    this sync already tried.
  - Bytes are kept only when they open and match the `md5digest`.
  - They are stored in a guarded write that also checks the document still has the revision they
    were fetched for (`Store::fill_blob`).
  - A hit counts in `files_down`, a miss in `files_missing`.
  - It never stops the sync or the push after it (review-verify 4). A request that fails (offline,
    a 5xx, an answer over the cap) counts as a miss and ends this sync's refetching, since the
    rest would likely fail too and each can take minutes. Only a 429 `session-account` stops the
    sync, as on every route (`BackOff`).
  - A file it missed (gone from the server, not matching its `md5digest`, unreadable, or a failed
    request) waits a day before it is asked for again: `sync.v2.refetch_missed` maps its ID to that
    time. Bytes gone for good, such as imported attachments without them, cost one request a day,
    not eight a sync. At most 512 files wait, and past that the ones due soonest are forgotten.
    A wait already due, or more than a day and an hour ahead (saved while the clock ran ahead), is
    dropped. The pull's own fetch of a new version's bytes doesn't wait.
- **Push.** Before each push request, `mark_sent` records the batch's revisions (guarded). Results:
  - `Conflict` with a current version that opens: applied by the rules above. A local win gets the bumped generation and goes in the next round.
  - `Lost` (`current: null` with a `base_rev`): `mark_unsynced` (the floor stays), then pushed again with `base_rev = null` in the next round (M1).
  - `Behind`: kept pending; the engine pulls before the next round.
  - `Unreadable` (old format): repaired, as above. `Unreadable` (newer format): kept pending, and the report sets `too_new`.

  None of these lands in `rejected`, which goes away.
- **Quota** (review-sync 6). The Worker counts `writes-account` before it runs a batch, so a refused
  batch still costs the whole day's budget for its documents. A 413 `quota_exceeded` is therefore
  remembered, per family, in a guarded meta write:
  - Document quotas (`docs`, `doc_bytes`, and `server` from a push) go in `sync.v2.quota.docs`.
  - File quotas (`files`, `file_bytes`, and `server` from an upload) go in `sync.v2.quota.files`.

  Each entry is `<kind> <used> <limit> <at_ms> <level>`. It holds documents back from the push
  until usage drops:
  - `new` holds what the server doesn't have yet: documents with no `synced_rev` (new IDs and
    never-synced tombstones), and uploads of new files.
  - `edits` also holds live documents the server has (and their uploads). The engine moves to it when
    a batch with no new documents is refused.
  - `all` holds the whole family. It applies when even a batch of tombstones is refused.

  Tombstones and edits of documents the server has keep going, so a user can delete their way back
  under quota. A refused batch is retried at once, without what the new level holds; that is the old
  "tombstones only" retry, generalized.

  The first time a sync with an entry would hold a document back, the engine reads
  `GET /v2/account` (once a sync). Usage dropped when the kind's usage is below the `used` the 413
  reported, or its quota rose above `limit`. A `server` entry can't be measured there, so it retries
  after an hour. When usage dropped, held documents go out again, within the headroom, each charged
  once a sync:
  - new live documents up to `quota.docs − usage.docs`;
  - documents the level holds, at their estimated sealed size (an upper bound) plus 512 each, up to
    the `doc_bytes` headroom (a tombstone of a document the server has costs nothing);
  - uploads the level holds, at their size plus 1,088 each, up to the file-bytes headroom, and new
    ones up to the file-count headroom.

  If the request fails, the entry stays. A 413 later in the same sync takes no more from the room.
  A sync that ran to the end, held nothing back and got no 413 deletes the family's entry. A live
  file document held for any reason (its document, its upload, or the upload hold) keeps both
  entries, since its upload was never tried. While an entry is in place, `report.quota` stays set,
  so the app keeps saying "Account full". So writes only grow by what is actually sent: a refused
  batch is never sent again until usage drops, and then only what fits.
- **Pauses (429).** `report.paused_limit` keeps the 429's `limit`, so the app can tell the server's
  cap from the account's own.
  - `writes-account` or `uploads-account` stops pushing for this sync and sets
    `report.paused_until`, which gives the problem `Paused`.
  - `writes-global` does the same, and gives `ServerBusy`.
  - `uploads-global` on an upload doesn't stop the sync or block pulls. Uploads are held until
    `retry_after`, in `sync.v2.uploads_until` (guarded), along with the live file documents waiting
    for them. Everything else keeps syncing, and `report.uploads_held_until` says until when.
  - `session-account` (the per-account request limit on every session route, about a minute) on
    any sync request (changes, push, upload, download) is a short back-off, not a pause. The sync
    stops where it is with `SyncError::BackOff { until }`, `until` being now plus `retry_after`.
    Nothing after the last finished step is marked done: the cursor stays at the last page applied,
    and a document whose push or bytes didn't go stays pending. `problem` gives `BackOff { until }`,
    whose schedule waits until then.
  - **Bounded waits** (review-verify 5). Every time a 429 sets (a pause, the upload hold, a back-off,
    and so the app's schedule) is now plus `retry_after`, clamped to 24 hours: a server's answer,
    however large, never holds a Mac back longer than a day. The saved times are read back with the
    clock they were saved by, which may have run ahead: a quota entry whose `at_ms`, or an upload
    hold whose time, is more than 25 hours ahead of now is deleted when read, so nothing waits on
    it (before, only a re-bind cleared it). A deleted quota entry costs at most one more refused
    request, which saves it again with the right time.
- **The guard** (fix 2 / M6, security re-check 3). Every `apply_remote`, `mark_synced`,
  `mark_sent`, `mark_unsynced`, `mark_missing_unsynced`, `fill_blob` and meta write (cursor, epoch,
  quota entries, holds) carries `SyncGuard { expect: &[("sync.v2.origin", session.origin),
  ("sync.v2.account_id", session.account_id)] }` (`binding::guard_pairs`). The store checks every
  pair inside the same write transaction and fails with `StoreError::GuardFailed` otherwise. The
  engine then stops with `SyncError::Cancelled`. `check_bound` also runs right before every push and
  upload.

Every remote write still goes through `apply_remote`, which snapshots the version it replaces.

### Problems

| `SyncProblem` | From | Sidebar | Settings help | Schedule |
|---|---|---|---|---|
| `SignedOut(reason)` | 401 | "Signed out" | Reason-specific: "Your password was changed on another Mac." / "This Mac was signed out from another Mac." / "Your account was recovered with its recovery key." / "Signed out to make room for another Mac." / "Signed out after 180 days without syncing." Any other reason: "This Mac was signed out." Each ends "Sign in again to keep syncing." | Stops |
| `AccountDisabled` | 403 | "Account turned off" | "This account is turned off on the server. Ask the server's owner." | Stops; Sync now retries |
| `Paused { until }` | 429 `writes-account` or `uploads-account` | "Sync paused" | "This account made a lot of changes today. Sync resumes at *time*." | Waits until then |
| `ServerBusy { until }` | 429 `writes-global` | "Server busy" | "The server has taken all the changes it can for today. Sync resumes at *time*." | Waits until then |
| `BackOff { until }` | 429 `session-account` (`SyncError::BackOff`) | "Syncing shortly" | "This Mac sent the server a lot of requests at once. Sync resumes in a moment." | Waits until then (about `retry_after`) |
| `QuotaExceeded(kind)` | report | "Account full" | "Your account is full (*which*: notes and notebooks / attachments / number of attachments / number of items). Delete something or ask the server's owner for more space." | Keeps running |
| `ServerFull` | report (`server`) | "Server full" | "The server is out of space. Ask its owner." | Keeps running |
| `Skipped { count }` | report (`skipped`: undecryptable documents not repaired) | "Some items skipped" | "*N* items on the server couldn't be read. They were skipped; your copies here are unchanged." | Keeps running |
| `UpdateKanso` | report (too new) | "Update Kanso" | "Another Mac uses a newer Kanso. Update this one to keep syncing." | Stops; Sync now retries |
| `WrongKey` | key ID | "Keys don't match" | "The server's account keys don't match this Mac's. Sync is stopped; your notes are safe here. Sign out and in again." | Stops |
| `ServerOutdated` | 404 health | "Server needs an update" | "That server doesn't run this version of Kanso sync." | Stops; Sync now retries |
| `Offline` | transport | "Offline" | "Can't reach the server. Your notes are safe on this Mac." | Keeps running |
| `Other` | anything else | "Couldn't sync" | "Something went wrong. Try Sync now." | Keeps running |

`report_problem` picks, in this order of precedence, `UpdateKanso`, `Paused` or `ServerBusy` (from
`paused_until`: `ServerBusy` when `paused_limit` ends with `-global`), `ServerFull`, `QuotaExceeded`,
`Skipped`. An `uploads-global` hold sets `uploads_held_until` and no problem, so the schedule keeps
pulling. Stale and repaired documents aren't
problems: the Mac fixed them. `SyncState::Synced` gains `problem: Option<SyncProblem>`, and `SyncState::Failed`
carries a `SyncProblem`. `SYNC_PROBLEMS` (sidebar) and `SYNC_PROBLEM_HELP` (settings) become
functions of `SyncProblem`, which removes substring matching.

### Keychain items

The service is `Kanso`. Item names use the library ID, so data directories never share secrets.

| Account (item name) | Value (JSON) | Lifetime |
|---|---|---|
| `sync-account:<library_id>` | `{"v":1,"server","origin","account_id","username","device_id","token","data_key"}` | Sign-up, sign-in or recovery until sign-out, deletion, a 401 or unbind |
| `sync-recovery-key:<library_id>` | `{"v":1,"origin","account_id","username","server","recovery_key","unconfirmed"}` (`server` and `unconfirmed` left out when empty or false) | From just before the request that sets the key leaves (sign-up, recovery, a password change, "New recovery key") until "I've saved it" or sign-out. A refused request puts back what was there. A confirmed key also goes when this Mac's session ends with `password_changed` or `recovered`: another device replaced it. While it exists, the Account page shows the key screen again. |
| `sync-logout:<library_id>` | `{"v":1,"pending":[{"server","token"}]}` | Until each logout succeeds |
| `sync-token` (legacy) | The old shared token | Never read or deleted by the new app; the owner removes it after the purge |

On load, the item's `(origin, account_id)` must equal the library's `(sync.v2.origin,
sync.v2.account_id)` and the config's account (security re-check 3). If it doesn't, the item is
ignored, and the Mac counts as signed out. The same pair decides everywhere else whether two
accounts are the same. `Session.server` and
`origin` always come from this item; `config.sync.server` only pre-fills the signed-out forms. The
data key stays in memory once loaded, and is zeroized on sign-out or quit.

### Library binding

**Meta keys** (H5). Only `unbind` touches the legacy `sync.last_seq`.

| Key | Holds |
|---|---|
| `sync.v2.origin` | The origin of the account's server |
| `sync.v2.account_id` | The account the library syncs with |
| `sync.v2.username` | Its username |
| `sync.v2.server` | The server URL |
| `sync.v2.last_seq` | The cursor |
| `sync.v2.epoch` | The epoch last seen (`remerging` while a re-merge runs) |
| `sync.v2.quota.docs` | A document quota hit still holding documents back: `<kind> <used> <limit> <at_ms> <level>` |
| `sync.v2.quota.files` | The same for file quotas |
| `sync.v2.uploads_until` | Uploads held until this time (ms) after an `uploads-global` 429 |
| `sync.v2.refetch_after` | The file ID where the last sync's search for missing bytes stopped |
| `sync.v2.refetch_missed` | The files a refetch missed, as a JSON object of file ID to the time (ms) each may be asked for again |
| `sync.v2.library_id` | Device identity |
| `sync.v2.device_secret` | Device identity |
| `sync.v2.hardware` | Device identity |

**kanso-store APIs** (frozen in step 1):

```rust
/// Every (meta key, value) pair must hold; the engine passes origin and account ID.
pub struct SyncGuard<'a> { pub expect: &'a [(&'a str, &'a str)] }
pub enum ApplyOutcome { Applied, Unchanged, RemoteWon, LocalWon { new_rev: String }, Stale { new_rev: String, conflict: bool } }
/// One row's sync facts (see "Sync with encryption › What a Mac remembers per document").
pub struct SyncState { pub id: String, pub rev: String, pub deleted: bool, pub synced_rev: Option<String>,
                       pub synced_gen: u64, pub sent_rev: Option<String> }
impl SyncState { pub fn pending(&self) -> bool; pub fn never_on_account(&self) -> bool; }
impl Store {
    /// One transaction: live documents get synced_rev = NULL, tombstones get synced_rev = rev
    /// and synced_gen = 0 (never on this account: never pushed, never protected), every floor
    /// and sent_rev is cleared, and the meta entries are written (Some) or removed (None).
    pub fn reset_sync(&self, meta: &[(&str, Option<&str>)]) -> Result<()>;
    /// One transaction: removes every document, note-tag row, blob, revision, FTS row and archive
    /// row, then writes the meta entries. Broadcasts a deletion per document (Origin::Sync).
    pub fn replace_library(&self, meta: &[(&str, Option<&str>)]) -> Result<usize>;
    pub fn apply_remote(&self, doc: &SyncDoc, attachment: Option<Attachment>, guard: Option<&SyncGuard>) -> Result<ApplyOutcome>;
    pub fn mark_synced(&self, id: &str, rev: &str, guard: Option<&SyncGuard>) -> Result<()>;
    /// Right before a push: each (id, rev) is in flight (sent_rev).
    pub fn mark_sent(&self, docs: &[(&str, &str)], guard: Option<&SyncGuard>) -> Result<()>;
    /// synced_rev = NULL; the floor stays.
    pub fn mark_unsynced(&self, id: &str, guard: Option<&SyncGuard>) -> Result<()>;
    /// After a re-merge's full pull: documents with a synced_rev (tombstones included, not those
    /// never on this account) whose IDs aren't in `present` get synced_rev = NULL, so they are
    /// pushed with base_rev = null. The floor stays.
    pub fn mark_missing_unsynced(&self, present: &HashSet<String>, guard: Option<&SyncGuard>) -> Result<Vec<String>>;
    pub fn sync_states(&self) -> Result<Vec<SyncState>>;
    /// Live file documents with no blob matching their md5digest: (id, rev, md5), in ID order after `after`.
    pub fn files_without_bytes(&self, after: &str, limit: usize) -> Result<Vec<(String, String, String)>>;
    /// Stores refetched bytes iff `id` is still live at `rev` and they match its md5digest.
    pub fn fill_blob(&self, id: &str, rev: &str, blob: Attachment, guard: Option<&SyncGuard>) -> Result<bool>;
    pub fn set_meta_if(&self, key: &str, value: &str, guard: &SyncGuard) -> Result<()>;
    /// Removes a meta entry (a quota entry, the upload hold) while `guard` holds.
    pub fn remove_meta_if(&self, key: &str, guard: &SyncGuard) -> Result<()>;
    pub fn pending_count(&self) -> Result<usize>;
}
```

`StoreError` gains `GuardFailed`.

`binding::local_library(store, origin, account_id)` returns one of these, where the same account
means the same `(origin, account_id)`:

- `Empty`: no live documents.
- `Same`
- `Unbound { notes, unsynced }`
- `Other { username, notes, unsynced }`

`unsynced` counts documents with changes that never reached any server (`pending_count`). The plan
for each flow and library:

| Flow | Library | What happens |
|---|---|---|
| Sign up | Empty or unbound | Bind fresh, then upload. The form says "Your *N* notes on this Mac will be encrypted and uploaded." |
| Sign up | Bound to another account | Before sign-up: "This Mac's notes came from *alex*. Upload them to the new account?" [Upload to new account] [Cancel]. Upload means merge. |
| Sign in or recover | Empty | Bind fresh, then pull. |
| Sign in or recover | Same | Resume. |
| Sign in or recover | Unbound with notes, or bound to another account | After authenticating, before saving anything (below). Cancel logs the new session out (`discard_session`); the library's Keychain items, which may be the signed-in account's or hold a recovery key still on screen, stay. |

The question in the last row:

- Title: "This Mac already has notes".
- Body:
  - "*N* notes on this Mac (*U* with changes that were never synced) aren't in *ngoni*'s account."
  - Or, when the library came from another account: "came from *alex*".
- Choices:
  - [Add them to *ngoni*]: "They're encrypted and uploaded. A note deleted in *ngoni*'s account but
    still on this Mac comes back, and a note changed on both sides keeps both versions."
  - [Replace them with *ngoni*'s notes]: "Kanso saves a backup of this Mac's library first, then
    downloads *ngoni*'s notes."
  - [Cancel]

What the choices do:

- **Fresh and merge (add).** `reset_sync` with the origin, account, username, server, `last_seq = 0`,
  and the epoch, quota entries, upload hold and refetch position removed. The first sync pulls first:
  identical revisions become `Unchanged`; the account's live copy of a note deleted here comes back
  (the tombstone was never on this account); the rest resolve as conflicts with both versions kept.
- **Replace.**
  1. `Store::backup_now(paths::backups_dir(data_dir), now)`: the same manual backup as Settings ›
     Back up now, outside the daily rotation. The path is shown with "Show in Finder".
  2. `replace_library` with the same meta, in one transaction (fix 3). A crash leaves either the old
     library or an empty one bound fresh, which pulls.
- **Unbind** (after deleting the account, and for rollback). `reset_sync` that removes the origin,
  account keys, cursor, server and epoch, keeps the device identity, and sets the legacy `sync.last_seq =
  0`. It also deletes `sync-account:<library>`.

**Order and cancellation.** Every flow that calls `reset_sync`, `replace_library` or `unbind`, or
touches the Keychain item, first cancels the running sync and joins its thread (fix 2). On sign-in:

1. Answer the question, if there is one.
2. Cancel and join the running sync. For Replace, every open editor saves first, so the backup has
   its edits; editor saves then wait and the local API (with MCP) stops until the bind ends (A10).
   An editor with edits typed meanwhile stays on its note, saves them afterwards, and asks about
   them instead when the Replace took the note away.
3. Save the Keychain item (A1, sync 7): a library is never bound to an account whose item is
   missing.
4. Bind. If it fails, the Keychain item is put back as it was (gone, or the previous account's),
   the new session is logged out, and a backup Replace made first is still shown.
5. Write the config (`account`, `enabled = true`).
6. Start sync.

A chain of tasks that stopped the running sync and leaves the same session signed in (it failed,
or its question was cancelled) starts an automatic sync again once its last task ends (A8). A sync
asked for while another runs, or while that one is being stopped, runs right after it: as Sync now
if any of the requests was, otherwise as an automatic sync that respects the schedule's hold
(A9, sync 8).

Startup counts the Mac as signed in only when the config, the Keychain item and the binding agree.

### Command line

Run `cargo run --release -p kanso-sync -- <command>` from the deployed commit. Each command takes
the data-directory lock, so quit Kanso first. macOS asks once to let `kanso-sync` read the
`sync-account` item.

```text
status   --data-dir DIR                             account, server, device, cursor, pending changes
sync     --data-dir DIR [--full]                    sync once with this library's session
verify   --data-dir DIR [--repair] [--legacy M.json --out R.json]
estimate --data-dir DIR                             charged document and file bytes this library needs
unbind   --data-dir DIR                             sign out locally and reset bookkeeping for the old app
```

`verify` pulls the whole feed into memory without applying it, then decrypts every document and
live file. It compares them with the library: every local document whose `synced_rev == rev` must
be on the server at that revision with equal fields, and every live file's bytes must match. It
prints the counts (matched, missing, extra, different, undecryptable, still pending), and exits 0
only when nothing is missing, different, undecryptable or pending. With `--repair`, it pushes this
Mac's version over each undecryptable server copy of a document the Mac has, with a new
generation, and counts them as repaired.

With `--legacy`, it also checks each row of the manifest from `admin legacy-export` (H4):

- **A live legacy document** must be in the library, either at that revision or at a higher
  generation, and must have passed the checks above.
- **A live legacy file** must have a local blob with the manifest's `md5digest`, and must be on the
  encrypted server.

It then writes `R.json` (`{"manifest_sha256", "account_id", "ok", "rows", "failed"}`).

**The manifest hash** (security re-check 6) covers the rows only, never `exported_at`. Sort the rows
by `id` (bytewise UTF-8); write each as `id TAB rev TAB ("1" if deleted, else "0") TAB (md5digest,
or nothing when null) LF`; the hash is SHA-256 of those bytes as 64 lowercase hex digits
(`verify::manifest_sha256`). `purge-legacy` recomputes it the same way from a fresh export. Test
vector: the rows `["note:b","2-bb",false,null]`, `["file:a","1-aa",false,
"d41d8cd98f00b204e9800998ecf8427e"]` and `["tag:c","3-cc",true,null]` hash to
`a0dbdfeedc42d0d264500b4faad3718aa823528946c653f34e62b2be42575b92`; no rows hash to SHA-256 of the
empty string.

## App

### Config (`config.rs`, `CONFIG_VERSION = 2`)

```rust
pub struct SyncSettings {
    pub enabled: bool,                    // set true on sign-up/sign-in; sync only runs while signed in
    pub interval_secs: u64,
    pub server: Option<String>,           // forms only; None = DEFAULT_SERVER
    pub account: Option<AccountRef>,      // Some while signed in
    pub endpoint: Option<String>,         // legacy, preserved untouched so a rolled-back app finds it
}
pub struct AccountRef { pub origin: String, pub id: String, pub username: String }
```

Startup compares `(origin, id)` with the Keychain item and the binding (security re-check 3).

The migration to version 2 copies `endpoint` into `server` when it isn't the default, keeps
`endpoint` and `enabled` as they are, and leaves `account` as `None`. The owner's Mac therefore
starts signed out with its notes intact. Older app builds ignore the new fields.

### Services (`services.rs`)

- `TokenState` goes. `AccountState` is one of `SignedOut`, `SignedIn { username }`, `SessionEnded {
  username, reason }` or `Busy(Step)`.
- `Services` holds:
  - `Arc<Session>` once loaded;
  - `Arc<dyn Backend>`: `HttpBackend` in `main.rs`, `MemoryServer` in tests;
  - `Arc<dyn SecretStore>`;
  - the running sync's `SyncHandle { cancel, thread }`.
- **Account tasks** run on a named `std::thread` and send `Step` updates and the result over an
  `async_channel`. Any task that re-binds or signs out cancels and joins `SyncHandle` first, on that
  background thread, with "Stopping sync…" as its step.
- On `SignedOut(reason)`, the app deletes the Keychain item, shows `SessionEnded`, and stops the
  schedule. With `password_changed` or `recovered`, a confirmed unsaved recovery key is deleted
  too, and leaves the key screen: the server no longer holds it (sec-client 3). An unconfirmed one
  stays, since it may be the only working key.
- **This Mac's own recovery** (review-verify 3). "Forgot your password?" while signed in ends this
  Mac's old session too, so a request still in flight on it (the Account page's storage and
  devices) gets a 401 before the recovery has signed in again: `unknown` once the new session
  replaced the device's row, or `recovered`. While a recovery task runs, such a 401 never deletes
  the unsaved recovery key, which is the recovery's own new one. And whatever the reason, the
  session item is deleted only while it still holds the ended session
  (`keychain::delete_ended_session`), so a session the recovery saved meanwhile stays for the next
  launch.
- **The launch check** (`restore`) reads the Keychain on a background thread. Account tasks wait for
  it (`restoring`), and its answer is dropped if a session appeared, a task started or the config's
  account changed meanwhile, so it never overwrites a newer state (A4). Pending logouts are retried
  after it answers. An unsaved recovery key shows whatever the account.
- **Writes held** (`writes_held`, `hold_writes`, `release_writes`): around a Replace, as in Library
  binding › Order and cancellation. The local API starts again only if it ran.

### Account page (Settings › Account)

`SettingsPage::Sync` becomes `SettingsPage::Account`: title "Account", a person icon, id
`settings-account`, between Appearance and Local API. The action `OpenSyncSettings` becomes
`OpenAccountSettings`. The page lives in a new `account_page.rs`, with its forms as `InputState`
entities.

**Signed out.** The status row reads "Not signed in" (quiet), with the detail "Sync your notes
between Macs, end-to-end encrypted. Your notes stay on this Mac either way." Below it: [Create
account] [Sign in].

- **Create account.** Rows:
  - Username, with the hint "3–32 lowercase letters, digits, - or _".
  - Password, with the hint "At least 12 characters. A few random words work well."
  - Confirm password.
  - Server: "kanso-sync.imngonii.workers.dev (default)" or the custom URL, with "Use a different
    server…". That opens a prompt checked by `check_endpoint` and `server_info`, and a custom server
    shows "Use the default server" next to it.

  Then the line "Your *N* notes on this Mac will be encrypted and uploaded. Kanso can't reset your
  password; you'll get a recovery key instead." and [Create account]. When `server_info` isn't
  `Open`, the button is disabled and the line reads "This server isn't taking new accounts."
  `server_info` is asked again each time the form opens, one question per server at a time, and an
  answer counts only while the form still shows that server (A6).
- **Sign in.** Username, Password and Server rows, [Sign in], and "Forgot your password?", which
  switches to Username, Recovery key, New password, Confirm and [Set new password].
- **Session ended.** The Sign in form, prefilled, under the reason's sentence from the problems table.
- **While a task runs.** Inputs are disabled, and the button spins with the step: "Stopping sync…",
  "Securing your keys…", "Contacting the server…", "Saving…". The password fields are cleared as
  soon as their task starts.

**Recovery key screen,** after sign-up, recovery and "New recovery key":

- Title "Save your recovery key". Text: "If you forget your password, this key and your username are
  the only way back into your account. Kanso and the server's owner can't reset it. Anyone with it
  can take over your account and read your notes, so keep it somewhere safe, like a password manager
  or on paper. Each time it's used, Kanso gives you a new one."
- The key in large monospace.
- **[Copy]** goes through a small macOS helper, `clipboard::copy_secret`, built on `NSPasteboard`.
  It calls `prepareForNewContentsWithOptions(NSPasteboardContentsCurrentHostOnly)` so Universal
  Clipboard doesn't send the key to other devices. It adds the `org.nspasteboard.ConcealedType` and
  `org.nspasteboard.TransientType` markers, and clears the pasteboard after 60 s if its change count
  is still ours. Quitting sooner clears it too, on the same condition
  (`clipboard::clear_secret_on_quit`, from `on_app_quit`). It notifies "Copied. It leaves the
  clipboard in a minute."
- **[Save as file…]** opens a save panel suggesting `Kanso recovery key.txt` and writes a 0600 file
  holding the key, username, server, date and warning. It never overwrites silently. If the chosen
  folder syncs to iCloud (`NSURLIsUbiquitousItemKey`, or a path under `~/Library/Mobile
  Documents/`), it asks first: "This folder syncs to iCloud. Save the key there anyway?"
- **"I've saved my recovery key"** enables [Done], which deletes `sync-recovery-key:<library>` if it
  still holds that key.
- **A note over the key**, in the warning colour, when it needs one: after a password change, "Your
  password changed, and so did your recovery key: the one you saved no longer works." When the
  answer was lost, that Kanso couldn't tell whether the server took the key (or created the
  account, or changed the password), and to save it anyway. When the account was created or the
  password changed but this Mac couldn't finish signing in, that sentence, with "Sign in with your
  password" (or the new password).
- **Elsewhere.** A key that arrives while Settings › Account isn't showing (from a task, a failure
  carrying one, or the launch check) raises the notification "Save your new recovery key. It's on
  the Account page." with [Show key]; a second key replaces it.

**Signed in.** Sections:

- **Status row:** as today, with [Sync now].
- **Account:**
  - Username and server.
  - Storage, from `usage`, for all kinds of item: "850 items · 3.1 MB of 100 MB · attachments 14 MB
    of 500 MB (15 of 10,000)".
- **Sync:** Sync automatically and Sync every.
- **Devices:** active devices with their opened names ("A Mac" while unnamed), "This Mac", and "Last
  active 2 hours ago". Other devices have [Sign out], which confirms: "Sign out *name*? It stops
  syncing until someone signs in on it again. Its notes stay on it." Afterwards a notification
  says how it went: "Signed out *name*.", or the error's words (A5). A device the server no longer
  knows counts as signed out.
- **Security:**
  - Change password… (Current, New, Confirm), described as "Your other Macs sign out and ask for
    the new password. Your recovery key changes too: the one you saved stops working, and Kanso
    shows the new one to save." It leads to the key screen.
  - New recovery key… (Current password), which leads to the key screen.
  - Forgot your password? (Recovery key, New, Confirm).
- **Conflict handling:** as today.
- **Footer:**
  - [Sign out]: "Sign out of *ngoni*? Notes stay on this Mac. Sync stops until you sign in again."
    With an unsaved recovery key, the question starts "Your new recovery key hasn't been saved yet;
    signing out deletes it." When the Keychain refuses to delete the sign-in, Kanso is signed out
    all the same and says "Signed out. Kanso couldn't remove this Mac's sign-in from the Keychain,
    but it won't use it again."
  - [Delete account…]: "This deletes *ngoni*'s encrypted notes, attachments and devices from the
    server. Notes on this Mac stay here, and other Macs keep their copies. The server's backups keep
    a copy for up to 30 days. This can't be undone." Fields: "Type *ngoni* to confirm" and Password.

`dialogs.rs` gains three things:

- `form`: several fields, async submit, an inline error, and the dialog staying open while the task runs.
- `choose`: up to three described choices.
- `confirm_typed`: a confirm dialog that requires typing a word.

### Elsewhere in the app

- **Sidebar footer.** The username and its initials when signed in, the macOS user name when
  signed out. The sync line reads "Sign in to sync" (quiet; clicking it opens Settings › Account),
  or one of the labels in the problems table, or today's states.
- **First-run welcome** (`empty_state.rs`, used from `editor.rs`). While signed out, a row reading
  "Already use Kanso on another Mac? Sign in to sync", which dispatches `ShowSignIn`.
- **Palette** (`commands.rs`). `Needs` gains `SignedIn` and `SignedOut`.

  | Id | Label | Needs | Runs |
  |---|---|---|---|
  | `settings-account` | Settings: Account | — | `OpenAccountSettings` |
  | `sign-in` | Sign in to sync… | `SignedOut` | `ShowSignIn` |
  | `create-account` | Create account… | `SignedOut` | `ShowCreateAccount` |
  | `change-password` | Change password… | `SignedIn` | `ChangePassword` |
  | `new-recovery-key` | New recovery key… | `SignedIn` | `NewRecoveryKey` |
  | `sign-out` | Sign out | `SignedIn` | `SignOut` |
  | `sync` | Sync now | `SignedIn` | `SyncNow` |
- **Menu.** "Sync Now" is disabled while signed out.

### Plain-language errors

| Cause | Words |
|---|---|
| `invalid_credentials` (sign-in) | "That username and password don't match." |
| `invalid_credentials` (recovery) | "That recovery key doesn't match this account." |
| `invalid_credentials` (current password) | "That isn't your current password." |
| Recovery-key checksum | "That recovery key has a typo. Check it and try again." |
| `username_taken` | "That username is taken. Try another." |
| `invalid_username` | "Use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit." |
| `reserved_username` | "That username is reserved. Try another." |
| Password too short | "Use at least 12 characters. A few random words work well." |
| Password equals username | "Choose a password that isn't your username." |
| Confirm mismatch | "The passwords don't match." |
| `signup_closed` | `closed`: "This server isn't taking new accounts." `full`: "This server is full." `busy`: "This server has taken enough new accounts today. Try tomorrow." |
| `rate_limited` | "Too many attempts. Try again in *N* minutes." |
| `account_disabled` | "This account is turned off on the server. Ask the server's owner." |
| `KeyMismatch`, `UnsafeKdf` | "Something's wrong with the server's copy of your keys. Nothing was changed on this Mac." |
| Transport | "Can't reach the server. Your notes are safe on this Mac." |
| `ServerOutdated`, or a URL that doesn't answer `/v2/health` | "That server doesn't run this version of Kanso sync." |

Sync problems use the words in the problems table.

## Admin script

`npm run -s admin -- [--local] <command>` (`cloud/scripts/admin.ts`, planning in
`src/accounts.ts`). It runs through `wrangler d1 execute` (plus `wrangler d1 export`, `wrangler r2
object get`, `wrangler secret delete` and `wrangler d1 time-travel restore` where stated), with
every value validated and quoted:

```bash
CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN" npm run -s admin -- accounts
```

| Command | Effect |
|---|---|
| `accounts` | Each account's username, ID, created date, usage against quotas, writes today against its budget, bytes uploaded today, active devices, last seen and status. Also the settings, global sums and pending purges. |
| `disable <username>` / `enable <username>` | Set or clear `disabled_at`. |
| `delete <username> --confirm <username>` | The deletion batch, plus the purge row. |
| `signups open` / `signups close` / `signups only <username>` | `settings.signup`. `only` then checks whether that name already has an account, and says so: the reservation is spent once it does. Its help and answers say to run it immediately before that name signs up, and, if someone took the name first, to `delete <username> --confirm <username>` and run `only` again (security review 2, finding 6). |
| `max-accounts <n>` | Subject to the overcommit check unless `--force` |
| `quota <username> [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--writes-per-day N]`, `quota <username> --default` | Overrides, in MiB; overcommit check. `--writes-per-day` sets `quota_writes_per_day`, refused over `global_writes_per_day` unless `--force` |
| `default-quota [--docs N] [--doc-mib N] [--files N] [--file-mib N]` | Defaults; overcommit check |
| `writes-per-day <n>` / `upload-bytes-per-day <n>` | `settings.writes_per_day` / `settings.upload_bytes_per_day`, the per-account daily budgets |
| `global-writes-per-day <n>` / `global-upload-bytes-per-day <n>` | The server-wide daily caps |
| `recount [<username>]` | Recompute the counters from the tables, setting `recounted_at` |
| `restore <time \| bookmark> [--confirm <time \| bookmark>]` | The only way to restore D1 (security review 2, finding 10). The point is a Unix time in seconds or an RFC 3339 time with a zone, within the last 30 days, or a bookmark as `wrangler d1 time-travel info` prints it (8-8-8-32 lowercase hex). Refused with `--local`. Without `--confirm` repeating the point, it only prints what it would run. With it, it runs `wrangler d1 time-travel restore kanso-sync --timestamp=… \| --bookmark=… --json`, then at once `restored --all`, and prints the bookmark to undo with. If the new epochs fail, it says to run `restored --all` before any Mac syncs. It also says what a restore undoes: accounts and devices added since are gone, and every sign-out, deletion and password or recovery key change since is undone. |
| `restored [<username> \| --all]` | A new epoch (`lower(hex(randomblob(8)))`), so Macs re-merge (M1). `restore` runs it for every account; alone, it is for a restore that went wrong halfway. |
| `legacy-status` | Whether `docs`/`counter` exist, their live document and file counts and size, and the `files/` purge state |
| `legacy-export --dir DIR` | `manifest.json` (`{"exported_at", "rows": [id, rev, deleted, md5digest]}`), `legacy.sql` (`wrangler d1 export --table docs --table counter`), and every live legacy file from R2 into `DIR/files/` |
| `purge-legacy --account <username> --verified R.json` | Refuses unless `R.json` is `ok`, names that account, and its `manifest_sha256` equals the manifest hash (under "Command line") of a fresh export of the legacy rows (nothing changed since). Then drops `docs` and `counter`, queues the `files/` purge, and runs `wrangler secret delete SYNC_TOKEN` with stdin attached. |

The four daily commands refuse, unless given `--force`, a value after which one account's budget
(the default, any override, or the upload budget) is over its server-wide cap. They and
`max-accounts` print each server-wide cap's fair share (the cap ÷ `max_accounts`), how many
accounts at their whole budget reach the cap, and the day's bound of about twice the cap.

## Tests

The conventions hold: proptest with `PROPTEST_RNG_SEED=20261017`, fast-check with the pinned
`FAST_CHECK_SEED`, regression files kept, and example tests only as a supplement. The engine and
property tests that used to read server state directly (`remote.file(id).md5_hex` at
`tests/sync.rs:218` and `properties.rs:489`, `server_docs`, the plaintext `remote.push` at
`sync.rs:357` and `properties.rs:614`) use `MemoryServer::plain_view(&session) -> PlainView`. It decrypts the server's
documents and files under the test session and can push plaintext as another client.
`MemoryServer::drop_next_responses(route, n)` replaces `drop_next_push_responses` for any route.

| Area | Tests | Where |
|---|---|---|
| Crypto | Round trips for documents, files, wraps and device names. Every bit flip, wrong key, AAD change (account, wire ID, wire rev, deleted, kind, format byte) and shape mismatch is rejected. A format byte above 1 is "too new", not undecryptable. Rewrap keeps the data key; recovery rotates the recovery key; old KEKs can't open new wraps. NFC and NFD passwords derive the same key. Changing the origin or username in the salt input changes `auth_key`. | `kanso-sync/tests/crypto.rs` |
| Recovery key | Round trip; every single substitution and adjacent data transposition detected; lenient parsing | same |
| Golden vectors | `crypto-v1.json` (owner: Rust builder, step 2a): Argon2 at test parameters with a fixed origin and username, every HKDF info, both wraps, sealed document, file and name with fixed nonces, wire ID and rev, key ID, recovery-key text. One `#[ignore]` vector at production parameters. | `kanso-sync/tests/fixtures/` |
| KDF | `prop_prelogin_outside_floor_or_ceiling_is_refused` (no auth key is sent) | `tests/accounts.rs` |
| Usernames | Shared fixture; idempotent; case-insensitive; accepted names match the pattern; non-ASCII refused | Rust and `cloud/test/usernames.test.ts` |
| Generations (M2) | `prop_accepted_generations_only_grow` over random multi-Mac histories; a local win's new revision beats the remote one; replayed older revisions and tombstones come back `Stale`, except a live version a Mac never held over its deletion, which wins (review-verify 2); `Lost` and the re-merge's missing pass keep the floor | `kanso-store` tests, `tests/properties.rs` |
| Integrity of `synced_rev` | Add never pushes a tombstone over the account's live copy (`prop_add_never_pushes_a_tombstone_over_a_live_copy`, and the example for 0–3 edits on either side, notes and files); a restore never silently drops an edit (`prop_no_live_edit_is_lost_silently` with restores, and the two-Mac example); our own echo after a lost answer never reverts or flags (`prop_our_own_echo_never_reverts_an_edit`); missing bytes are fetched again within the bound (`prop_missing_bytes_come_back`); over quota, a refused batch isn't sent again until usage drops (`prop_over_quota_writes_only_grow_by_what_is_sent`) | `tests/integrity.rs` |
| Sync over encryption | The existing convergence and idempotence properties and `sync.rs` cases, run over `Sealed<MemoryServer>` with `PlainView` | `tests/properties.rs`, `tests/sync.rs` |
| Hostile server | `prop_a_hostile_server_never_changes_local_data_wrongly`, with a sequence of attacks per sync. A wrapper flips bits; swaps sealed values between documents and accounts; forges tombstones; relabels revisions; replays old versions; rewinds the account (lower `head_seq`); changes the epoch; leaves documents out of a re-merge; returns `current: null`; answers a push with an old version as the conflict's `current`; and sends unknown format bytes. Invariants: local content is always some client's write for that same ID; the floor never drops; a document's content only changes to a version above the floor it had before the sync; nothing pending is lost; replays are `Stale`; rewinds re-merge; a wrong key ID writes nothing; too-new never overwrites. | `tests/properties.rs` |
| Repair (M3) | A broken server copy (format 1) is replaced from the local copy; one bad document never stops sync | `tests/sync.rs` |
| Rebinding (fix 2) | `prop_rebinding_during_a_sync_never_mixes_accounts`: a remote that rebinds the library mid-sync (between any two calls); no write of A's lands after B's binding; B's cursor is never A's | `tests/binding.rs` |
| Binding | `prop_binding_plan` (model-based): notes reach another account only through an explicit add; Replace always leaves a backup with every document; sign-out never changes the library; sign-up into an empty account pushes no tombstones; a crash between any two steps of Replace leaves either the old library or an empty one bound fresh | `tests/binding.rs` |
| Rollback (fix 1 / H5) | `v2_then_unbind_then_v1_loses_no_edit`: a library synced under v1 rules, then v2 with edits, creations and deletions, then `unbind`, then v1 rules again. Every v2-window edit and creation reaches the v1 server; v2-window deletions come back (asserted, as the rollback steps say). v1 rules come from a plaintext legacy server in `tests/common/legacy.rs` (today's `MemoryRemote`, conformance-tested against the v1 Worker) and the cursor key `sync.last_seq`, so no live old Worker is needed. | `tests/rollback.rs` |
| Client retries | A lost response for sign-up, password change, recovery finish, new recovery key, push and file upload is recovered by the documented retry | `tests/accounts.rs` |
| Client quota and pause | Deletions still go through after a 413; a refused batch isn't sent again until usage drops, and then only what fits (`over_quota_syncs_spend_writes_only_on_what_the_server_stores`, and `prop_over_quota_writes_only_grow_by_what_is_sent`); a file-upload quota error holds new uploads; a 429 pauses and keeps its `limit`; `writes-global` is `ServerBusy`; an `uploads-global` 429 holds uploads without blocking pulls; a `session-account` 429 on the feed, a push, an upload or a download backs off briefly with the cursor kept and nothing marked done | `tests/sync.rs`, `tests/integrity.rs` |
| Keychain | An item with a different origin or account ID is ignored; sign-out deletes an unconfirmed recovery key; pending logouts are retried until 204 or 401 | `tests/accounts.rs` (with `MemorySecrets`) |
| Device identity | Same account → same ID; different accounts → unlinkable IDs; a hardware hash mismatch regenerates the identity and leaves the binding | `tests/binding.rs` |
| Plaintext canary | `prop_nothing_plain_leaves_the_mac`: every byte the client sends (push, files, auth, device name) is captured. No marker string, local ID, local rev, file MD5, password, recovery key, data key or device name appears in it, raw, hex or base64 (all alignments, standard and URL). | `tests/canary.rs` |
| Conformance | `conformance.json` (v2 sync cases, quotas and the global caps included) and `auth.json` (owner: Worker builder): same-device replacement, 11th-device eviction, password change signing others out with its reason, recovery rotating the key, logout then 401 `signed_out`, disabled after credentials, sign-up closed / full / busy, expiry, `account_id_taken`, the known-device proof, the idempotent sign-up retry (and the right auth key in a different request not being one), the per-account daily budgets and their overrides, and every rate-limit rule. Both run by the Worker and by `MemoryServer`; `tests/fixtures.rs` keeps their shapes equal to `api.rs` from step 1. The v1 cases in `conformance-v1.json` now only check the rollback test's stand-in for the old Worker. | `cloud/test`, `tests/conformance.rs` |
| Isolation (Worker) | b4e2733's two properties with three signed-up accounts. R2 keys stay under each account's own prefix. A device revocation can't reach another account's session, even one with the same device ID. | `cloud/test/isolation.test.ts` |
| Races (H1) | Through `handle(request, env, now)` with `Promise.all`: N parallel sign-ups at `max_accounts − 1` → exactly one succeeds; N parallel wrong logins → at most the rule's maximum are checked; parallel pushes and file uploads near every quota → no counter ever exceeds its quota; parallel uploads of one ID → `files` and `file_bytes` stay ≥ 0 and `recount` restores them | `cloud/test/races.test.ts` |
| Auth (Worker) | Every route and status; `prop_only_the_registered_auth_key_signs_in`; `prop_sessions_follow_the_model`; idempotent retries; nothing in D1 equals a submitted proof or token; prelogin is deterministic and shaped like a real answer; `Origin` → 403; wrong content type → 415; non-canonical base64url → 400 | `cloud/test/auth.test.ts` |
| Rate limits | `prop_limiter_matches_the_model` (fixed windows, counting first); known-device exemption; keys hold no raw IP or username; IPv6 grouping | `cloud/test/limits.test.ts` |
| Quotas | `prop_counters_equal_recomputed_sums`; `prop_growth_past_quota_is_refused_and_shrinking_always_passes`; the reported quota order; global caps; the row-cost measurement; the overcommit refusal | `cloud/test/quota.test.ts` |
| Migration | `prop_0002_leaves_legacy_rows_untouched`; settings seeded closed; the file applies through `applyD1Migrations`; e2e applies it through `wrangler d1 migrations apply --local` | `cloud/test/migration.test.ts`, e2e |
| Admin | Each command's SQL; the overcommit check; the `purge-legacy` manifest-hash guard; b4e2733's injection properties | `cloud/test/admin.test.ts` |
| Purges and cron | Purges finish within the round budget; a row is kept an hour after its prefix empties; limits rows expire; sessions expire at 180 days; revoked rows go after 90 | `cloud/test/purge.test.ts` |
| App | `prop_every_sync_problem_reads_plainly`, `prop_every_error_has_words`, form validation, the config v2 migration, and headless flows against `MemoryServer` + `MemorySecrets` (sign-up through Done, sign-in with each binding answer, session ended with each reason, password change, new recovery key, deletion, sign-out during a sync). Snapshots of every Account page state. | `kanso-app` |
| End to end | Below | `kanso-sync/tests/e2e.rs`, `cloud/scripts/e2e-local.sh` |

Existing tests that must change:

- `services.rs`: `syncing_two_libraries_reports_synced`.
- `config.rs`: `prop_migration_only_lifts_the_old_default_size`, which becomes a v2 property.
- `sidebar.rs`: the `SYNC_PROBLEMS` properties and `sync_line_examples`.
- `settings.rs`: `sync_status_examples`, `every_sync_problem_has_help` and
  `prop_sync_status_agrees_with_the_sidebar`.
- `palette.rs`: the tests that type "sync" and expect "Sync now" (about lines 2151 and 2478–2503).
  They now run signed in.
- `settings_window_tests.rs`: `sync_now_needs_a_worker` and `a_token_can_always_be_set`, which are
  replaced.

**End to end.** `cloud/scripts/e2e-local.sh`:

1. Make a temporary state directory, and beside it a directory for the canary list.
2. Apply the migrations with `wrangler d1 migrations apply kanso-sync --local --persist-to $dir`.
3. Set `signup` to `open` with `wrangler d1 execute --local`.
4. Start `wrangler dev --local --persist-to $dir --port $KANSO_E2E_PORT --test-scheduled --var
   AUTH_PEPPER:<test>` (port 8787 by default; a port something already listens on is refused, so
   the test never talks to another server). This is all Miniflare: no Cloudflare credentials.
5. Run `KANSO_SYNC_E2E_ENDPOINT=http://127.0.0.1:$port KANSO_SYNC_E2E_CANARIES=$list cargo test
   -p kanso-sync --test e2e -- --nocapture`.
6. Stop `wrangler dev`, then run the ignored test `the_stopped_state_holds_no_canary` with
   `KANSO_SYNC_E2E_STATE=$dir`. While the Worker runs, D1's recent writes aren't in its files yet;
   the scan's positive control caught that.

The test runs at production KDF parameters:

1. Two libraries sign up and sign in, then exchange notes, a conflict, a 2 MB image and deletions.
2. A password change gives the other device 401 `password_changed`.
3. Recovery rotates the key: the old key gets 401.
4. Wrong passwords end in 429 with `limit` `login-user` (the binding's 429s are waited out), and the
   known Mac then signs in anyway: its device proof exempts it.
5. A full-size push (4 MiB) and an 8 MiB changes page complete; their wall time is printed as the
   CPU bench.
6. Deleting the account gives 401, and its R2 objects are gone after `/__scheduled`. A second
   account, the witness, stays, with one file.
7. Finally, once the Worker has stopped, every file under the state directory (D1's SQLite and R2's
   blobs) is scanned for the canaries: the marker, the password, the recovery key, the local
   revision hashes and the deleted account's sealed files. As a positive control the scan must find
   the witness's username (stored as it is in D1) and its sealed file (in R2).

The `e2e` CI job runs the script on port 18787. After deploy, the Workers dashboard's CPU-time metric for `/v2/push` and
`/v2/changes` during the owner's first upload confirms the bench (expected well under 100 ms,
against a 30 s limit).

## Rollout

### Build order

**Step 1: the contract.** One PR, assembled by the Rust builder, with the Worker builder's files in
it. Merging it freezes the files below. After that, a change needs the review of every builder in
the "Consumers" column.

| Frozen file | Author | Consumers |
|---|---|---|
| `docs/ACCOUNTS.md` | — | all |
| `cloud/migrations/0002_accounts.sql`, applied by `wrangler d1 migrations apply --local` and by a minimal vitest | Worker | Worker, Rust (`MemoryServer` mirrors it) |
| `cloud/test/fixtures/usernames.json` | Worker | Worker, Rust |
| `cloud/test/fixtures/conformance.json` (v2) and `cloud/test/fixtures/auth.json`, checked against `api.rs` by `crates/kanso-sync/tests/fixtures.rs` | Worker | Worker, Rust |
| `crates/kanso-sync`: every public item in "Public API", with bodies `todo!()` except constants and pure validators. Also `wire.rs` types, `SyncReport` (new fields `skipped`, `stale`, `repaired`, `too_new`, `remerged`, `quota`, `paused_until`; `rejected` removed), `SyncProblem`, `problem`, `report_problem`, the binding types (`LocalLibrary`, `Flow`, `Plan`, `Choice`, `local_library`, `plan`, `bind`, `unbind`, `DeviceIdentity`, `device_identity`), the keychain types (`SecretStore`, `Keychain`, `MemorySecrets`, save/load/delete), and `MemoryServer` with its controls (below). `Send + Sync` on `Backend` and `Session`. | Rust | App |
| `crates/kanso-store`: `SyncGuard`, `ApplyOutcome`, `StoreError::GuardFailed`, `reset_sync`, `replace_library`, `mark_unsynced`, `mark_missing_unsynced`, `set_meta_if`, `pending_count`, and the guard parameter on `apply_remote`/`mark_synced`, with call sites updated. Bodies may be `todo!()` only where no current test calls them. | Rust | App, Rust |

`MemoryServer` controls: `new()`; `set_now(ms)`; `set_setting(name, value)`;
`set_signup(Signup::{Open, Closed, Only(name)})`; `set_max_accounts`; `set_default_quota`;
`set_quota(username, …)`; `set_writes_per_day`; `disable`/`enable(username)`; `devices(username)`;
`revoke_all(username, reason)`; `expire_sessions(unseen_for)`; `set_epoch(username)` and
`rewind(username, to_seq)` (restores); `drop_next_responses(route, n)`; `stats()`;
`plain_view(&Session)`.

**Contract notes.** Decisions made while freezing step 1, binding on every builder. The sections
above already say each of them; this list is the index.

*Security re-check, applied in step 1:*

1. **Settings fail closed.** Every trigger reads its setting with `coalesce(…, 0)` (`'closed'` for
   `signup`); `settings_keep` and `daily_keep` refuse deleting a row, and renaming one is refused
   too. Proven in a scratch database with the guards dropped: each missing setting refuses.
2. **Server-wide daily caps.** Settings `global_writes_per_day` (300,000) and
   `global_upload_bytes_per_day` (5,000,000,000; uploads weighted by size, replacements included).
   The `daily` table holds the UTC day's sign-ups, document writes and upload bytes; only triggers
   write it, so deleting accounts can't lower a count. `signups_per_day` is per UTC day. Past a cap:
   429 `rate_limited` with `limit` `writes-global` or `uploads-global` and `retry_after` to the next
   UTC midnight.
3. **The same account is the same `(origin, account_id)`.** Meta key `sync.v2.origin`;
   `SyncGuard { expect }` over both (`binding::guard_pairs`); `local_library(store, origin,
   account_id)`; `keychain::load_session` and `load_recovery_key` take both, and the recovery item
   holds `origin`; `AccountRef` gains `origin`. The wrap AAD gains `lp(account_id)`.
4. **Re-merge keeps `synced_rev`** and pulls from 0. `Stale { new_rev }` rebases the local version
   and pushes it like a local win; `mark_missing_unsynced` re-pushes synced documents (tombstones
   included) the full feed lacks, with `base_rev = null`.
5. **Device proof.** `HKDF(device_secret, "kanso/v1/device-proof" || lp(origin) ||
   lp(account_id))`, sent as `device.proof` at sign-up, sign-in and recovery; `sessions.proof_hash`
   holds its SHA-256. Only a matching proof exempts a sign-in from `login-user`. Types:
   `crypto::DeviceProof`, `DeviceIdentity::device_proof`, `api::NewDevice { id, sealed_name, proof }`,
   `api::LoginDevice { id, proof }`.
6. **Legacy manifest hash**: rows only, sorted by `id`, the byte format and test vector under
   "Command line"; `verify::manifest_sha256` implements it.
7. **Sign-up retry is idempotent on the server**: the same request again (username, account ID,
   KDF parameters, salt, auth key, wrapped keys, recovery auth key and key ID) → 201 with the
   existing account, whatever the settings. The right auth key with anything else different is not
   a retry. `auth.json` has both cases.
8. **R2 and D1 drift**: GET needs the `account_files` row; the cron reconciles orphans and sizes.
   The only schema addition is `account_files.uploaded_at`, which the upload cap needs anyway.
9. `login-user` is keyed by account ID, with one shared row for unknown names; the binding groups
   IPv6 by /48; `limits` rows go when their window ends.
10. Automatic repair only when the broken wire revision is `wire_rev(id, synced_rev)`, pushed one
    generation higher; anything else is skipped and left to `verify --repair`. The store method for
    that bump is the Rust builder's to add (additive, not frozen).
11. Credential updates (recovery finish, password change, new recovery key) are compare-and-swap.
12. Deletion: the client counts only 401 `unknown` as success. The login batch keeps at most 90
    revoked rows per account.
13. `signups only <name>` checks for an existing account afterwards; `check_endpoint` refuses a
    host ending in a dot.
14. `DEFAULT_SERVER` stays `https://kanso-sync.imngonii.workers.dev`.

*Other step-1 decisions:*

- **Fixtures.** The v1 file is renamed `conformance-v1.json` and still runs against the v1 Worker
  and `MemoryRemote` until v1 code goes. `conformance.json` is v2. The step language is in each
  file's `about`. Every case starts with sign-ups open and the clock at a UTC midnight. Raw steps
  are Worker-only; `MemoryServer` runs the rest through `Api`. The expected values were checked
  against an independent model before freezing.
- **What the fixtures pin.** Health says `closed` for `closed` and `only:`, `full` when open at
  `max_accounts`, else `open`. A missing or malformed bearer is 401 `unknown`. Revoking the current
  device is a logout (`signed_out`). Sign-up triggers answer before `UNIQUE`, and
  `account_id_taken` before `username_taken`. Deletion checks the key before the username. Device
  proofs must be canonical like auth keys. `login-ip` counts before `login-user`, and the first
  over its maximum is named. `retry_after` is the seconds to the window's end, rounded up.
  `quota_exceeded` with `server` reports the server-wide sum and cap. Push validates all but the
  sealed alphabet before counting `writes-account`; a refused body counts 1, and an empty push
  counts nothing (security review 2); file uploads check
  path and headers before counting. `limit` outside 1–500, or a negative `since`, is 400.
- **`MemoryServer`.** `new()` starts with sign-ups open and the migration's other settings
  (`DEFAULT_SETTINGS`, checked against the migration by a test). `set_setting` panics where D1's
  `CHECK` would refuse (`is_valid_setting`). `set_now` fixes its clock.
- **API surface.** `Api` has every v2 route with explicit tokens, so the conformance runner can
  replay any token; `WireRemote` is the session-bound sync subset, and `Backend::remote` defaults
  to an adapter over `Api`. `Route` names are the fixtures' route names.
- **Account.** `Step` gains `StoppingSync`. `AccountError` gains `Protocol`, `Endpoint` and
  `Keychain`. `check_new_username` and `UsernameError::Reserved` refuse reserved names on the
  client. `kanso_sync::origin(server)` computes the origin.
- **Binding.** `Machine { hardware_uuid, computer_name }` feeds `device_identity`, so tests can fake
  the Mac. `device_name` trims again after cutting, so it is idempotent. `bind(store, session,
  choice, backups_dir, now_ms)` returns the backup's path after a Replace; `Cancel` binds nothing.
  `local_library` checks Same, Empty, Other, Unbound, in that order. The meta keys live in
  `kanso_sync::meta_keys`.
- **Keychain.** `set_pending_logouts` replaces the list; `PendingRecoveryKey` and `PendingLogout`
  are the item types.
- **Engine.** `SyncReport.skipped` holds the wire IDs of undecryptable documents not repaired;
  `stale` and `repaired` aren't problems. `report_problem` precedence: `UpdateKanso`, `Paused`,
  `ServerFull`, `QuotaExceeded`, `Skipped`. `problem` maps a 429 to `Other`; the engine itself turns
  a push or upload 429 into `paused_until`. The v1 engine (`sync_once`, `sync_with`, `MemoryRemote`,
  `HttpRemote`, `credentials`, `SyncConfig`) keeps working until 2b removes it.
- **Left to the Rust builder:** `verify` gains `repair` and `VerifyReport.repaired`; `PlainView` has
  inherent methods. `Sealed<W>`, the plaintext types in `remote.rs` and crypto internals beyond
  `Password`, `RecoveryKey`, `KdfParams` and the key types are internal and may change.
- **Implemented in step 1** (pure or trivial): username, password and recovery-key validation; wire
  patterns and `ApiError` parsing; `problem`, `report_problem` and `schedule`; `plan`;
  `device_name`; `manifest_sha256`; `origin`; the guard helpers, `mark_unsynced`,
  `mark_missing_unsynced`, `set_meta_if` and `pending_count`; `MemorySecrets`. `LocalWon.new_rev`
  is the unchanged local revision until 2a adds the generation bump.

*Step 3 (integration) decisions:*

- **v1 is gone.** The plaintext engine, `HttpRemote`, `MemoryRemote`, the shared-token Keychain
  code, `SyncConfig`, `CURSOR_KEY` and the legacy tests are deleted; the Worker serves no `/v1`.
  `kanso-sync unbind` (the rollback path) stays. The rollback test's stand-ins for the old app and
  Worker live in `tests/common/legacy.rs`, checked against `conformance-v1.json`; the old app's
  store rules are behind `kanso-store`'s `legacy-v1-rules` feature, which only `kanso-sync`'s
  dev-dependencies enable.
- **Daily budgets.** `writes_per_day` is 20,000 (a fifteenth of `global_writes_per_day`, 300,000);
  `accounts.quota_writes_per_day` (NULL: the setting) overrides it per account, set with `quota
  <user> --writes-per-day N`, and `/v2/account` reports the effective value. The per-account
  upload cap is the setting `upload_bytes_per_day` (1 GiB), rule `uploads-account`, counted from
  `Content-Length` after `writes-account` passes; its 429 pauses sync like any push or upload 429.
  Both maxima are read from D1 in the counting batch, and a missing value refuses. The admin
  script has `upload-bytes-per-day`, and its four daily commands refuse, unless `--force`, a value
  that lets one account's budget exceed the server's cap. Amended in `0002_accounts.sql` (not yet
  applied anywhere), the fixtures, `MemoryServer` and the Worker together.
- **Sign-up retry** (note 7) is the same request again, not just the same auth key; `MemoryServer`
  now checks every field as the Worker does.
- **Cancelled binding questions** log the new session out with `kanso_sync::discard_session`,
  which leaves `sync-account` and `sync-recovery-key` alone (`sign_out` deleted them: the signed-in
  account's session, or a cancelled recovery's new key). A failed sign-up still deletes the
  recovery key it saved, which nobody saw. An account task that stopped the sync and leaves the
  same session signed in starts it again.
- **The editor and sync revisions.** A local win, a stale rebase or a repair gives the open note a
  new `_rev` with the same content (`Change` with `Origin::Sync`). `editor::same_content`
  compares versions apart from `_rev` and the derived task counts; the editor adopts such a
  revision without asking, unsaved typing and all, and a save that meets one first retries once
  on top of it.
- **Tests.** The account flow tests run the real sync (`Services::no_sync` is gone);
  `services.rs` has its two-library test back over `MemoryServer`. The e2e canary scan runs after
  the Worker stops, with a positive control, and the wrong-password step asserts `login-user`.

**Step 2: in parallel.**

- **W, the Worker builder.** Everything in `cloud/`, against the frozen migration and fixtures.
  Done when `npm test` and `npm run typecheck` are green.
- **R, the Rust client builder.**
  - *2a*: `crypto.rs`, `username.rs`, `account.rs`, `binding.rs`, `keychain.rs`, `MemoryServer`,
    the store APIs, and `crypto-v1.json`. Its tests are green and `tests/conformance.rs` passes
    `auth.json`.
  - *2b*: `sealed.rs` and the engine changes, `http.rs`, `verify.rs`, the CLI, and the canary,
    rebinding, rollback and hostile-server tests. `cargo test -p kanso-sync -p kanso-store` is green.
- **A, the app builder.**
  - *A1*, from step 1: config v2, services, `account_page.rs`, dialogs, sidebar, welcome, palette,
    menu, the clipboard helper, and snapshots, using the stubs and stub states.
  - *A2*, after Rust 2a merges: the headless flow tests against `MemoryServer`, and the existing
    tests listed above.

**Step 3: integration.** `cloud/scripts/e2e-local.sh` is green (needs W and 2b). Then the app
against `wrangler dev` by hand, and CI green: fmt, clippy `-D warnings`, workspace tests, worker,
e2e.

**Step 4: deploy and migrate**, below.

### Deploy and the owner's migration

0. **While v1 still works.**
   1. Tag the current head `sync-v1`.
   2. In a worktree of `sync-v1`, on every Mac, quit Kanso and run `cargo run --release -p
      kanso-sync -- --data-dir DIR --endpoint https://kanso-sync.imngonii.workers.dev --full`.
      Repeat until the report shows `pending`, `files_missing` and `invalid` at zero.
   3. Reopen Kanso and don't edit until step 7 (H4, fix 10).
1. **Backups.**
   1. Settings › General › Back up now.
   2. Copy `/Applications/Kanso.app` to `Kanso-old.app`.
   3. Record the D1 Time Travel bookmark (`npx wrangler d1 time-travel info kanso-sync`).
   4. Confirm `wrangler d1 migrations list kanso-sync --remote` shows only `0001_init`.
   5. Run `npm run -s admin -- legacy-export --dir ~/KansoLegacy-<date>`. Keep that folder with
      your backups; it holds the old plaintext.
2. **Pepper.** Set `AUTH_PEPPER` as above.
3. **Migrate.** `npm run migrate:remote` applies `0002_accounts` (additive; sign-ups closed). The old
   Worker keeps working.
4. **Deploy the Worker.** `npm run deploy`, with the cron, the rate limiter and invocation logs off.
   From here `/v1` answers 404.
5. **Install the app.** Quit Kanso, then `scripts/bundle-macos.sh` and copy it to `/Applications`.
   It opens signed out with every note in place. Run `kanso-sync estimate` and compare it with the
   quotas.
6. **Create the owner's account.**
   1. Open Settings › Account › Create account and fill it in, but don't submit yet.
   2. Immediately before submitting: `npm run -s admin -- signups only <you>`. Until the account
      exists, anyone who guesses the name can take it, so leave no gap.
   3. Submit. Save the recovery key, then Done.
   4. If the name was taken (the app says so, or `signups only` said the reservation is spent
      before you signed up), someone else got it first: `npm run -s admin -- delete <you> --confirm
      <you>`, then repeat from 2.
   5. If the estimate exceeded a default, run `quota <you> …` and Sync now.
7. **Verify.** Wait for "Synced". Quit Kanso, then run `cargo run --release -p kanso-sync -- verify
   --data-dir ~/Library/Application\ Support/Kanso --legacy ~/KansoLegacy-<date>/manifest.json
   --out ~/KansoLegacy-<date>/verified.json`. It must exit 0.
8. **Other Macs, one at a time.** Install the new build, Sign in, then choose [Add them to *you*].
   They were synced in step 0, so this only adds their local-only changes. Then run `kanso-sync
   verify` on each.
9. **Purge the legacy data.** `npm run -s admin -- purge-legacy --account <you> --verified
   ~/KansoLegacy-<date>/verified.json`. Within 15 minutes, `legacy-status` shows the `files/` purge
   done. Time Travel still holds the purged plaintext for 30 days, and `~/KansoLegacy-<date>` holds
   it for good.
10. **Open sign-ups:** `npm run -s admin -- signups open`.
11. **Clean up.**
    1. `security delete-generic-password -s Kanso -a sync-token`.
    2. Update `cloud/DEPLOYED.md`, `cloud/README.md` and `README.md`.
    3. Delete `Kanso-old.app`.

**Other people's Macs.** The builds are ad-hoc signed and not notarized, so:

- **Gatekeeper.** On first launch, Control-click the app and choose Open, or use System Settings ›
  Privacy & Security › Open Anyway.
- **Keychain.** Each new build has a different signature, so macOS may ask once per build to let
  Kanso use its Keychain items. Choose Always Allow.

`README.md` says this.

### Rollback

- **Before step 9.** The legacy tables, `SYNC_TOKEN` and the Keychain `sync-token` item are intact.
  1. On every Mac, quit Kanso and run `cargo run --release -p kanso-sync -- unbind --data-dir DIR`
     (from the new commit). This removes the `sync.v2.*` account keys and the Keychain item, sets
     live `synced_rev` to NULL, and sets the legacy `sync.last_seq` to 0.
  2. `npx wrangler rollback` to the previous Worker version.
  3. Reinstall `Kanso-old.app`. Its Worker URL is still in `config.json` (`endpoint` was preserved);
     re-enter it if missing.

  The old app pulls everything and pushes every change made during the v2 window. Newer edits win,
  and both versions are kept. Notes deleted during the v2 window come back; delete them again. The
  v2 tables stay, unused.
- **After step 9.** There is no return to plaintext sync; fix forward. `npm run -s admin --
  restore <step 1 bookmark>` (within 30 days) restores the database and then makes every Mac
  re-merge. The files are in `~/KansoLegacy-<date>`.
- **Restoring D1, at any time.** Only through `admin restore <time | bookmark>`, never with
  `wrangler d1 time-travel restore` or the dashboard. A restore alone rewinds every account's feed
  while each Mac still trusts its cursor and epoch, so pushes and pulls would go wrong silently;
  `restore` gives every account a new epoch at once, which makes each Mac re-merge (M1). A
  restore also brings back the sessions, passwords and recovery keys of that time: a device token
  revoked since works again until it is revoked again, and anyone who changed a password or
  recovery key since must change it again. Run it first without `--confirm` to see what it would
  do.
- **Client-only problems.** Sign out (notes stay), install a fixed build, and sign in. "Same"
  resumes.

## Threat model

Protected against: a curious or compromised server or owner; a leaked D1 database, R2 bucket or
Time Travel copy; network observers; malicious web pages (M7); and a malicious second Kanso server
(M5).

- They get ciphertext and the metadata listed above.
- A D1 leak alone doesn't allow signing in: auth hashes are hashes of 256-bit keys, and session
  tokens are stored hashed.
- Writes are authenticated, so the server can't forge, alter, relabel, cross-link or move documents,
  files, deletions or device names without the Mac noticing.
- Replaying an older version is refused once a Mac has seen a newer one (M2), also after the server
  claims it lost the document or rotates its epoch. The exception is a live version that a Mac
  never held, over its deletion: an edit beats a deletion (see the stale reads below).
- Rewinding a whole account makes the Macs re-merge rather than lose changes (M1).

Residual risks:

- **Offline password guessing.** Anyone holding an account's D1 row (the owner, a D1 leak, or Time
  Travel within 30 days) can guess passwords against the wrapped data key, at one Argon2id (1 GiB,
  t = 3) per guess. A weak password falls. Mitigations: the 12-character minimum, the passphrase
  hint, and memory-hard parameters.
- **Password changes don't replace the data key (M4).** Anyone who once knew the password and
  fetched the wrapped key, or who read the data key from a signed-in Mac, can decrypt anything they
  later obtain from the server, until a rekey. v1 has no rekey; see "Deferred". A password change
  does replace the recovery key, in the same update (security review 2, finding 1): a recovery key
  that such a person set while they knew the password stops working, so they can't take the
  account over by recovery after the owner changes it. What remains is reading, not control.
- **Stale reads and withholding.** A malicious server can withhold changes, delete data, or replay
  an old version to a Mac that never saw a newer one. It can bring a deleted note back on a Mac
  with a version another Mac wrote that this Mac never held, since an edit always beats a deletion
  (review-verify 2). It can't fabricate content, and every replaced local version is kept in history.
- **Metadata.** Sizes, counts per kind, timing and IPs are visible, as listed above.
- **The client is trusted.** Friends run ad-hoc builds the owner makes. End-to-end encryption
  protects them against the server, but not against whoever builds the app: a modified build could
  send keys anywhere. Anyone who doesn't trust the owner should build Kanso from source, and can
  point it at their own Worker.
- **The recovery key.**
  - It is as powerful as the password. It sits in the Keychain until the user confirms it's saved,
    and wherever the user puts it after that. The clipboard copy stays on this Mac and is cleared
    after a minute, and saving to iCloud warns.
  - A used key stops working, and so does every key from before a password change.
  - A key replaced on another Mac with "New recovery key" can still show on this one as unsaved:
    that change signs no session out, and the Worker gives no key ID to compare. Saving it saves a
    key that no longer works; the one the other Mac showed does. A password change or a recovery
    elsewhere ends this Mac's session, and the unsaved key goes with it (sec-client 3).
- **A compromised Mac.** The local library is plaintext, and the Keychain holds the data key and the session.
- **Lost password and recovery key.** The server copy is unrecoverable, by design. Local copies remain.
- **Lockout.** Up to 10 sign-in attempts per account per 15 minutes from devices that aren't
  signed in can be used up by anyone. Signed-in Macs, which prove themselves with their device
  proof, are unaffected (M8, security re-check 5). A Mac that was signed out (by itself, another
  Mac, a password change, recovery, eviction or expiry) counts like a stranger until it signs in
  again, so an old copy of its library can't be used to guess passwords past the limit.
- **Floods and cost.** Without a custom domain and a WAF rule, a large flood can drive Workers
  request costs. Per-account write budgets and `SESSION_LIMITER` cap what one account costs, the
  global caps bound storage, and the server-wide daily caps bound a day's writes and uploads at
  about twice each cap. A few accounts at their own budgets can no longer pause everyone else's
  sync: an account within its fair share is never refused by a server-wide cap.
- **Unaudited crates.** The crypto crates are recent RustCrypto releases (argon2 0.6,
  chacha20poly1305 0.11), pinned in `Cargo.lock`.

## Deferred (v1 behaviour stated)

- **Rekey (M4).** v1 has no way to replace the data key. Later: `POST /v2/account/rekey`, taking
  `current_auth_key` plus new `kdf`, `salt`, `auth_key`, `wrapped_key`, `recovery_auth_key`,
  `recovery_wrapped_key` and `key_id`. In one batch it would:
  1. Delete the account's documents and files (and queue the R2 purge).
  2. Replace both wraps, both hashes and the key ID, with a new epoch.
  3. Revoke every other session (`rekeyed`).

  The calling Mac then unbinds, rebinds fresh and re-uploads. The schema, the `rekeyed` reason and
  the epoch logic already support it; it needs no format change.
- **Moving a server's URL.** v1 binds accounts to their server's origin (M5) and has no flow for a
  changed URL; owners must keep the URL. Later, a move would work like a password change:
  connected through the new URL, derive the current key with the old origin from the Keychain item,
  and the new one with the new origin. No format change.
- **Sharing, email, 2FA, web client:** out of scope.

## Critical Files for Implementation

**Step 1, contract**

- Rust builder:
  - `crates/kanso-sync/src/{lib.rs, wire.rs, api.rs, account.rs, crypto.rs, username.rs, binding.rs, keychain.rs, memory.rs, remote.rs, engine.rs, sealed.rs, verify.rs, http.rs}` (stubs and types)
  - `crates/kanso-sync/tests/fixtures.rs` (fixture shapes)
  - `crates/kanso-store/src/{sync.rs, lib.rs}` (store API signatures)
- Worker builder:
  - `cloud/migrations/0002_accounts.sql`
  - `cloud/test/fixtures/{usernames.json, conformance.json, auth.json}`; `conformance-v1.json` (renamed)
  - `cloud/test/migration.test.ts` (minimal), with `MIGRATION_DB` in `cloud/vitest.config.ts` and `cloud/test/env.d.ts`

**Worker builder**

- Create:
  - `cloud/src/auth.ts` (prelogin, sign-up, login, recovery, logout, password, recovery key, delete, devices, session lookup)
  - `cloud/src/usernames.ts`
  - `cloud/src/limits.ts` (binding filter and counting upsert)
  - `cloud/src/sync.ts` (push, changes and files, with quota error mapping)
  - `cloud/src/purge.ts` (purges and the cron jobs)
  - `cloud/scripts/e2e-local.sh`
- Change:
  - `cloud/src/index.ts` (v2 router, `Origin` and content-type checks, `handle(request, env, now)`, `scheduled`)
  - `cloud/src/protocol.ts` (v2 patterns and limits)
- From b4e2733, rewritten: `cloud/src/accounts.ts` (admin planner, overcommit check, legacy commands), `cloud/scripts/admin.ts`, `cloud/scripts/tsconfig.json`
- Config: `cloud/wrangler.jsonc`, `cloud/package.json`, `cloud/vitest.config.ts`
- Tests:
  - `cloud/test/{helpers.ts, env.d.ts, setup.ts, worker.test.ts, conformance.test.ts, properties.test.ts}`
  - `cloud/test/{isolation.test.ts, auth.test.ts, usernames.test.ts, limits.test.ts, quota.test.ts, races.test.ts, migration.test.ts, admin.test.ts, purge.test.ts}`
- Docs and config: `cloud/README.md`, `cloud/DEPLOYED.md`, `.gitignore`, `.github/workflows/ci.yml` (e2e job)

**Rust client builder**

- Workspace: `Cargo.toml` (crypto dependencies), `crates/kanso-sync/Cargo.toml`
- 2a:
  - `crates/kanso-sync/src/{crypto.rs, username.rs, account.rs, binding.rs, keychain.rs, memory.rs, api.rs}`
  - `crates/kanso-store/src/{sync.rs, write.rs, lib.rs}` (`reset_sync`, `replace_library`, guard, `Stale`, generation bump)
  - `crates/kanso-sync/tests/{crypto.rs, accounts.rs, binding.rs, conformance.rs}`
  - `crates/kanso-sync/tests/fixtures/crypto-v1.json`
- 2b:
  - `crates/kanso-sync/src/{sealed.rs, wire.rs, remote.rs, engine.rs, http.rs, verify.rs, bin/kanso-sync.rs}`
  - `crates/kanso-sync/tests/{common/mod.rs, common/legacy.rs, sync.rs, properties.rs, canary.rs, rollback.rs, e2e.rs}`
- Delete: `crates/kanso-sync/src/credentials.rs`

**App builder**

- Create:
  - `crates/kanso-app/src/account_page.rs`
  - `crates/kanso-app/src/clipboard.rs` (`copy_secret`)
- Change:
  - `crates/kanso-app/src/config.rs`
  - `crates/kanso-app/src/services.rs`
  - `crates/kanso-app/src/settings.rs`
  - `crates/kanso-app/src/dialogs.rs`
  - `crates/kanso-app/src/sidebar.rs`
  - `crates/kanso-app/src/empty_state.rs` and `crates/kanso-app/src/editor.rs`
  - `crates/kanso-app/src/commands.rs` and `crates/kanso-app/src/palette.rs`
  - `crates/kanso-app/src/workspace.rs` (actions `OpenAccountSettings`, `ShowSignIn`, `ShowCreateAccount`, `ChangePassword`, `NewRecoveryKey`, `SignOut`; menu)
  - `crates/kanso-app/src/main.rs` (`HttpBackend`, startup reconciliation, `flush_pending_logouts`)
- Tests: `crates/kanso-app/src/{snapshots.rs, settings_window_tests.rs, ui_tests.rs}`
- Docs: `README.md` (accounts, Gatekeeper and Keychain prompts), `docs/PLAN.md` (Sync section points here)
