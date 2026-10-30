# Updating the WordCraft dependency

The GNOME client consumes unmodified upstream crates at a pinned Git revision.
There is no forked engine code, egui patch or copied renderer to merge.

## Boundary

- `src/lib.rs`: opening and rasterization adapter.
- `selection.rs`, `editing.rs`, `formatting.rs`, `saving.rs`, `tables.rs`: command adapters.
- `preferences.rs`: GNOME host settings; no engine internals.
- `gnome_host.rs`: tab lifecycle, recovery scheduling and native host dialogs.
- `gnome_format_panels.rs`: native advanced text/paragraph formatting via public engine commands.
- `styles.rs`, `gnome_styles.rs`: style command adapter, isolated previews and inline popover pages.
- `gnome_tables.rs`: native insertion, contextual table controls and properties.
- `bin/gnome.rs`: GTK/libadwaita document view and input.

The host uses public Session, DocLayout and command APIs. Recovery encoding,
locking and background I/O remain upstream responsibilities. The GNOME settings
choose when to tick recovery or save the document; these policies are host code.

## Upgrade procedure

1. Change the three WordCraft `rev` values in Cargo.toml together.
2. Check upstream's Cargo.toml for its required `krilla` patch revision and Rust MSRV.
3. Regenerate Cargo.lock, inspect its dependency diff and run `cargo test`.
4. Run the GUI build and CI screenshots, including tabs, independent windows,
   find/replace, preferences and dark/narrow layouts.
5. Manually round-trip representative DOCX and ODT documents with formatting,
   lists, tables and multiple pages. Verify undo, hit testing and clipboard.
6. Verify recovery after forced process termination and confirm explicit discard
   does not offer an obsolete copy at the next start.

A source-compatible upstream update should be a revision bump plus validation.
Command names, public Rust types, file fidelity or layout changes can require
adapter changes. Avoid unpinned upstream main and changes to engine internals.

## Production performance

Distribute `cargo build --release --locked --features gui --bin wordcraft-gnome`,
never a stripped development binary. The release profile enables speed
optimization, ThinLTO, one codegen unit, and symbol stripping. Panic unwinding
remains enabled because upstream catches panics in command and rendering paths.

The GTK adapter consumes native premultiplied RGBA without an extra full-page
copy. Straight RGBA remains available for headless callers. Page lookup uses
binary search over ordered page positions; equivalence is checked against
the original full scan. Cached thumbnail repainting is coalesced to 120 ms
while editing. These changes do not modify the upstream engine.

`cargo run --example performance` and `cargo run --release --example performance`
exercise the same pages at full and thumbnail sizes. The benchmark excludes
pixel checksum calculation from raster timing. Compare checksums and timings
on the same runner; results describe the sample workload, not every document.

## Table integration

The pinned engine marks `text.tab` as a mutation even when only its selection
changes. `tables::navigate` uses pure `caret.set`/`select.range` for existing
cells, preserving dirty state, redo and typing boundaries. In the last cell it
uses `text.tab` to let upstream add the row. Literal tabs use `text.insertTab`.
Recheck this distinction on engine upgrades. The adapter never mutates the
table structure itself. Cross-cell dragging uses extending caret commands so
WordCraft owns rectangular cell selections; highlights use public layout boxes.

Table regressions cover structure, undo/redo, selected-column shading, empty
cell selection, merges/splits, DOCX/ODT common properties and multipage rendering.
CI also sends real Tab/Shift+Tab and text input under Xvfb with xdotool.

## Style popover

The list reads every paragraph/character style from the current stylesheet.
The host keeps list, detail and name-entry pages in one GtkPopover/GtkStack.
Preview renders a fresh sample Session with a cloned stylesheet, with proofing
disabled; it never runs preview commands on the live document. Its small
premultiplied raster is cropped for display. CRUD uses public commands, with
client-side empty/duplicate-name checks and built-in deletion disabled.
Creation supplies the current paragraph style as basedOn.

CI tests preview immutability and style operations, then exercises real widget
callbacks for creation, rename, update, deletion, navigation and filtering.
It asserts that no additional application window is opened.

## Caret motion and initial sidebar

`caret_motion.rs` interpolates only the painted caret, for 100 ms, with retargeting
from its current visual position. Engine geometry, selection, hit testing and
scroll-to-caret always use the exact destination. A GTK frame-clock callback is
registered only during movement; it respects gtk-enable-animations. Selection
drags, missing carets, new sessions and page changes snap. No upstream patch.
The sidebar's initial visibility is set before attaching the tab, with no delayed
show call. Explicit user toggles retain libadwaita's normal transitions.
