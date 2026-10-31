Leaf 0.1.0-alpha.6 fixes retained GTK document and window state after closing tabs/windows.

Window owners are explicit and signal callbacks borrow their state weakly. Closed tabs stop their autosave timer; deferred UI callbacks no longer keep them alive. The same CSS provider is no longer registered on every tab. Obsolete rich clipboard data is released when clipboard ownership changes. Document editing and the pinned engine are unchanged.

A repeatable ownership/memory audit checks real DOCX opening, sidebar search, menus, tab/window closure and object release. A second regression covers properties/columns dialogs and clipboard replacement. Native and installed sandbox tests must pass before publication. The report distinguishes object leaks from allocator/font caches; these measurements are not a general speed benchmark.

Install Leaf-0.1.0-alpha.6-x86_64.flatpak over alpha.5. GNOME 49 runtime; no automatic update feed. AI-assisted development. Keep document copies. The existing Orca/canvas and GTK print-backend limitations remain documented.
