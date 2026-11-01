# App Privacy ("nutrition label") answers: a draft for the owner

For App Store Connect › App Privacy. Derived from `docs/ACCOUNTS.md`
("What the server can and can't see"), `docs/METRICS.md`, the privacy page on
kansoapp.app, and the app's code as of 0.9.0. **The owner submits these answers
and is responsible for them** (Apple: [App privacy
details](https://developer.apple.com/app-store/app-privacy-details/)). Anything
marked **Check** is a judgement call that is not settled by the code.

Apple's word "collect" means sending data off the device "in a way that allows
you and/or your third-party partners to access it for a period longer than what
is necessary to service the transmitted request in real time". Notes, titles, names
and files leave the phone only as ciphertext the sync server cannot open, so they
are not collected by sync.

## Do you or your third-party partners collect data from this app?

**Yes.** After the one-time notice is acknowledged, Kanso sends a small anonymous
usage count (at launch, every hour while open, and when the app goes to the background). With a sync account, the sync service also holds the
account's username. A crash report is sent only when the person taps Send.

No account is needed for the usage count, and no advertising, attribution or
analytics SDK is linked into the app. PostHog receives the checked, anonymous
count from Kanso's own Worker; the PostHog SDK is not in the app.

## Data used to track you

None. (`NSPrivacyTracking` is `false` and there are no tracking domains.) The
usage count is not combined with data from other companies for advertising or
measurement across apps, and it carries no advertising identifier or account ID.

## Data linked to you (only with a sync account)

| Data type (Apple's name) | What it is here | Purpose | Linked to identity | Tracking |
|---|---|---|---|---|
| Identifiers › **User ID** | The account's username. For a sign-in with Apple, Google or GitHub, the provider's account number for the person (never an email, name or profile), stored beside the username | App Functionality | Yes | No |
| User Content › **Other User Content** | The note excerpts, note titles and question sent only when the person turns on Ask your notes and asks. They go from the phone to the model provider the person configured with their own key, never to Kanso's sync server | App Functionality | Yes | No |

**Check: Other Data Types.** The server also holds, per account, counts and rough
sizes of notes, notebooks, tags and files, when each was written, how many devices
are signed in and when each was last seen (to the hour). None of it says what a note
contains. Apple has no category named for this; "Other Data Types" (App
Functionality, linked) is the honest label if the owner reads "collect" strictly,
and omitting it is defensible if the owner reads these as operational records of
the sync service. The safe answer is to declare it.

**Check: IP address.** The sync server's rate limiter keeps a keyed hash of the IP
address for at most two days. A keyed hash the owner cannot reverse is not an
address; Apple lists no "IP address" type apart from Device ID and Location. The
draft does not declare it as location. The coarse country derived from a metrics
request is declared below.

## Data not linked to you

| Data type (Apple's name) | What it is here | Purpose | Linked to identity | Tracking |
|---|---|---|---|---|
| Usage Data › **Product Interaction** | Anonymous use: app active once a day, app opened, session length as a range, platform, phone/tablet form, app version and release channel, OS version, sync mode, sync errors and note, notebook and attachment-size ranges, whether AI is on, the UI language and theme or editor mode as plain labels (a custom theme is just "custom"), and a closed list of feature names with how often they were used as a range. Never note text, titles, tags, file names, theme or plugin names, or anything typed (`docs/METRICS.md`) | Analytics | No | No |
| Identifiers › **Device ID** | The pseudonymous metrics ID: `a_` plus a value derived from the account's encryption key while signed in, else a random `i_` install ID kept in the local library. It is not an advertising or hardware ID, and neither Kanso nor PostHog can trace it back to an account | Analytics | No | No |
| Location › **Coarse Location** | The two-letter country Cloudflare reports for the metrics request, added by the Worker before PostHog. No IP address is stored or sent to PostHog | Analytics | No | No |
| Diagnostics › **Crash Data** | A crash report the person chose to send: where in Kanso's code it failed (stack frames), the app and OS versions and the device model. Never note text, titles or file names (`docs/CRASH-REPORTS.md`). Sent without a session or device ID; the server stores no IP address with it and deletes it after 90 days | App Functionality | No | No |

The usage count is sent only after the one-time notice is acknowledged. It cannot
be turned off afterwards, as the notice says. Crash reports remain optional and are
sent only when the person taps Send. Ask your notes is off until the person enables
it, chooses a model provider and asks a question; it sends to that provider, not to
Kanso.

## Data not collected

Contacts, health, financial info, precise location, sensitive info, browsing
history, search history (searches run on the device), purchases, performance data,
photos and the content of notes except the excerpts a person explicitly sends through
Ask your notes. Sync stores notes as end-to-end encrypted ciphertext. App interactions
are declared above only in the narrow anonymous form the metrics page lists; there is
no session replay, screen recording, advertising identifier or cross-app tracking.

## Things that change these answers

- **Ask your notes.** AI is off by default. It is declared above as *User Content ›
  Other User Content*: when the person turns it on and asks, the chosen excerpts and
  question go to the model provider they configured, from their own key. The current
  public store copy does not advertise the feature until live-model QA is complete.
- **Crash reporting.** On-device reports the person chooses to send, no SDK. Declared
  above. A crash SDK would add its own data and linking.
- **Recordings and dictation with Parakeet.** On-device; they do not change these
  answers.
- **Self-hosted sync servers.** A person can point the app at their own sync server.
  The answers above describe the default one. The anonymous usage count still goes
  to Kanso's metrics Worker even when notes sync to a self-hosted server.

## The privacy manifest

`mobile/ios/Runner/PrivacyInfo.xcprivacy` declares the same: no tracking; the User
ID; anonymous Product Interaction, the pseudonymous Device ID, coarse country and
Crash Data; and two required-reason APIs (file timestamps `C617.1`, user defaults
`CA92.1`). Flutter's engine and each plugin carry manifests of their own. After the
first TestFlight upload, Apple emails any required-reason API it finds that is not
declared (ITMS-91053); add it to the manifest and this file if it does.
