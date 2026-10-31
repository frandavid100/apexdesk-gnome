# Leaf 0.1.0-alpha.1 (x86_64)

This is an experimental GitHub build, not a Flathub release. The interface is primarily Spanish. Use copies of important documents.

Download `Leaf-0.1.0-alpha.1-x86_64.flatpak` from this GitHub prerelease. Open it with GNOME Software and choose Install. If the graphical installer does not offer to add the runtime source, use:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user ./Leaf-0.1.0-alpha.1-x86_64.flatpak
flatpak run io.github.frandavid100.leaf
```

The bundle needs the GNOME 49 runtime from Flathub. It is downloaded separately on first installation, so the total download can be considerably larger than the Leaf bundle. An internet connection is needed if the runtime is missing.

Leaf appears in your launcher with its own icon and can be selected in Files → Open With for DOCX and ODT. It does not change your default editor automatically. Choose documents through the native file selector; the package has no broad access to your home folder and no network permission.

This single-file alpha has no automatic Leaf update feed. Download a subsequent GitHub bundle with the same application ID and install it over the previous version; do not uninstall or delete application data to update. Flatpak keeps its application-specific data under `~/.var/app/io.github.frandavid100.leaf`. Preferences/recovery from the earlier unpacked executable are not automatically imported.

Run `sha256sum -c SHA256SUMS` in the directory containing the release assets to verify their checksums. Installation testing under the CI runtime is recorded in `flatpak-smoke.log`; this does not replace testing on your own GNOME OS installation.

Report problems at https://github.com/frandavid100/leaf/issues/new/choose with this version, your distribution and reproducible steps. See the testing guide in the repository. Development, tests, packaging, documentation and artwork have used substantial AI assistance.
