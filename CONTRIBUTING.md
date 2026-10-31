# Contributing to Leaf

Bug reports and testing with real documents are especially useful at this stage. English and Spanish are welcome. Use the [testing guide](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TESTING.md) and [issue forms](https://github.com/frandavid100/leaf/issues/new/choose).

For code, start from `wordcraft-saving`, read AGENTS.md, PROJECT_STATUS.md and UPDATING_WORDCRAFT.md, and keep unrelated changes separate. Do not modify the pinned WordCraft engine to simplify frontend work. Target pull requests at `wordcraft-saving` until the maintainer changes the development branch.

Run `cargo test --locked` and build the release GUI for functional changes. Interaction regressions should exercise the actual GTK widget path. Describe what was tested, the source revision and any remaining limits. Documentation-only changes do not need a new executable build.

Avoid uploading personal or confidential documents. A minimal file with invented text is preferable. AI-assisted contributions should describe the affected parts and validation performed; a passing test is not a substitute for understanding a change.

There is no promised response time. The maintainer should prioritize lost/corrupted content, crashes and incorrect saving before visual refinements. Please keep reports focused on one reproducible problem.
