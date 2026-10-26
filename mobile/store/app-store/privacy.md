# App Privacy ("nutrition label") answers: a draft for the owner

For App Store Connect › App Privacy. Derived from `docs/ACCOUNTS.md` ("What the
server can and can't see"), the privacy page on kansoapp.app, and the app's code
as of step 13. **The owner submits these answers and is responsible for them**
(Apple: [App privacy details](https://developer.apple.com/app-store/app-privacy-details/)).
Anything marked **Check** is a judgement call that is not settled by the code.

Apple's word "collect" means sending data off the device "in a way that allows you
and/or your third-party partners to access it for a period longer than what is
necessary to service the transmitted request in real time". Notes, titles, names
and files leave the phone only as ciphertext the sync server cannot open, so they
are not collected.

## Do you or your third-party partners collect data from this app?

**Yes, but only if the person signs in to sync.** Without an account the app sends
nothing anywhere. Kanso has no analytics, advertising, attribution or crash
reporting SDK in the app, and no third-party partners whose code is in the app.

## Data used to track you

None. (`NSPrivacyTracking` is `false` and there are no tracking domains.)

## Data linked to you (only with a sync account)

| Data type (Apple's name) | What it is here | Purpose | Linked to identity | Tracking |
|---|---|---|---|---|
| Identifiers › **User ID** | The account's username. For a sign-in with Apple, Google or GitHub, the provider's account number for the person (never an email, name or profile), stored beside the username | App Functionality | Yes | No |

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
draft does not declare it.

## Data not collected

Contacts, health, financial info, location, sensitive info, browsing history,
search history (searches run on the device), purchases, usage data, diagnostics,
photos, audio and the content of notes, as far as the server is concerned
(end-to-end encrypted).

## Things that change these answers

- **Crash reporting** (issue #48 open question). The recommended option (the person
  taps Send on a report without note text) is optional disclosure under Apple's
  rules only if it meets every criterion listed on the page above; a Sentry-style
  SDK would add *Diagnostics › Crash Data* (not linked, App Functionality or
  Analytics). Update this file with the owner's answer.
- **Ask your notes** (step 12). If it ships in this release, the person's chosen AI
  service receives the note text they ask about, from their own key. Declare
  *User Content › Other User Content* as collected by that third party, linked
  to the person, purpose App Functionality, and say so in the description. This
  draft assumes it does not ship in 0.9.0.
- **Recordings and dictation with Parakeet** (step 12) are on-device and do not
  change the answers.
- **Self-hosted servers.** A person can point the app at their own sync server.
  The answers above describe the default one.

## The privacy manifest

`mobile/ios/Runner/PrivacyInfo.xcprivacy` declares the same: no tracking, the User
ID above, and two required-reason APIs (file timestamps `C617.1`, user defaults
`CA92.1`). Flutter's engine and each plugin carry manifests of their own. After the
first TestFlight upload, Apple emails any required-reason API it finds that is not
declared (ITMS-91053); add it to the manifest and this file if it does.
