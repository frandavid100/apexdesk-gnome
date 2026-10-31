# First public alpha: release gate

Proposed version: `v0.1.0-alpha.1`. This is a preparation checklist, not evidence that a release exists. The repository currently has no public release or tested Flatpak.

## Completed preparation

- Leaf name and editable application/symbolic icon sources are in the development branch.
- Public README, real development screenshots with provenance, testing guide, contribution guide and issue forms are prepared.
- Draft release notes and Reddit announcement are stored in this repository.
- Existing executable baseline: `c744dac3f553b0304e631dd80dde7fbe3669e1cd`, successful CI 41144295677. This is the old development binary, not a packaged Leaf alpha.

## Required before inviting general testers

- [ ] Integrate Leaf branding into the application, version output and installed launcher; assign and retain a stable application ID owned by this project.
- [ ] Provide and validate desktop/AppStream metadata, icon installation, DOCX/ODT associations and file opening from Files.
- [ ] Obtain a human-authored packaging manifest if the project intends to preserve a Flathub submission path under its current AI policy. No Flatpak manifest has been created by this preparation work.
- [ ] Build the Flatpak from a pinned source revision, with an identified runtime and architecture; verify portal-based open/save and recovery storage inside the sandbox.
- [ ] Install on a clean GNOME OS system, start via launcher, open from Files, edit/save/reopen both formats, export PDF, test recovery and install a subsequent build over the first without losing preferences.
- [ ] Record the exact installed package, architecture, runtime, commit and manual results. Do not infer this from native-binary CI.
- [ ] Run relevant tests and the release build on the final tagged revision; inspect dependencies/licenses and record known limitations.
- [ ] Publish a GitHub prerelease with the `.flatpak`, checksums, exact source tag, runtime requirements, installation/update instructions tested against that package, and completed release notes.
- [ ] Refresh screenshots from the packaged Leaf build and replace pre-branding development captures.
- [ ] Ensure the repository's default landing page and issue forms direct visitors to the current source and release.
- [ ] Check the destination community's current posting rules immediately before posting the announcement; replace draft instructions with the verified release permalink.

## Release installation instructions

Write these after building and testing the actual package. Do not guess a Flatpak ID, filename, runtime remote, CPU architecture or working update command. A standalone bundle is not an automatic update repository. State how testers obtain the next GitHub release and whether it installs over the previous package.

## Flathub is a separate later step

A GitHub alpha is not a Flathub submission. Review the official requirements at the time of submission. The policy source checked on 31 October 2026 requires disclosure of AI-generated material, prohibits AI-generated/assisted Flathub manifests and agent-generated submission interactions, and evaluates acceptance case by case. This project needs human packaging/maintenance participation; acceptance is not guaranteed.

Sources: [requirements](https://github.com/flathub-infra/documentation/blob/main/docs/02-for-app-authors/02-requirements.md), [submission process](https://github.com/flathub-infra/documentation/blob/main/docs/02-for-app-authors/05-submission.md).
