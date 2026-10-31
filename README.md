# Leaf

<img src="data/icons/hicolor/scalable/apps/leaf.svg" width="96" height="96" alt="Leaf icon">

**Create and edit documents.**

Leaf is an experimental document editor for Linux, built with GTK4 and libadwaita. It uses the [WordCraft](https://github.com/storytold/wordcraft) engine for document layout, rendering and editing.

**Alpha development:** use copies of your documents. Compatibility with complex DOCX and ODT files is still being tested. Leaf is an independent project, not an official GNOME application.

![Leaf development build: document editing with page thumbnails](docs/screenshots/editor.png)

## What works today

- Open, edit and save DOCX and ODT documents; export to PDF.
- Text formatting, paragraph settings, section columns, styles, tables and inline images.
- Undo/redo, rich clipboard operations and sidebar search with replacement.
- Multiple tabs and windows; page thumbnails and heading navigation.
- Adaptive editing controls, dark appearance and configurable autosave/recovery.

These are implemented features, not a claim of complete Microsoft Word or LibreOffice compatibility. See the [testing guide](docs/TESTING.md) for the areas that need real-document testing.

## Try Leaf

The current **x86_64 Flatpak alpha** is available on [GitHub Releases](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.4). Download `Leaf-0.1.0-alpha.4-x86_64.flatpak` and open it with GNOME Software, or follow [installation instructions](packaging/INSTALL.md). It uses the GNOME 49 runtime from Flathub, downloaded separately if missing. Leaf itself is not currently on Flathub.

The installed package has passed automated launcher, interface, document-portal opening and persistent-data upgrade/reinstall checks on Ubuntu CI. Manual testing on GNOME OS and with your own documents is still needed. This standalone bundle has no automatic Leaf update feed.

Experienced testers can build the development branch. Requirements: Rust 1.95 or newer, Git, pkg-config, GTK 4.10+ and libadwaita 1.4+ development libraries. The existing CI runs on Ubuntu 24.04; other host environments have not been comprehensively tested.

```sh
git clone --branch wordcraft-saving https://github.com/frandavid100/leaf.git
cd leaf
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
./target/release/wordcraft-gnome
```

The source-build target retains its internal name, `wordcraft-gnome`; the installed application is Leaf, ID `io.github.frandavid100.leaf`, and its installed executable is `leaf`. GNOME OS users should install the Flatpak rather than add development packages to the host.

## Help test it

Start with the [15-minute testing guide](docs/TESTING.md), then [report a bug or document compatibility issue](https://github.com/frandavid100/leaf/issues/new/choose). Spanish and English reports are welcome. Do not upload confidential documents; a small, anonymized reproducer is more useful.

![Sidebar search and replacement in a development build](docs/screenshots/search.png)

<details>
<summary>Dark appearance</summary>

![Leaf development build with dark window chrome](docs/screenshots/dark.png)

</details>

Screenshots are actual captures of the last validated development build, using a synthetic test document. They predate Leaf branding integration. [Capture provenance](docs/screenshots/README.md).

## Limits of this alpha

Alpha.4 repairs the ODT header/footer and repeating-table-row defects found in our [one-off audit of 24 fixtures](docs/CORPUS_AUDIT_2026-10-31.md). [Follow-up results and remaining limits](docs/ODT_PRESERVATION_2026-10-31.md). Older releases still have those defects; this focused correction is not a general compatibility guarantee.

Complex document fidelity, fonts, pagination and format preservation need broader testing. Images are inserted inline; floating-image positioning is not exposed. Tables do not yet have mouse resize handles or formulas. Unequal column widths are retained in DOCX but equalized by the current ODT exporter. Printing and a complete English interface remain future work. Automated checks do not establish fidelity for arbitrary documents.

For everyday work, keep your original and check saved copies in another editor. Autosave to the original file is an explicit preference; use recovery mode while testing.

## Development and credits

The active branch is **`wordcraft-saving`**. Detailed [Spanish usage notes](docs/USER_GUIDE_ES.md) are also available. See [project status](PROJECT_STATUS.md), [roadmap](ROADMAP.md), [contributing](CONTRIBUTING.md) and [updating WordCraft](UPDATING_WORDCRAFT.md). The repository's default branch still contains an earlier prototype; follow the branch link when browsing the source.

WordCraft remains pinned; Leaf carries one isolated ODT compatibility patch in wordcraft-formats, documented in [patches/](patches/README.md). Other engine crates and the renderer remain unchanged; the native frontend is maintained separately. Leaf is licensed under [GPL-3.0-or-later](LICENSE). Development, tests, documentation and artwork have used AI assistance; [scope and limitations](docs/AI_ASSISTANCE.md).