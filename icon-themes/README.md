# Kanso icon themes

Icon themes for [Kanso](https://github.com/iamngoni/kanso), the Markdown
notes app. Each theme is a folder: one JSON file and the SVGs it names.
Kanso compiles the themes in this folder in, and reads more from the
`icon-themes` folder in its data folder (Settings › Icon Themes › Icon theme
files › Open Folder), so you can try a theme by dropping its folder there:
Kanso picks up the change within a couple of seconds.

The icon theme draws every icon in Kanso: the sidebar, the note list, the
toolbar, menus, the palette and every control, the component library's own
(a checkbox's check, a menu's arrow) included. Choose one in
Settings › Icon Themes, or with **Change icon theme…** in the
palette; the new icons show at once, in every window. An icon a theme
leaves out stays Kanso's own, icon by icon.

These are built in:

- **Lucide**, Kanso's own icons ([Lucide](https://lucide.dev), ISC);
- **Phosphor**, the regular weight of [Phosphor](https://phosphoricons.com)
  (MIT), drawn for every icon Kanso has, with its own notebook, Inbox and
  Journal icons and 34 more icons for notebooks;

and these, converted from Zed icon themes, which draw a notebook (folded and
open), a note and attachments, and leave the rest to Kanso's own:

- **Catppuccin Icons** by Catppuccin ([source](https://github.com/catppuccin/zed-icons), MIT);
- **Charmed Icons** by littensy, jmesrje ([source](https://github.com/jmesrje/zed-charmed-icons), MIT);
- **FantastIcons** by Caio Landgraf ([source](https://github.com/caiolandgraf/fantasticons-zed), MIT);
- **JetBrains Icons** by Zeeshan Ahmad ([source](https://github.com/ziishaned/zed-jetbrains-icons), Apache-2.0);
- **JetBrains New UI Icons** by ankddev ([source](https://github.com/ankddev/zed-jetbrains-newui-icons), Apache-2.0);
- **Kokedera Icons** by Florian Mühlhans ([source](https://github.com/cappuccinotogo/kokedera-icons-extension-zed), MIT);
- **Material Icon Theme** by Zed Industries ([source](https://github.com/zed-extensions/material-icon-theme), MIT);
- **Min Icons** by Phillip Bronzel ([source](https://github.com/phibr0/zed-min-theme), MIT);
- **Monospace Icons** by Isheunopa Ralph Mhonde ([source](https://github.com/irmhonde/monospace-icon-theme), Apache-2.0);
- **Phosphor Icons Theme** by Theo Luciano ([source](https://github.com/theoluciano/phosphor-icons-theme), MIT);
- **Serendipity Icons** by Nguyen Dang Vinh ([source](https://github.com/meocoder31099/Serendipity-Icon-Theme-Zed), MIT);
- **Symbols** by Joan Garcia ([source](https://github.com/joangarciaa-dev/zed-symbols), MIT).

## The format

A theme file follows
[`schema/kanso-icon-theme-v1.json`](schema/kanso-icon-theme-v1.json) (JSON
Schema 2020-12). The smallest theme gives a name and one icon:

```
icon-themes/
  sketch/                the theme's ID: lowercase letters, digits, - and _
    sketch.json          one .json file
    icons/search.svg     the SVGs, anywhere inside the folder
    icons/notebook.svg
    icons/planet.svg
```

```json
{
  "$schema": "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/icon-themes/schema/kanso-icon-theme-v1.json",
  "schema_version": 1,
  "name": "Sketch",
  "author": "You",
  "license": "CC0-1.0",
  "icons": { "search": "icons/search.svg" },
  "library": { "notebook": "icons/notebook.svg" },
  "notebook_icons": {
    "planet": { "file": "icons/planet.svg", "label": "Planet", "keywords": "space orbit" }
  }
}
```

Top-level keys:

| Key | |
|---|---|
| `name` | Shown under the theme's card. Unique among icon themes. Required. |
| `author`, `license`, `source`, `description` | Who drew the icons, their SPDX license (e.g. `MIT`), where they come from, a sentence about them. |
| `schema_version` | `1`. |
| `icons`, `library`, `notebook_icons` | The icons, below. Every one is optional. |

A path is relative to the theme's folder, with `/` between its parts, each
of letters, digits, `.`, `-` and `_`, and ends in `.svg`. One file can draw
several icons.

### `icons`

Kanso's UI icons by name, which are [Lucide](https://lucide.dev/icons)'s:
`search`, `settings`, `close`, `check`, `chevron-down`, `notebook`, `pin`,
`trash` and the rest. The schema lists every one
Kanso draws, and a name it doesn't draw is an error, so a typo doesn't pass
silently.

### `library`

The library's own icons, which a theme can draw apart from the UI icon
they show by default. A slot left out falls back to the one in brackets,
which the theme may draw in turn:

| Key | Draws |
|---|---|
| `notebook` | A notebook without an icon of its own (`notebook`). |
| `notebook-open` | One opened in the sidebar to show the ones inside it (`library.notebook`). |
| `notebook-locked` | A locked notebook, in place of its icon (`lock`). |
| `fold-closed`, `fold-open` | The arrow beside a notebook or tag level, folded and open (`chevron-right`, `chevron-down`). |
| `inbox`, `journal`, `templates` | The Inbox, the Journal and the Templates notebooks (`library.notebook`). |
| `note` | A note in the palette (`file-text`). |
| `note-pinned`, `note-locked`, `note-shared`, `note-published` | The marks on a note (`pin`, `lock`, `users`, `globe`). |
| `tag` | A tag (`hash`). |
| `all-notes`, `pinned`, `recent`, `tasks`, `shared`, `trash` | The places in the sidebar and the palette (`file-text`, `pin`, `clock`, `circle-check`, `users`, `trash`). |
| `notebooks`, `tags` | The collapsed sidebar's notebooks and tags (`notebook`, `hash`). |
| `board`, `calendar`, `graph` | The board, the journal calendar and the graph (`square-kanban`, `calendar-days`, `waypoints`). |
| `attachment-pdf`, `attachment-image`, `attachment-audio` | Kept for attachments; nothing draws them yet (`file`, `image`, `music`). |

### `notebook_icons`

More icons for a notebook's **Change Icon…** picker, in a section named
after the theme, found by their names, labels, keywords and the theme's
name. Each is a path, or `{ "file", "label", "keywords" }`. A notebook keeps
one as `<theme>/<icon>` and syncs it; under another icon theme, or on a
device without the theme, it shows the default notebook icon.

## Drawing the icons

Kanso draws an icon as a shape in the theme's text color, so an icon theme
matches every light and dark color theme:

- **One color.** Only the shape matters: colors in the file are ignored, and
  a multicolored icon shows as a single-color silhouette.
- Draw on a square `viewBox`, such as `0 0 24 24`. Strokes and fills both
  work; `stroke="currentColor"` is fine.

## What Kanso reads

A theme's folder comes from anyone, so it is read with care: the JSON is
at most 512 KB and names at most 2,000 icons; an SVG is at most 128 KB, and
a theme at most 16 MB of them; a path never leaves the folder (no `..`, no
absolute paths, and a link that leads out isn't followed); Kanso reads up to
64 themes. Nothing is fetched: a theme is only ever local files.

A theme file that can't be read (not JSON, an unknown key, a name that's
taken) is skipped, and Settings › Icon Themes lists it with the reason. An SVG
that can't be used (missing, too large, not an SVG) is listed too, and the
rest of the theme still draws.

## Importing

**Import…** under Settings › Icon Themes › Icon theme files, **Import Icon
Theme…** in the palette, or `kanso icon-theme import <path>`, converts:

- a **Zed** icon theme: an extension's `icon_themes/*.json`, its folder, or
  its `.tar.gz`;
- a **theme package** (an npm-style package folder, or the `.tgz` its
  registry serves) carrying one, in an `icon_themes/` folder or named by
  `iconTheme` in its `package.json` (Zed's format or Kanso's);
- a **Kanso icon theme** folder, checked and copied.

A Zed theme's folder icons become `notebook` and `notebook-open`, its
chevrons `fold-closed` and `fold-open`, its default file icon `note`, its
named folders the special notebooks where the names line up (`inbox`,
`journal`, `templates`, `trash`, `tags`), and its PDF, image and audio file
icons the attachment slots (by the theme's own `file_suffixes`, else by
Zed's default suffixes, as Zed does). Everything else is left to Kanso's
own icons. A theme file to convert may be up to 8 MB; what Kanso writes is
held to the limits above.

The result lands in the icon themes folder as an ordinary theme folder to
edit, with only the SVGs it uses (each checked as above) and every license
file the source has (`LICENSE`, `LICENSE-MIT`, `COPYING`…). Its author and
license are the ones `extension.toml` or `package.json` give, its `source`
the repository or homepage they name. The import says what mapped and what
was left out; importing again updates the same folder. A theme with no icon
Kanso can use isn't imported, and variants drawing the same shapes in other
colors (a light and a dark one, say) are imported once, since Kanso draws
icons in the text color anyway. An archive is unpacked by Kanso itself,
regular files only, at most 64 MB and 20,000 files, and never outside its
own temporary folder.

`kanso icon-theme check <folder>` reads a theme as Kanso does and lists
anything wrong, and `kanso icon-theme schema` prints the schema.

## Contributing a theme

1. Add `your-theme/` here, with its JSON, its SVGs (only the ones it names)
   and its LICENSE, and `author`, `license` and `source` saying where the
   icons come from; only add icons whose license allows it.
2. Run `kanso icon-theme check icon-themes/your-theme`, and
   `cargo test -p kanso-app icon_themes`: every SVG in the folder has to be
   used.
3. Open a pull request with a screenshot of Settings › Icon Themes.

## Licenses

Each theme carries its icons' license in its `license` field and its folder's
LICENSE file: Lucide is ISC, Phosphor is MIT (© 2023 Phosphor Icons). The
theme files themselves are MIT.
