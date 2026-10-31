#!/usr/bin/env bash
# Run inside a private D-Bus session and an existing virtual display.
set -euo pipefail
binary=${1:?tested executable}
print_test_tmp=$(mktemp -d)
export TMPDIR="$print_test_tmp"
for mode in success cancel error; do
  /usr/bin/python3 tools/print_portal_test.py "$mode" "print-$mode.pdf" >"print-portal-$mode.log" 2>&1 &
  portal=$!
  trap 'kill "$portal" 2>/dev/null || true' EXIT
  for attempt in $(seq 1 100); do
    if grep -q PRINT_PORTAL_READY "print-portal-$mode.log"; then break; fi
    kill -0 "$portal"; sleep 0.1
  done
  grep -q PRINT_PORTAL_READY "print-portal-$mode.log"
  LEAF_PRINT_TEST="$mode" GSK_RENDERER=cairo timeout 45s "$binary" --test-document --print-ui-test 2>"print-ui-$mode.log" || { cat "print-ui-$mode.log" "print-portal-$mode.log"; exit 1; }
  grep -q PRINT_UI_OK "print-ui-$mode.log"
  grep -q "PRINT_PORTAL_OK $mode" "print-portal-$mode.log"
  pdfinfo "print-$mode.pdf" >"print-$mode-info.txt"
  pdftotext "print-$mode.pdf" "print-$mode.txt"
  grep -q 'The Open Studio Handbook' "print-$mode.txt"
  kill "$portal"; wait "$portal" || true
done
test -z "$(find "$print_test_tmp" -maxdepth 1 -name 'leaf-print-*' -print -quit)"
rmdir "$print_test_tmp" || true
echo PRINT_INTEGRATION_OK
