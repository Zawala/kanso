#!/usr/bin/env bash
# Checks the settings the mobile app states twice (#48) are the same twice.
# Usage: scripts/mobile-parity.sh
#
# 1. mobile/hook/build.dart builds the library with cargo's release profile
#    and gives it the settings of `[profile.mobile]` in the root Cargo.toml
#    as CARGO_PROFILE_RELEASE_* variables (the builder always builds
#    `--release`). The two lists must name the same settings with the same
#    values, or the app's library is not the one scripts/mobile-size.sh and
#    `cargo build --profile mobile` measure.
# 2. mobile/rust/rust-toolchain.toml (read by the hook's cargo) pins the same
#    channel as the root rust-toolchain.toml.
#
# CI runs it in the Mobile workflow; run it after changing either side.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"

python3 -I - "$root" <<'PY'
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
failures = []


def toml_section(path, name):
    """The `key = value` lines of one [section], values without quotes."""
    values, inside = {}, False
    for raw in path.read_text().splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        header = re.fullmatch(r"\[([^\]]+)\]", line)
        if header:
            inside = header.group(1).strip() == name
            continue
        pair = re.fullmatch(r'([A-Za-z0-9_-]+)\s*=\s*"?([^"]*)"?', line)
        if inside and pair:
            values[pair.group(1)] = pair.group(2).strip()
    return values


# 1. The cargo profile.
profile = toml_section(root / "Cargo.toml", "profile.mobile")
profile.pop("inherits", None)
hook_source = (root / "mobile/hook/build.dart").read_text()
block = re.search(r"_mobileProfile\s*=\s*<String, String>\{(.*?)\};", hook_source, re.S)
if not block:
    sys.exit("mobile/hook/build.dart has no _mobileProfile map.")
hook = {
    m.group(1): m.group(2)
    for m in re.finditer(r"'CARGO_PROFILE_RELEASE_([A-Z_]+)':\s*'([^']*)'", block.group(1))
}
# opt-level in Cargo.toml is OPT_LEVEL in the variable's name.
wanted = {key.upper().replace("-", "_"): value for key, value in profile.items()}
for key in sorted(set(wanted) | set(hook)):
    if key not in hook:
        failures.append(f"[profile.mobile] sets {key.lower().replace('_', '-')} = {wanted[key]}, hook/build.dart does not")
    elif key not in wanted:
        failures.append(f"hook/build.dart sets CARGO_PROFILE_RELEASE_{key} = {hook[key]}, [profile.mobile] does not")
    elif wanted[key] != hook[key]:
        failures.append(f"{key.lower().replace('_', '-')}: [profile.mobile] has {wanted[key]}, hook/build.dart has {hook[key]}")
if not profile:
    failures.append("Cargo.toml has no [profile.mobile] settings.")

# 2. The toolchain.
channels = {}
for path in ("rust-toolchain.toml", "mobile/rust/rust-toolchain.toml"):
    channels[path] = toml_section(root / path, "toolchain").get("channel")
if None in channels.values() or len(set(channels.values())) != 1:
    failures.append(f"toolchain channels differ: {channels}")

if failures:
    for failure in failures:
        print(f"::error::{failure}")
    sys.exit(1)
print(f"mobile profile: {len(profile)} settings match; toolchain {channels['rust-toolchain.toml']} in both.")
PY
