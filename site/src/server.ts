/**
 * The Worker's entry. It sends www.kansoapp.app to the apex with a permanent
 * redirect, hands every other request to TanStack Start, and adds the
 * security headers every response should carry. Pages vary by User-Agent (the
 * download button), so HTML is never cached publicly.
 */
import handler, { createServerEntry } from "@tanstack/react-start/server-entry";

const APEX = "kansoapp.app";

const SECURITY_HEADERS: Readonly<Record<string, string>> = {
  "strict-transport-security": "max-age=63072000; includeSubDomains; preload",
  "x-content-type-options": "nosniff",
  "referrer-policy": "strict-origin-when-cross-origin",
  "permissions-policy": "camera=(), microphone=(), geolocation=(), interest-cohort=()",
  "x-frame-options": "DENY",
  "cross-origin-opener-policy": "same-origin",
};

export default createServerEntry({
  async fetch(request) {
    const url = new URL(request.url);
    if (url.hostname === `www.${APEX}`) {
      url.hostname = APEX;
      return Response.redirect(url.toString(), 301);
    }
    const response = await handler.fetch(request);
    const headers = new Headers(response.headers);
    for (const [name, value] of Object.entries(SECURITY_HEADERS)) headers.set(name, value);
    if ((headers.get("content-type") ?? "").startsWith("text/html")) {
      headers.set("cache-control", "private, no-cache");
      headers.append("vary", "User-Agent, Sec-CH-UA-Platform");
    }
    return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
  },
});
