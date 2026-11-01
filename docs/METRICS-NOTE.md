# Kanso counts, it doesn't watch

Kanso sends a small anonymous count, so we know how many people use it and on which devices.
Here is everything it sends, and everything it doesn't.

## What's sent

When you open Kanso, every hour while it's open, and when you close it (or send it to the
background on a phone):

- The kind of device: Mac, Windows, Linux, iPhone, iPad or Android, its system version, and
  Kanso's version and release channel (stable, beta or dev).
- Whether sync is on, whether it's Kanso's server or your own, and how many sync errors there were,
  as a range.
- Whether AI is turned on.
- Your language, if Kanso ships it.
- Your theme (light, dark or system, and which of Kanso's own themes; a theme you made or
  downloaded is just "custom") and your default editor mode.
- Roughly how many notes, notebooks and plugins you have, and how much room attachments take, each
  as a range.
- That Kanso was opened, and how long it was open: under a minute, 1-5 minutes, 5-15, 15-60,
  1-3 hours, or more.

And which of these you used since the last count, and roughly how often (once, 2-5, 6-20, 21 or
more), never what with: creating, editing, deleting, restoring, pinning or moving a note, creating
a notebook, adding a tag, using a template, the daily note, searching, following a link, opening
backlinks or the graph, ticking a task, attaching a file, inserting a table or a diagram, opening a
sticky note or a new window, focus mode, installing a plugin or running its command, changing the
theme, resolving a sync conflict, voice input, publishing, importing (and which app you imported
from), locking, AI, quick capture, sharing a note with someone, or exporting. Phones count only the
ones they have.

The country your connection comes from is added on our server. Your IP address is not kept or
passed on.

## What isn't

- Your notes, their titles, notebooks, tags, files or links
- Anything you type, search for or ask AI
- The name of a theme or a plugin
- Your name, username or email address
- Your IP address
- Anything about your other apps or your computer beyond its system version

## How you're counted

If you're signed in to an account, your devices share one ID made from your account's encryption
key, so your laptop and phone count as one person. Our server never has that key, so neither we
nor anyone else can tell which account the ID belongs to. If you're not signed in, each device
has a random ID of its own.

## Where it goes

To Kanso's own server first, which checks every field against a fixed list and refuses anything
else, then on to [PostHog](https://posthog.com) (EU) to draw the charts. There's no tracking code
from anyone else in Kanso.

## Can I turn it off?

No. It's how we know Kanso is being used at all, and it's kept to the list above so it never
needs to be. Debug and development builds don't send anything.

## The code

Every detail is in [docs/METRICS.md](https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md):
the exact fields and values, and links to the code that sends them and the server that checks them.
