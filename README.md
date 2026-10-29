# WordCraft GNOME — rama experimental

Una interfaz de documentos GTK4/libadwaita para GNOME, utilizando
**WordCraft**, no ApexDesk.

**Estado:** primera migración, limitada a lectura y navegación por páginas.
No hay edición, guardado desde la UI, selección ni fidelidad de impresión
comprobada. La interfaz usa imágenes de página generadas por WordCraft.

## Motores

Dependencias con revisión fija:
[storytold/wordcraft@165d572](https://github.com/storytold/wordcraft/commit/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc).

- `wordcraft-engine`: comandos, archivos, sesión y paginación.
- `wordcraft-render`: rasterización de páginas.
- `wordcraft-doc`: modelo de documento.

Estos módulos son independientes de egui; solo usamos la representación
RGBA de sus páginas. WordCraft está licenciado MIT OR Apache-2.0.
Este repositorio experimental mantiene GPL-3.0-or-later para el código propio;
conservar los avisos de licencia de las dependencias originales.

## Compilar / probar

Sistema GNOME con GTK4 >= 4.10, libadwaita >= 1.4, Rust >= 1.95.

```sh
cargo test
cargo run -- --png pagina.png
cargo run --features gui --bin wordcraft-gnome -- --sample
```

`--sample` abre el documento de ejemplo del motor. También hay un botón
**Abrir** para elegir DOCX u ODT. Se puede navegar entre las páginas.
La ventana es **solo lectura**; no modifica los originales.

GitHub Actions valida que no entren dependencias egui, compila Rust,
prueba el render sin interfaz y arranca la ventana en una pantalla virtual.

La versión antigua basada en ApexDesk continúa intacta en la rama `main`
hasta que la migración se valide.
