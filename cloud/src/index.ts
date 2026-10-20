// Kanso's sync server, protocol v2 (docs/ACCOUNTS.md): accounts with
// end-to-end encrypted libraries. The Worker stores and returns sealed values
// it can't read, keyed by account; D1 holds accounts, sessions, documents and
// the counters that enforce quotas and limits, R2 holds sealed file bytes
// under `accounts/<account_id>/<wire_id>`.
//
//   GET    /v2/health                   liveness, protocol, whether sign-ups are open
//   POST   /v2/auth/prelogin            KDF parameters, salt and account ID for a username
//   POST   /v2/auth/signup              create an account and this device's session
//   POST   /v2/auth/login               sign in a device
//   POST   /v2/auth/recover/start       prove the recovery key, get the recovery-wrapped data key
//   POST   /v2/auth/recover/finish      set a new password and recovery key
//   POST   /v2/auth/logout              sign out this device                          (session)
//   GET    /v2/account                  account, usage and quotas                     (session)
//   POST   /v2/account/password         change the password                           (session)
//   POST   /v2/account/recovery-key     replace the recovery key                      (session)
//   POST   /v2/account/delete           delete the account and its server data        (session)
//   PUT    /v2/account/device           set this device's sealed name                 (session)
//   GET    /v2/account/devices          list active devices                           (session)
//   DELETE /v2/account/devices/<id>     sign out one device                           (session)
//   POST   /v2/push                     write sealed documents                        (session)
//   GET    /v2/changes?since=&limit=    the account's change feed                     (session)
//   PUT    /v2/files/<wire_id>          store sealed file bytes                       (session)
//   GET    /v2/files/<wire_id>          fetch sealed file bytes                       (session)
//   PUT    /v2/shares/<id>              publish a sealed note, or replace it          (session)
//   DELETE /v2/shares/<id>              unpublish it                                  (session)
//   GET    /v2/public/<id>              a published note's sealed bytes               (public)
//   GET    /p/<id>                      the page that opens it with the link's key    (public)
//
// /v1 and every other path answer 404. Any request with an Origin header gets
// 403 before anything else, except on the two public routes, which hold no
// credentials and only ever read; a body with the wrong Content-Type gets 415.

import * as auth from "./auth";
import { pepperKeys } from "./crypto";
import type { Env } from "./env";
import { error, isContentType, methodNotAllowed, notFound } from "./http";
import { bindingFilter } from "./limits";
import { runCron } from "./purge";
import * as shares from "./shares";
import * as sync from "./sync";

export type { Env } from "./env";
export { runCron } from "./purge";

type Handler = (request: Request, env: Env, now: number) => Promise<Response>;
type PepperHandler = (request: Request, env: Env, now: number, keys: import("./crypto").PepperKeys) => Promise<Response>;

interface Method {
	handler: Handler | PepperHandler;
	/** The Content-Type a body must have. */
	body?: "json" | "bytes";
	/** Needs AUTH_PEPPER: 503 `misconfigured` without a valid one. */
	pepper?: boolean;
}

interface Route {
	/** For logs: the route's name, never its parameters. */
	name: string;
	methods: Record<string, Method>;
	/** Behind the AUTH_LIMITER binding (every /v2/auth/* route and /v2/health). */
	binding?: boolean;
	/** Readable by anyone, from a browser too: the Origin rule doesn't apply. */
	public?: boolean;
}

const json = (handler: Handler | PepperHandler, pepper = false): Method => ({ handler, body: "json", pepper });

const ROUTES: Record<string, Route> = {
	"/v2/health": { name: "health", binding: true, methods: { GET: { handler: auth.health, pepper: true } } },
	"/v2/auth/prelogin": { name: "prelogin", binding: true, methods: { POST: json(auth.prelogin, true) } },
	"/v2/auth/signup": { name: "signup", binding: true, methods: { POST: json(auth.signup, true) } },
	"/v2/auth/login": { name: "login", binding: true, methods: { POST: json(auth.login, true) } },
	"/v2/auth/recover/start": { name: "recover_start", binding: true, methods: { POST: json(auth.recoverStart, true) } },
	"/v2/auth/recover/finish": { name: "recover_finish", binding: true, methods: { POST: json(auth.recoverFinish, true) } },
	"/v2/auth/logout": { name: "logout", binding: true, methods: { POST: { handler: auth.logout } } },
	"/v2/account": { name: "account", methods: { GET: { handler: auth.account } } },
	"/v2/account/password": { name: "password", methods: { POST: json(auth.password) } },
	"/v2/account/recovery-key": { name: "recovery_key", methods: { POST: json(auth.recoveryKey) } },
	"/v2/account/delete": { name: "delete", methods: { POST: json(auth.deleteAccount) } },
	"/v2/account/device": { name: "set_device", methods: { PUT: json(auth.setDevice) } },
	"/v2/account/devices": { name: "devices", methods: { GET: { handler: auth.devices } } },
	"/v2/push": { name: "push", methods: { POST: json(sync.push) } },
	"/v2/changes": { name: "changes", methods: { GET: { handler: sync.changes } } },
};

/** A path segment, percent-decoded; "" (which no handler accepts) when it can't be. */
function segment(raw: string): string {
	try {
		return decodeURIComponent(raw);
	} catch {
		return "";
	}
}

function match(path: string): Route | null {
	const fixed = ROUTES[path];
	if (fixed) return fixed;
	const device = /^\/v2\/account\/devices\/([^/]+)$/.exec(path);
	if (device) return { name: "revoke_device", methods: { DELETE: { handler: auth.revokeDevice(segment(device[1])) } } };
	const file = /^\/v2\/files\/([^/]+)$/.exec(path);
	if (file) {
		const id = segment(file[1]);
		return { name: "files", methods: { GET: { handler: sync.getFile(id) }, PUT: { handler: sync.putFile(id), body: "bytes" } } };
	}
	const share = /^\/v2\/shares\/([^/]+)$/.exec(path);
	if (share) {
		const id = segment(share[1]);
		return { name: "shares", methods: { PUT: { handler: shares.putShare(id), body: "bytes" }, DELETE: { handler: shares.deleteShare(id) } } };
	}
	const published = /^\/v2\/public\/([^/]+)$/.exec(path);
	if (published) return { name: "public", public: true, methods: { GET: { handler: shares.getPublic(segment(published[1])), pepper: true } } };
	const viewer = /^\/p\/([^/]+)$/.exec(path);
	if (viewer) return { name: "viewer", public: true, methods: { GET: { handler: shares.getViewer(segment(viewer[1])) } } };
	return null;
}

/** Answers one request as of `now` (Unix ms); tests pass their own clock. */
export async function handle(request: Request, env: Env, now: number): Promise<Response> {
	let name = "unknown";
	try {
		const route = match(new URL(request.url).pathname);
		// Browsers always send Origin on the requests that matter; the native client never does (M7).
		if (request.headers.has("origin") && !route?.public) return error(403, "forbidden_origin", "Browser requests are refused.");
		if (!route) return notFound();
		name = route.name;
		const method = route.methods[request.method];
		if (!method) return methodNotAllowed(Object.keys(route.methods).join(", "));
		if (route.binding) {
			const limited = await bindingFilter(env.AUTH_LIMITER, request);
			if (limited) return limited;
		}
		const type = request.headers.get("content-type");
		if (method.body === "json" && !isContentType(type, "application/json", true)) {
			return error(415, "unsupported_media_type", "Send application/json.");
		}
		if (method.body === "bytes" && !isContentType(type, "application/octet-stream", false)) {
			return error(415, "unsupported_media_type", "Send application/octet-stream.");
		}
		if (method.pepper) {
			const keys = pepperKeys(env.AUTH_PEPPER);
			if (!keys) return error(503, "misconfigured", "The server has no valid AUTH_PEPPER.");
			return await (method.handler as PepperHandler)(request, env, now, await keys);
		}
		return await (method.handler as Handler)(request, env, now);
	} catch (err) {
		// Route names and error codes only: no paths, bodies or values.
		console.error(JSON.stringify({ route: name, error: "internal", kind: (err as Error)?.name ?? typeof err }));
		return error(500, "internal", "Internal error.");
	}
}

export default {
	fetch(request, env): Promise<Response> {
		return handle(request, env, Date.now());
	},
	scheduled(_controller, env, ctx): void {
		ctx.waitUntil(
			runCron(env, Date.now()).then(
				() => undefined,
				(err) => console.error(JSON.stringify({ route: "cron", error: "internal", kind: (err as Error)?.name ?? typeof err })),
			),
		);
	},
} satisfies ExportedHandler<Env>;
