# Play Console "Data safety" answers: a draft for the owner

For Play Console › App content › Data safety. Derived like
`../app-store/privacy.md` (read its caveats; they apply here too). The owner
submits and answers for these.

## Does your app collect or share any of the required user data types?

**Collects: yes.** After the one-time notice is acknowledged, Kanso sends a small
anonymous usage count once a day. With a sync account, the sync service holds the
account's username. A crash report is sent only when the person taps Send.

**Shares: only when Ask your notes is used.** Note excerpts, titles and the question
go to the model provider the person configured, with their own key; Kanso receives none
of it. Sending data to Kanso's own sync server to run sync is collection, not sharing;
the server's host (Cloudflare) processes it on Kanso's behalf, which Play does not count
as sharing. PostHog receives the checked metrics batch from Kanso's Worker as a service
provider; the PostHog SDK is not in the app, and no data is sold or used for advertising
or cross-app tracking.

## Is all of the user data collected by your app encrypted in transit?

Yes. Every connection is TLS (the platform's certificate verifier), and notes are
sealed end to end before they are sent. The metrics batch contains no notes, titles,
tags, file names or anything typed.

## Do you provide a way for users to request that their data is deleted?

Yes: the account can be deleted. **Check, before release:** Play requires that an
app that lets people create an account has an in-app way to delete it **and** a
web link where a deletion can be requested. The desktop app deletes an account
(`docs/ACCOUNTS.md`, "Deleting the account"). Confirm the same is in the mobile
app (step 8), and publish a web page or form on kansoapp.app for the link Play asks
for. The same applies to the App Store (guideline 5.1.1(v)).

The anonymous metrics ID cannot be traced back to an account, so a deletion request
cannot remove an individual's anonymous counts; the public metrics page says this.
Account deletion covers sync data.

## Data types

| Play category › type | Collected | Shared | Optional | Purpose | Notes |
|---|---|---|---|---|---|
| Personal info › **User IDs** | Yes | No | Yes (only with sync) | App functionality; Account management | The account's username; for Google, GitHub or Apple sign-in, the provider's account number, never an email or name |
| Location › **Approximate location** | Yes | No | No (after the notice) | Analytics | The two-letter country Cloudflare reports for a metrics request. No precise location, GPS access or stored IP address; PostHog gets the country with geo-IP disabled |
| App activity › **App interactions** | Yes | No | No (after the notice) | Analytics | Anonymous app-active and closed feature-name counts from `docs/METRICS.md`; no note content, titles or anything typed, no session replay |
| Device or other IDs | Yes | No | No (after the notice) | Analytics | The pseudonymous metrics ID (`a_` derived from the account's encryption key, else a random `i_` install ID). Not an advertising ID or Android hardware identifier |
| App info and performance › **Crash logs** | Yes | No | Yes (only when the person taps Send) | App functionality | Stack frames, app and OS version and device model only; never note text, titles or file names (`docs/CRASH-REPORTS.md`). Not tied to an account; kept 90 days |
| App info and performance › Diagnostics | No | No | n/a | n/a | No performance or diagnostic data beyond the crash report above |
| App activity › **Other user-generated content** | Yes | Yes | Yes (only when AI is on and used) | App functionality | The note excerpts, note titles and question sent to the model provider the person configured when they use Ask your notes. Kanso never receives them. Ordinary note text, titles, notebook and tag names and files leave the phone for sync only as ciphertext the server cannot open |
| Web browsing, contacts, messages, files and docs, photos, audio, calendar, health, finance | No | No | n/a | n/a | The app does not collect them |

**Check:** the server also holds counts, rough sizes and times of notes, and the
last-seen hour of each device (see `privacy.md`). Play has no category for it; if the
owner declares *Other Data Types* on the App Store, declare *App activity › Other
actions* (App functionality) here too.

**Ask your notes.** It is off by default. The chosen model provider receives the note
excerpts and question only when the person enables the feature and asks. It is declared
as *App activity › Other user-generated content* (collected, shared with the provider
the person chose, optional, App functionality). The current public listing does not
advertise the feature until live-model QA is complete.

## Other Play Console forms

- **Content rating:** no user-generated content shared between people in the app;
  no ads; no purchases. Expect "Everyone".
- **Ads:** no. **App access:** all features work without an account (give reviewers
  no login). **Target audience:** 18 and over, not for children.
- **Permissions:** none beyond the network (note: check `INTERNET` is in the main
  manifest before sync ships; see `docs/RELEASING-MOBILE.md`, "Known gaps"). The
  share target and widgets need none.
- **Government apps, financial features, health:** no.
