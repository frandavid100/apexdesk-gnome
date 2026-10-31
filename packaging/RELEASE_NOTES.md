Leaf 0.1.0-alpha.2 adds document columns and native symbolic formatting icons.

**Columns:** open Paragraph → Columnas…. Choose 1–12 equal columns, asymmetric presets or custom widths/gaps, and an optional separator. Apply to the current section, the whole document, selected body text or from the cursor onward. Selection mode creates continuous sections through WordCraft; the change is one undo step. Going back without applying leaves the document unchanged.

**Icons:** Style, Paragraph and Text now use GTK symbolic icons with native sizing and alignment instead of Cairo-drawn letters.

Download **Leaf-0.1.0-alpha.2-x86_64.flatpak** and open it with GNOME Software. It uses the same application ID and GNOME 49 runtime as alpha.1. No automatic Leaf update feed. See INSTALL.md.

**Known limitations:** primarily Spanish interface and experimental file fidelity. Unequal column widths round-trip in DOCX; the pinned WordCraft ODT exporter writes equal widths (count, spacing and separator remain). The popover explains this limitation. Test copies and compare saved documents in another editor.

Native release tests and installed-Flatpak checks run before publication, including upgrading from alpha.1. Manual GNOME OS and broader real-document testing remain necessary. Development, tests, packaging, documentation and artwork have used substantial AI assistance.

[Testing guide](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TESTING.md) · [Report a problem](https://github.com/frandavid100/leaf/issues/new/choose)
