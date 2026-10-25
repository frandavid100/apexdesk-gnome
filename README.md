# ApexDesk GNOME

Prototipo de un lector ofimático **nativo GNOME**, construido con GTK4,
libadwaita y los motores Rust de ApexDesk.

## Estado

- **Fase 1, superada:** DOCX/ODT se abren, editan y guardan en copias, con
  comprobaciones de integridad y preservación básica del formato.
- **Fase 2, en desarrollo:** ventana GNOME, selector de archivos, vista de lectura
  de texto, negritas, cursivas, subrayados y títulos.
- **Todavía no** es un editor: no hay modificación de documentos desde la
  interfaz, maquetación paginada, tablas ni representación de imágenes.

## Licencia y procedencia de los motores

El código fuente **actual** de ApexDesk utiliza PolyForm Strict, que no permite
crear derivados sin permiso. Este prototipo utiliza **únicamente** la revisión
previa con licencia GPLv3+ fijada al commit
[`f35e6fa`](https://github.com/apexdesk/apexdesk/commit/f35e6fa10470156a4e55febdf9f8bc2920031b82).
No se incorporan modificaciones posteriores. Nuestra aplicación también
está licenciada bajo GPL-3.0-or-later.

## Instalación desde fuentes en GNOME

Requisitos: Rust estable, desarrollo GTK4 4.10+ y libadwaita 1.4+,
`pkg-config` y Git. Por ejemplo, en Ubuntu 24.04:

```sh
sudo apt install git cargo pkg-config libgtk-4-dev libadwaita-1-dev
cargo run --features gui --bin apexdesk-gnome
```

El botón **Abrir** permite seleccionar un DOCX u ODT. La ventana indica que
es una vista previa de **solo lectura**. No modifica el archivo original.

Para ejecutar la fase 1 desde consola:

```sh
cargo test
cargo run -- smoke target/phase1-smoke
```

Para generar los documentos externos usados en las pruebas:

```sh
python3 -m pip install python-docx==1.2.0 odfpy==1.4.1
python3 scripts/make_fixtures.py
python3 scripts/verify_fixtures.py
cargo test
```

## Verificaciones CI

El flujo [**Fase 2 — GNOME GTK4 libadwaita**](.github/workflows/gnome-preview.yml)
compila el binario y lo adjunta como artefacto de desarrollo para Ubuntu 24.04.
El flujo también verifica que la aplicación arranca en Xvfb y adjunta una captura
de la pantalla inicial. Este binario requiere GTK4/libadwaita instalados y
todavía no es un Flatpak.

La prueba no valida el aspecto visual ni la accesibilidad en una sesión real
de GNOME; esto requerirá una revisión manual posterior.
