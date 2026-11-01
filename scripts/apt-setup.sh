#!/usr/bin/env bash
# One-time setup of the apt repository at kansoapp.app/apt (docs/APT.md),
# run by the maintainer from the repository's root:
#
#   scripts/apt-setup.sh            # the first time
#   scripts/apt-setup.sh --rotate   # to replace the key (every user must
#                                   # download kanso.gpg again)
#
# It makes the apt signing key (ed25519) in a temporary GnuPG home, stores
# it and a random passphrase as secrets of the repository's `release`
# environment (APT_SIGNING_KEY, APT_SIGNING_PASSPHRASE; only v* tags reach
# them), and writes two public files to commit:
#
#   site/public/apt/kanso.gpg    the public key users download
#   packaging/linux/apt.conf     the suites and the key's fingerprint
#
# The private key and passphrase never touch the repository or the disk
# outside the temporary home, except the backup it offers to write to a
# folder you choose outside the repository. Nothing is echoed.
set -euo pipefail

REPO="iamngoni/kanso"
ENVIRONMENT="release"
KEY_FILE="site/public/apt/kanso.gpg"
CONF_FILE="packaging/linux/apt.conf"
BACKUP_NAME="kanso-apt-signing-key-BACKUP"

rotate=false
case "${1:-}" in
"") ;;
--rotate) rotate=true ;;
-h | --help)
	sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
	exit 0
	;;
*)
	echo "Unknown option: $1 (try --help)" >&2
	exit 2
	;;
esac

die() {
	echo "apt-setup: $*" >&2
	exit 1
}

# ---- Preflight -------------------------------------------------------------

for tool in gpg gh openssl git; do
	command -v "$tool" >/dev/null || die "$tool is missing; install it first."
done
[ -f Cargo.toml ] && [ -d site ] && [ -f scripts/apt-setup.sh ] ||
	die "run this from the repository's root (scripts/apt-setup.sh)."
repo_root=$(pwd -P)
[ -z "$(git status --porcelain)" ] ||
	die "the working tree isn't clean; commit or stash first, so the two new files are all that changes."
gh auth status >/dev/null 2>&1 || die "gh isn't signed in; run: gh auth login"
admin=$(gh api "repos/$REPO" --jq .permissions.admin 2>/dev/null || true)
[ "$admin" = "true" ] || die "you need admin rights on $REPO to set its secrets."

maintainer=$(sed -n 's/^maintainer = "\(.*\)"$/\1/p' crates/kanso-app/Cargo.toml | head -1)
email=$(printf '%s' "$maintainer" | sed -n 's/.*<\(.*\)>.*/\1/p')
[ -n "$maintainer" ] && [ -n "$email" ] ||
	die "no 'maintainer = \"Name <email>\"' in [package.metadata.deb] of crates/kanso-app/Cargo.toml."

existing=$(gh secret list --env "$ENVIRONMENT" -R "$REPO" --json name --jq '.[].name' 2>/dev/null || true)
if printf '%s\n' "$existing" | grep -qx APT_SIGNING_KEY && [ "$rotate" != true ]; then
	die "APT_SIGNING_KEY already exists in the $ENVIRONMENT environment. Replacing it breaks every user's apt update until they download kanso.gpg again; run with --rotate if that's what you want."
fi

# ---- Questions -------------------------------------------------------------

ask() { # ask <prompt> <default>
	local answer
	read -r -p "$1 [$2]: " answer
	printf '%s' "${answer:-$2}"
}

echo "The packages' maintainer, and the key's name:"
echo "  $maintainer"
echo "  Kanso APT Signing Key <$email>"
[ "$(ask "Is that right? (yes/no)" yes)" = yes ] ||
	die "change maintainer in [package.metadata.deb] of crates/kanso-app/Cargo.toml, commit, and run again."

expiry=$(ask "Key expiry: 2y, 5y or never" 2y)
case $expiry in
2y | 5y | never) ;;
*) die "expiry must be 2y, 5y or never." ;;
esac
if [ "$expiry" != never ]; then
	echo "Before it expires, extend it (gpg --quick-set-expire) and store the key again, or users' apt update fails."
fi

echo "Channels: only stable is implemented. unstable needs prerelease tags and isn't published yet."
suites=$(ask "Suites" stable)
[ "$suites" = stable ] || die "only stable is implemented (docs/APT.md)."

echo "Back up the key: without it, a lost secret means a new key, and every"
echo "user's apt update breaks until they download kanso.gpg again."
backup_dir=$(ask "Folder for the backup, outside the repository (empty to skip)" "")
backup_real=""
if [ -n "$backup_dir" ]; then
	backup_dir="${backup_dir/#\~/$HOME}"
	mkdir -p "$backup_dir"
	backup_real=$(cd "$backup_dir" && pwd -P)
	case "$backup_real/" in
	"$repo_root"/*) die "$backup_real is inside the repository; choose a folder outside it." ;;
	esac
fi

if [ "$rotate" = true ]; then
	echo
	echo "ROTATING: once the next release is signed with the new key, everyone"
	echo "who installed with apt must download $KEY_FILE's new key again:"
	echo "  curl -fsSL https://kansoapp.app/apt/kanso.gpg | sudo tee /usr/share/keyrings/kanso.gpg >/dev/null"
	[ "$(ask "Rotate the key? (yes/no)" no)" = yes ] || die "nothing changed."
fi

# ---- The key, in a temporary home ------------------------------------------

GNUPGHOME=$(mktemp -d)
export GNUPGHOME
chmod 700 "$GNUPGHOME"
cleanup() {
	gpgconf --kill gpg-agent 2>/dev/null || true
	rm -rf "$GNUPGHOME"
}
trap cleanup EXIT

passphrase="$GNUPGHOME/passphrase"
(
	umask 077
	openssl rand -base64 32 | tr -d '\n' >"$passphrase"
)
loopback=(--batch --pinentry-mode loopback --passphrase-file "$passphrase")

echo "Making the key…"
gpg "${loopback[@]}" --quiet --quick-gen-key \
	"Kanso APT Signing Key <$email>" ed25519 sign "$expiry"
fingerprint=$(gpg --batch --with-colons --list-secret-keys | awk -F: '/^fpr:/ { print $10; exit }')
[[ $fingerprint =~ ^[0-9A-F]{40}$ ]] || die "couldn't read the new key's fingerprint."

# ---- Secrets, straight from gpg to GitHub ----------------------------------

echo "Storing APT_SIGNING_KEY and APT_SIGNING_PASSPHRASE in $REPO's $ENVIRONMENT environment…"
gpg "${loopback[@]}" --armor --export-secret-keys "$fingerprint" |
	gh secret set APT_SIGNING_KEY --env "$ENVIRONMENT" -R "$REPO"
gh secret set APT_SIGNING_PASSPHRASE --env "$ENVIRONMENT" -R "$REPO" <"$passphrase"

# ---- The public files ------------------------------------------------------

mkdir -p "$(dirname "$KEY_FILE")" "$(dirname "$CONF_FILE")"
gpg --batch --export "$fingerprint" >"$KEY_FILE"
cat >"$CONF_FILE" <<EOF
# The apt repository (docs/APT.md), written by scripts/apt-setup.sh.
# The suites the release workflow publishes, and the fingerprint of the key
# that signs them (site/public/apt/kanso.gpg); APT_SIGNING_KEY must be it.
suites=$suites
fingerprint=$fingerprint
EOF

# ---- Backup ----------------------------------------------------------------

if [ -n "$backup_real" ]; then
	(
		umask 077
		gpg "${loopback[@]}" --armor --export-secret-keys "$fingerprint" >"$backup_real/$BACKUP_NAME.asc"
		cp "$passphrase" "$backup_real/$BACKUP_NAME.passphrase"
	)
	echo "Wrote $backup_real/$BACKUP_NAME.asc and $BACKUP_NAME.passphrase."
	echo "Keep them offline, apart from each other if you can."
else
	echo "!!! NO BACKUP WRITTEN. The key exists only in GitHub's secrets, which can't be read back. !!!"
fi

# ---- Done ------------------------------------------------------------------

cat <<EOF

The apt signing key is ready.
  Fingerprint: $fingerprint

Next:
  1. Commit the public files:
       git add $KEY_FILE $CONF_FILE && git commit -m "apt: add signing key"
  2. Deploy the site once, so https://kansoapp.app/apt/kanso.gpg exists:
       cd site && npm run deploy      (or however you usually deploy it)
  3. Tag the next release: its workflow publishes the repository.
EOF
