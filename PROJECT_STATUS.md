# Estado de WordCraft GNOME

Actualizado: 30 de octubre de 2026. Punto de entrada para continuar desde otra conversación.

## Cómo retomar

Repositorio: https://github.com/frandavid100/apexdesk-gnome
Rama de trabajo: **`wordcraft-saving`**. No asumir que la rama predeterminada contiene estos avances.

Leer este archivo, [ROADMAP.md](ROADMAP.md), [AGENTS.md](AGENTS.md) y
[UPDATING_WORDCRAFT.md](UPDATING_WORDCRAFT.md). Comprobar después el código y
los últimos commits: este documento es una instantánea, no sustituye al repositorio.
El usuario puede dar estos enlaces en un chat nuevo; no depender del historial del chat.

## Objetivo y decisiones acordadas

Cliente nativo GTK4/libadwaita para GNOME OS. WordCraft aporta el documento,
maquetación, renderizado, comandos, historial e importación/exportación.
Conservar el motor sin modificaciones y fijar sus revisiones para facilitar actualizaciones.

- Documentos en pestañas; `+` y Ctrl+N crean una pestaña. También hay ventanas independientes.
- Abrir y Nueva ventana pertenecen al menú general del panel lateral.
- Panel lateral estrecho y abierto por defecto. Fondo del visor del color de la ventana; hojas blancas.
- Barra de edición abajo, como Apostrophe. Deshacer y Rehacer a la izquierda.
- Formato agrupado en Texto, Estilo y Párrafo, con opciones ampliadas en paneles nativos.
- Zoom a la derecha: +, − y un tercer botón con 100 % y Ajustar al ancho.
- Sin botones Página anterior/siguiente; navegación mediante scroll y miniaturas.
- Buscar/reemplazar en el menú de tres puntos del documento.
- Preferencias de autoguardado: desactivado, recuperación o propio archivo.
- Mantener la interfaz GNOME compacta; la captura de Word es referencia de funciones.

## Implementado

| Área | Estado |
| --- | --- |
| Visualización | Páginas, miniaturas bajo demanda, zoom, ajuste al ancho, tema oscuro y ventana estrecha. |
| Fase 3A | Cursor, selección con ratón/teclado, doble/triple clic y copiar. |
| Fase 3B | Escritura Unicode, borrar, párrafos, saltos, portapapeles, deshacer y rehacer. |
| Fase 3C | Fuente, tamaño, negrita, cursiva, subrayados, colores, resaltado, tachados, sub/superíndice, espaciado y efectos admitidos por WordCraft. |
| Párrafos | Alineación/dirección, sangrías, interlineado, espaciado, listas multinivel, ordenación, bordes, sombreado, tabulaciones y marcas ¶. |
| Estilos | Aplicar estilos de párrafo/carácter; crear desde selección, actualizar, renombrar y eliminar personalizados. |
| Fase 3D | Abrir y guardar DOCX/ODT, guardar como, cambios pendientes y confirmaciones de cierre. |
| Documentos | Pestañas y ventanas, nuevo documento y apertura conservando documentos existentes. |
| Autoguardado | Preferencia persistente; recuperación cada 10 s por defecto, propio archivo cada 30 s; recuperación tras cierre inesperado. |
| Búsqueda | Buscar y reemplazar una/todas las coincidencias, con deshacer. |
| Optimización | Release con ThinLTO, píxeles premultiplicados directos a GTK, búsqueda binaria de páginas y miniaturas agrupadas a 120 ms. |

Las funciones de formato son las expuestas por el motor, sin equivalencia completa con Microsoft Word.
Las tablas de documentos existentes pueden representarse; todavía no hay una interfaz dedicada
para insertar y editar su estructura. Tampoco hay una interfaz dedicada para insertar imágenes.

## Última entrega y pruebas

Código del último ejecutable validado: `c0493a3fe213a32ecad1b084db667c8fbe9cbb61`.
Los commits posteriores a esa revisión pueden ser exclusivamente documentación: comprobar el historial.

[CI completada correctamente](https://github.com/frandavid100/apexdesk-gnome/actions/runs/41057985488).
Artefacto de esa ejecución: `wordcraft-gnome-fase3d-linux`, ID `12976989747`.
El usuario recibió `wordcraft-gnome-fase3d.zip` con el ejecutable release y un LEEME.
Los artefactos de Actions caducan: si no están disponibles, recompilar esa revisión.

Ejecutable: 33.554.600 bytes frente a 92.516.752 del desarrollo anterior.
ZIP entregado: 15.642.305 bytes. Benchmark sobre el mismo runner y adaptador:
20 imágenes de páginas/miniaturas, desarrollo 1463,016 ms, release 21,464 ms.
Coinciden todos los píxeles comprobados; el cálculo de su checksum queda fuera del tiempo.
Es una muestra concreta, no una aceleración garantizada de toda la aplicación.
También se comparó una región de la página en las capturas anterior/actual sin diferencias.

Pasaron los tests de integración con el motor, de los adaptadores y de búsqueda de páginas frente al recorrido anterior;
la CI compiló y probó el ejecutable release en ventanas, pestañas y paneles nativos.

**Pendiente de confirmación del usuario:** formato avanzado y última optimización en su
GNOME OS Preview 51. Había confirmado que las entregas anteriores funcionaban.
No interpretar el éxito de la CI como una prueba manual del usuario.

## Límites y comprobación pendiente

Probar documentos reales, especialmente guardar/reabrir DOCX y ODT con formato complejo.
La fidelidad depende de los importadores/exportadores de WordCraft. La prueba ampliada
de formato avanzado cubre DOCX; no extrapolar esa cobertura íntegra a ODT.
Probar texto con tildes/IME, selecciones entre páginas, deshacer, recuperación y varias pestañas.
Los estilos recién creados aparecen en la lista al volver a abrir el panel.

## Mapa del código

- `src/bin/gnome.rs`: widgets GTK, selección visual, entrada y vista de páginas.
- `src/gnome_host.rs`: pestañas, ventanas, preferencias, búsqueda y recuperación.
- `src/gnome_format_panels.rs`: paneles de formato avanzado y estilos.
- `src/lib.rs`: apertura y adaptador de renderizado; conserva API de RGBA recto y nativo premultiplicado.
- `src/{selection,editing,formatting,saving,preferences,viewer_layout}.rs`: adaptadores y geometría.
- `tests/`: regresiones funcionales; `examples/performance.rs`: comparación de renderizado.
- `.github/workflows/wordcraft-gnome.yml`: pruebas, compilación release y capturas.

WordCraft fijado a `165d5725dc25b4a7887d8025c3b1fe506ccc3dcc`;
patch de krilla a `92d392cff132eaf53e13afff77f38b5c11b73aa0`.
Consultar Cargo.toml/Cargo.lock por si una revisión posterior los actualiza.

## Mantener este registro

Después de cada entrega, actualizar estado, validación, revisión, limitaciones y siguiente
paso. Separar propuestas, trabajo implementado y confirmaciones reales del usuario.
Registrar en ROADMAP los cambios de prioridad para que el plan sobreviva al chat.
