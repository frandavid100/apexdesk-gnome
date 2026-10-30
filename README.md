# WordCraft GNOME — interfaz adaptativa libadwaita

Esta rama (`adwaita-layout`) reemplaza la estructura manual del visor por
los contenedores recomendados por las **GNOME Human Interface Guidelines**.

## Componentes

- `AdwApplicationWindow`: ventana.
- `AdwOverlaySplitView`: panel lateral y documento, con ancho proporcional
  administrado por libadwaita. El panel se superpone en ventanas pequeñas.
- `AdwToolbarView` y `AdwHeaderBar`: cabeceras y barras integradas con GNOME.
- `AdwBreakpoint`: cambio de disposición por debajo de **760sp**, incluidos
  ajustes de accesibilidad para texto grande.
- `GtkOverlay` con estilo `.osd`: controles de zoom sobre el lienzo.
- El renderizado, paginación y formatos siguen procediendo de WordCraft.

El menú de la **aplicación** está en la cabecera lateral; el menú del
**documento** está en la cabecera principal.

## Funciones

Permite abrir DOCX/ODT, leerlos, desplazarse de forma continua, ajustar zoom,
consultar miniaturas y saltar a páginas. No permite editar ni guardar todavía.

Los atajos principales: `Ctrl+O` abrir, `F9` panel, `Ctrl++` ampliar,
`Ctrl+-` reducir, `Ctrl+0` ajustar ancho.

## Pruebas

Requiere GTK4 >= 4.10, libadwaita >= 1.4 y Rust estable.

```sh
cargo test
cargo run --features gui --bin wordcraft-gnome -- --sample --sidebar
```

Para pruebas visuales sin modificar preferencias del usuario:

```sh
./wordcraft-gnome --sample --narrow --sidebar
./wordcraft-gnome --sample --narrow
./wordcraft-gnome --sample --dark --sidebar
./wordcraft-gnome --version
```

El flujo de GitHub Actions genera capturas de las cinco disposiciones,
compila los motores sin egui y crea un ejecutable Linux reducido (sin símbolos
de depuración). Las capturas automatizadas no sustituyen las pruebas
interactivas con teclado, lector de pantalla o Wayland real.

## Ramas y licencias

- `main`: prototipo previo basado en ApexDesk.
- `wordcraft-migration`: primera integración de WordCraft.
- `wordcraft-viewer`: visor anterior con contenedores manuales.
- `adwaita-layout`: refactorización HIG/libadwaita (esta rama).

Las dependencias de WordCraft proceden de
[storytold/wordcraft, revisión 165d5725](https://github.com/storytold/wordcraft/commit/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc),
con licencia MIT OR Apache-2.0; el código del proyecto es GPL-3.0-or-later.

Documentación de referencia:
[GNOME HIG](https://developer.gnome.org/hig/),
[OverlaySplitView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.OverlaySplitView.html),
[ToolbarView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ToolbarView.html).
