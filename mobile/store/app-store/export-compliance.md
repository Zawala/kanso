# Export compliance answers: a draft for the owner

For App Store Connect › the app › App Information › App Encryption Documentation, or
the questions App Store Connect asks on the first build marked "Missing Compliance".
These are **provisional decision 2 of #48 step 13 (2026-10-26)**. The owner can overrule
them, submits them and answers for them. The background is in
[RELEASING-MOBILE.md › Export compliance](../../../docs/RELEASING-MOBILE.md#export-compliance).
Nothing here is legal advice, and the points marked **Check** were not confirmed from
primary sources.

## The facts from the code

- Kanso seals note text, titles, notebook and tag names, and files before they are synced,
  with **standard algorithms that are not Apple's**: Argon2id, HKDF-SHA256,
  XChaCha20-Poly1305, AES-256-GCM, X25519 and Ed25519. They come from Rust crates in the
  app, not from CryptoKit or CommonCrypto (`docs/ACCOUNTS.md`).
- Connections use TLS (rustls through the platform's certificate verifier).
- No proprietary or non-standard algorithm is used anywhere.
- Kanso's source code, including all of this cryptography, is public on GitHub under the
  AGPL (`LICENSE`).

## Info.plist

`ITSAppUsesNonExemptEncryption` = **`YES`** (`mobile/ios/Runner/Info.plist`, unchanged).
Kanso uses encryption that Apple doesn't provide, so it can't claim the "only exempt
encryption" answer that `NO` means.

## The questions in App Store Connect, and the answers

| Question (Apple's wording may vary) | Answer |
|---|---|
| What type of encryption algorithms does your app implement? | **Standard encryption algorithms instead of, or in addition to, using or accessing the encryption within Apple's operating system** |
| Is your app going to be available on the App Store in France? | **No, for now** (see France below). Answer Yes and upload the declaration once it is filed |
| *(Older form)* Does your app qualify for any of the exemptions provided in Category 5, Part 2 of the U.S. Export Administration Regulations? | **Yes**: mass-market encryption (Note 3 to Category 5, Part 2; 5D992.c), and publicly available encryption source code |

When the documentation is approved, Apple returns a code. Put it in the `mobile-release`
environment's variable `IOS_EXPORT_COMPLIANCE_CODE`, and the release workflow adds it to
Info.plist as `ITSEncryptionExportComplianceCode`. App Store Connect then stops asking for
every build.

## France

Apple's table: an app that uses standard encryption that isn't Apple's needs the
**French encryption declaration** (ANSSI) if it is on the App Store in France.

**Provisional decision:** France is left off the app's availability (App Store Connect ›
Pricing and Availability) until the owner has filed the declaration. Leaving it off costs
nothing to undo. To file it:

- Use ANSSI's declaration form for "fourniture de moyens de cryptologie" (supplying a
  cryptology product, mass-market software). The figures it asks for are the ones in "The
  facts from the code" above. The publisher is the owner; the product is
  "Kanso for iPhone and iPad", a notes app.
- ANSSI acknowledges the declaration. Upload that acknowledgement in App Store Connect, then
  add France back.

**Check:** ANSSI's current form and its delays. Play has no equivalent question.

## United States

- **Publicly available source code:** Kanso's encryption source code is published. Since
  BIS's rule of 29 March 2021, publicly available encryption source code that uses only
  standard cryptography leaves the EAR without the notification email that used to be
  needed (15 CFR 742.15(b), 734.3(b)(3)).
- **The app binary:** a mass-market item (5D992.c under Note 3), self-classified. The same
  rule ended the annual self-classification report for most mass-market items. It is still
  needed for chips, chipsets, components and their "executable software", which Kanso is
  not.
- **So nothing needs filing with BIS.** **Check** this with counsel if Kanso is ever sold to
  government end users or embargoed destinations. The App Store's own country restrictions
  already cover the embargoed ones.

Sources: [Apple: Complying with encryption export regulations](https://developer.apple.com/documentation/security/complying-with-encryption-export-regulations);
[Jones Day on the 2021 rule](https://www.jonesday.com/en/insights/2021/04/commerce-reduces-requirements-relating-to-massmarket-encryption-items-and-publicly-available-software);
[Thompson Hine on the 2021 rule](https://www.thompsonhine.com/publications/bis-eliminates-most-mass-market-encryption-reporting-obligations);
[BIS: Encryption](https://www.bis.gov/encryption).
