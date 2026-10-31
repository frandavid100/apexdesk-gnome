#!/usr/bin/env bash
# Real desktop portal/backend dialog; dismiss it without printing a sheet.
set -euo pipefail
LEAF_PRINT_TEST=cancel GSK_RENDERER=cairo "$@" --test-document --print-ui-test 2>print-dialog.log &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
dialog=""
for attempt in $(seq 1 200); do
  dialog=$(xdotool search --onlyvisible --name '^Imprimir — ' 2>/dev/null | head -n1 || true)
  if test -n "$dialog"; then break; fi
  kill -0 "$pid" || { cat print-dialog.log; exit 1; }
  sleep 0.1
done
test -n "$dialog" || { cat print-dialog.log; exit 1; }
scrot print-dialog.png
xdotool windowfocus --sync "$dialog"
xdotool key --clearmodifiers Escape
wait "$pid" || { cat print-dialog.log; exit 1; }
grep -q 'PRINT_UI_OK Dismissed' print-dialog.log
echo PRINT_NATIVE_DIALOG_OK
