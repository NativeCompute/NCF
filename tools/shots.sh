#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")/.."
out="${1:-docs/shots}"
mkdir -p "$out"
cargo build -q -p ncf-gallery || exit 1
bin=target/debug/ncf-gallery
display="${DISPLAY:-}"
if [ -z "$display" ] || ! xdpyinfo -display "$display" >/dev/null 2>&1; then
  display=:97
  Xvfb "$display" -screen 0 1440x960x24 >/dev/null 2>&1 &
  xvfb=$!
  trap 'kill $xvfb 2>/dev/null || true' EXIT
  for _ in $(seq 1 50); do xdpyinfo -display "$display" >/dev/null 2>&1 && break; sleep 0.2; done
fi
export DISPLAY="$display"
home="$(mktemp -d)"
failed=0
for cast in dark light; do
  for section in type controls surfaces colour icons marks notices motion; do
    log="$home/$section-$cast.log"
    XDG_DATA_HOME="$home" HOME="$home" LIBGL_ALWAYS_SOFTWARE=1 "$bin" --"$cast" --no-splash --section "$section" >"$log" 2>&1 &
    pid=$!
    win=""
    for _ in $(seq 1 100); do
      win="$(xdotool search --onlyvisible --pid "$pid" 2>/dev/null | tail -1)"
      [ -n "$win" ] && break
      kill -0 "$pid" 2>/dev/null || break
      sleep 0.2
    done
    if [ -z "$win" ]; then
      echo "no window for $section ($cast)"
      cat "$log"
      failed=1
      kill "$pid" 2>/dev/null
      continue
    fi
    sleep 2
    import -window "$win" "$out/$section-$cast.png" || failed=1
    kill "$pid"
    wait "$pid" 2>/dev/null
  done
done
rm -rf "$home"
[ "$failed" = 0 ] || exit 1
echo "shots in $out"
