# One-off DOCX/ODT preservation audit — 2026-10-31

## Findings

All **24 files opened, saved through Leaf's save adapter, and reopened without process failures**. This is a targeted sample of regression fixtures, not a representative compatibility percentage.

| Group | Result |
|---|---|
| 14 DOCX | No differences in the measured semantic signatures; all checked page images pixel-identical. |
| 3 ODT | Confirmed content/structure loss in headers or footers, including two nested tables. |
| 2 ODT | First table row changes from an ordinary row to a repeating header row. |
| 5 ODT | Only small page-dimension/margin rounding or raster differences found in these checks. |

**Avoid saving ODT files containing headers or footers in the current version.** This includes tables placed in those areas. Use copies and verify any alternative output in another editor. No fix or new application release is included in this audit.

The normalized body text and page count survived in all 24 cases. Imported media-byte fingerprints survived in all 24. That does not prove all original images imported correctly, or that body layout is identical to Word/LibreOffice.

## Reproducible losses

| Fixture | Before → after saving as ODT | Evidence |
|---|---|---|
| nestedTableInFooter.odt | 1 footer part → 0; 2 nested tables → 0 | Footer table visibly disappears on page 1. Cells are empty, so text-only checking would miss this. |
| tdf135338_firstLeftPageFooter.odt | 7 header/footer parts → 0 | Alternate/first-page footer strings disappear; body text stays. |
| tdf104254_noHeaderWrapping.odt | 1 header part → 0 | “Some text, all on one line.” disappears from the page header. |
| table_styles_1.odt and table_styles_2.odt | First-row repeating-header property false → true | Changes table behavior even though the one-page raster is nearly unchanged. |

Code inspection of the pinned engine supports the first finding: [ODT master-page export](https://github.com/storytold/wordcraft/blob/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc/crates/formats/src/odt/package.rs#L49) emits empty master-page elements without header/footer content, whereas its importer reads those stories. For table rows, [the common export model](https://github.com/storytold/wordcraft/blob/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc/crates/formats/src/model/from_doc.rs#L300) promotes a visually styled first row to header status; the ODT writer serializes it as table-header-rows. Findings apply to **the pinned revision**, not a claim about every upstream version.

Small differences include margins rounded by roughly 0.003 points and an ODT page whose raster height changes from 793 to 792 pixels at a rounding boundary. These are recorded separately from loss. The two table-header cases have only 3 and 62 pixels differing by more than two channel levels on the checked page; visual similarity does not establish semantic preservation.

## Method and limits

- Current Leaf application source: 45037abc0ce0e323ea03f8f2c20218eda08c77a2 (alpha.3); audit additions do not change application behavior.
- Pinned WordCraft: 165d5725dc25b4a7887d8025c3b1fe506ccc3dcc. Dependencies/engine unchanged.
- Worker calls the same Leaf open_document and saving::save adapters as the GUI. No editing is performed; each file is saved to a new temporary destination in its original format.
- Full imported versus reopened body/part structure, text, effective paragraph/character properties, table grids/row/cell properties, inline objects, media fingerprints and physical section/page properties are compared. Character/paragraph resolution does not fully account for conditional table-style context; rendered checks provide an additional signal.
- Numeric comparison tolerance: 0.001 points. ID or equivalent-serialization differences require interpretation.
- Renders use the engine PNG exporter before saving and after reopening: first two pages at most, **28 page pairs** total. No independent Word/LibreOffice visual reference was used.
- Original XML text versus imported text is a supplementary word-token diagnostic, not a full import-fidelity proof. The verified run has no missing-token flags. It ignores punctuation/order and does not validate all source formatting or unused style definitions.
- All original files are untouched. The audit downloads fixtures from a fixed revision and verifies each Git blob hash; final provenance also records SHA256.
- Initial run 41215402571 found the same preservation defects. A false import-text flag caused by splitting words at XML run boundaries was corrected; verification run 41216038237 reproduces the findings with zero such flags.
- Success of the Actions job means the audit completed, **not that all files passed preservation**.

## Reproduction and retained evidence

[Verified Actions run](https://github.com/frandavid100/leaf/actions/runs/41216038237), job 124580648151, source 6c866cb12a996889f9b8b7d643d485ef87df3c24.

Artifact leaf-corpus-audit, ID 13047846452, ZIP SHA256 ce3a2a5548a5c4a5a85cd6baeee7b043f487bf0b39e7f587824266a26f018d20. The artifact includes per-case before/after/difference PNGs, logs and license notices; Actions artifacts expire. [Permanent detailed results](audits/2026-10-31-results.json) retain measured differences and fixture hashes.

Optional commands:
```sh
cargo build --release --locked --example corpus_audit
python -m pip install pillow==11.3.0
python tools/corpus_audit.py
```
The separate workflow is manual-only after this requested run. It is not scheduled, not called by the release workflow, and not a release prerequisite.

## Next priority

Address ODT header/footer export preservation before adding more editing features; add small focused regressions when implementing the fix. Review repeating table-header conversion separately. Broader independent visual import testing remains future work.

---

# One-off DOCX/ODT corpus audit

LibreOffice revision: `6e0b39149b549bd05065d451006c4d060589891d`. 24 fixtures. Original files were not modified. No release gate was added.

Import token differences are diagnostic flags, not proof of loss: generated fields, repeated/header stories and XML text boundaries can differ. Round-trip semantic differences and first-two-page raster comparisons are separate checks. Float tolerance is 0.001 points; differing style/list/part IDs and equivalent serializations require inspection. No Word/LibreOffice visual baseline is claimed.

| File | Open/save/reopen | Changed semantic categories | Pages | First 2 pages identical | Source token flags |
|---|---|---|---|---|---|
| defaultStyle.docx | unchanged |  | [1, 1] | True | 0 |
| 2_table_doc.docx | unchanged |  | [1, 1] | True | 0 |
| Table_cell_auto_width_fdo69656.docx | unchanged |  | [1, 1] | True | 0 |
| floating-table-position.docx | unchanged |  | [1, 1] | True | 0 |
| 2col-header.docx | unchanged |  | [1, 1] | True | 0 |
| multi-column-separator-with-line.docx | unchanged |  | [1, 1] | True | 0 |
| ImageCrop.docx | unchanged |  | [1, 1] | True | 0 |
| image_data.docx | unchanged |  | [1, 1] | True | 0 |
| footnote.docx | unchanged |  | [1, 1] | True | 0 |
| firstheadernofooter.docx | unchanged |  | [2, 2] | True | 0 |
| footer-margin-lost.docx | unchanged |  | [1, 1] | True | 0 |
| margins_from_style.docx | unchanged |  | [1, 1] | True | 0 |
| NumberedList.docx | unchanged |  | [1, 1] | True | 0 |
| TextEffects_InStyle.docx | unchanged |  | [1, 1] | True | 0 |
| listformat.odt | differences | page_properties | [1, 1] | False | 0 |
| table_styles_1.odt | differences | tables, page_properties | [1, 1] | False | 0 |
| table_styles_2.odt | differences | tables, page_properties | [1, 1] | False | 0 |
| nestedTableInFooter.odt | differences | text, character_format, paragraph_format, tables, objects, page_properties | [1, 1] | False | 0 |
| image-mimetype.odt | differences | page_properties | [1, 1] | True | 0 |
| ooo321_stylepagenumber.odt | differences | page_properties | [4, 4] | False | 0 |
| tdf133507_contextualSpacingSection.odt | differences | page_properties | [2, 2] | False | 0 |
| tdf135338_firstLeftPageFooter.odt | differences | text, character_format, paragraph_format, objects, page_properties | [5, 5] | False | 0 |
| arabic-zero-numbering.odt | differences |  | [1, 1] | False | 0 |
| tdf104254_noHeaderWrapping.odt | differences | text, character_format, paragraph_format, objects, page_properties | [1, 1] | False | 0 |

Detailed, capped property differences and token flags are in results.json. Per-case images and worker logs accompany the Actions artifact. Count a case as fully measured-unchanged only if all compared semantic categories and rendered pages match; that still does not establish source import fidelity or full-document visual fidelity.

## Provenance and licensing

Fixtures are downloaded for this audit from LibreOffice/core at the pinned revision and verified against their Git blob SHA. LibreOffice has MPL/LGPL and other applicable notices; fixture-specific provenance may vary. This repository does not relicense or vendor the fixtures. Source links and project notices: https://www.libreoffice.org/licenses/ and https://github.com/LibreOffice/core/tree/6e0b39149b549bd05065d451006c4d060589891d. Output images are derived test evidence; retain original attribution/notices when redistributing. The artifact contains the upstream COPYING, COPYING.LGPL and COPYING.MPL notices.
