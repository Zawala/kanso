// One account's live event room (design: Kanso accounts spec, revision 3 ›
// GET /v2/account/events): a Durable Object, keyed by account ID, that holds
// the account's signed-in devices' WebSockets and tells them when a device
// approval changed, so an approver doesn't poll.
//
// Events are hints, never state. A frame carries only its type and an
// approval ID; the device then reads the row over HTTPS. No frame carries a
// device name, a code, an ephemeral key, a wrapped key or note content. The
// room keeps no event log: D1's approval row is the authority, and a frame
// that is lost is caught up by the app's GET after a launch, a Settings open,
// a foreground or a reconnect.
//
// The room is also where an account's push notifications are sent from
// (src/push.ts). It stores one value, the account ID, and holds one alarm:
// when a change marks a device due, the Worker calls `schedule`, which arms
// the alarm for the earliest due time, and the alarm sends what is due and
// arms itself again for what is still waiting. That is what coalesces a burst
// of changes into one push per device per 30 s. D1 holds what is due; a lost
// alarm is re-armed by the next change.
//
// The room is reachable only through the Worker, which authenticates the
// session and the device proof before forwarding the upgrade. Sockets use the
// hibernation API, tagged with their device ID: a device has one socket, and
// a second replaces the first (close 4000, "replaced"), so the room's sockets
// are bounded by the account's active devices. Pings are answered by the
// runtime's auto-response without waking the room; any other client frame
// closes the socket with 1008.

import { DurableObject } from "cloudflare:workers";
import type { Env } from "./env";
import { deliverDue, pushSender } from "./push";

/** The internal headers the Worker sends with an upgrade. */
export const ACCOUNT_ROOM_HEADERS = {
	account: "x-kanso-room-account",
	device: "x-kanso-room-device",
} as const;

/** The event types a room publishes. */
export type AccountEvent = "approval.created" | "approval.named" | "approval.cancelled";

/** What a socket carries through hibernation: only whose it is. */
interface Attachment {
	account: string;
	device: string;
}

/** Close code for a socket another of the same device's replaced. */
export const REPLACED = 4000;
/** Close code for a socket whose session was signed out, revoked or replaced by a recovery. */
export const SIGNED_OUT = 4001;

export class AccountRoom extends DurableObject<Env> {
	constructor(ctx: DurableObjectState, env: Env) {
		super(ctx, env);
		// Answered by the runtime while the room sleeps: idle heartbeats never wake it.
		ctx.setWebSocketAutoResponse(new WebSocketRequestResponsePair("ping", "pong"));
	}

	/** The upgrade, from the Worker, with the account and device it authenticated. */
	override async fetch(request: Request): Promise<Response> {
		if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") return new Response(null, { status: 426 });
		const account = request.headers.get(ACCOUNT_ROOM_HEADERS.account) ?? "";
		const device = request.headers.get(ACCOUNT_ROOM_HEADERS.device) ?? "";
		if (account === "" || device === "") return new Response(null, { status: 400 });
		for (const old of this.ctx.getWebSockets(device)) {
			try {
				old.close(REPLACED, "replaced");
			} catch {
				// Already closing.
			}
		}
		const [client, server] = Object.values(new WebSocketPair());
		this.ctx.acceptWebSocket(server, [device]);
		server.serializeAttachment({ account, device } satisfies Attachment);
		server.send(JSON.stringify({ type: "hello" }));
		return new Response(null, { status: 101, webSocket: client });
	}

	/** Tells every open socket of the account that approval `id` changed. Never throws for a socket that is gone. */
	async publish(type: AccountEvent, id: string): Promise<void> {
		const frame = JSON.stringify({ type, id });
		for (const ws of this.ctx.getWebSockets()) {
			try {
				ws.send(frame);
			} catch {
				// Closing or gone: its close event cleans up.
			}
		}
	}

	/**
	 * Closes the sockets of `devices`, or every socket of the account when it
	 * is null (a password change or a recovery), with 4001. A socket is
	 * authenticated only at its upgrade, so this is what ends one whose session
	 * has since been revoked. A device whose session is still live reconnects.
	 */
	async signOut(devices: string[] | null): Promise<void> {
		const sockets = devices === null ? this.ctx.getWebSockets() : devices.flatMap((device) => this.ctx.getWebSockets(device));
		for (const ws of sockets) {
			try {
				ws.close(SIGNED_OUT, "signed_out");
			} catch {
				// Already closing.
			}
		}
	}

	/** Arms the push alarm for `dueAt` (Unix ms) unless one is set for an earlier time. */
	async schedule(accountId: string, dueAt: number): Promise<void> {
		await this.ctx.storage.put("account", accountId);
		await this.arm(dueAt);
	}

	private async arm(at: number): Promise<void> {
		const set = await this.ctx.storage.getAlarm();
		if (set === null || at < set) await this.ctx.storage.setAlarm(at);
	}

	/** Sends the pushes that are due, then arms for the next. D1 errors are logged and rethrown, so the runtime retries. */
	override async alarm(): Promise<void> {
		const accountId = await this.ctx.storage.get<string>("account");
		if (accountId === undefined) return;
		const now = Date.now();
		try {
			const { next } = await deliverDue(this.env, accountId, now, pushSender(this.env));
			if (next !== null) await this.arm(Math.max(next, now + 1));
		} catch (err) {
			console.error(JSON.stringify({ route: "push", error: "alarm_failed", kind: (err as Error)?.name ?? typeof err }));
			throw err;
		}
	}

	/** How many sockets are open, per device (tests). */
	async sockets(): Promise<string[]> {
		return this.ctx.getWebSockets().map((ws) => (ws.deserializeAttachment() as Attachment).device);
	}

	/** The app sends only pings, which the auto-response answers; anything else is a protocol error. */
	override async webSocketMessage(ws: WebSocket): Promise<void> {
		try {
			ws.close(1008, "unexpected frame");
		} catch {
			// Already closing.
		}
	}

	override async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
		try {
			ws.close(code === 1005 || code === 1006 ? 1000 : code, reason);
		} catch {
			// Already closed.
		}
	}

	override async webSocketError(): Promise<void> {
		// Nothing is kept per socket.
	}
}

/** The account's room. */
export const accountRoom = (env: Env, accountId: string): DurableObjectStub<AccountRoom> =>
	env.ACCOUNT_ROOM.get(env.ACCOUNT_ROOM.idFromName(accountId));

/**
 * Publishes after the D1 write has committed. A room that can't be reached
 * never fails the HTTP route: the event is only a hint, and the next GET
 * catches up.
 */
export async function publish(env: Env, accountId: string, type: AccountEvent, id: string): Promise<void> {
	try {
		await accountRoom(env, accountId).publish(type, id);
	} catch (err) {
		console.error(JSON.stringify({ route: "account_events", error: "publish_failed", kind: (err as Error)?.name ?? typeof err }));
	}
}

/**
 * Closes the account-event sockets of `devices` (all of the account's when
 * null) after the D1 revocation has committed. Like `publish`, a room that
 * can't be reached never fails the route.
 */
export async function signOutSockets(env: Env, accountId: string, devices: string[] | null): Promise<void> {
	try {
		await accountRoom(env, accountId).signOut(devices);
	} catch (err) {
		console.error(JSON.stringify({ route: "account_events", error: "sign_out_failed", kind: (err as Error)?.name ?? typeof err }));
	}
}
