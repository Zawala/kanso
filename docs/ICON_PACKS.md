# Kanso icon packs

An icon pack is a folder of SVG icons. Kanso reads packs from the
`icon-packs` folder in its data folder (Settings › Themes › Icon packs ›
Open Folder) and picks up changes within a couple of seconds, or on Reload.

## Getting packs

**Browse Icon Packs…** (Settings › Themes › Icon packs, the palette, View ›
Icon Packs…, or **Get More Icons…** in a notebook's icon picker) lists the
open-source icon sets on [Iconify](https://iconify.design): Lucide, Phosphor,
Tabler, Material Symbols, Remix, Heroicons and nearly two hundred more, each
with its author, licence, size and a few of its icons. **Install** downloads
the set whole (one file, from its package on jsDelivr) into the icon packs
folder; **Uninstall** removes the files it wrote, and the folder once nothing
else is in it (files you added there stay, and Kanso says which). An install
carries on if the gallery is closed, and gives up after three minutes if the
download stalls. Only one-color sets are listed, since
Kanso draws icons in the theme's color. Nothing is fetched until the gallery
opens, and nothing about your notes is sent: only the names of the sets and
icons asked for. A set installed from the gallery keeps its licence in its
`pack.json`.

You can also make or copy a pack by hand, as below.

## What a pack does

A pack does two things:

- **Notebook icons.** Its icons join the picker in a notebook's menu
  (**Change Icon…**), in a section named after the pack, and the search finds
  them by their names, labels and keywords, and by the pack's name. A
  notebook keeps the icon as `<pack>/<icon>` and syncs it. A device without
  the pack shows the default notebook icon instead until the pack is added.
- **Kanso's own icons.** An SVG named like one of the icons Kanso draws
  (`pin.svg`, `chevron-right.svg`, `notebook.svg`, `search.svg`…) replaces it
  everywhere: the sidebar, the toolbar, menus and the palette. Choose the pack
  with **Use for App Icons**: the window opens again in its place with the
  new icons, the note you're writing saved first. A pack that
  covers only some icons leaves the rest as they are. Kanso's names are
  [Lucide](https://lucide.dev/icons)'s, so a pack made for Lucide names fits.

## The format

A pack holds its icons as SVG files, or all of them in one `icons.json` in
[Iconify's format](https://iconify.design/docs/types/iconify-json.html), as the
gallery installs them:

```
icon-packs/
  sketch/            the pack's ID: lowercase letters, digits, - and _
    pack.json        optional
    circle-dot.svg   one icon per file, named the same way
    wave.svg
```

`pack.json` names the pack and, optionally, its icons:

```json
{
  "name": "Sketch",
  "author": "Your name",
  "license": "CC0",
  "icons": {
    "wave": { "label": "Wave", "keywords": "sea water" }
  }
}
```

Every field is optional. Without a `name`, the folder's name is used
("my-icons" shows as "My icons"); without a `label`, the file's name is.

## Drawing the icons

Kanso draws an icon as a shape filled with the theme's text color, like
its built-in icons, so a pack matches every light and dark theme. That means:

- **One color.** Only the shape matters: colors in the file are ignored, and a
  multicolored icon shows as a single-color silhouette.
- Draw on a square `viewBox`, such as `0 0 24 24`. Strokes and fills both
  work; `stroke="currentColor"` is fine.
- An icon is at most 256 KB, an `icons.json` at most 48 MB, and a pack at most
  40,000 icons and 64 MB of them, an icon's other names (Iconify's aliases)
  counted as icons but sharing its file; Kanso reads up to 64 packs.

A file that can't be used (a name with capitals or spaces, a file that isn't
an SVG, one that is too large) is skipped, and Settings › Themes lists it with
the reason. It never stops the rest of the pack.
