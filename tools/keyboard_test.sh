#!/usr/bin/env bash
set -euo pipefail
binary=${1:?tested executable}
rm -f keyboard-stage-*
GSK_RENDERER=cairo "$binary" --keyboard-ui-test 2>keyboard-ui.log &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; rm -f keyboard-stage-*' EXIT
wait_ready() {
  for attempt in $(seq 1 150); do
    if grep -q "KEYBOARD_READY $1$" keyboard-ui.log; then return; fi
    kill -0 "$pid" || { cat keyboard-ui.log; exit 1; }
    sleep 0.1
  done
  cat keyboard-ui.log; exit 1
}
wait_ready 0
window=$(xdotool search --onlyvisible --pid "$pid" | head -n1)
xdotool windowfocus --sync "$window"
keys=(F6 F6 F6 F10 Escape Escape shift+F6 shift+F6 shift+F6)
for stage in "${!keys[@]}"; do
  wait_ready "$stage"
  xdotool key --clearmodifiers "${keys[$stage]}"
  sleep 0.3
  touch "keyboard-stage-$stage"
done
wait_ready 9
touch keyboard-stage-9
wait "$pid"
grep -q KEYBOARD_UI_OK keyboard-ui.log
