#!/usr/bin/env bash
# GitHub alpha bundle from the tested native build. Not a Flathub manifest.
set -euo pipefail
app_id=io.github.frandavid100.leaf
runtime_version=49
version=0.1.0-alpha.2
binary=${1:?Usage: build-bundle.sh TESTED_BINARY}
work=flatpak-build
repo=flatpak-repo
out=dist
mkdir -p "$out"
flatpak --user remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak --user install -y --noninteractive flathub "org.gnome.Platform//$runtime_version"
# No SDK compilation is performed here; the tested Ubuntu-built executable is staged.
flatpak build-init "$work" "$app_id" org.gnome.Platform org.gnome.Platform "$runtime_version"
install -Dm0755 "$binary" "$work/files/bin/leaf"
install -Dm0644 data/io.github.frandavid100.leaf.desktop "$work/files/share/applications/$app_id.desktop"
install -Dm0644 data/io.github.frandavid100.leaf.metainfo.xml "$work/files/share/metainfo/$app_id.metainfo.xml"
install -Dm0644 data/icons/hicolor/scalable/apps/leaf.svg "$work/files/share/icons/hicolor/scalable/apps/$app_id.svg"
install -Dm0644 data/icons/hicolor/symbolic/apps/leaf-symbolic.svg "$work/files/share/icons/hicolor/symbolic/apps/$app_id-symbolic.svg"
install -Dm0644 LICENSE "$work/files/share/licenses/$app_id/LICENSE"
install -Dm0644 docs/AI_ASSISTANCE.md "$work/files/share/doc/leaf/AI_ASSISTANCE.md"
# Portals grant individual documents; no home/host filesystem or network access.
flatpak build-finish --command=leaf --share=ipc --socket=wayland --socket=fallback-x11 --device=dri "$work"
flatpak build-export "$repo" "$work" alpha
flatpak build-bundle --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo "$repo" "$out/Leaf-$version-x86_64.flatpak" "$app_id" alpha
if test -f prior-release/Leaf-0.1.0-alpha.1-x86_64.flatpak; then
  flatpak --user install -y --noninteractive prior-release/Leaf-0.1.0-alpha.1-x86_64.flatpak
  flatpak run --command=sh "$app_id" -c 'mkdir -p "$XDG_DATA_HOME"; printf retained > "$XDG_DATA_HOME/upgrade-test"'
fi
flatpak --user install -y --noninteractive "$out/Leaf-$version-x86_64.flatpak"
if test -f prior-release/Leaf-0.1.0-alpha.1-x86_64.flatpak; then
  flatpak run --command=sh "$app_id" -c 'grep -q retained "$XDG_DATA_HOME/upgrade-test"'
  echo FLATPAK_UPGRADE_OK
fi
flatpak --user info --show-metadata "$app_id" > "$out/flatpak-metadata.txt"
flatpak --user info --show-runtime "$app_id" > "$out/runtime.txt"
flatpak --user remote-info --show-commit flathub "org.gnome.Platform//$runtime_version" > "$out/runtime-commit.txt"
printf '%s\n' "${GITHUB_SHA:-local-build}" > "$out/source-commit.txt"
cp packaging/INSTALL.md "$out/INSTALL.md"
