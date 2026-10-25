#!/usr/bin/env bash
set -euo pipefail
tmpfile="$(mktemp)"
trap 'rm -f "$tmpfile"' EXIT
cargo tree "$@" --edges normal --prefix none --format '{p}' > "$tmpfile"
if grep -E '^(egui|eframe|egui-[a-z0-9_-]+|epaint) v' "$tmpfile"; then
    echo 'ERROR: dependencias gráficas egui detectadas' >&2
    exit 1
fi
echo 'OK: árbol de dependencias sin egui/eframe/epaint'
