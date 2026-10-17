# Kanso sync Worker

A Cloudflare Worker that Kanso clients sync through. Documents and their change sequence live in
D1; attachment bytes live in R2. Every route except `/v1/health` needs
`Authorization: Bearer <SYNC_TOKEN>`.

| Route | |
|---|---|
| `GET /v1/health` | Liveness, no token needed |
| `POST /v1/push` | Up to 200 changed documents, each with the revision it was based on |
| `GET /v1/changes?since=&limit=` | Documents changed after a sequence number |
| `PUT` / `GET /v1/files/<id>` | Attachment bytes, checked against their MD5 |

## Deploying to your account

Needs Node and a Cloudflare API token that can edit Workers, D1 and R2.

```bash
cd cloud
npm ci
export CLOUDFLARE_API_TOKEN=...            # never commit it
npx wrangler d1 create kanso-sync          # put the printed database_id in wrangler.jsonc
npx wrangler r2 bucket create kanso-files
npm run migrate:remote
openssl rand -hex 32 | npx wrangler secret put SYNC_TOKEN
npm run deploy
```

Then give each Mac the Worker URL and the same token in Kanso's Settings › Sync. The token is kept
in the Keychain (service "Kanso", account "sync-token"). From the command line,
`kanso-sync token set` reads it from stdin, and `kanso-sync check --endpoint <url>` checks the
URL and token.

The current deployment is listed in [DEPLOYED.md](DEPLOYED.md).

## Tests

```bash
npm test          # vitest in the Workers runtime, with fast-check seed 20261017
npm run typecheck
```

`test/fixtures/conformance.json` holds cases that the Worker and the Rust client's in-memory
remote (`kanso_sync::MemoryRemote`) must both pass.
