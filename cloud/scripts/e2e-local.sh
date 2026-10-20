#!/usr/bin/env bash
# End to end against a local Worker (docs/ACCOUNTS.md › Tests › End to end).
#
# Everything here is Miniflare: a throwaway state directory, `wrangler dev`
# on 127.0.0.1:8787 with a fixed test pepper, and the Rust e2e test
# (`cargo test -p kanso-sync --test e2e`) pointed at it. Once the Worker has
# stopped, a second test scans the state directory for the first one's
# canaries. No Cloudflare credentials are read or needed; the script unsets
# them to be sure.
#
#   cloud/scripts/e2e-local.sh            from anywhere in the repository
#
# KANSO_E2E_KEEP=1 keeps the state directory for inspection.
# KANSO_E2E_PORT picks the Worker's port (default: a free one the system hands
# out, so two runs on one machine never share a Worker); a port something else
# already listens on is refused, so the test never talks to another server.
set -euo pipefail

unset CLOUDFLARE_API_TOKEN CLOUDFLARE_ACCOUNT_ID IMNGONII_CLOUDFLARE_API_TOKEN

CLOUD="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="$(cd "$CLOUD/.." && pwd)"
PORT="${KANSO_E2E_PORT:-}"
if [[ -z "$PORT" ]]; then
	PORT="$(python3 -I -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')"
fi
ENDPOINT="http://127.0.0.1:$PORT"
# A fixed test pepper (bytes 0..31, as in vitest.config.ts), never a deployed one.
TEST_PEPPER="AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"

# 1. A temporary state directory, and beside it (never inside it) the list of
#    canaries the scan looks for.
STATE="$(mktemp -d "${TMPDIR:-/tmp}/kanso-e2e.XXXXXX")"
LISTS="$(mktemp -d "${TMPDIR:-/tmp}/kanso-e2e-canaries.XXXXXX")"
CANARIES="$LISTS/canaries.json"
DEV_PID=""
stop_worker() {
	if [[ -n "$DEV_PID" ]] && kill -0 "$DEV_PID" 2>/dev/null; then
		kill "$DEV_PID" 2>/dev/null || true
		wait "$DEV_PID" 2>/dev/null || true
	fi
	DEV_PID=""
}
cleanup() {
	stop_worker
	rm -rf "$LISTS"
	if [[ "${KANSO_E2E_KEEP:-}" == "1" ]]; then
		echo "State kept in $STATE"
	else
		rm -rf "$STATE"
	fi
}
trap cleanup EXIT

if (exec 3<>"/dev/tcp/127.0.0.1/$PORT") 2>/dev/null; then
	echo "Something already listens on 127.0.0.1:$PORT; set KANSO_E2E_PORT to a free port." >&2
	exit 1
fi

cd "$CLOUD"
if [[ ! -x node_modules/.bin/wrangler ]]; then
	echo "Run npm ci in cloud/ first." >&2
	exit 1
fi
WRANGLER=(node_modules/.bin/wrangler)

# 2. The migrations.
"${WRANGLER[@]}" d1 migrations apply kanso-sync --local --persist-to "$STATE"

# 3. Sign-ups open.
"${WRANGLER[@]}" d1 execute kanso-sync --local --persist-to "$STATE" --command "UPDATE settings SET value = 'open' WHERE name = 'signup'"

# 4. The Worker, with /__scheduled for the cron.
"${WRANGLER[@]}" dev --local --persist-to "$STATE" --ip 127.0.0.1 --port "$PORT" --inspector-port 0 \
	--test-scheduled --show-interactive-dev-session=false --var "AUTH_PEPPER:$TEST_PEPPER" >"$STATE/wrangler-dev.log" 2>&1 &
DEV_PID=$!

for _ in $(seq 1 120); do
	if curl -fsS "$ENDPOINT/v2/health" >/dev/null 2>&1; then break; fi
	if ! kill -0 "$DEV_PID" 2>/dev/null; then
		echo "wrangler dev exited:" >&2
		cat "$STATE/wrangler-dev.log" >&2
		exit 1
	fi
	sleep 0.5
done
# wrangler dev may move to another port when this one is taken; then the
# health check above answered for someone else's Worker.
if ! grep -q "127.0.0.1:$PORT" "$STATE/wrangler-dev.log"; then
	echo "wrangler dev isn't listening on $PORT:" >&2
	cat "$STATE/wrangler-dev.log" >&2
	exit 1
fi
if ! curl -fsS "$ENDPOINT/v2/health"; then
	echo "The Worker didn't answer /v2/health:" >&2
	cat "$STATE/wrangler-dev.log" >&2
	exit 1
fi
echo

# 5. The Rust end-to-end test.
cd "$REPO"
KANSO_SYNC_E2E_ENDPOINT="$ENDPOINT" KANSO_SYNC_E2E_CANARIES="$CANARIES" \
	cargo test -p kanso-sync --test e2e -- --nocapture

# 6. The canary scan, once wrangler dev has exited and D1 has written
#    everything to its files.
stop_worker
KANSO_SYNC_E2E_STATE="$STATE" KANSO_SYNC_E2E_CANARIES="$CANARIES" \
	cargo test -p kanso-sync --test e2e -- --ignored --exact the_stopped_state_holds_no_canary --nocapture
