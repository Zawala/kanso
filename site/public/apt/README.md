# kansoapp.app/apt

`kanso.gpg`, the apt repository's public signing key, goes here.
`scripts/apt-setup.sh` writes it; it is served as a static file at
https://kansoapp.app/apt/kanso.gpg.

Everything else under /apt redirects to the GitHub Release assets
(`src/lib/apt.ts`). docs/APT.md explains the whole repository.
