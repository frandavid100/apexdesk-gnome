# ODT preservation follow-up — 2026-10-31

This follows the [original one-off audit](CORPUS_AUDIT_2026-10-31.md). It concerns defects in releases through alpha.3; the original measurements remain historical evidence.

## Correction

The ODT exporter previously wrote empty master pages, discarding header/footer stories. Alpha.4 writes those blocks with the existing ODT Writer, including nested tables, automatic styles and embedded image resources. It resolves linked section headers and keeps parts belonging to unused imported page styles in unreferenced master pages, so their content survives without appearing in the body. ODT conversion now distinguishes explicit repeated rows from visual first-row styling.

Only wordcraft-formats is overridden, from the same pinned upstream revision. The source diff, dependency boundary and update procedure are recorded in [patches/](../patches/README.md). Other engine crates and the renderer are unchanged.

## Validation

Source: `d5f969801fb8f069396922cb6603ff4cfa68b35c`. [CI run 41219266814](https://github.com/frandavid100/leaf/actions/runs/41219266814), native job `124590219910`: successful `cargo test --locked`, release GUI build, existing GTK regressions and HiDPI checks at scales 1 and 2.

| Check | Result |
|---|---|
| 14 DOCX | Measured signatures unchanged; checked page images identical. |
| 10 ODT | No text-token losses or changes to measured character/paragraph formatting, objects or media fingerprints. |
| nestedTableInFooter.odt | 1 → 1 footer; 2 → 2 nested tables. Only dimensional rounding remains. |
| tdf135338_firstLeftPageFooter.odt | 7 → 7 page parts, including unused page-style stories; all text retained. |
| tdf104254_noHeaderWrapping.odt | 1 → 1 header; text retained and visible in LibreOffice. |
| table_styles_1/2.odt | Ordinary first rows stay ordinary; explicit repeated rows covered separately by regression. |
| All 24 | Normalized body text and page counts retained; no process failures. |

Native artifact `13049921141`, ZIP SHA256 `af934ca273e37a417346f17ff0b0096fbe816a37fe93b407d10837c302880081`. Downloaded evidence matched this digest. The saved ODT package also contains “Footer” and the three other unused footer stories in separate unreferenced master pages.

Visual review of the four LibreOffice exports confirms the nested empty tables and header/footer text are visible. Three ODT checked page images have 3, 62 and 59 changed RGB pixels respectively above the audit threshold (maximum channel delta > 2); some other files have only sub-threshold differences. The Arabic numbering fixture retains its existing 793 → 792 pixel image-height rounding.

Seven focused regressions cover all six active header/footer slots, bold text, embedded PNG bytes, nested tables, repeated-row flags, linked sections, empty documents, unused page-style stories, disabled first/even modes, export immutability and save failure/success state. They run through public engine I/O or Leaf's normal save adapter.

The same 24 LibreOffice fixtures (14 DOCX / 10 ODT) are opened, saved and reopened. Their pinned source revision and fingerprints are in the committed [results JSON](audits/2026-10-31-odt-fix-results.json). This checks imported document state before/after save, not full fidelity to the original office application.

LibreOffice independently opens four repaired ODT files and exports PDFs: both nested footer tables, header text, first/even/default footer variants, and an ordinary table fixture. PDF text assertions check the actual visible header/footer phrases; the empty table fixtures require visual inspection. The earlier assertion searching for “Footer” in the rendered current-page footer was incorrect: that text belongs to an unused page style. The corrected exporter preserves it in an unused master page; it must not be expected in visible PDF text.

## Remaining limits

Small ODT numeric rounding remains (typically under 0.005 pt), with some changed edge pixels; one existing page-height rounding case changes raster height by a pixel. This patch does not implement arbitrary ODF page styles, exact geometry, missing import features, unequal ODT column widths or universal Word/LibreOffice fidelity. Disabled first/even variants preserve their content via unused master pages, but their disabled-slot association is not retained by the existing importer.

The 24 fixtures are a small, selected corpus. Engine-rendered before/after pages cover only the first two pages per document (28 page pairs); independent LibreOffice checking covers four saved files, not the whole corpus or all original-vs-saved layouts. Media checks only cover data that imported successfully. Manual editing and recovery behavior on the user's real desktop still need testing.

This was a one-off recheck to validate the fix. The corpus remains manually runnable; it is not required before every version.
