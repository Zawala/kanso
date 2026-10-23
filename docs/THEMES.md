# Themes and the gallery

Kanso colors itself with a UI theme, an editor syntax theme and a preview
theme, and draws its icons with an icon theme. All of them work offline: the
compiled-in themes are always there, and a person's own themes live in the
`themes/` and `icon-themes/` folders of Kanso's data folder.

The **gallery** is the same kind of themes, published in
[`iamngoni/kanso-themes`](https://github.com/iamngoni/kanso-themes) as a
signed catalog, so a new theme or a fix reaches Kanso without a release.

## Browse and install

Settings › Themes and Settings › Icon Themes have an **Installed | Browse**
switch.

- **Browse** lists every theme in the catalog. Search looks through the name,
  family, author, licence and description; the filters are **All**, **Light**,
  **Dark**, **Installed** and **Updates**.
- A color theme's card is a live miniature drawn from the same file Kanso
  would install. Before that file arrives the card uses the five headline
  colors in the catalog; if the fetch fails it says "Connect to see this
  theme."
- A family with exactly one light and one dark theme is one **paired card**;
  a family with three variants stays separate cards under the family name.
- **Install** adds the theme under `<data>/gallery/`, apart from your own
  folders, and does not change the theme in use. **Use** selects it (both
  slots of a pair when Kanso follows the system). **Update** appears when a
  newer signed version is ready. **Remove** deletes only the gallery copy; if
  the theme was in use Kanso falls back to Kanso Light or Kanso Dark (or
  Lucide for icons) and says so.
- **Preview** applies a color theme to the whole window from the fetched,
  checked file, without writing a file or a setting. Escape or **Stop**
  restores the themes in force before the preview. A theme that fails the
  legibility checks does not preview; Browse lists the failing pairs.
- **Needs Kanso <version>** means the catalog's entry is for a newer app; the
  rest of the card still reads.

## Badges

| Badge | Meaning |
|---|---|
| **Built in** | Compiled into Kanso. Always available, never updated by the gallery. |
| **Yours** | A file you added or imported. The gallery never touches it. |
| **Gallery** | Installed from the gallery; the card includes its version, author and licence. |
| **Edited** | The installed copy no longer matches what Kanso wrote. It is never overwritten; remove it or leave it. |
| **Missing** | The installed files are gone. Reinstall is offered; nothing happens automatically. |
| **Not in the gallery** | The theme disappeared from the catalog but was not revoked. It keeps working and is no longer updated. |

## Offline

Installed and built-in themes always work. Browse shows the last catalog it
verified; nothing blocks on the network. A manual **Refresh** reports "Kanso
couldn't reach GitHub. It tries again later." Automatic checks fail
silently, and Settings shows a quiet line while installed themes keep
working.

## Automatic updates and revocation

Kanso checks the catalog on its app-update timer: 20 seconds after launch,
then every half hour, five minutes after a failure and within a minute of
waking. Automatic updates only touch themes you installed, and only while
the installed copy is still exactly what Kanso wrote. **Keep gallery themes
up to date** (on by default) turns this off without turning off app updates.
Failures are written to `<data>/logs/themes.log`.

A **revoked** theme was withdrawn by the catalog owner with a signed entry.
Kanso removes it, falls back if it was selected, and says why. If you edited
the revoked copy, it is kept on disk but disabled and unselected until the
owner decides that case (issue #49, question 4); a notice says why. A theme
that merely disappears from the catalog is **not** revoked and stays
installed.

## Where gallery themes live

```
<data>/gallery/
  index.json, index.signature   the last verified catalog
  meta.json                     the serial floor, ETags and last check
  state.json                    installed records and a pending swap
  themes/<id>.json              installed color themes
  icon-themes/<id>/             installed icon themes
  cache/                        verified downloads
  staging/                      installs in progress
```

Open Folder still shows only your own themes. Removing a gallery theme never
touches a file you put in `themes/` or `icon-themes/`, and your own theme
with the same name always wins over a gallery one.

## What is compiled in

This release keeps every color theme and icon theme compiled in. They are
the offline, always-there set, so first run works without a catalog and
fallbacks always exist. A later release may shrink to Kanso Light, Kanso
Dark and Lucide, but only with a migration that installs gallery copies of
any selected theme before removing the built-in one.

## Security

The catalog is verified with an Ed25519 signature over its exact bytes
against keys compiled into Kanso, before it is parsed. Every file's SHA-256
and size are in that signed catalog and are checked before a file is
written. The highest serial Kanso has accepted is stored, so an older index
is refused. Icon SVGs are parsed and rebuilt against an allowlist before
they are drawn, at install, at import and at load. The themes signing key is
separate from the app-update key; see `docs/RELEASING.md` for its setup and
rotation.
