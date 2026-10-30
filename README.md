# WordCraft GNOME — visor de documentos

Interfaz **GTK4 + libadwaita**, con motores de [WordCraft](https://github.com/storytold/wordcraft)
independientes de egui. Código de desarrollo experimental.

## Ramas

- `main`: prototipo anterior, basado en ApexDesk; permanece intacto.
- `wordcraft-migration`: primera integración de WordCraft, visor de una sola página.
- **`wordcraft-viewer`**: visor con páginas continuas, zoom, miniaturas y cabecera organizada como GNOME Papers.

## Funciones de esta versión

- Abrir documentos **DOCX y ODT** en modo solo lectura.
- Desplazamiento vertical continuo entre páginas.
- **Zoom** del 50 % al 200 % con botones flotantes, menú de opciones, `Ctrl++`, `Ctrl+-` o `Ctrl+rueda`.
- **Ajustar al ancho** con el botón correspondiente o `Ctrl+0`.
- Panel lateral con **cabecera independiente** y miniaturas completas, que se puede mostrar y ocultar (F9).
- Pulsar en una miniatura para saltar a esa página.
- Navegación por páginas mediante los botones de cabecera.
- Renderizado bajo demanda de las páginas y miniaturas próximas a la zona visible para contener el uso de memoria.
- Documento de ejemplo de WordCraft, para probarlo sin abrir archivos.

El zoom cambia el tamaño de representación, **no el formato interno del documento**.
La paginación, tablas y estilos proceden de WordCraft, no de `GtkTextView`.

**Limitaciones:** no permite escribir, seleccionar texto o guardar documentos.
Se necesitarán pruebas adicionales con archivos grandes, escalado HiDPI,
rotaciones y compatibilidad de formatos. El lanzamiento automatizado bajo Xvfb
no sustituye las pruebas manuales en GNOME OS.

## Compilación y pruebas

Requiere Rust 1.95+, GTK4 4.10+ y libadwaita 1.4+ de desarrollo.

```sh
cargo test
cargo run --features gui --bin wordcraft-gnome -- --sample
```

Para fotografiar la interfaz con las miniaturas abiertas:

```sh
cargo run --features gui --bin wordcraft-gnome -- --sample --sidebar
```

El [workflow CI](.github/workflows/wordcraft-gnome.yml) ejecuta pruebas
del motor y de la geometría del visor, comprueba que no haya egui/eframe,
compila la aplicación y la arranca bajo Xvfb. Publica una captura y un
binario Linux de desarrollo como artefacto. Para reducir el tamaño del ZIP se
retiran los símbolos de depuración del binario distribuido; el código conserva
las pruebas y la información necesaria en el repositorio.

Los motores WordCraft están fijados a la revisión
[`165d5725`](https://github.com/storytold/wordcraft/commit/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc)
y se distribuyen bajo MIT OR Apache-2.0. Nuestro código está bajo GPL-3.0-or-later.
