Leaf 0.1.0-alpha.5 adds native printing and improves keyboard navigation.

Print from the document menu or Ctrl+P. Leaf prepares a vector PDF snapshot of the current document, including unsaved changes, then uses GTK's native print dialog and the desktop print portal. Printing does not change your editable file, dirty state or undo history. Temporary files are private and removed after the operation. Cancellation and errors are handled without saving the document.

F6 / Shift+F6 move between the document, sidebar and formatting controls; F10 opens the main menu. Search fields, image/page dimensions and icon controls have clearer accessible names. About now shows the same version as the executable. This improves control accessibility; full screen-reader access to the custom document canvas is still unfinished.

Install Leaf-0.1.0-alpha.5-x86_64.flatpak over alpha.4. GNOME 49 runtime; source builds now require GTK 4.14+. No automatic Leaf update feed. AI-assisted development. Use document copies. ODT compatibility fixes from alpha.4 remain included. Physical printer testing is still needed.
