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
	/** The write-free filter of published notes and their viewer page, keyed by IP group; absent in unit tests. */
	PUBLIC_LIMITER?: RateLimit;
	/** Rerout's API key for published notes' short links; `wrangler secret put REROUT_API_KEY`. Without it there are none. */
	REROUT_API_KEY?: string;
	/** The Rerout project the key belongs to (a plain var); a link Rerout made elsewhere is deleted and not used. */
	REROUT_PROJECT_ID?: string;
	/** The custom domain short links are made on (a plain var, `go.kansoapp.app`), added and verified in that project; without it, Rerout's own. */
	REROUT_DOMAIN?: string;
}
