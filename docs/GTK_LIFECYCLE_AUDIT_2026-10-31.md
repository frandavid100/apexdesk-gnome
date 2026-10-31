# GTK ownership and memory audit — 2026-10-31

This audit concerns the GNOME host, not WordCraft's document model or renderer. No Rust dependency or engine revision changes are required.

## Reproduction and evidence

The diagnostic opens a synthetic 60-paragraph DOCX using the normal open path. Each of three waves opens three tabs and two independent windows, exercises sidebar search and style/context menus, schedules a thumbnail refresh, and closes the documents through the normal tab/window APIs. Five documents close per wave, fifteen in total. The initial document/window close separately at the end.

Probes hold only weak references to document UI state, window state, the first document block, page pictures and representative GTK controls. The follow-up strict test also checks dialog and column-control lifetime and replacement of rich clipboard contents. It must report zero surviving closed objects; a lower RSS alone is insufficient.

Linux RSS comes from `/proc/self/status`. A diagnostic-only `malloc_trim(0)` distinguishes some allocator retention from reachable objects. It is never called by normal document editing. Measurements use release binaries, Cairo rendering and Xvfb on Ubuntu 24.04 CI (GTK 4.14.5, libadwaita 1.5.0, Rust 1.99.1). Font, GTK, renderer and allocator caches can remain; these numbers are a workload measurement, not a general RAM/speed promise or proof that every possible leak has been excluded.

### Before the fix

Instrumentation-only commit: `b7b5b7f792856fc5ca02f1f3b7769ad96320bd3a`, based on the alpha.5 host. [Baseline run 41234617745](https://github.com/frandavid100/leaf/actions/runs/41234617745), job `124635662105`. This build was not published.

| Closed documents | Live Ui | Live Workspace | Live document blocks/page pictures | RSS, KiB | RSS after trim, KiB |
| --- | --- | --- | --- | --- | --- |
| 5 | 5 | 2 | 0 / 0 | 225192 | 188684 |
| 10 | 10 | 4 | 0 / 0 | 255588 | 212208 |
| 15 | 15 | 6 | 0 / 0 | 271924 | 235676 |

The final root Workspace, Ui and GTK window also remained alive. This confirms host-state retention; it does **not** show that the document blocks and page images were leaked. Those were already released by the previous teardown.

### After the fix

Source `d988fec5e350ab6388e597fe891919f12f3bcda1`, [native run 41238438224](https://github.com/frandavid100/leaf/actions/runs/41238438224), job `124647025276`: all native checks passed, including cargo tests, the release GTK build, both lifecycle workloads, keyboard/print portal, graphical interaction and HiDPI checks.

| Closed documents | Live Ui / Workspace / GTK windows / probed controls / controllers | RSS, KiB | RSS after trim, KiB |
| --- | --- | --- | --- |
| 5 | 0 / 0 / 0 / 0 / 0 | 229212 | 168028 |
| 10 | 0 / 0 / 0 / 0 / 0 | 231040 | 169988 |
| 15 | 0 / 0 / 0 / 0 / 0 | 231028 | 171560 |

Document block/page-picture probes also remained zero. The initial root Ui, Workspace and GTK window all expired after closing. The additional dialog/clipboard workload passed all three waves too: zero retained probed objects and no obsolete rich fragment after external replacement. Its trimmed RSS was 161396 / 163396 / 164856 KiB, but it is a separate workload/process and should not be used to claim that opening dialogs lowers memory.

After fifteen closes, raw RSS was 40896 KiB (39.9 MiB, 15.0%) lower than the baseline. After diagnostic trimming, the difference was 64116 KiB (62.6 MiB, 27.2%). Between waves one and three, trimmed RSS grew by 46992 KiB before the fix and 3532 KiB after it. Some growth/caching remains; this is not a claim of perfectly flat memory or an overall speedup.

[Machine-readable measurements and provenance](audits/2026-10-31-lifecycle-results.json). The native artifact ZIP was independently downloaded and its SHA-256 matched GitHub: `47ef113b794572e71a5e7e8e5c8d573bfa27ad749dae674ebe99798cfe68b598` (artifact 13057140097, 24616786 bytes). Wide/narrow, style/text context menus, dark appearance and tab-overview screenshots were inspected; this is screenshot review, not manual GNOME OS testing.

## Changes

- Open workspaces have an explicit application owner, removed after confirmed close. GTK callbacks borrow host/document state weakly instead of forming `Ui → widget → callback → Ui` and corresponding Workspace cycles.
- Closing a document explicitly removes its autosave source, cancels pending printing, finishes recovery and clears page/thumbnail resources. Deferred callbacks use weak references and check that the tab remains open.
- Context menus return borrowed formatting widgets before teardown, including a popup dismissed before mapping. Restored leases immediately discard their strong references. Normal contextual action dispatch still defers unparenting until GTK has finished activating the action.
- Embedded `AdwTabBar` controls detach their strong `TabView` reference on document close. Without that step, closing a whole window leaves `TabView → page → toolbar → TabBar → TabView` alive even after the Rust host state drops. [libadwaita 1.5 source](https://github.com/GNOME/libadwaita/blob/1.5.0/src/adw-tab-bar.c) confirms that `adw_tab_bar_set_view()` uses `g_set_object()` for this reference.
- Main format popovers, transient table/image popovers and the window's per-tab outside-click controller detach at document close.
- The CSS provider registers once per display/session instead of once per tab.
- A single clipboard-change subscription releases the previous rich fragment when GTK no longer owns its provider. An active copied fragment intentionally stays available after closing its source document.
- Style, paragraph, text and property-dialog callbacks avoid owning their containing state/window. Existing editing commands, selection behavior and pinned menu interaction remain covered by the GUI regressions.

## Validation history

The first corrected build (`be329c46`, run 41235784470) released Rust Ui/Workspace state but failed the strict check with 18 remaining GTK controls. Explicit popover teardown then released those controls; the next build (`5b384298`, run 41236751836) exposed the separate embedded TabBar cycle in two closed windows (eight remaining controls). Neither failed build was distributed. The assertions were retained and extended to GTK windows and window-attached controllers. Detaching the mapped bar before unmapping exposed an animation teardown crash in libadwaita 1.5 (run 41237402660, debugger reproduction 41237928574). The teardown now unmaps the bar first, allowing animation callbacks to finish before their tab records are freed. Failed diagnostic builds were not distributed.

## Repeating the focused check

```sh
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d) GSK_RENDERER=cairo xvfb-run -a ./target/release/wordcraft-gnome --lifecycle-ui-test
XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d) GSK_RENDERER=cairo xvfb-run -a ./target/release/wordcraft-gnome --lifecycle-dialog-test
```

The first workload can be compared with the baseline. The second adds document-properties/columns dialogs and clipboard replacement; its RSS is not directly comparable to the simpler workload. The installed Flatpak also runs the first strict check. These focused regressions do not rerun the 24-file compatibility corpus.

## Limits and follow-up

No manual GNOME OS, Wayland, multi-monitor, physical-printer or Orca result is implied by CI. Active clipboard data, caches and an in-flight print worker's document snapshot/application hold are intentional owners. Window-level responsive breakpoint declarations still live until their window closes; libadwaita stores their widget targets weakly. The check does not claim that every small piece of window bookkeeping disappears after closing a single tab. Modal dialogs and asynchronous native selectors should continue to be reviewed when new functionality is added. The test observes selected objects and bounded workloads, not arbitrary files, every widget or an unbounded editing session.

## Installed Flatpak and release

[Leaf v0.1.0-alpha.6](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.6) is published. Tag, native source and package source are `d988fec5e350ab6388e597fe891919f12f3bcda1`. The package job `124649600736` in the same run passed upgrade from alpha.5, metadata validation, installed UI/HiDPI checks, strict lifecycle checks, a real print-dialog cancellation, three mock print-portal outcomes, desktop launcher, external document portal and persistent-data reinstall checks.

The sandbox lifecycle workload also released all probed objects over three waves and the root window. Its RSS uses a different GNOME 49 runtime and must not be compared directly with the Ubuntu-native baseline. Native dialog and clipboard tests passed separately. The full 24-file corpus was not rerun.

Bundle: `Leaf-0.1.0-alpha.6-x86_64.flatpak`, **10178880 bytes**, SHA-256 `11b146713f469a8dec062192e990852c6e12c0f2bb98f268d8b1aadbaf1baaac`. GNOME runtime commit `2bb946ea4ebb050f6a32f90e979feb975b08280d1ff1aac881dec3d1361ad84d`.

The final package artifact ZIP was independently downloaded and matched GitHub's digest `1e1e7a4fcf902ff970d2987db812945247f416b8131784ed838c71744202ba30` (artifact 13056876901, 10361386 bytes). Every entry in SHA256SUMS passed locally; source/runtime metadata and native/sandbox logs were inspected. The installed blank-document launcher screenshot was also reviewed. These are automated tests and artifact inspection, not a manual test on the user's machine.
