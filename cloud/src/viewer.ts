// The page a published note is read on: `GET /p/<share id>#<key>`
// (docs/ACCOUNTS.md › Published notes).
//
// Static HTML with one inline style and one inline script, both pinned in the
// Content-Security-Policy by their SHA-256, and nothing loaded from anywhere
// else. The script reads the key from location.hash (the fragment never
// reaches the server), fetches the ciphertext from /v2/public/<id>, opens it
// with WebCrypto's AES-256-GCM, and shows the note in an iframe sandboxed
// without allow-scripts and without allow-same-origin.
//
// A srcdoc frame inherits this page's policy, so the note's own styling has
// to fit it: the script gathers the note's <style> blocks and style=""
// attributes into one stylesheet the frame loads as a data: URL (`style-src
// data:`), and its images are data: URLs already (`img-src data:`). Both are
// taken out as text before the page is parsed: a DOMParser document inherits
// the policy too, so the browser would refuse to parse them there. Nothing in
// the note can run, load from the network, or reach this page.

import { DEFAULT_TITLE, PREVIEW_DESCRIPTION, PREVIEW_IMAGE_PATH } from "./rerout";

/** The page's only stylesheet. Calm: Kanso's paper and graphite, in both schemes. */
export const VIEWER_STYLE = `:root{color-scheme:light dark;--paper:#fbfaf7;--ink:#2b2a27;--muted:#76736c;--line:#e6e2da;--card:#ffffff}
@media (prefers-color-scheme:dark){:root{--paper:#1c1b19;--ink:#e9e6df;--muted:#9a968d;--line:#34322e;--card:#232220}}
*{box-sizing:border-box}
html,body{margin:0;height:100%;background:var(--paper);color:var(--ink);font:15px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif}
#note{position:fixed;inset:0;width:100%;height:100%;border:0;display:none;background:var(--paper)}
#note.shown{display:block}
#status{min-height:100%;display:flex;align-items:center;justify-content:center;padding:24px}
#status.hidden{display:none}
.card{max-width:420px;padding:28px 30px;border:1px solid var(--line);border-radius:12px;background:var(--card)}
.card h1{margin:0 0 8px;font-size:17px;font-weight:600;letter-spacing:-.01em}
.card p{margin:0;color:var(--muted)}
.quiet{color:var(--muted)}`;

/** The page's only script. Every step that can fail ends in a calm message. */
export const VIEWER_SCRIPT = `(function(){
"use strict";
var status=document.getElementById("status"),frame=document.getElementById("note");
function say(title,detail){
status.textContent="";
var card=document.createElement("div");card.className="card";
var h=document.createElement("h1");h.textContent=title;card.appendChild(h);
var p=document.createElement("p");p.textContent=detail;card.appendChild(p);
status.appendChild(card);status.className="";frame.className="";document.title=title;
}
function b64url(text){
if(!/^[A-Za-z0-9_-]*$/.test(text)||text.length%4===1)return null;
var s=text.replace(/-/g,"+").replace(/_/g,"/");while(s.length%4)s+="=";
try{var bin=atob(s);}catch(e){return null;}
var out=new Uint8Array(bin.length);for(var i=0;i<bin.length;i++)out[i]=bin.charCodeAt(i);return out;
}
var id=(location.pathname.match(/^\\/p\\/([A-Za-z0-9_-]{22})$/)||[])[1];
var key=b64url(location.hash.replace(/^#/,""));
if(!id){say("This link isn\\u2019t complete","Check that the whole link was copied.");return;}
if(!key||key.length!==32){say("This link is missing its key","The part after # unlocks the note. Ask whoever shared it for the whole link.");return;}
function clean(html){
var css="";
html=html.replace(/<style\\b[^>]*>([\\s\\S]*?)<\\/style>/gi,function(_,text){css+=text.replace(/<!\\[CDATA\\[|\\]\\]>/g,"")+"\\n";return "";});
html=html.replace(/(<[^<>]*?\\s)style\\s*=/gi,"$1data-kanso-css=");
var doc=new DOMParser().parseFromString(html,"text/html");
doc.querySelectorAll("script,style,template,iframe,frame,object,embed,base,link,meta,form,noscript").forEach(function(el){el.remove();});
var n=0;
doc.querySelectorAll("*").forEach(function(el){
for(var i=el.attributes.length-1;i>=0;i--){var name=el.attributes[i].name;if(/^on/i.test(name)||name==="style")el.removeAttribute(name);}
if(el.hasAttribute("data-kanso-css")){
var rules="";
(el.getAttribute("data-kanso-css")||"").split(/;(?![^(]*\\))/).forEach(function(decl){
var at=decl.indexOf(":");if(at<1)return;
var p=decl.slice(0,at).trim().toLowerCase(),v=decl.slice(at+1).trim().replace(/!\\s*important$/i,"").trim();
if(/^-?[a-z][a-z0-9-]*$/.test(p)&&v&&!/[{}<>\\\\]/.test(v))rules+=p+":"+v+" !important;";
});
el.removeAttribute("data-kanso-css");
if(rules){n++;el.setAttribute("data-kanso-style",String(n));css+="[data-kanso-style=\\""+n+"\\"]{"+rules+"}\\n";}
}
});
doc.querySelectorAll("a[href]").forEach(function(a){
var href=a.getAttribute("href")||"";
if(href.charAt(0)==="#")return;
if(/^(https?:|mailto:)/i.test(href)){a.setAttribute("target","_blank");a.setAttribute("rel","noopener noreferrer");}
else a.removeAttribute("href");
});
var head=doc.head||doc.documentElement.insertBefore(doc.createElement("head"),doc.body);
var ref=doc.createElement("meta");ref.setAttribute("name","referrer");ref.setAttribute("content","no-referrer");head.insertBefore(ref,head.firstChild);
var link=doc.createElement("link");link.setAttribute("rel","stylesheet");link.setAttribute("href","data:text/css;charset=utf-8,"+encodeURIComponent(css));head.appendChild(link);
var title=(doc.title||"").trim();
return {html:"<!doctype html>"+doc.documentElement.outerHTML,title:title};
}
fetch("/v2/public/"+id,{cache:"no-store",credentials:"omit",referrerPolicy:"no-referrer"}).then(function(r){
if(r.status===404)throw "gone";
if(r.status===410)throw "expired";
if(r.status===429)throw "busy";
if(!r.ok)throw "server";
return r.arrayBuffer();
}).then(function(buf){
var bytes=new Uint8Array(buf);
if(bytes.length<29||bytes[0]!==1)throw "key";
var aad=new TextEncoder().encode("kanso-share\\u0001"+id);
return crypto.subtle.importKey("raw",key,{name:"AES-GCM"},false,["decrypt"]).then(function(k){
return crypto.subtle.decrypt({name:"AES-GCM",iv:bytes.slice(1,13),additionalData:aad,tagLength:128},k,bytes.slice(13));
}).catch(function(){throw "key";});
}).then(function(plain){
var page=clean(new TextDecoder().decode(plain));
document.title=page.title||"A note";
frame.srcdoc=page.html;frame.className="shown";status.className="hidden";
}).catch(function(why){
if(why==="gone")say("This note isn\\u2019t published any more","It may have been unpublished, or the link may be mistyped.");
else if(why==="expired")say("This link has expired","Whoever shared it chose for it to stop working after a while.");
else if(why==="busy")say("Too many requests","Wait a minute, then reload the page.");
else if(why==="key")say("This key doesn\\u2019t open the note","Check that the whole link was copied, including the part after #.");
else say("Couldn\\u2019t load the note","Check your connection, then reload the page.");
});
})();`;

const escapeAttr = (text: string): string => text.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/**
 * The page: the style and script above, exactly as hashed. Its preview tags
 * are the same for every share (the page never reads D1): "A note", one line
 * about Kanso and Kanso's image on `origin`.
 */
export const viewerHtml = (origin: string): string => `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex, nofollow">
<meta name="referrer" content="no-referrer">
<title>A note</title>
<meta property="og:site_name" content="Kanso">
<meta property="og:title" content="${DEFAULT_TITLE}">
<meta property="og:description" content="${PREVIEW_DESCRIPTION}">
<meta property="og:image" content="${escapeAttr(origin + PREVIEW_IMAGE_PATH)}">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta name="twitter:card" content="summary_large_image">
<style>${VIEWER_STYLE}</style>
</head>
<body>
<main id="status"><p class="quiet">Opening the note…</p></main>
<iframe id="note" title="The note" sandbox="allow-popups allow-popups-to-escape-sandbox" referrerpolicy="no-referrer"></iframe>
<script>${VIEWER_SCRIPT}</script>
</body>
</html>
`;

async function cspHash(text: string): Promise<string> {
	const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)));
	let binary = "";
	for (const b of digest) binary += String.fromCharCode(b);
	return `'sha256-${btoa(binary)}'`;
}

let policy: Promise<string> | null = null;

/**
 * The viewer's Content-Security-Policy: its own script and style by hash,
 * fetches to this origin only, and for the note in the frame (which inherits
 * the policy) data: images, fonts and the gathered stylesheet. No frame may
 * hold the page, and nothing posts anywhere.
 */
export function viewerPolicy(): Promise<string> {
	policy ??= (async () =>
		[
			"default-src 'none'",
			`script-src ${await cspHash(VIEWER_SCRIPT)}`,
			`style-src ${await cspHash(VIEWER_STYLE)} data:`,
			"img-src data:",
			"font-src data:",
			"connect-src 'self'",
			"frame-src 'none'",
			"base-uri 'none'",
			"form-action 'none'",
			"frame-ancestors 'none'",
		].join("; "))();
	return policy;
}

/** Headers every public answer carries: no referrer, no sniffing, no framing, not indexed. */
export const PUBLIC_HEADERS: Record<string, string> = {
	"referrer-policy": "no-referrer",
	"x-content-type-options": "nosniff",
	"x-frame-options": "DENY",
	"x-robots-tag": "noindex, nofollow",
	"cross-origin-opener-policy": "same-origin",
	"cross-origin-resource-policy": "same-origin",
};

/** `GET /p/<id>`: the page, the same for every share. */
export async function viewerPage(origin: string): Promise<Response> {
	return new Response(viewerHtml(origin), {
		headers: {
			"content-type": "text/html; charset=utf-8",
			"content-security-policy": await viewerPolicy(),
			"cache-control": "public, max-age=300",
			"permissions-policy": "camera=(), microphone=(), geolocation=(), interest-cohort=()",
			...PUBLIC_HEADERS,
		},
	});
}
