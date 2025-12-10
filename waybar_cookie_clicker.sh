#!/usr/bin/env bash
# Wrapper that runs the binary with the save file kept inside the repo directory.
# Waybar modules should reference this script instead of the binary directly.
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$DIR/target/release/waybar_cookie_clicker" --state "$DIR/state.json" "$@"
