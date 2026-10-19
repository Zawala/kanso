# Kanso themes

Color themes for [Kanso](https://github.com/iamngoni/kanso), the Markdown
notes app. Each theme is one JSON file. Kanso compiles the themes in this
folder in, and reads more from the `themes` folder in its data folder
(Settings › Themes › Open Folder), so you can try a theme by dropping its file
there: Kanso picks up the change within a couple of seconds.

As in Inkdrop, three themes are in use at once, and you choose each in
Settings › Themes:

- the **UI theme** colors the sidebar, the note list, menus and every control;
- the **syntax theme** colors the Markdown source in the editor, and code in it;
- the **preview theme** colors the rendered note.

Syntax and preview follow the UI theme unless you pick otherwise. With
*Switch between a light and a dark theme with the system* on, you pick one
light and one dark UI theme, and Kanso switches between them with macOS.

## The format

A theme file follows [`schema/kanso-theme-v1.json`](schema/kanso-theme-v1.json)
(JSON Schema 2020-12). The smallest theme gives a name, an appearance and
three colors; Kanso derives every other color from them, shading text until
it reads on the surfaces it sits on:

```json
{
  "$schema": "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/schema/kanso-theme-v1.json",
  "schema_version": 1,
  "name": "Night Pine",
  "author": "You",
  "license": "MIT",
  "appearance": "dark",
  "ui": { "editor": "#1B2420", "text": "#E6EFE9", "accent": "#7FC8A9" }
}
```

Top-level keys:

| Key | |
|---|---|
| `name` | Shown under the theme's card. Unique among themes. Required. |
| `appearance` | `"light"` or `"dark"`. Required. |
| `author`, `license`, `source`, `description` | Who made the palette, its SPDX license (e.g. `MIT`), where it comes from, a sentence about it. |
| `schema_version` | `1`. |
| `ui`, `syntax`, `preview` | The colors, below. Every key is optional except `ui.editor`, `ui.text` and `ui.accent`. |

Colors are `#RRGGBB`, or `#RRGGBBAA` with an alpha. A translucent UI or
preview color is painted over the editor surface.

### `ui`

| Key | Colors |
|---|---|
| `editor` | The editor and preview pane: the main surface. **Required.** |
| `text` | Body text on every surface. **Required.** |
| `accent` | Focus rings, the caret, the Active status. **Required.** |
| `app` | Behind everything: the window's top band. |
| `sidebar` | The sidebar of notebooks and tags. |
| `list` | The note list. |
| `elevated` | Menus, popovers and the command palette. |
| `fill` | Chips, banners and other quiet fills. |
| `text_secondary` | Text that is read but not primary: note previews, details. |
| `text_muted` | Quiet text: times, counts, hints. |
| `divider` | Lines between panes and rows; control borders. |
| `link` | Link text in the UI. |
| `row_active`, `row_hover` | The selected note in the list, and one under the pointer. |
| `sidebar_active`, `sidebar_hover` | The same in the sidebar. |
| `hover`, `selected` | Menu items and ghost buttons under the pointer; a selected ghost button. |
| `active_line` | The editor line with the cursor, when line numbers show. |
| `button`, `button_text` | Primary buttons and their text. |
| `success`, `warning`, `danger` | Statuses (Completed, On hold), errors and destructive actions. |
| `selection` | Selected text in the editor; usually translucent. |
| `caret` | The editor's caret. |

### `syntax`

Each value is a color, or an object with any of `color`, `font_style`
(`"normal"` or `"italic"`) and `font_weight` (100–900):

```json
"syntax": {
  "heading": { "color": "#88C0D0", "font_weight": 700 },
  "comment": { "color": "#616E88", "font_style": "italic" },
  "keyword": "#81A1C1"
}
```

Headings and strong text stay bold, and emphasis italic, when a theme gives
them only a color.

| Key | Colors |
|---|---|
| `text` | Plain Markdown text. |
| `heading` | Heading text. |
| `block_marker` | Heading `#` and quote `>` markers. |
| `marker` | Inline markers: `*`, backticks, code fences, table pipes, link brackets. |
| `list_marker` | List bullets and numbers. |
| `task` | Task boxes, `[ ]` and `[x]`. |
| `hr` | Horizontal rules. |
| `quote` | Quoted text. |
| `link`, `url` | Link text, and link destinations. |
| `code`, `code_block` | Inline code, and code blocks without a language. |
| `emphasis`, `strong` | *Emphasis* and **strong** text. |
| `front_matter`, `html`, `escape` | YAML front matter, HTML blocks, backslash escapes. |
| `comment`, `comment_doc` | Code: comments, documentation comments. |
| `keyword`, `string`, `string_special` | Code: keywords, strings, regular expressions and the like. |
| `function`, `type`, `type_constructor` | Code: function names, types, constructors and variants. |
| `number`, `boolean`, `constant` | Code: literals and constants. |
| `variable`, `variable_special`, `property`, `attribute` | Code: names. |
| `tag`, `label`, `operator`, `punctuation`, `bracket`, `embedded` | Code: the rest. |

The code keys are tree-sitter capture names, as in Zed's themes.

### `preview`

| Key | Colors |
|---|---|
| `text`, `heading`, `link` | Body text, headings, links. |
| `quote` | Quoted text and finished tasks. |
| `border` | Table borders and horizontal rules. |
| `code_background`, `code_text` | Code blocks, and their text when they have no language. |
| `inline_code_background`, `inline_code_text` | Inline code. |
| `table_header_background` | A table's header row. |
| `selection` | Selected text. |

Code blocks in the preview take the preview theme's `syntax` colors.

### What is derived

Anything left out is worked out from what is given: the panes are set back
from `editor` (darker in a dark theme, a step toward the text in a light
one), secondary and quiet text are mixed from `text` and `editor`, links and
buttons come from `accent`, and syntax and preview colors from the UI
colors. When a syntax or preview theme is paired with another UI theme, its
colors are shaded until they read on that theme's surfaces.

## Importing

`Import Theme…` on the Themes page, or `kanso theme import <path>`, converts:

- an **Inkdrop** theme: a v6 package (`package.json` with `"theme": true`,
  `ui.css`, `syntax.css`, `preview.css`) as a folder or `.tgz`, and, as well as
  it can, a v5 UI, syntax or preview package;
- a **Zed** theme family (`.json`, one Kanso theme per theme in it).

The result lands in the themes folder as an ordinary theme file to edit.

## Contributing a theme

1. Add `your-theme.json` here, with `author`, `license` and `source` saying
   where the palette comes from; only add palettes whose license allows it.
2. Keep it legible: body text at 4.5:1 on every surface, secondary and
   quiet text at 3:1, every syntax color at 3:1 on the editor.
   `kanso theme check your-theme.json` lists anything that falls short.
3. Run `scripts/check.sh`: it checks every theme against the schema, rewrites
   `index.json` and renders `thumbnails/your-theme.png`.
4. Open a pull request with the thumbnail.

`kanso theme fmt <files>` writes a file in the canonical key order, and
`kanso theme schema` prints the schema the app reads.

## Licenses

Each theme carries its palette's license in its `license` field, with the
palette's home in `source`. The palettes here are all MIT licensed; the
files themselves are MIT, like Kanso.
