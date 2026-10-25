# Highlight queries

These `.scm` files are copied unchanged from `gpui-component` 0.7.3 (the
component library the desktop app draws with), from
`src/highlighter/languages/<language>/`, where they are the queries the
desktop editor highlights with:

| Here | There |
|---|---|
| `markdown.scm` | `markdown/highlights.scm` |
| `markdown_injections.scm` | `markdown/injections.scm` |
| `markdown_inline.scm` | `markdown_inline/highlights.scm` |
| `<language>.scm` for `go`, `html`, `javascript`, `json`, `kotlin`, `lua`, `rust`, `typescript` | `<language>/highlights.scm` |

Licence: Apache-2.0, Copyright 2024 - 2026 Longbridge
(<https://longbridge.com>); the licence text is `LICENSE-APACHE` here. The
rust query is adapted there from tree-sitter-rust's. Kanso is AGPL-3.0-or-later,
which Apache-2.0 code can be part of.

Every other grammar uses the highlight query its own crate ships, as the
library does. The library's injection queries for code languages (Rust
macros, JavaScript templates, HTML scripts, PHP) are not copied: injected code
is highlighted with its language's highlight query alone, with no injections of
its own, which is what the library does too.

The Markdown queries are patched when they are read (`highlight::queries`), so
the desktop and the mobile app patch them identically. A test in `kanso-app`
(`markdown_source`) checks that these copies, patched, equal what the
library's registry holds after the desktop's own patching, so a library
update that changes a query fails a test instead of drifting.
