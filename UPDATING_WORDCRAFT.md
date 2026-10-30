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

## Sidebar search and paragraph popover

`search.rs` invokes literal `edit.advancedFind` in the body and consumes all public
`Session.find.results`, not the JSON response's 200-item sample. It preserves
selection while collecting context, pages and highlight geometry. Hit actions
validate document identity and revision before selecting or replacing through
public engine commands. Empty replacement deletes the range. UI result buttons
refresh after mutations/undo, and each tab owns its query and options.
`gnome_search.rs` owns the sidebar stack and weak callbacks; highlights are a
separate GTK overlay and do not change the document or the raster cache.
`gnome_paragraph.rs` replaces the old paragraph dialog with icon controls and
scrollable internal pages with a fixed back row. New paragraph/replace icons
are drawn as small foreground-colour vectors, so they work without optional
icon-theme assets and follow light/dark appearance. It retains the public paragraph commands; no engine
formatting model is copied. The source engine remains unmodified.

## Adaptive page canvas and focus

The GTK host uses a Fixed canvas with geometry from viewer_layout::arrange.
Two consecutive pages share a row only when they fit at the current zoom. Mixed
page heights and an odd last page retain document order; visibility lookup is
indexed by row. Text hit testing still uses the actual GTK page-frame bounds and
the upstream layout. No engine pagination or document properties are changed.
An explicit GtkViewport has scroll-to-focus disabled: the focusable canvas spans
the document and must not be automatically panned into view on popover dismissal.
Keyboard caret visibility remains an explicit host operation. CI reproduces a
bottom menu click, its native dismissing click, and a subsequent text click on
a later page with xdotool. Both dismissal and canvas focus must preserve scroll.

## Outline sidebar and native main menu

`navigation.rs` reads resolved outline levels through the public document/style
API and advances the engine's public numbering counters. TOC styles are omitted;
font size and localized style names are never used to guess headings. The host
only selects the heading position; collapse state belongs to each sidebar.
Search/navigation share a native GtkStack crossfade. Bulk replacement invokes
one literal edit.replaceAll command in the body, retaining one undo step.
`gnome_menus.rs` uses GMenu and window actions rather than styled GtkButtons.
Menu accelerator attributes display Undo/Redo shortcuts without globally
intercepting Ctrl+Z in native search/name fields. Other window actions dispatch
to the selected tab. AdwTabOverview supplies the native all-tabs view.


## Native document chrome

The document menu uses the same GMenu/GtkPopoverMenu pipeline as the main menu,
but owns Undo/Redo and Save/Save As entries. Window actions resolve the selected
tab at activation time; history accelerator labels do not intercept native text
field shortcuts. New tab and Save have no header buttons. The example action is
removed; --test-document is an internal CI fixture, normal startup stays blank.
Sidebar navigation uses a nested AdwToolbarView bottom bar, not content padding.
A retained vertical GtkSizeGroup aligns it with the editor toolbar. Both use Flat
style for native scroll undershoots. Thumbnail buttons enclose image plus label
with uniform 12px padding. These are host changes only, no engine updates.


## Pinned editing popovers

`gnome_popover_pin.rs` supplies a FormatPopoverGroup per tab and shared title,
back and pin header for Style, Paragraph, Text and Table. Only one panel is
visible; opening another hides its predecessor and inherits the group's pin.
Every editing popover is nonmodal from construction. Never change autohide on
a mapped popup: GTK recreates its surface and visibly blinks even if remapped
synchronously. Pinning changes only the host's dismissal policy. A non-claiming
window capture gesture dismisses unpinned panels on outside clicks, excluding
their popup surfaces and menu-button anchors. GTK still handles explicit menu
button close and Escape. Applying dismisses only when the shared pin is off.
The single header owns a replaceable back callback (cloned before invocation to
avoid RefCell reentrancy); subpages no longer insert another navigation row.
Paragraph split controls use native flat AdwSplitButton, including shared hover
background, half-button feedback and native separator/arrow. The documented child
hierarchy supplies the internal MenuButton; its lazy-popup callback resets the
button and switches the existing paragraph stack, without a nested popup. Main
actions retain selected-state feedback. Thumbnail pictures use Papers-style 6px
radii, a fine border and hidden overflow to clip the raster too. No engine changes
or added dependencies. Recheck split child hierarchy when upgrading libadwaita.
Tests wait for GTK frames, assert mapped surfaces, no hide signal/surface
replacement during pin toggles, and exclusive switching with pin inheritance.



## Images, page setup and PDF

`pictures.rs` selects exactly one picture through pure select.range, uses insert.picture, picture.change/size and text.delete, and leaves all media/layout handling upstream. New images are inline; preserve the engine's object hit testing and UTF-8 object character length. Selection painting uses its ObjectHit bounds. The fifth editing popover joins the existing exclusive pin group.
`document_properties.rs` clones each target section, updates only paper/margins, prevalidates all sections, and calls layout.pageSetup for each. The engine checkpoint/join/merge and edit_snapshot/restore APIs provide one undo and rollback on errors; existing headers, footers, columns and other settings stay distinct. Temporary caret commands restore the user's selection. Millimetre display rounding does not dirty unchanged fields.
`pdf_export.rs` uses pure file.exportPdf with an explicit .pdf destination. The GUI clones the document and creates a separate Session inside a GIO blocking worker, retaining the live session's path, dirty state, selection and history. Native file dialogs confirm the exact output path; never silently append an extension after confirmation.
`gnome_document_tools.rs` owns native popover, staged modal properties and file selectors. Async chooser callbacks validate document identity/revision before acting. No dependency or engine revision changes.



## Native context menus and rich clipboard

`gnome_context.rs` supplies a native GtkPopoverMenu/GMenu for text, table cells and pictures; the secondary-button gesture resolves the target with the same layout hit testing as primary clicks. An existing text or cell selection is retained when the pointer falls inside it. Otherwise the host selects only the clicked picture or moves the caret. Menu/Shift+F10 use the current selection. Actions reuse the host's existing adapters and public engine commands, never document mutations. The former table-only custom popover is removed.

Copy publishes the engine's plain-text and HTML representations through a GdkContentProvider union. A single app-wide engine fragment allows rich paste across tabs/windows while GTK owns that exact provider; external clipboard changes invalidate it by provider identity. Native entry fields retain their own clipboard handling. Text fallback captures document identity/revision/selection, so asynchronous reads cannot paste into another tab or a moved selection. Cut deletes only after a successful clipboard write. Context actions and Ctrl+C/X/V share these functions. No new dependencies or engine changes.



### Native row activation lifetime

GtkModelButton closes its ancestor popover before invoking its action-dispatch handler. Never unparent a context popover synchronously in its closed handler: this detaches the action muxer before native row dispatch. Defer unparenting to an idle callback and retain the popover through it. Tests must exercise the GtkModelButton clicked/activate path (and a real pointer click), not widget.activate_action, which bypasses that ordering. Nautilus context popovers use has-arrow=false and halign=start; shortcut attributes use the native accelerator column.

Normal startup anchors table/picture tool popovers directly to the document canvas instead of exposing toolbar menu buttons. Their pointing rectangle is updated from the context click. The shared outside-click policy exempts an anchor only when it is a MenuButton, never the whole canvas. Public engine operations and dependencies remain unchanged.

Canvas clearing must preserve GtkPopover children: standalone tools belong to the tab and need the same realized ancestor throughout opened/page rebuilds. Removing every Fixed child detaches their native parent before popup, including at normal startup. Deferred transient-menu cleanup also checks that a parent still exists.



## Adaptive controls and contained context pages

`gnome_adaptive.rs` observes actual viewport adjustment changes, moving existing history/format boxes between header and bottom bar. No permanent frame tick is introduced. Width hysteresis avoids relayout loops. Capture open popovers before hiding their former toolbar; reopening suppresses the usual home-page reset until allocation settles, retaining pin and selection.

Text/Style/Paragraph use GtkStack interpolation and native slide transitions. GtkPopoverMenu custom slots lease the same root widgets for contained contextual submenus, restoring them synchronously on closed while retaining the previous deferred native-menu unparenting. Weak embedded-host routing sends close/pin actions to the current container. Hosts join per-tab exclusive dismissal without changing modality on a mapped surface. The duplicated native submenu back row is hidden after guarded GtkStack page lookup; check this small GTK hierarchy boundary on GTK upgrades. All document operations remain public upstream commands, with no engine or dependency changes.

Viewport size notifications capture open panels synchronously before GTK unmaps a resizing anchor. Do not postpone that capture to idle: visibility can already have been lost. Reopening waits briefly for the new allocation, with a generation guard. Nonmodal context hosts explicitly focus their first native menu row because GtkPopover only does that automatically with autohide enabled. Context hosts inherit the same style-popover/format-popover classes, keeping font weight and active-state styling identical. Pointer regression targets the left of the document so the new header popup does not intercept its dismissing click.

Closing a nonmodal contextual host explicitly restores canvas focus (as Nautilus does), before deferred unparenting. The real-click regression checks both focus acquisition on opening and focus return after Copy, allowing normal keyboard editing to continue. Final validated revision: c744dac3f553b0304e631dd80dde7fbe3669e1cd, CI 41144295677.
