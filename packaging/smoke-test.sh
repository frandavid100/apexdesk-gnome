#!/usr/bin/env bash
set -euo pipefail
app_id=io.github.frandavid100.leaf
export GSK_RENDERER=cairo
export XDG_CURRENT_DESKTOP=GNOME
export XDG_DATA_DIRS="$HOME/.local/share/flatpak/exports/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
dbus-update-activation-environment DISPLAY XDG_CURRENT_DESKTOP XDG_DATA_DIRS
mkdir -p "$HOME/.config/xdg-desktop-portal"
printf '[preferred]\ndefault=gtk\n' > "$HOME/.config/xdg-desktop-portal/portals.conf"
flatpak run "$app_id" --version | grep -F 'Leaf 0.1.0-alpha.1'
flatpak run --socket=x11 --nosocket=wayland "$app_id" --chrome-ui-test 2>dist/flatpak-chrome.log
grep -q DOCUMENT_CHROME_OK dist/flatpak-chrome.log
flatpak run --socket=x11 --nosocket=wayland "$app_id" --styles-ui-test 2>dist/flatpak-styles.log
grep -q STYLES_POPOVER_OK dist/flatpak-styles.log
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
flatpak run "$app_id" --package-storage-test | grep PACKAGE_STORAGE_OK
flatpak --user install -y --noninteractive --reinstall dist/Leaf-0.1.0-alpha.1-x86_64.flatpak
flatpak run --command=sh "$app_id" -c 'grep -q "autosave.*0" "$XDG_CONFIG_HOME/wordcraft-gnome/preferences.json"'
# This CI seed is not shipped: every tester receives the normal recovery-mode default.
echo FLATPAK_INSTALLED_LAUNCHER_OK
echo FLATPAK_PORTAL_FILE_OPEN_OK
echo FLATPAK_REINSTALL_DATA_OK
echo FLATPAK_SMOKE_OK
