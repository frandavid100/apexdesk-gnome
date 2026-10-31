# Testing Leaf

Leaf is an alpha. Test copies, not your only copy of an important document. Reports in English or Spanish are welcome.

## Before starting

Record the release tag (or source commit), installation method, distribution, GNOME version, architecture, display scaling and whether you use Wayland or X11. For a source build, record `git rev-parse HEAD`; the old `--version` string does not uniquely identify current development snapshots.

Copy a short DOCX or ODT document to a test folder. In Preferences, keep autosave in **recovery-file mode**. Autosave to the original file can overwrite it if you explicitly enable that mode.

## A useful 15-minute test

1. Open the copy. Compare it with the same file in LibreOffice or the application that created it. Note missing text, images, fonts, unexpected page breaks or misplaced tables **before editing**.
2. Type an accented word, select across paragraphs, replace a selection, copy/paste and undo/redo. Apply a heading style and one text-format change.
3. Save under a new name in the same format, close and reopen the saved copy. Check content and formatting in Leaf and the other editor. Record whether a problem appeared on opening, editing, saving or reopening.
4. Find a repeated word, replace one match and then all matches. Undo the bulk replacement. Try another document in a second tab and make sure the first document is unchanged.
5. Insert a small table and an inline picture. Change the picture size with aspect ratio locked, undo it, and save/reopen again.
6. Narrow the window, switch dark/light appearance and navigate using the thumbnails or headings. Open and dismiss an editing menu while scrolled to a later page: it should not jump to the beginning.

A successful test is useful too: add your environment, file origin/format and the checks you performed to a compatibility report. A successful round-trip of one file does not establish support for all files from that application.

## Longer tests

Try a long document, a document with mixed section sizes, lists, tables across pages, and documents created by different editors. Test missing fonts separately from missing content. Keep these variables separate when possible.

For recovery, use a disposable document: enable recovery mode, edit, wait at least 15 seconds, terminate Leaf unexpectedly, reopen and use Recover Documents. Record the installed build and exact outcome. Explicitly discarded changes should not be offered for recovery again. Never force-terminate an editor containing unsaved work you care about.

Export a PDF and inspect it in Papers, including page dimensions, image placement and text. Exporting must not change the editable file's destination or undo history.

## Reporting a problem

Use [the issue forms](https://github.com/frandavid100/leaf/issues/new/choose). Include exact steps, expected/actual behavior and whether undo or reopening restores the document. Report one problem per issue. Search existing issues first.

Attach the smallest non-confidential original and affected copy if you can. Use invented content, and check comments, document metadata, filenames and screenshots for personal information. If you cannot share a file, report the steps and a description without it. Logs are optional; remove paths and private text before posting.

## Current validation baseline

The last executable validated by automated CI is commit `c744dac3f553b0304e631dd80dde7fbe3669e1cd`, [run 41144295677](https://github.com/frandavid100/leaf/actions/runs/41144295677). It includes release compilation, engine integration tests and actual GTK interaction checks. Later documentation/icon commits do not imply a new executable. There is no verified public Flatpak build yet.

Record findings in [TEST_RESULTS.md](TEST_RESULTS.md) only when a person has actually performed the test. Do not turn automated success into a manual confirmation.
