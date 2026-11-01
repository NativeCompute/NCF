#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:-docs/shots}"
mkdir -p "$out"
cargo build -q -p ncf-gallery
bin=target/debug/ncf-gallery
display="${DISPLAY:-}"
if [ -z "$display" ] || ! xdpyinfo -display "$display" >/dev/null 2>&1; then
  display=:97
  Xvfb "$display" -screen 0 1440x960x24 >/dev/null 2>&1 &
  xvfb=$!
  trap 'kill $xvfb 2>/dev/null || true' EXIT
  sleep 1
fi
export DISPLAY="$display"
home="$(mktemp -d)"
for cast in dark light; do
  for section in type controls surfaces colour icons marks notices motion; do
    XDG_DATA_HOME="$home" HOME="$home" LIBGL_ALWAYS_SOFTWARE=1 "$bin" --"$cast" --no-splash --section "$section" >/dev/null 2>&1 &
    pid=$!
    sleep 2.5
    win="$(xdotool search --pid "$pid" | tail -1)"
    import -window "$win" "$out/$section-$cast.png"
    kill "$pid"
    wait "$pid" 2>/dev/null || true
  done
done
rm -rf "$home"
echo "shots in $out"
