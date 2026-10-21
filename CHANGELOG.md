# Changelog

Every release of Kanso, newest first. Each entry links its commit, and each
heading compares the release with the one before.

## [0.6.0](https://github.com/iamngoni/kanso/compare/v0.5.2...v0.6.0) - 2026-10-20

### New

- **cloud:** Short links are made on go.kansoapp.app ([2c13feb](https://github.com/iamngoni/kanso/commit/2c13febb560e61646d9276540646960054a91556))
- **site:** Kanso's website, kansoapp.app ([e339fd3](https://github.com/iamngoni/kanso/commit/e339fd36f17a7dc5a871e3161907a022603e1e27))
- **import:** Markdown folders, Obsidian, Logseq, Notion, Evernote, Bear, Joplin, Standard Notes, Simplenote and UpNote come in, with links, tags and attachments ([ccb4a03](https://github.com/iamngoni/kanso/commit/ccb4a034d1faf482ec2a4872fdcc0de88c053f39))
- **app:** Import… previews and imports other apps' notes, File › Open… brings in a Markdown file, and kanso import does both from a terminal ([d56f379](https://github.com/iamngoni/kanso/commit/d56f3799f4488399d70813b4b760a13529183869))
- **site:** The home page is Kanso, running in the browser ([4d03793](https://github.com/iamngoni/kanso/commit/4d037936d98b2a2f8383019880ff4d5bc9dc0dbb))

### Fixed

- **import:** A Joplin export's name comes from its file on Windows too ([df9b0aa](https://github.com/iamngoni/kanso/commit/df9b0aa56fbc4b413c51f36d8191ae2d2393a8b4))

### Under the hood

- **licence:** Kanso is free software under the AGPL-3.0-or-later ([9ff280e](https://github.com/iamngoni/kanso/commit/9ff280e1fea6c26cb3f969d959b00ec6a8c3ac22))
- **snapshots:** The demo roadmap lists wiki-links instead of naming another app ([40e8b93](https://github.com/iamngoni/kanso/commit/40e8b93deb8465296374e07787f96065fa20e771))
- **import:** The Notion export is written by its test, so Windows can check out the repository ([5c4da38](https://github.com/iamngoni/kanso/commit/5c4da38332106d8d864f587e79c8021d6f1e0ecb))

## [0.5.2](https://github.com/iamngoni/kanso/compare/v0.5.0...v0.5.2) - 2026-10-20

### New

- **cloud:** Published notes get a Rerout short link that previews the note's title ([46eb294](https://github.com/iamngoni/kanso/commit/46eb29486dc32741899efb2be47da632ef932923))
- **sync:** Publishing sends the note's title and shows the short link with the key ([d8dfc5f](https://github.com/iamngoni/kanso/commit/d8dfc5fa8b25ed5b3ab74d974eb99eb96d1a73b5))

### Fixed

- **import:** Importing a library on Windows builds a SQLite file URI it accepts ([515dcf8](https://github.com/iamngoni/kanso/commit/515dcf83f70ed27740ab84c071c2386f6bbf2ef7))
- **updates:** Restart quits and opens the new Kanso, instead of waiting for the next quit ([2cd9879](https://github.com/iamngoni/kanso/commit/2cd98798e2ca6d38f611f4d971180310e9e966a6))

### Under the hood

- Run every crate's tests but the app's on Windows and Linux too ([e4b2dfd](https://github.com/iamngoni/kanso/commit/e4b2dfd19183f4ed684bacc7d8afd386d0a83533))
- **e2e:** The wrong-password check holds across a rate-limit window boundary ([dedb0e3](https://github.com/iamngoni/kanso/commit/dedb0e355c433b4cf2bfb27d450e01037ec43db8))
- **changelog:** The 0.5.1 notes describe the Windows import fix without naming another app ([62594db](https://github.com/iamngoni/kanso/commit/62594dbefa32a50ff2bc30a28da8d4d6b77d6c20))
- Bind the e2e Worker's port in the same step that picks it ([2bd8f69](https://github.com/iamngoni/kanso/commit/2bd8f699e1f21ee7cf2b0fd14621ce82c5ffada2))

## [0.5.0](https://github.com/iamngoni/kanso/compare/v0.2.0...v0.5.0) - 2026-10-20

### New

- **release:** A per-user Windows installer beside the zip ([f66153a](https://github.com/iamngoni/kanso/commit/f66153a377dbab5b5aa139f56146421134c71a3a))
- **render:** Draw math, Mermaid diagrams and Excalidraw sketches in the theme's colours ([8594dfd](https://github.com/iamngoni/kanso/commit/8594dfd222cf8a03e8514d746db3e1ae7fbdaba2))
- **preview:** Show math, diagrams and Excalidraw drawings in the preview, and export notes as web pages or PDFs ([653584c](https://github.com/iamngoni/kanso/commit/653584c35a0a9b1dfda5e3eb734015af22a966b1))
- **mcp:** Add kanso-export-html to export a note as a standalone web page ([f39daf2](https://github.com/iamngoni/kanso/commit/f39daf284abae746c92179e2b19bed2f71635c67))
- **math:** Show $$…$$ on a line of its own as display math, centred, in the preview and exports ([4694608](https://github.com/iamngoni/kanso/commit/46946083be7502def1adbeb8a3e3f2b0399b57a2))
- **store:** Index each note's links, headings and tasks on this device ([14a021a](https://github.com/iamngoni/kanso/commit/14a021a1f20907eb93a9ae1c020421e2a815ba82))
- **mcp:** Add kanso-get-outline and kanso-resolve-wikilink tools ([489acc7](https://github.com/iamngoni/kanso/commit/489acc74adc4a8a20b72f6a8d7b54b8d3324a96b))
- **app:** An outline and backlinks beside the note, and wiki-links between notes ([aa87983](https://github.com/iamngoni/kanso/commit/aa87983703be72d9850c5cc38367265044ff93c6))
- **list:** Wiki-links read as their alias or title in note excerpts ([dd31e6f](https://github.com/iamngoni/kanso/commit/dd31e6f9f59a7afad74e8422db27024744d3e399))
- **list:** Math reads as its TeX in note excerpts ([7ffe1f5](https://github.com/iamngoni/kanso/commit/7ffe1f53a56f628077698d51e6bbf330ef895ae4))
- **cloud:** Publish a note as an encrypted page anyone with the link can read ([5f504da](https://github.com/iamngoni/kanso/commit/5f504da43a5069de48771180e5c938ca8b8bf2bc))
- **sync:** Seal and publish a note with its own key that never reaches the server ([99ef980](https://github.com/iamngoni/kanso/commit/99ef9805ee7937628839a484b42f39c0846eac84))
- **publish:** Publish a note to the web from its ⋯ menu, update it, or take it down ([28db263](https://github.com/iamngoni/kanso/commit/28db263732c6d77cbe482955bac2091ba21efec9))
- **store:** Daily notes, a Journal, Inbox and Templates, and tags nested with / ([8be2538](https://github.com/iamngoni/kanso/commit/8be2538476477481a700b18e4da74dadd5051a6b))
- **mcp:** Kanso-create-daily-note and kanso-create-from-template ([1e36f92](https://github.com/iamngoni/kanso/commit/1e36f9251fcc1170d954405e20236cde0df2fbee))
- **app:** Open at login on Windows, and as a macOS login item on macOS 13 and later ([4d93b00](https://github.com/iamngoni/kanso/commit/4d93b005aa815affde09893dcd0cfd6b46315f9e))
- **app:** Today's note, notes from templates, quick capture from any app and a tag tree ([028eb3f](https://github.com/iamngoni/kanso/commit/028eb3f7d33c6cba91ba84f3982071fde17b69e8))
- **cli:** Kanso capture --book saves to a notebook by its name or ID ([4b1ccc9](https://github.com/iamngoni/kanso/commit/4b1ccc962df0c81ea359b06fe525824b75078038))
- **editor:** Vim keys, with Normal, Insert and Visual modes in a note's text ([d19fc09](https://github.com/iamngoni/kanso/commit/d19fc096e6c2954969712807842a8a8aa0c8d6b5))
- **editor:** Vim keys gain H M L, ge, > and <, and . repeats Visual mode changes ([27e5964](https://github.com/iamngoni/kanso/commit/27e59645c93aa558f6c4aec47984b466ae8cf36c))
- **tasks:** Due dates show as chips in the preview and are tinted in the editor, with @ date completion ([6c91085](https://github.com/iamngoni/kanso/commit/6c9108560bca2fa856e22c0d873ebf74e0a95591))
- **tasks:** A Tasks view lists every task by when it's due, ticks them off in place and opens them at their line ([e436364](https://github.com/iamngoni/kanso/commit/e43636409cd846f1e2d2fc66cc0b9635c3b46e88))
- **reminders:** Tasks with a time send a notification when due, with Mark Done and Dismiss ([0e17f71](https://github.com/iamngoni/kanso/commit/0e17f71a1a1f2065e35062e7e553fe8d3a99cf7d))
- **replace:** Replace in all notes, with a live preview, options, and Undo replace ([ff34ddb](https://github.com/iamngoni/kanso/commit/ff34ddb4362f63af7587305af86543574f423b88))
- **history:** Note history shows what changed since each version, with each section to copy ([6e9bfc1](https://github.com/iamngoni/kanso/commit/6e9bfc161b9cbb4799be6e1ce011cd00f7cd7e0e))
- **images:** Drag an image's edge to resize it, size it with ![alt|300], or use Resize image… from the palette ([d8a3b0b](https://github.com/iamngoni/kanso/commit/d8a3b0b0ae7b04eaef89b4419611b813c618f3fa))
- **related:** Keep each note's embedding and the words read in its pictures on this device ([80b92c2](https://github.com/iamngoni/kanso/commit/80b92c2469a0a48252a410998e3aa8944d8b0fdd))
- **mcp:** Kanso-related-notes finds the notes closest in meaning to a note ([6a95ada](https://github.com/iamngoni/kanso/commit/6a95adaa7e83dcd6c80c46b89ad198ee5dcd9924))
- **graph:** See how notes link, around the open note in the inspector and across the whole library ([ef2cac7](https://github.com/iamngoni/kanso/commit/ef2cac740ee39b55f3213a572d76259a763387a4))
- **ocr:** Search finds notes by the words in their pictures, read with Vision on macOS ([1aaf62e](https://github.com/iamngoni/kanso/commit/1aaf62e3bb7abb4af28e06718c280a37aebd093e))

### Improved

- **app:** Typing never waits on the outline ([d95c0a0](https://github.com/iamngoni/kanso/commit/d95c0a06e73e743ade9627326f6de6458a880111))

### Fixed

- **app:** Plainer Local API and import wording ([76d2e0c](https://github.com/iamngoni/kanso/commit/76d2e0c6eba450fefca6ee37ddd30de5a8b63dc4))
- **release:** Build the Windows installer for dev versions too ([7b0fe29](https://github.com/iamngoni/kanso/commit/7b0fe29cb4e5e2f193f52fe0447fa6188ca59131))
- **render:** Draw Mermaid arrowheads smaller so they sit lightly on thin lines and clear the nodes ([58924a5](https://github.com/iamngoni/kanso/commit/58924a5e7f13b7b9d47dad967e008d80076c33ca))
- **preview:** Keep a drawing's actions in a quiet row under it that fades in on hover, never over the drawing ([f78cf44](https://github.com/iamngoni/kanso/commit/f78cf449c6f4aa8bee9dd2bc7dcbff74fb51cb9d))
- **export:** Export as PDF prints real pages on your default paper size, with margins ([c1de524](https://github.com/iamngoni/kanso/commit/c1de524adc81c3cc89246314e64e7ae44103cac2))
- **preview:** Text the Markdown parser can't handle shows as plain text instead of crashing ([77de02e](https://github.com/iamngoni/kanso/commit/77de02e393a94a29165553879a9546547ee43192))
- **cloud:** Retire unpublished links for good, serve only the checked upload, and let the owner take a note down ([ef1ef50](https://github.com/iamngoni/kanso/commit/ef1ef50d89bb1000d4154221816b3bc70a1be070))
- **sync:** Publish under a new link when the old one has expired or was retired, never reviving it ([fc31799](https://github.com/iamngoni/kanso/commit/fc317992986b9bf40027a718c7e6647ea6eb84dd))
- **api:** Keep a note's published link when the local API or MCP writes the note ([03563d8](https://github.com/iamngoni/kanso/commit/03563d8889ea63f3c84ed4c5663f0026424b712e))
- **publish:** Say your devices, not your Macs, in the Publish sheet ([225d09e](https://github.com/iamngoni/kanso/commit/225d09eba7c1439705e18c45ac18e27988cca1ee))
- **sync:** A Journal, Inbox or Templates notebook made on two devices offline is one notebook, not a conflict ([52d0a21](https://github.com/iamngoni/kanso/commit/52d0a21e35901a367d5e16540d0f8275275e86e4))
- **capture:** Quick notes are written in the editor's font and size from Settings ([ade79a3](https://github.com/iamngoni/kanso/commit/ade79a3dbdbce67252296674580bbd27f945710d))
- **app:** A new note made under a nested tag gets that tag ([b7803c0](https://github.com/iamngoni/kanso/commit/b7803c0af3f429c8800d48aaff1472bf175f8715))
- **reminders:** Reminders only reach the notification centre from an app bundle, and say when notifications are off ([185cb86](https://github.com/iamngoni/kanso/commit/185cb86e0bb9e086e30159d14910e859104476e5))
- **reminders:** Build cleanly on Windows and Linux, where the bundle check isn't needed ([f2f21f4](https://github.com/iamngoni/kanso/commit/f2f21f450fac76b56d88d195512e0d6320797209))
- **export:** Wiki-links in exported and published pages show as their label, and ![alt|300] images keep their width ([85f185e](https://github.com/iamngoni/kanso/commit/85f185e03dbf219002d00e2a6c325d7d706e1649))
- Say device instead of Mac on Windows and Linux, name each platform's secret store, and show Ctrl shortcuts off macOS ([13f3310](https://github.com/iamngoni/kanso/commit/13f33100387c6ccb6afd79839f87a7c459c08ee9))
- **related:** Say the same thing about the model download in the inspector and Settings ([29e496a](https://github.com/iamngoni/kanso/commit/29e496a64841f1345adfe057fd114ba7e833324f))

### Under the hood

- **release:** Publish each release automatically once every check passes ([9a01c36](https://github.com/iamngoni/kanso/commit/9a01c362e65336c3943af8baf09a32e74f05cce8))
- Describe Kanso on its own terms in the README, changelog and themes guide ([f54d468](https://github.com/iamngoni/kanso/commit/f54d46805699469586f30cef1d7f158b1eac2016))
- **readme:** Releases publish themselves once every check passes ([9a7f813](https://github.com/iamngoni/kanso/commit/9a7f813ec63c3330155e160a96302953940b1050))
- **readme:** The Mac download is notarized and updates itself ([45d8b8e](https://github.com/iamngoni/kanso/commit/45d8b8efa1a289e503d6f93377ae88b1f6125c5d))
- **app:** Measure typing latency with the outline open ([cb1bace](https://github.com/iamngoni/kanso/commit/cb1bacefb99d8b0a5b4c06c50ef4543f3916be36))
- **core:** Excerpt properties allow dots and brackets a note itself has ([618072b](https://github.com/iamngoni/kanso/commit/618072b9d757bd3be74962a8897cf61736ba4ed8))
- **app:** Snapshots date the notes they make by the demo clock ([437fb72](https://github.com/iamngoni/kanso/commit/437fb72cd09f6dbae676dc564d6e01167b1ededa))
- **e2e:** Give each end-to-end run its own Worker port so parallel runs never share one ([67c70c1](https://github.com/iamngoni/kanso/commit/67c70c1b2545f7cb6fb4515b73dc59b97336f774))
- **editor:** Snapshots of each Vim mode, and the typing benchmark with Vim keys on ([5771afc](https://github.com/iamngoni/kanso/commit/5771afc0d78cb634b6f48b7040c75291cafcadfc))
- **editor:** Vim snapshots show the caret as it is half a blink later ([30b016b](https://github.com/iamngoni/kanso/commit/30b016b19f14861025324408c6030471575d1992))
- **app:** Snapshots of the Tasks view, due chips, the replace sheet and the history changes ([436eeb2](https://github.com/iamngoni/kanso/commit/436eeb2499f2957b33f642c82de0ac40331c82f9))
- **editor:** Due dates stay tinted and @ completes dates with Vim keys on ([49b5243](https://github.com/iamngoni/kanso/commit/49b524353a1b046609663df6b77fcc29d07dbafd))
- Describe Kanso as it is now, with its newer shortcuts, kanso capture and device-neutral sync wording ([bff32fe](https://github.com/iamngoni/kanso/commit/bff32fe9adff9157a68681baa6127f0dd5fd8525))
- **changelog:** Release notes describe Kanso on its own terms ([c7be1b2](https://github.com/iamngoni/kanso/commit/c7be1b27da5fb98315d6756824dd95082611e174))

## [0.2.0](https://github.com/iamngoni/kanso/compare/v0.1.1...v0.2.0) - 2026-10-19

### New

- Close brackets, quotes and Markdown markers as you type ([4f383a9](https://github.com/iamngoni/kanso/commit/4f383a991bd116220cfe1848cb9345da67a87631))
- Tab and Shift-Tab indent list items, with what hangs under them ([6ea7aac](https://github.com/iamngoni/kanso/commit/6ea7aacf8eaa3624829e83373c36e91833525bf7))
- Format Table lines up a Markdown table ([3945fee](https://github.com/iamngoni/kanso/commit/3945fee951b59353366cbc4081e81f4e7cf18c6b))
- Smart paste: links over selected words, formatted text as Markdown, images as attachments ([0b023f6](https://github.com/iamngoni/kanso/commit/0b023f689f86e5dc15dc93744f054be2603723cc))
- Size attached images in the preview ([a1a7dfd](https://github.com/iamngoni/kanso/commit/a1a7dfdab76e0980cb2717da7b68edbd463ade5d))
- Word goal with a progress ring in focus mode ([8ee7204](https://github.com/iamngoni/kanso/commit/8ee7204287b572a1eafcd3b1b34db203217886e5))
- Typewriter scrolling, and the writing aids in Settings › Editor ([18e0836](https://github.com/iamngoni/kanso/commit/18e08363e197903d385588f2e0acc4b902173eef))

### Fixed

- Kanso comes to the front after Restart now installs an update, instead of opening behind other windows ([7982fed](https://github.com/iamngoni/kanso/commit/7982fed))

### Under the hood

- Snapshots and latency runs for the writing aids ([d3c7467](https://github.com/iamngoni/kanso/commit/d3c7467ec807eb709fd7a867e51f984a00f79a27))

## [0.1.1](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.1.1) - 2026-10-19

### Fixed

- **release:** Brace variables before an ellipsis in release.sh ([6f819da](https://github.com/iamngoni/kanso/commit/6f819da6a4ae2585478dad4c27d0eceb4c308ab8))

## [0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0) - 2026-10-19

The first release: a calm, local-first Markdown notes app built on GPUI
for macOS, Windows and Linux. What follows sums up
the history so far; every commit is in [the history up to
v0.1.0](https://github.com/iamngoni/kanso/commits/v0.1.0).

### New

- Kanso updates itself from signed releases, says clearly when an update is ready, and shows what's new after it installs ([6bb7570](https://github.com/iamngoni/kanso/commit/6bb7570))
- Notes, notebooks within notebooks, tags and statuses in a SQLite store with trigram search, revisions and daily backups ([169d0e4](https://github.com/iamngoni/kanso/commit/169d0e4))
- Import a PouchDB notes library, inside the app or from the command line, keeping Kanso's own edits on a re-import ([e34c7bc](https://github.com/iamngoni/kanso/commit/e34c7bc), [e5479c4](https://github.com/iamngoni/kanso/commit/e5479c4))
- The editor with a live preview, side by side or alone, a draggable split and focus mode ([a531b38](https://github.com/iamngoni/kanso/commit/a531b38), [d0e68c2](https://github.com/iamngoni/kanso/commit/d0e68c2))
- The note list grouped by day, with search matches shown in the excerpts ([d7744b9](https://github.com/iamngoni/kanso/commit/d7744b9))
- The command palette and Open Note ([d268159](https://github.com/iamngoni/kanso/commit/d268159))
- Backlinks and each note's history, with a way back to any earlier version ([4b45548](https://github.com/iamngoni/kanso/commit/4b45548))
- Export a note, or every note, as Markdown ([0e35966](https://github.com/iamngoni/kanso/commit/0e35966))
- A local API and a native MCP endpoint for agents and other clients, compatible with popular local-server clients ([88ba474](https://github.com/iamngoni/kanso/commit/88ba474), [d5fd4d5](https://github.com/iamngoni/kanso/commit/d5fd4d5))
- Accounts with end-to-end encrypted sync between your computers, recovery keys, and a Settings page for the account and its devices ([f2d5a5a](https://github.com/iamngoni/kanso/commit/f2d5a5a), [666c77f](https://github.com/iamngoni/kanso/commit/666c77f), [b5fbac9](https://github.com/iamngoni/kanso/commit/b5fbac9))
- Settings inside the main window, and opening at login ([157cd61](https://github.com/iamngoni/kanso/commit/157cd61))
- Twenty themes, separate UI, syntax and preview themes, and importing theme packages and Zed themes ([1f9d3ff](https://github.com/iamngoni/kanso/commit/1f9d3ff), [8f050bc](https://github.com/iamngoni/kanso/commit/8f050bc))
- Code blocks colored in 23 languages, with a copy button ([3709232](https://github.com/iamngoni/kanso/commit/3709232), [27f54a0](https://github.com/iamngoni/kanso/commit/27f54a0))
- An app icon: an ink ensō with a vermilion seal ([c403439](https://github.com/iamngoni/kanso/commit/c403439))
- Builds for Windows and Linux as well as macOS ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))

### Improved

- A warm graphite and paper look and a calmer writing surface ([2e1b76b](https://github.com/iamngoni/kanso/commit/2e1b76b))
- Typing stays fast in long notes ([e0d7ad0](https://github.com/iamngoni/kanso/commit/e0d7ad0))

### Fixed

- Typed text is never lost, through saves, syncs and a note's revision moving underneath it ([f6a6583](https://github.com/iamngoni/kanso/commit/f6a6583), [9442143](https://github.com/iamngoni/kanso/commit/9442143))
- Sync keeps local edits through its own echoes, lost answers and restores ([7b0d30d](https://github.com/iamngoni/kanso/commit/7b0d30d), [652b439](https://github.com/iamngoni/kanso/commit/652b439), [3ab3f29](https://github.com/iamngoni/kanso/commit/3ab3f29))
- Recovery keys are kept until they're saved, and replaced with every password change ([645c5cd](https://github.com/iamngoni/kanso/commit/645c5cd), [ff8f89f](https://github.com/iamngoni/kanso/commit/ff8f89f))
- Auth keys, proofs and tokens stay out of logs and freed memory ([3bda3bd](https://github.com/iamngoni/kanso/commit/3bda3bd))
- Three rounds of review findings in the main window ([61f005e](https://github.com/iamngoni/kanso/commit/61f005e), [f325bfc](https://github.com/iamngoni/kanso/commit/f325bfc), [18b218d](https://github.com/iamngoni/kanso/commit/18b218d))

### Under the hood

- Property tests with a pinned seed across the workspace, and PNG snapshots of the real views ([ff9515f](https://github.com/iamngoni/kanso/commit/ff9515f))
- A release workflow that builds every platform into a draft release ([62ec295](https://github.com/iamngoni/kanso/commit/62ec295))
