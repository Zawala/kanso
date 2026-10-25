#!/bin/sh
# Tests macos-profile.sh's checks on made-up profiles: an expired one, and
# one whose DeveloperCertificates do or don't hold the signing certificate.
# macOS only (plutil, base64 -D); needs no keychain.
set -eu
. "$(dirname "$0")/macos-profile.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

openssl req -x509 -newkey rsa:2048 -nodes -keyout "$work/key.pem" -out "$work/cert.pem" \
    -days 1 -subj "/CN=Kanso profile test" >/dev/null 2>&1
openssl x509 -in "$work/cert.pem" -outform DER -out "$work/cert.der"
sha1=$(shasum -a 1 "$work/cert.der" | cut -d' ' -f1)
other=0000000000000000000000000000000000000000

profile() { # expiry, then whether it lists the certificate
    plutil -create xml1 "$work/p.plist"
    plutil -insert ExpirationDate -date "$1" "$work/p.plist"
    plutil -insert DeveloperCertificates -array "$work/p.plist"
    plutil -insert DeveloperCertificates.0 -data AAAA "$work/p.plist"
    if [ "$2" = yes ]; then
        plutil -insert DeveloperCertificates.1 -data "$(base64 -i "$work/cert.der")" "$work/p.plist"
    fi
}
fail() { echo "test-macos-profile: $1" >&2; exit 1; }

profile 2099-01-01T00:00:00Z yes
profile_unexpired "$work/p.plist" || fail "a profile until 2099 counted as expired"
profile_has_certificate "$work/p.plist" "$sha1" || fail "the listed certificate wasn't found"
profile_has_certificate "$work/p.plist" "$(echo "$sha1" | tr '[:lower:]' '[:upper:]')" ||
    fail "an uppercase SHA-1 wasn't found"
! profile_has_certificate "$work/p.plist" "$other" || fail "another certificate was found"

profile 2001-01-01T00:00:00Z no
! profile_unexpired "$work/p.plist" || fail "an expired profile counted as valid"
! profile_has_certificate "$work/p.plist" "$sha1" || fail "an unlisted certificate was found"

plutil -create xml1 "$work/p.plist"
! profile_unexpired "$work/p.plist" || fail "a profile without ExpirationDate counted as valid"
echo "test-macos-profile: ok"
