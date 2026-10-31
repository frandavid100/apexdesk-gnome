Leaf 0.1.0-alpha.3 fixes document and thumbnail raster resolution on HiDPI screens.

Page rendering now accounts for GTK display density as well as document zoom. A logical-size paintable keeps page geometry, scrolling and hit testing independent of texture resolution. Cached pages and thumbnails are invalidated when the widget scale factor changes. The old 2 pixels-per-point cap is removed; the existing 12-million-pixel page allocation guard remains.

Automated checks cover 100% and 200% display scaling, 100% and 200% document zoom, texture dimensions, logical page geometry and unchanged document revision. Real multi-monitor Wayland/fractional-scale testing remains pending. This fixes a concrete integration defect; confirmation from the original Reddit reporter is still needed.

Download Leaf-0.1.0-alpha.3-x86_64.flatpak and install over alpha.2. GNOME 49 runtime required. No automatic Leaf update feed. AI-assisted development. Experimental DOCX/ODT fidelity; test copies. Unequal column widths remain DOCX-only on round-trip.
