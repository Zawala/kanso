# Play Console "Data safety" answers: a draft for the owner

For Play Console › App content › Data safety. Derived like
`../app-store/privacy.md` (read its caveats; they apply here too). The owner
submits and answers for these.

## Does your app collect or share any of the required user data types?

**Collects: yes, only with a sync account. Shares: no.** Without an account the
app sends nothing. Sending data to Kanso's own sync server to run sync is
collection, not sharing; the server's host (Cloudflare) processes it on Kanso's
behalf, which Play does not count as sharing.

## Is all of the user data collected by your app encrypted in transit?

Yes. Every connection is TLS (the platform's certificate verifier), and notes are
sealed end to end before they are sent.

## Do you provide a way for users to request that their data is deleted?

Yes: the account can be deleted. **Check, before release:** Play requires that an
app that lets people create an account has an in-app way to delete it **and** a
web link where a deletion can be requested. The desktop app deletes an account
(`docs/ACCOUNTS.md`, "Deleting the account"). Confirm the same is in the mobile
app (step 8), and publish a web page or form on kansoapp.app for the link Play asks
for. The same applies to the App Store (guideline 5.1.1(v)).

## Data types

| Play category › type | Collected | Shared | Optional | Purpose | Notes |
|---|---|---|---|---|---|
| Personal info › **User IDs** | Yes | No | Yes (only with sync) | App functionality; Account management | The account's username; for Google, GitHub or Apple sign-in, the provider's account number, never an email or name |
| App activity › **Other user-generated content** | No | No | n/a | n/a | Note text, titles, notebook and tag names and files leave the phone only as ciphertext the server cannot open: not collected |
| App info and performance › Crash logs, Diagnostics | No | No | n/a | n/a | **Check** against the crash reporting decision (issue #48, open question 1) |
| Device or other IDs | No | No | n/a | n/a | Device IDs are per-account sync sessions, not Android advertising or hardware IDs |

**Check:** the server also holds counts, rough sizes and times of notes, and the
last-seen hour of each device (see `privacy.md`). Play has no category for it; if the
owner declares it on the App Store, declare *App activity › Other actions*
(App functionality) here too.

## Other Play Console forms

- **Content rating:** no user-generated content shared between people in the app;
  no ads; no purchases. Expect "Everyone".
- **Ads:** no. **App access:** all features work without an account (give reviewers
  no login). **Target audience:** 18 and over, not for children.
- **Permissions:** none beyond the network (note: check `INTERNET` is in the main
  manifest before sync ships; see `docs/RELEASING-MOBILE.md`, "Known gaps"). The
  share target and widgets (step 10) need none.
- **Government apps, financial features, health:** no.
