#!/usr/bin/env bash
set -euo pipefail
exec flatpak run --env="LEAF_PRINT_TEST=${LEAF_PRINT_TEST:-success}" --socket=x11 --nosocket=wayland io.github.frandavid100.leaf "$@"
