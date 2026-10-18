# Deployed

https://kanso-sync.imngonii.workers.dev

As of 2026-10-18 this still runs the single-user v1 Worker (`SYNC_TOKEN`, migration `0001_init`
only). Moving it to accounts (`0002_accounts`, `AUTH_PEPPER`, protocol v2) follows
[docs/ACCOUNTS.md › Rollout](../docs/ACCOUNTS.md#rollout); update this file when that is done.

## Restoring D1

Only with `npm run -s admin -- restore <time | bookmark>`, run first without `--confirm` to see
what it does. Never with `wrangler d1 time-travel restore` or the dashboard: a restore alone
rewinds every account while each Mac still trusts its cursor, and `restore` follows it at once with
a new epoch for every account so that each Mac re-merges. It prints the bookmark to undo with. A
restore also brings back the sessions, passwords and recovery keys of that time: revoke again
any device revoked since, and anyone who changed a password or recovery key since must change it
again.

## What the deployment records

`wrangler.jsonc` pins what Cloudflare keeps: Workers Logs stores only the Worker's own log lines
(route names and error codes), while invocation logs, which would store each request's URL, method
and metadata, and traces are off. Metrics stay on.

Live tailing can't be switched off. While someone with access to the Cloudflare account runs
`wrangler tail`, or opens the dashboard's live logs, they see each request as it happens: its URL
(document, file and device IDs, which are opaque), method, headers and Cloudflare's metadata about
the connection, such as the country it came from. Cloudflare redacts some values, such as the
`Authorization` header, by heuristic rather than by guarantee. A tail stores nothing unless its
viewer saves the output, so tail only to debug, and don't keep what it prints.
