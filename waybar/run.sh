#!/usr/bin/env bash
# Launcher for waybar modules.
# Works whether this directory is symlinked into ~/.config/waybar/ or copied there.
#   - Symlinked: resolves to the real repo, uses the built binary and state.json there.
#   - Copied:    falls back to waybar_cookie_clicker in PATH and XDG state dir.
REAL_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
BINARY="$REAL_DIR/../target/release/waybar_cookie_clicker"
STATE="$REAL_DIR/../state.json"

if [ ! -x "$BINARY" ]; then
  BINARY="waybar_cookie_clicker"
  STATE="$HOME/.local/share/waybar_cookie_clicker/state.json"
fi

exec "$BINARY" --state "$STATE" "$@"
