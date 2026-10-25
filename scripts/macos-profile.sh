# Checks on a decoded Developer ID provisioning profile, sourced by
# bundle-macos.sh and tested by test-macos-profile.sh. POSIX sh.

# Whether the decoded profile's ExpirationDate is still ahead.
profile_unexpired() {
    expires=$(plutil -extract ExpirationDate raw "$1" 2>/dev/null) || return 1
    expires_at=$(date -j -u -f '%Y-%m-%dT%H:%M:%SZ' "$expires" +%s 2>/dev/null) || return 1
    [ "$expires_at" -gt "$(date -u +%s)" ]
}

# The SHA-1 of the certificate codesign will sign with, uppercase: the
# identity itself when it is one, else the valid code-signing identity of
# that name (`security find-identity`; `find-certificate -Z` may also list
# expired certificates with the same name).
signing_sha1() {
    case "$1" in
        *[!0-9A-Fa-f]* | '') ;;
        *) [ ${#1} -eq 40 ] && { echo "$1" | tr '[:lower:]' '[:upper:]'; return; } ;;
    esac
    security find-identity -v -p codesigning 2>/dev/null |
        awk -v name="\"$1\"" 'index($0, name) { print $2; exit }' | tr '[:lower:]' '[:upper:]'
}

# Whether the decoded profile's DeveloperCertificates include the signing
# certificate: AMFI kills an app whose restricted entitlements come with a
# profile for another certificate.
profile_has_certificate() {
    want=$(signing_sha1 "$2")
    [ -n "$want" ] || return 1
    i=0
    while cert=$(plutil -extract "DeveloperCertificates.$i" raw "$1" 2>/dev/null); do
        have=$(printf '%s' "$cert" | base64 -D | shasum -a 1 | cut -d' ' -f1 | tr '[:lower:]' '[:upper:]')
        [ "$have" = "$want" ] && return 0
        i=$((i + 1))
    done
    return 1
}
