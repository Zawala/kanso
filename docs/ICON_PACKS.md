# Kanso icon packs

An icon pack is a folder of SVG icons. Kanso reads packs from the
`icon-packs` folder in its data folder (Settings › Themes › Icon packs ›
Open Folder) and picks up changes within a couple of seconds, or on Reload.

A pack does two things:

- **Notebook icons.** Its icons join the picker in a notebook's menu
  (**Change Icon…**), in a section named after the pack, and the search finds
  them by their names, labels and keywords, and by the pack's name. A
  notebook keeps the icon as `<pack>/<icon>` and syncs it. A device without
  the pack shows the default notebook icon instead until the pack is added.
- **Kanso's own icons.** An SVG named like one of the icons Kanso draws
  (`pin.svg`, `chevron-right.svg`, `notebook.svg`, `search.svg`…) replaces it
  everywhere: the sidebar, the toolbar, menus and the palette. Choose the pack
  with **Use for App Icons**. Kanso shows the new icons the next time it
  opens, since a window keeps the icons it has already drawn. A pack that
  covers only some icons leaves the rest as they are. Kanso's names are
  [Lucide](https://lucide.dev/icons)'s, so a pack made for Lucide names fits.

## The format

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
- An icon is at most 256 KB, and a pack at most 4,000 icons; Kanso reads up to
  64 packs.

A file that can't be used (a name with capitals or spaces, a file that isn't
an SVG, one that is too large) is skipped, and Settings › Themes lists it with
the reason. It never stops the rest of the pack.
