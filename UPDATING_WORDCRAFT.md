# Updating the WordCraft dependency

The GNOME client consumes unmodified upstream crates at a pinned Git revision.
There is no forked engine code, egui patch or copied renderer to merge.

## Boundary

- `src/lib.rs`: opening and rasterization adapter.
- `selection.rs`, `editing.rs`, `formatting.rs`, `saving.rs`: typed command adapters.
- `preferences.rs`: GNOME host settings; no engine internals.
- `gnome_host.rs`: tab lifecycle, recovery scheduling and native host dialogs.
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
