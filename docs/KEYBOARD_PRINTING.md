# Keyboard, control accessibility and printing — 2026-10-31

## Changes

F6 / Shift+F6 cycle between the document, sidebar and formatting controls. Tab keeps its existing table-cell navigation. F10 opens the visible main menu; Menu / Shift+F10 retain the context menu. Escape from a format popover restores focus to its anchor.

Explicit accessible names cover sidebar/search/zoom icons, replacement fields/actions, paragraph icons and split halves, colors, style actions, outline expanders, table insertion sizes and numeric formatting fields. Property rows associate their controls. About and --version share one application version.

Imprimir… in the document menu and Ctrl+P use GTK PrintDialog. The application exports a private vector PDF from a cloned document on a background worker. The live document, save path, dirty state, selection and history remain unchanged. Repeated requests are blocked while a job is pending. Temporary sources are retained until the callback, then cleaned on success, dismissal or error. Closing a tab cancels its pending operation; an application hold keeps cleanup running if the last window closes during export.

## Platform and maintenance

Source builds require GTK 4.14+ for PrintDialog; the GNOME 49 Flatpak runtime already satisfies this. There is no new Rust dependency, PDF rendering library, engine revision or compatibility patch. No extra Flatpak permissions are added.

The print adapter is src/printing.rs (private disposable snapshot) plus src/gnome_printing.rs (GTK lifecycle). It uses the existing PDF export command. Rendering/layout and file fidelity remain engine responsibilities. Temp directories use exclusive creation and Unix mode 0700; RAII removes them. Cancellation prevents a later dialog but cannot interrupt the engine's synchronous PDF conversion midway through its worker.

References reviewed: [GNOME keyboard HIG](https://developer.gnome.org/hig/guidelines/keyboard.html), [GTK PrintDialog](https://docs.gtk.org/gtk4/class.PrintDialog.html), [print_file](https://docs.gtk.org/gtk4/method.PrintDialog.print_file.html), [accessible properties](https://docs.gtk.org/gtk4/enum.AccessibleProperty.html).

## Validation

Native source `b1b42907f2059353518480899a3d4d56beb2996f`, [CI run 41225574820](https://github.com/frandavid100/leaf/actions/runs/41225574820), job `124608903617`: successful `cargo test --locked`, release GTK build, keyboard test, print portal test and the existing graphical/HiDPI suite at 1× and 2×.

- Headless regression checks independent private PDF jobs, file permissions, RAII cleanup and live document/selection/path/dirty/history preservation.
- Real xdotool events verify F6 and Shift+F6 cycling, F10/escape and search dismissal without document edits.
- Native GTK printing is tested through real D-Bus descriptor handoff for submission, dismissal and error; PDF text is checked with Poppler and temporary print directories must be gone afterwards.
- Existing context, menu chrome, styles, paragraph, table, document tools, adaptive and HiDPI regressions also pass.
- The 24-file corpus is not rerun for this UI/printing change; its optional trigger remains off.

Native artifact `13051823942` reports SHA256 `ca176fa5dc604a53fdc85dfd349aa071c85649090b10b066fb2305f047e12534` (GitHub-provided digest; not independently downloaded in this session). Poppler reads the PDFs and their text successfully; it still emits an upstream PDF metadata warning about the Suspects value. No PDF exporter change is included here.

Final packaging source/tag `3bb562d04129d668261b6aafb25833628d801480`, [run 41226912994](https://github.com/frandavid100/leaf/actions/runs/41226912994), job `124612866903`: passed. It reuses the proven native artifact only after git diff confirms src, Cargo.toml, Cargo.lock, tests, vendor and patches are identical to the native source above.

The installed Flatpak passes the real native print-dialog/Cancel action (PRINT_NATIVE_DIALOG_OK), simulated portal success/dismissal/error with PDF text assertions (PRINT_INTEGRATION_OK), upgrade from alpha.4, launcher, external file portal and persistent-data reinstall checks. Final artifact `13051049778` reports digest `a63e32c3e3c1bc1a57b5b183f0f06fe302abc17b2a2a724bc1bf415a6e47460e`. Release bundle: **10,151,304 bytes**, SHA256 `0a9d56137e03d2d622e3f224b6ac866851ece7a2766c18e4d750d3dc350ef34d` (GitHub asset digest). [Release alpha.5](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.5).

Initial validation failures were corrected before publication: a test-only GTK action receiver, the existing document-menu test's old item count, the portal Escape/Cancel distinction, and missing Poppler tools in the packaging test environment. No failed build was published.

The hermetic print test service runs only in private CI D-Bus sessions. It receives the real GTK Print call and Unix descriptor, verifies PDF bytes/text, and returns success, dismissal or error. It checks duplicate suppression, callback handling and cleanup without sending anything to hardware. The installed bundle additionally opens the real system/backend print dialog and activates its Cancel button through Alt+C. The service and scripts are not installed as application components.

### Portal dismissal caveat

The Ubuntu CI GTK portal backend (1.15.1) maps GTK_RESPONSE_DELETE_EVENT, including its Escape close path, to portal response 2 (failure). Its Cancel action maps to response 1 (dismissal). [Backend source](https://github.com/flatpak/xdg-desktop-portal-gtk/blob/1.15.1/src/print.c). The initial real-dialog test exposed this difference. The final test activates Cancel via Alt+C. Leaf keeps real failure reporting rather than treating every response-2 error as a silent cancellation. On affected backends, Escape/WM-close can therefore show a print error; use Cancel to dismiss normally. This is a backend-dependent UI limitation, not evidence that a file was printed or saved.

## Limits and manual checks

The custom canvas does not yet implement GTK AccessibleText or expose the complete editable text and selection to Orca. Better control labels and keyboard navigation are not full screen-reader accessibility. Screen-reader, high-contrast and physical-printer verification remain manual work. Existing PDF/font/layout limitations still apply.

The screenshot of the real print dialog is captured by CI; visual review was not completed in this session after the local execution environment became unavailable. Do not confuse this automated dialog/cancellation test with a physical print or manual GNOME OS test.

1. With the pointer unused, cycle F6 both ways, open/navigate/dismiss menus, and edit after returning. Repeat in a narrow window and a table. Search fields should retain their own text shortcuts.
2. Open Print with unsaved changes. Cancel without changing dirty state/history. Print a copy or use Print to File; inspect page dimensions, orientation, text and images. Test ranges, copies and duplex on a suitable printer.
3. With Orca, check names and states of sidebar, formatting, search/replacement and numeric fields. Record the custom-canvas limitation separately.
