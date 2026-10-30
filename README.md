# WordCraft GNOME · Fase 3C: formato

Esta rama `wordcraft-formatting` añade herramientas de formato a la
interfaz GTK4/libadwaita, reutilizando el motor WordCraft sin egui.

## Incluido

- Negrita **Ctrl+B**, cursiva **Ctrl+I** y subrayado **Ctrl+U**.
- Estilos de párrafo: Normal y Títulos 1, 2 y 3.
- Alineación: izquierda, centrada, derecha y justificada.
- Tamaños de fuente: 10, 11, 12, 14, 16, 18, 24 y 36 puntos.
- Atajos de párrafo: Ctrl+L/E/R/J, Ctrl+Shift+N,
  Ctrl+Alt+1/2/3.
- Barra de formato integrada en `AdwToolbarView`, desplazable
  horizontalmente en ventanas estrechas.
- Cambio de formato aplicado a selecciones y texto futuro en el cursor.
- Paginación, miniaturas, zoom, escritura, copia, pegado, borrado y
  deshacer/rehacer de las fases anteriores.

El formato se realiza con comandos del motor (`format.bold`,
`format.italic`, `format.underline`, `format.size`,
`para.heading1` y `para.alignCenter`, entre otros).

## Importante

**Todavía no se guardan cambios en DOCX/ODT**. Los cambios en contenido y
formato permanecen en la sesión. El programa avisa antes de cerrar o
reemplazar documentos modificados. En esta etapa usa solo documentos de
prueba y evita editar originales que necesites conservar.

Esta versión añade funcionalidades sobre la fase 3B, que aún está
pendiente de verificación interactiva en GNOME OS. Las pruebas de CI
no garantizan por sí solas el funcionamiento de todos los teclados,
tipos de documento o entradas IME.

## Ejecutar

```sh
chmod +x wordcraft-gnome
./wordcraft-gnome --version
./wordcraft-gnome --sample --sidebar
```

La versión debe indicar **fase 3C**. Se puede comparar con la rama
`wordcraft-editing` (fase 3B) y la rama `wordcraft-selection` (fase 3A).

## Pruebas de desarrollo

```sh
cargo test
cargo build --features gui --bin wordcraft-gnome
cargo run --features gui --bin wordcraft-gnome -- --sample --sidebar --format-demo
```

El flujo automático produce `vista-formato.png` y verifica la
compilación, el motor y el arranque en Xvfb.

WordCraft revisión
[`165d5725`](https://github.com/storytold/wordcraft/commit/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc)
(MIT OR Apache-2.0). Código de esta aplicación: GPL-3.0-or-later.
