# Kanso sync Worker

A Cloudflare Worker that Kanso clients sync through, protocol v2: accounts with end-to-end
encrypted libraries. Each Mac seals documents and files with a key the server never sees; the
Worker stores and returns those sealed values per account. D1 holds accounts, sessions, documents
and the counters behind quotas and rate limits; R2 holds sealed file bytes under
`accounts/<account_id>/<wire_id>`. The full design is [docs/ACCOUNTS.md](../docs/ACCOUNTS.md).

| Route | Session | |
|---|---|---|
| `GET /v2/health` | | Liveness, protocol version, whether sign-ups are open |
| `POST /v2/auth/prelogin` | | KDF parameters, salt and account ID for a username |
| `POST /v2/auth/signup` | | Create an account and this device's session |
| `POST /v2/auth/login` | | Sign in a device |
| `POST /v2/auth/recover/start`, `/finish` | | Recover with the recovery key, setting a new password |
| `POST /v2/auth/logout` | yes | Sign out this device |
| `GET /v2/account` | yes | Account, usage and quotas |
| `POST /v2/account/password`, `/recovery-key`, `/delete` | yes | Change the password, replace the recovery key, delete the account |
| `PUT /v2/account/device`, `GET /v2/account/devices`, `DELETE /v2/account/devices/<id>` | yes | Device names, the device list, signing one out |
| `POST /v2/push` | yes | Up to 200 sealed documents, each with the revision it was based on |
| `GET /v2/changes?since=&limit=` | yes | The account's change feed |
| `PUT` / `GET /v2/files/<wire_id>` | yes | Sealed file bytes, checked against `x-kanso-sha256` |

A session is `Authorization: Bearer kanso_…`. `/v1` and every other path answer 404. Any request
with an `Origin` header gets 403, and a body with the wrong `Content-Type` gets 415. There is no
admin route: accounts are administered from the command line (below).

## Configuration

`wrangler.jsonc` binds the D1 database `kanso-sync` (`DB`), the R2 bucket `kanso-files` (`FILES`)
and the rate limiter `AUTH_LIMITER` (20 requests a minute per IP group, in front of `/v2/auth/*`
and `/v2/health`). It runs the cron every 15 minutes (purges, expired rate-limit rows and
sessions, R2 reconciliation) and keeps invocation logs off; the Worker itself logs only route
names and error codes.

One secret is required, `AUTH_PEPPER`: 32 random bytes as base64url. It keys the fake answers
`prelogin` gives for unknown names and the rate-limit keys of IP addresses. Without it, health and
the sign-in routes answer 503 `misconfigured`; signed-in devices keep working. `SYNC_TOKEN` is no
longer read.

## Deploying

Needs Node, and a Cloudflare API token that can edit Workers, D1 and R2. The token is only ever
referenced by name; never write it into a file.

```bash
cd cloud
npm ci
export CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN"

# First deployment only: create the database and bucket, and put the printed
# database_id in wrangler.jsonc.
npx wrangler d1 create kanso-sync
npx wrangler r2 bucket create kanso-files

# The pepper, once (read from stdin, never echoed or saved).
pepper=$(openssl rand 32 | base64 | tr '+/' '-_' | tr -d '=\n')
printf %s "$pepper" | npx wrangler secret put AUTH_PEPPER
unset pepper

npm run migrate:remote     # 0002_accounts is additive; sign-ups start closed
npm run deploy
```

Losing the pepper only changes the fake prelogin answers and resets rate-limit counters, so a
backup in a password manager is optional.

Moving the existing single-user deployment to accounts (legacy export, verification, the purge of
the old plaintext) follows the rollout in [docs/ACCOUNTS.md](../docs/ACCOUNTS.md#rollout). The current
deployment is listed in [DEPLOYED.md](DEPLOYED.md).

## Administering accounts

```bash
CLOUDFLARE_API_TOKEN="$IMNGONII_CLOUDFLARE_API_TOKEN" npm run -s admin -- accounts
npm run -s admin -- --local accounts        # the local database (wrangler dev's), no token needed
```

| Command | |
|---|---|
| `accounts` | Every account's usage against its quotas, writes today against its budget, bytes uploaded today, devices, last seen and status; the settings, global sums and pending purges |
| `settings` | The settings |
| `signups open` / `signups close` / `signups only <username>` | Open sign-ups, close them, or reserve them for one name |
| `disable <username>` / `enable <username>` | Refuse, or allow again, the account's sign-ins and sessions |
| `delete <username> --confirm <username>` | Delete the account's rows and queue its R2 purge |
| `max-accounts <n> [--force]` | At most n accounts |
| `quota <username> [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--writes-per-day N] [--force]` | Per-account quotas, including the account's own daily write budget; `quota <username> --default` goes back to the defaults |
| `default-quota [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--force]` | The default quotas |
| `writes-per-day <n> [--force]` | Documents pushed plus files uploaded, per account per UTC day (the default) |
| `upload-bytes-per-day <n> [--force]` | Uploaded file bytes, per account per UTC day |
| `global-writes-per-day <n> [--force]` / `global-upload-bytes-per-day <n> [--force]` | Server-wide daily caps |
| `recount [<username>]` | Recompute the usage counters from the tables |
| `restored [<username> \| --all]` | After a D1 Time Travel restore: a new epoch, so each Mac re-merges |
| `legacy-status` | The single-user tables and the `files/` purge |
| `legacy-export --dir DIR` | `manifest.json`, `legacy.sql` and every live legacy file, into DIR |
| `purge-legacy --account <username> --verified R.json` | Drop the single-user tables, queue the `files/` purge and delete `SYNC_TOKEN`, once `R.json` (from `kanso-sync verify --legacy`) proves everything arrived |

**Daily budgets.** Each account may push documents plus upload files up to `writes_per_day`
(20,000) per UTC day, and upload at most `upload_bytes_per_day` (1 GiB, 1,073,741,824 bytes) of
file bytes, refused uploads and replacements included. The server as a whole stores at most
`global_writes_per_day` (300,000) document writes and `global_upload_bytes_per_day`
(5,000,000,000) upload bytes a day. So fifteen accounts at their write budget, or five at their
upload budget, use up the server's day for everyone until UTC midnight; each daily command prints
that count. Past its budget an account's Macs show "Sync paused" and carry on after UTC midnight.
A library that needs more in one day (a first upload or a re-merge of a large library) can be given
its own budget with `quota <username> --writes-per-day N`; the owner does this for their own
account rather than raising the default. The daily commands refuse, unless given `--force`, a
value that lets one account's budget exceed the server's.

`max-accounts`, `quota` and `default-quota` refuse a change after which the quotas would promise
more than a global cap, unless given `--force`. Every value is validated before it reaches SQL.
The exit status is 0 on success, 1 when a command is refused or wrangler fails, and 2 for a usage
error. With `--local`, `--persist-to DIR` selects a state directory other than `.wrangler/`
(`legacy-export` can't use it: `wrangler d1 export` only reads the default one).

## Local development

```bash
npm run migrate:local
npx wrangler d1 execute kanso-sync --local --command "UPDATE settings SET value = 'open' WHERE name = 'signup'"
npx wrangler dev --var AUTH_PEPPER:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8
```

That pepper is the fixed test value (bytes 0 to 31) that `vitest.config.ts` also uses; never deploy
it. `.dev.vars` and `.wrangler/` are ignored by git.

## Tests

```bash
npm test          # vitest in the Workers runtime; fast-check seed 20261017 (FAST_CHECK_SEED, FAST_CHECK_RUNS)
npm run typecheck # the Worker and tests, then scripts/
scripts/e2e-local.sh
```

`test/fixtures/` holds cases the Worker and the Rust client's in-memory server must both pass:
`usernames.json`, `auth.json` and `conformance.json` (protocol v2), and `conformance-v1.json`,
which this Worker no longer serves: only the Rust rollback test's stand-in for the old Worker
still runs it. `scripts/e2e-local.sh` starts the Worker under `wrangler
dev` on a throwaway state directory and runs `cargo test -p kanso-sync --test e2e` against it,
all in Miniflare with no Cloudflare credentials.
