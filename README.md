# Leaf

**Create and edit documents.**

Leaf is an experimental GTK4/libadwaita document editor for Linux, powered by [WordCraft](https://github.com/storytold/wordcraft). It is an independent project designed for GNOME.

**The active project is on [`wordcraft-saving`](https://github.com/frandavid100/leaf/tree/wordcraft-saving).** This default branch contains an older ApexDesk prototype and is retained as history. Do not build this branch to try the current editor.

[Read the current README and see screenshots](https://github.com/frandavid100/leaf/blob/wordcraft-saving/README.md).

## Status

The current development version implements DOCX/ODT editing and saving, PDF export, section columns, styles, tables, inline images, tabs and sidebar search/replacement. Complex document fidelity is still being tested: use copies of your files.

**[Download the current x86_64 Flatpak alpha](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.3).** Open the `.flatpak` with GNOME Software or follow the release's INSTALL.md. The GNOME 49 runtime is downloaded from Flathub if required. Leaf itself is not on Flathub. This is experimental: use copies of your documents.

## Get involved

- [Testing guide](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TESTING.md)
- [Report a bug or document compatibility result](https://github.com/frandavid100/leaf/issues/new/choose) — English and Spanish welcome.
- [Development status](https://github.com/frandavid100/leaf/blob/wordcraft-saving/PROJECT_STATUS.md)
- [Release preparation](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/RELEASE_CHECKLIST.md)
- [AI assistance disclosure](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/AI_ASSISTANCE.md)

Current frontend development uses substantial AI assistance; scope and validation limits are documented above. WordCraft is consumed unmodified at a pinned revision. Current Leaf source is GPL-3.0-or-later.

Pull requests for the current editor should target `wordcraft-saving`.
