# Notes for App Review

(App Store Connect › App Review Information › Notes.)

Kanso is a Markdown notes app. Every feature works with no account: open the app
and write. Sync is optional. To try it, create an account from Settings › Sync with
a username and password (no email is asked for), or use the demo account below.

Demo account (on the review server): *the owner adds a username and password here
when the sync server's sign-ups are open; never commit it.*

Encryption: the app seals notes end to end with standard algorithms (Argon2id,
HKDF, XChaCha20-Poly1305, X25519, Ed25519) before they are synced. Export
compliance is answered in Info.plist (`ITSAppUsesNonExemptEncryption`).

Sign in with Apple is offered beside Google and GitHub (guideline 4.8).
