# Builder fixture

`builder-index.json` is a byte-for-byte copy of `gallery/index.json` from the
`kanso-themes` builder (`node scripts/gallery.mjs`). It carries every theme
the builder publishes, including the icon preview pins, so Kanso's parser is
tested against the real shape rather than a hand-written example.

Regenerate it after a builder change:

```sh
git clone https://github.com/iamngoni/kanso-themes /tmp/kanso-themes
cd /tmp/kanso-themes && node scripts/gallery.mjs
cp gallery/index.json <this directory>/builder-index.json
```
