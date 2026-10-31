# First public alpha: release gate

Published GitHub prerelease: [v0.1.0-alpha.1](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.1). The x86_64 bundle passed automated installed-Flatpak checks. Manual GNOME OS and broader file testing remain pending.

## Completed preparation

- Leaf name and editable application/symbolic icon sources are in the development branch.
- Public README, real development screenshots with provenance, testing guide, contribution guide and issue forms are prepared.
- Draft release notes and Reddit announcement are stored in this repository.
- Existing executable baseline: `c744dac3f553b0304e631dd80dde7fbe3669e1cd`, successful CI 41144295677. This is the old development binary, not a packaged Leaf alpha.

## Required before inviting general testers

- [x] Integrate Leaf branding into the application, version output and installed launcher; assign and retain a stable application ID owned by this project.
- [x] Provide and validate desktop/AppStream metadata, icon installation, DOCX/ODT associations and file opening from Files.
- [ ] Obtain a human-authored packaging manifest if the project intends to preserve a Flathub submission path under its current AI policy. No Flatpak manifest has been created by this preparation work.
- [x] Build the Flatpak from a pinned source revision, with an identified runtime and architecture; verify portal-based open/save and recovery storage inside the sandbox.
- [ ] Install on a clean GNOME OS system, start via launcher, open from Files, edit/save/reopen both formats, export PDF, test recovery and install a subsequent build over the first without losing preferences.
- [ ] Record the exact installed package, architecture, runtime, commit and manual results. Do not infer this from native-binary CI.
- [ ] Run relevant tests and the release build on the final tagged revision; inspect dependencies/licenses and record known limitations.
- [x] Publish a GitHub prerelease with the `.flatpak`, checksums, exact source tag, runtime requirements, installation/update instructions tested against that package, and completed release notes.
- [ ] Refresh screenshots from the packaged Leaf build and replace pre-branding development captures.
- [x] Ensure the repository's default landing page and issue forms direct visitors to the current source and release.
- [ ] Check the destination community's current posting rules immediately before posting the announcement; replace draft instructions with the verified release permalink.

## Release installation instructions

Write these after building and testing the actual package. Do not guess a Flatpak ID, filename, runtime remote, CPU architecture or working update command. A standalone bundle is not an automatic update repository. State how testers obtain the next GitHub release and whether it installs over the previous package.

## Flathub is a separate later step

A GitHub alpha is not a Flathub submission. Review the official requirements at the time of submission. The policy source checked on 31 October 2026 requires disclosure of AI-generated material, prohibits AI-generated/assisted Flathub manifests and agent-generated submission interactions, and evaluates acceptance case by case. This project needs human packaging/maintenance participation; acceptance is not guaranteed.

Sources: [requirements](https://github.com/flathub-infra/documentation/blob/main/docs/02-for-app-authors/02-requirements.md), [submission process](https://github.com/flathub-infra/documentation/blob/main/docs/02-for-app-authors/05-submission.md).


## Actual first-alpha evidence

## Primera prerelease Flatpak publicada — 31 de octubre de 2026

Publicación autorizada por el usuario: [Leaf v0.1.0-alpha.1](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.1).
Asset: `Leaf-0.1.0-alpha.1-x86_64.flatpak`, **10.062.992 bytes**, SHA-256
`7f7bc26489e73838caf5bcdfdbc63d80254b4975b41b46fd9cb7b498501c9366`.
ID `io.github.frandavid100.leaf`, rama Flatpak `alpha`, runtime `org.gnome.Platform/x86_64/49`.
Incluye nombre/iconos Leaf, lanzador, MIME DOCX/ODT, argumentos de ruta/URI y metadatos AppStream.
Sin permiso de red ni acceso general home/host; usa portales. Preferencias anteriores del ZIP no se importan automáticamente.

Código nativo compilado y probado en `497cd6c5aad7a71f4b580e933f4c3e505b9126c5`: job `test` **124494863399**, run 41186663675. Ese job pasó; el flujo completo falló inicialmente en la fase de empaquetado por ausencia del lector SVG del host. La corrección solo del entorno y de pruebas de reinstalación se validó después sin repetir pruebas ya superadas.
Tag y fuentes de la release: `07d5ff9a6f7b1dd0ff0a0fa399e2b258a254769a`.
[CI final de empaquetado](https://github.com/frandavid100/leaf/actions/runs/41188080151) completada correctamente; job **124498983142**. Una comparación automática exige que src, Cargo.toml, Cargo.lock y tests sean idénticos al código nativo validado antes de reutilizar su artefacto (13037715609).

Pasaron validación desktop/AppStream, instalación del bundle, arranque mediante gtk-launch, pruebas GTK DOCUMENT_CHROME_OK y STYLES_POPOVER_OK dentro del sandbox, apertura por portal de un DOCX externo con nombre/contenido comprobados, escritura en el estado persistente del runtime y su fallback, y desinstalación/reinstalación sin borrar datos conservando preferencias y ambos archivos de estado. Marcadores FLATPAK_INSTALLED_LAUNCHER_OK, FLATPAK_PORTAL_FILE_OPEN_OK, FLATPAK_REINSTALL_DATA_OK y FLATPAK_SMOKE_OK. La captura del lanzador instalado se revisó visualmente; arranca en blanco. La prueba de reinstalación no es una prueba de actualización entre dos versiones distintas.

Release con bundle, instrucciones, SHA256SUMS, runtime/commit, revisión fuente, permisos, log y captura. Artefacto final de CI 13038065980, SHA-256 del ZIP `5b42816e1ada2c5be81559a6ebf1e4ad73c0438613c8842f0c45cba3264622dc`.
No se ha publicado en Reddit ni en Flathub, ni se ha creado un manifiesto de Flathub. Es un bundle de GitHub montado con comandos básicos de Flatpak sobre el ejecutable compilado en Ubuntu. Scripts de empaquetado asistidos por IA: declaración actualizada. Para Flathub sigue haciendo falta participación humana y una construcción conforme a sus requisitos.

Pendiente: instalación/prueba manual en GNOME OS del usuario, fidelidad con archivos reales, guardar/reabrir mediante los selectores nativos en el equipo del usuario, actualización entre versiones distintas, captura promocional definitiva y localización. Sin feed de actualizaciones automáticas de Leaf. No presentar pruebas de CI como confirmación manual.
