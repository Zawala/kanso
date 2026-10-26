# Store listings

The words and pictures for the App Store and Google Play, written once here so they
are reviewed like code. Nothing in this folder is uploaded by CI except
`play/whatsnew/whatsnew-en-US`, which goes to Play with each internal release.
The rest is pasted or uploaded by the owner into the consoles
([docs/RELEASING-MOBILE.md](../../docs/RELEASING-MOBILE.md), "What the owner does").
`scripts/mobile-store-check.sh` (run by the Mobile release workflow) holds every text
to its store's limit and keeps other notes apps out of them.

All text is English (US). Kanso's voice: plain, calm, no hype.

## Files

| File | Where it goes | Limit |
|---|---|---|
| `app-store/name.txt` | App Store Connect › App Information › Name | 30 |
| `app-store/subtitle.txt` | Subtitle | 30 |
| `app-store/promotional-text.txt` | Version › Promotional Text (can change without a review) | 170 |
| `app-store/description.txt` | Description | 4000 |
| `app-store/keywords.txt` | Keywords, comma-separated, no spaces | 100 |
| `app-store/whats-new.txt` | Version › What's New (0.9.0) | 4000 |
| `app-store/support-url.txt`, `marketing-url.txt`, `privacy-policy-url.txt` | URLs | |
| `app-store/primary-category.txt`, `secondary-category.txt`, `copyright.txt` | | |
| `app-store/privacy.md` | App Privacy answers (draft), with the open points | |
| `app-store/review-notes.md` | App Review Information › Notes | |
| `play/title.txt` | Main store listing › App name | 30 |
| `play/short-description.txt` | Short description | 80 |
| `play/full-description.txt` | Full description | 4000 |
| `play/whatsnew/whatsnew-en-US` | Release notes (sent by CI) | 500 |
| `play/category.txt`, `support-url.txt`, `website-url.txt`, `privacy-policy-url.txt` | | |
| `play/data-safety.md` | Data safety form (draft), with the open points | |

The support URL is the project's GitHub issues page: it works, but asks people for a
GitHub account. A page on kansoapp.app (with the account deletion link both stores want)
is better; change the files when it exists.

## What each claim depends on

The descriptions are written for the app as designed in
[#48](https://github.com/iamngoni/kanso/issues/48). Before submitting, strike anything that
has not shipped.

| Claim | Build step |
|---|---|
| Notebooks, tags, pinned, recent, library; search with highlights; themes; three panes on iPad | 5 (merged) |
| Markdown editor, styling as you type, the bar above the keyboard, dictation, autocorrect | 6 |
| Preview, math and diagrams, wiki-links followed | 7 |
| End-to-end encrypted sync, sign-in methods, device approval, recovery key | 8 |
| Locked notes, Face ID / fingerprint, the cover in the app switcher | 9 |
| Share sheet, widgets | 10 |
| Not claimed (drop-in later): canvases and ink (11), Ask your notes, recordings (12) | |
| Kept out of iCloud and device backups | 5 (merged) |
| No ads, trackers or analytics | true while crash reporting stays undecided |

If Ask your notes ships in 0.9.0, add it to the descriptions and to the privacy answers.

## Screenshots

Apple needs screenshots for the largest iPhone and iPad sizes (smaller ones are scaled
from them); Play needs 2 to 8 per device type, JPEG or 24-bit PNG without alpha, each side
between 320 and 3,840 px (tablet screenshots 1,080 px or more), the long side at most twice
the short one; at least 4 at 1,080 px or more are needed for some featuring.
Sources: [App Store Connect screenshot specifications](https://developer.apple.com/help/app-store-connect/reference/screenshot-specifications),
[Play Console: add preview assets](https://support.google.com/googleplay/android-developer/answer/9866151).

| Set | Device to use | Pixels (portrait) | Used for |
|---|---|---|---|
| `ios-iphone-6.9` | iPhone 17 Pro Max simulator | 1320 × 2868 | App Store, 6.9-inch iPhone |
| `ios-ipad-13` | iPad Pro 13-inch (M5) simulator | 2064 × 2752 | App Store, 13-inch iPad |
| `android-phone` | an emulator at 1080 × 1920 | 1080 × 1920 | Play phone |
| `android-tablet` | an emulator at 1600 × 2560 | 1600 × 2560 | Play 10-inch tablet (and 7-inch) |

Five screens, in this order, each in light (the dark variants as a second row if the stores'
count allows): **01 All notes** (the list, with the pinned note), **02 Library** (notebooks,
tags), **03 A notebook**, **04 A note**, **05 Search** with a highlighted match. Add **06
the editor with the keyboard bar**, **07 a locked note** and **08 an iPad in three panes**
as steps 6 and 9 land. The pictures show the app's own seeded library, not real notes.

```sh
scripts/mobile-store-screenshots.sh ios-iphone-6.9 "iPhone 17 Pro Max"
scripts/mobile-store-screenshots.sh ios-ipad-13 "iPad Pro 13-inch (M5)"
scripts/mobile-store-screenshots.sh android-phone emulator-5554
scripts/mobile-store-screenshots.sh android-tablet emulator-5554
```

The script runs `integration_test/store_screenshots_test.dart` with
`test_driver/store_screenshot_driver.dart` (`flutter drive`), writes PNGs to
`mobile/store/screenshots/<set>/` (git-ignored) and fails if a picture isn't the set's
size. Boot the simulator or emulator first, at the set's size; set the status bar to 9:41
and full battery for the iPhone (`xcrun simctl status_bar booted override --time 9:41
--batteryState charged --batteryLevel 100 --cellularBars 4`). They are not generated in CI:
look at each picture before it goes to a store. It has not been run for this change (it needs
the simulators and a full build of the Rust library); `flutter analyze` passes on it.

A 512 × 512 icon and a 1024 × 500 feature graphic are also needed for Play, and the 1024 × 1024
icon for the App Store comes from the asset catalog; none is made here.
