#!/usr/bin/env bash
set -euo pipefail
app_id=io.github.frandavid100.leaf
export GSK_RENDERER=cairo
export XDG_CURRENT_DESKTOP=GNOME
export XDG_DATA_DIRS="$HOME/.local/share/flatpak/exports/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
dbus-update-activation-environment DISPLAY XDG_CURRENT_DESKTOP XDG_DATA_DIRS
mkdir -p "$HOME/.config/xdg-desktop-portal"
printf '[preferred]\ndefault=gtk\n' > "$HOME/.config/xdg-desktop-portal/portals.conf"
flatpak run --command=sh "$app_id" -c 'grep -q retained "$XDG_DATA_HOME/upgrade-test"'
echo FLATPAK_UPGRADE_SANDBOX_OK
flatpak run "$app_id" --version | grep -F 'Leaf 0.1.0-alpha.3'
flatpak run --socket=x11 --nosocket=wayland "$app_id" --columns-ui-test 2>dist/flatpak-columns.log
grep -q COLUMNS_UI_OK dist/flatpak-columns.log
flatpak run --socket=x11 --nosocket=wayland "$app_id" --chrome-ui-test 2>dist/flatpak-chrome.log
grep -q DOCUMENT_CHROME_OK dist/flatpak-chrome.log
flatpak run --socket=x11 --nosocket=wayland "$app_id" --styles-ui-test 2>dist/flatpak-styles.log
grep -q STYLES_POPOVER_OK dist/flatpak-styles.log
flatpak run --env=GDK_SCALE=2 --socket=x11 --nosocket=wayland "$app_id" --hidpi-ui-test 2>dist/flatpak-hidpi.log
grep -q 'HIDPI_UI_OK scale=2' dist/flatpak-hidpi.log
# A real exported desktop launcher, using the installed icon and application ID.
gtk-launch "$app_id" 2>dist/flatpak-launcher.log &
launcher_pid=$!
for attempt in $(seq 1 100); do
  if flatpak ps --columns=application | grep -qx "$app_id"; then break; fi
  sleep 0.1
done
flatpak ps --columns=application | grep -qx "$app_id"
sleep 3
scrot dist/leaf-installed.png
flatpak kill "$app_id"
wait "$launcher_pid" || true
# An outside document must enter through the document portal, not --filesystem=home.
python3 - <<'DOCX'
from pathlib import Path
import zipfile
p=Path('/tmp/launch-test.docx')
with zipfile.ZipFile(p,'w') as z:
 z.writestr('[Content_Types].xml','<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
 z.writestr('_rels/.rels','<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
 z.writestr('word/document.xml','<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Leaf launch test</w:t></w:r></w:p><w:sectPr/></w:body></w:document>')
DOCX
flatpak run --socket=x11 --nosocket=wayland --file-forwarding "$app_id" --launch-file-test @@u file:///tmp/launch-test.docx @@ 2>dist/flatpak-file-open.log
grep -q LAUNCH_FILE_OK dist/flatpak-file-open.log
# Preferences written inside application data survive reinstalling the same bundle.
flatpak run --command=sh "$app_id" -c '
  leaf_state_root="${XDG_STATE_HOME:-$XDG_DATA_HOME/state}/wordcraft-gnome/recovery"
  leaf_app_root="${XDG_DATA_HOME%/data}"
  case "$leaf_state_root" in "$leaf_app_root"/*) ;; *) echo "State path outside persistent app data" >&2; exit 1;; esac
  test -d "$leaf_state_root"
  printf persistent > "$leaf_state_root/package-default-test"
'
# Also exercise the host fallback used when a runtime does not define XDG_STATE_HOME.
flatpak run --command=sh "$app_id" -c 'unset XDG_STATE_HOME; exec /app/bin/leaf --package-storage-test' | grep PACKAGE_STORAGE_OK
flatpak --user uninstall -y --noninteractive "$app_id"
# Intentionally do not pass --delete-data: verify normal reinstall preserves app data.
flatpak --user install -y --noninteractive dist/Leaf-0.1.0-alpha.3-x86_64.flatpak
flatpak run --command=sh "$app_id" -c '
  grep -q "autosave.*0" "$XDG_CONFIG_HOME/wordcraft-gnome/preferences.json"
  leaf_state_root="${XDG_STATE_HOME:-$XDG_DATA_HOME/state}/wordcraft-gnome/recovery"
  grep -q persistent "$leaf_state_root/package-default-test"
  grep -q persistent "$XDG_DATA_HOME/state/wordcraft-gnome/recovery/package-test"
'
# This CI seed is not shipped: every tester receives the normal recovery-mode default.
echo FLATPAK_INSTALLED_LAUNCHER_OK
echo FLATPAK_PORTAL_FILE_OPEN_OK
echo FLATPAK_REINSTALL_DATA_OK
echo FLATPAK_SMOKE_OK
