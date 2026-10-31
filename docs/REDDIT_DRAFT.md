# Reddit announcement — draft for David

Status: not posted. Use only after the public prerelease and its installable asset exist, and after checking r/gnome's current rules. Add the actual release permalink before posting. Do not describe the current source snapshot as an installed Flatpak release.

## Title

Leaf: a native GTK4 document editor — looking for alpha testers

## Post

I've been working on Leaf, an experimental document editor built with GTK4 and libadwaita. It uses WordCraft for document layout, rendering and editing, with a separate native frontend designed for GNOME.

It can open, edit and save DOCX and ODT files, export PDFs, and work with styles, tables and inline images. It also has tabs, sidebar search and replacement, and an adaptive interface.

I'm looking for people willing to test copies of real documents, especially files created in Word or LibreOffice. The most useful test is to open a copy, make a few changes, save it and reopen it in both Leaf and the original editor. I'd like to know what survives correctly and what breaks.

This is an early alpha, not a replacement for a mature office suite. Complex formatting needs more testing, and the interface is currently primarily Spanish. Development has used substantial AI assistance; the repository explains its scope and the checks performed.

Source: https://github.com/frandavid100/leaf/tree/wordcraft-saving
Testing guide: https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TESTING.md
Bug reports: https://github.com/frandavid100/leaf/issues/new/choose

Before posting: insert the verified release download permalink here and attach screenshots from that packaged build.

Feedback on keyboard navigation, accessibility and the GNOME interaction patterns would also be useful. Please use non-confidential files when reporting problems.
