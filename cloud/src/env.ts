/** The Worker's bindings (docs/ACCOUNTS.md › Worker configuration). */
export interface Env {
	DB: D1Database;
	FILES: R2Bucket;
	/** 32 bytes of base64url; `wrangler secret put AUTH_PEPPER`. Missing or malformed answers 503 where it is needed. */
	AUTH_PEPPER?: string;
	/** The write-free front filter of the auth routes, keyed by IP group; absent in unit tests. */
	AUTH_LIMITER?: RateLimit;
	/** The write-free filter of the session routes, keyed by account ID; absent in unit tests. */
	SESSION_LIMITER?: RateLimit;
}
