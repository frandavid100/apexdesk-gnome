Leaf 0.1.0-alpha.4 repairs ODT header/footer preservation and unintended repeating table rows.

ODT output now writes page-part content, automatic styles and media into its master pages. Default, first-page and even-page slots and linked section headers are covered by focused regressions. First-row styling no longer implies a repeating header in ODT.

The fix is an isolated Cargo override of wordcraft-formats from the same pinned upstream revision; patches/ records provenance and the source diff. Other engine crates remain pinned and unchanged. New tests and targeted corpus checks run before publication. Broader document fidelity still needs testing; this is not complete Word/LibreOffice compatibility.

Install Leaf-0.1.0-alpha.4-x86_64.flatpak over alpha.3. GNOME 49 runtime; no automatic Leaf update feed. AI-assisted development. Use document copies. Unequal column widths still round-trip only in DOCX on this engine.
