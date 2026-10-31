# Isolated WordCraft formats compatibility patch

Upstream: https://github.com/storytold/wordcraft at 165d5725dc25b4a7887d8025c3b1fe506ccc3dcc, crate wordcraft-formats 0.12.0 (MIT OR Apache-2.0).

Cargo overrides only this crate with vendor/wordcraft-formats. The snapshot retains upstream source and license notices; only src/odt/package.rs and the standalone Cargo manifest differ. All other engine crates stay on the existing Git revision. This is an explicit exception to the earlier unmodified-engine state, justified by confirmed ODT data loss. It fixes all engine save paths, including autosave, without replacing the engine's atomic save or session bookkeeping.

wordcraft-formats-odt.patch records the exporter change. Cargo.toml expands inherited workspace version, license and dependencies into a standalone manifest with the same pinned doc/geom sources and dependency ranges. No new runtime libraries are introduced.

On an engine update: compare the upstream ODT implementation first. If fixed upstream, remove the Cargo patch and vendor snapshot, regenerate Cargo.lock, and run tests/odt_preservation.rs plus the affected corpus cases. Otherwise recopy this crate at the new fixed revision, reapply/review the small exporter patch, regenerate the standalone manifest/lock and validate. Never silently keep a vendor snapshot from a different engine revision.

The exporter shares one Writer across body and page parts so automatic-style and picture identifiers cannot collide. Both XML style scopes receive those definitions. Page parts include nested tables and picture resources. The ODT-only export view disables the common flow model's visual-header heuristic; actual RowProps.header flags remain. Other format converters are unchanged.

Unused imported header/footer stories are retained in unreferenced master pages. Disabled first/even content is also retained there, without enabling those modes; its disabled-slot association is an existing importer limitation. This is a targeted preservation fix, not full ODF page-style support.
