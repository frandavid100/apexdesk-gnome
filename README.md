# WordCraft GNOME — Fase 3B (edición en memoria)

Rama experimental `wordcraft-editing` desarrollada sobre la fase 3A
(`wordcraft-selection`) y los contenedores nativos libadwaita.

## Lo que funciona

- Clic, selección, arrastre y copia de la fase 3A.
- **Escritura** directamente en el documento mediante el contexto de entrada
  de GTK. El motor de edición es WordCraft, no un GtkTextView.
- **Intro**: párrafo nuevo; **Shift+Intro**: salto de línea.
- **Retroceso/Supr**: borrar caracteres o reemplazar la selección.
- **Ctrl+V**: pegar texto plano; **Ctrl+X**: cortar texto.
- **Ctrl+Z**, **Ctrl+Shift+Z** y **Ctrl+Y**: deshacer y rehacer.
- Repaginación y recálculo del cursor tras los cambios, incluidas las
  modificaciones que hacen crecer o reducir el número de páginas.
- Miniaturas, navegación y zoom siguen utilizando WordCraft.

## Limitación importante: aún no se puede guardar

Este prototipo **no sobrescribe ni guarda en disco** los archivos DOCX/ODT.
Los cambios existen únicamente durante la sesión. Verás un indicador de
cambios sin guardar y se pedirá confirmación antes de cerrar o abrir otro
documento. Si necesitas conservar el resultado provisional,
**Ctrl+A → Ctrl+C** permite copiar el texto a otra aplicación.

La fase 3D incorporará el guardado y recuperación. Hasta entonces usa
**copias de documentos**, nunca archivos que contengan datos irremplazables.

La composición del teclado se conecta al `GtkIMMulticontext`, aunque aún
quedan por probar el subrayado visual del texto en preedición IME,
accesibilidad y ediciones grandes.

## Ejecutar

Extrae el artefacto Linux de GitHub en una carpeta nueva y ejecuta:

```sh
chmod +x wordcraft-gnome
./wordcraft-gnome --version
./wordcraft-gnome --sample --sidebar
```

La identificación debe incluir «fase 3B».

La interfaz se compila y prueba automáticamente con:

```sh
cargo test
cargo build --features gui --bin wordcraft-gnome
```

El workflow genera una captura adicional `vista-edicion.png` tras ejecutar
una mutación real del documento en WordCraft.

## Código y licencias

El motor WordCraft está fijado a
[`165d5725`](https://github.com/storytold/wordcraft/commit/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc)
(MIT OR Apache-2.0). Este proyecto es GPL-3.0-or-later.
