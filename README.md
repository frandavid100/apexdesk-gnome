# WordCraft GNOME — Fase 3A: cursor y selección

Esta rama (`wordcraft-selection`) incorpora la primera interacción real
con texto sobre el visor GTK4/libadwaita.

**Qué puedes probar**

- Clic sobre una palabra: posiciona el cursor según la maquetación de WordCraft.
- Doble clic: selecciona una palabra. Triple clic: selecciona el párrafo.
- Arrastrar sobre las páginas: selecciona texto, incluso entre páginas.
- Flechas, Inicio, Fin y combinaciones con **Shift**: mover o extender la selección.
- `Ctrl+Izquierda/Derecha`: mover por palabras; `Ctrl+Inicio/Fin`: principio/final.
- `Ctrl+A`: seleccionar el documento; `Ctrl+C`: copiar texto al portapapeles.
- Zoom, miniaturas, desplazamiento y la interfaz adaptativa de las fases anteriores.

La posición y el resaltado proceden de las API `DocLayout::hit`,
`DocLayout::caret_on` y `DocLayout::selection_rects` de WordCraft; el estado
se modifica con sus comandos `caret.set`, `select.range` y `edit.copy`.
No usamos `egui`.

**Limitaciones actuales:** el documento todavía no admite escritura, cambios
de formato ni guardado. La selección mediante arrastre fuera de la ventana
no dispone todavía de desplazamiento automático. `Ctrl+C` copia texto
plano, no HTML ni formato enriquecido. La compatibilidad de selección
en tablas complejas y documentos con varias orientaciones requiere pruebas.

## Ejecución

En GNOME con GTK4 >= 4.10 y libadwaita >= 1.4:

```sh
./wordcraft-gnome --version
./wordcraft-gnome --sample --sidebar
```

Desde fuentes (Rust estable):

```sh
cargo test
cargo run --features gui --bin wordcraft-gnome -- --sample --sidebar
```

## Calidad y pruebas

GitHub Actions compila el editor, verifica las pruebas del motor
(incluido un test de selección entre páginas), comprueba ausencia de
`egui` y genera capturas de pantalla con libadwaita. Eso no sustituye
una prueba real de clic, arrastre, accesibilidad y portapapeles en GNOME OS.

El código está disponible en GitHub bajo GPL-3.0-or-later y reutiliza
WordCraft [revisión 165d5725](https://github.com/storytold/wordcraft/commit/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc)
(MIT OR Apache-2.0). La rama anterior `adwaita-layout` permanece intacta.
