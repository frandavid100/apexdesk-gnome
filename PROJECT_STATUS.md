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

- Documentos en pestañas; Pestaña nueva en el menú principal y Ctrl+N/Ctrl+T crean una pestaña vacía. También hay ventanas independientes. Sin botones de nueva pestaña ni guardar en la cabecera.
- Abrir y Nueva ventana pertenecen al menú general del panel lateral.
- Panel lateral estrecho y abierto por defecto desde su construcción, sin apertura diferida. Miniaturas con esquinas de 6 px y borde fino como Papers; margen uniforme de 12 px alrededor de imagen y número, ambos dentro del botón seleccionado; filas a altura natural. Selector inferior en AdwToolbarView, con altura compartida con la barra de edición y sombras nativas al desplazar contenido. Fondo del visor del color de la ventana; hojas blancas.
- Cabecera del documento de una línea: nombre y ● a su izquierda cuando hay cambios pendientes, sin subtítulo ni contador de páginas.
- Barra adaptable: en ancho, Deshacer/Rehacer arriba a la izquierda y Estilo/Párrafo/Texto arriba a la derecha, solo iconos con nombres accesibles. En estrecho los mismos cinco controles pasan abajo; no hay duplicados.
- Los tres desplegables de barra (Estilo, Párrafo, Texto) y los dos paneles contextuales (Tabla e Imagen) comparten un grupo por pestaña: solo uno puede estar abierto y la chincheta se conserva al pasar de uno a otro. La modalidad GTK es siempre no modal; fijar/desfijar solo cambia la política de cierre, sin ocultar, recrear ni volver a mostrar la superficie. Sin fijar se cierra al aplicar o pulsar fuera. Cabecera única con nombre, vuelta atrás contextual y chincheta. Controles divididos de Párrafo con AdwSplitButton plano: fondo compartido al pasar el puntero y resaltado independiente de cuerpo/chevrón, como Nautilus; subopciones en la misma subpágina, sin otro popup.
- Orden de formato: Estilo → Párrafo → Texto, en la cabecera o en la barra inferior según el ancho útil del documento. Tabla e Imagen se insertan desde Añadir en el menú del documento y se editan con clic derecho. Estilos y Párrafo se gestionan en desplegables con subpáginas internas; Texto conserva su panel ampliado.
- Zoom a la derecha: +, − y un tercer botón con 100 % y Ajustar al ancho.
- Sin botones Página anterior/siguiente; navegación mediante scroll y miniaturas.
- Selector inferior del panel lateral con iconos Páginas/Títulos, inspirado en Papers. Títulos jerárquicos con secciones contraíbles, numeración y navegación al texto mediante WordCraft.
- Lupa en la cabecera lateral: alterna navegación/búsqueda mediante fundido de 180 ms. Resultados con contexto y página; opciones de mayúsculas y reemplazo en los tres puntos del campo. Reemplazo individual desde cada resultado y Reemplazar todo a la derecha del segundo campo, en un solo paso de deshacer. Sin contador de coincidencias; botón de opciones plano. Ctrl+F/Ctrl+H abren ese mismo panel.
- Menú principal GtkPopoverMenu/GMenu como Nautilus: ventana/pestaña nuevas, vista de todas las pestañas, abrir, recuperación, preferencias, atajos, ayuda y acerca de. Menú de documento nativo separado: Deshacer/Rehacer y Guardar/Guardar como. Propiedades del documento y Exportar a PDF están también en el menú del documento. Sin acción ni opción de documento de ejemplo; el arranque normal abre un documento vacío. El contenido de prueba solo está disponible para diagnósticos internos con --test-document. Se conservan Ctrl+N (pestaña) y Ctrl+Mayús+N (ventana); Ctrl+T añade una pestaña también.
- Preferencias de autoguardado: desactivado, recuperación o propio archivo.
- Mantener la interfaz GNOME compacta; la captura de Word es referencia de funciones.

## Implementado

| Área | Estado |
| --- | --- |
| Visualización | Una o dos páginas por fila según ancho disponible y zoom, sin reducirlo para forzar la vista doble. Miniaturas bajo demanda, zoom, ajuste al ancho, tema oscuro y ventana estrecha. |
| Fase 3A | Cursor con desplazamiento visual suave de 100 ms, selección con ratón/teclado, doble/triple clic y copiar. |
| Fase 3B | Escritura Unicode, borrar, párrafos, saltos, portapapeles, deshacer y rehacer. |
| Fase 3C | Fuente, tamaño, negrita, cursiva, subrayados, colores, resaltado, tachados, sub/superíndice, espaciado y efectos admitidos por WordCraft. |
| Párrafos | Tres filas de iconos: alineación; listas/sangrías; interlineado/sombreado/bordes. Flechas a subpáginas y Más opciones dentro del mismo desplegable, con vuelta atrás. Se conservan las funciones anteriores. |
| Estilos | Lista completa con búsqueda, aplicar por nombre, gestión en subpáginas con vuelta atrás, muestra real, crear/actualizar/renombrar/eliminar sin ventanas adicionales. |
| Fase 3D | Abrir y guardar DOCX/ODT, guardar como, cambios pendientes y confirmaciones de cierre. |
| Documentos | Pestañas y ventanas, nuevo documento y apertura conservando documentos existentes. |
| Autoguardado | Preferencia persistente; recuperación cada 10 s por defecto, propio archivo cada 30 s; recuperación tras cierre inesperado. |
| Búsqueda | Panel lateral con fundido, coincidencias con contexto/página y resaltado, mayúsculas, reemplazo individual o total con un solo deshacer; sin contador. Estado por pestaña. |
| Tablas | Inserción, filas/columnas, selección de celdas, combinar/dividir, tamaño, alineación, bordes, sombreado y cabeceras; menú contextual y panel de opciones. |
| Imágenes | Inserción en línea, selección con clic, tamaño en mm con proporción bloqueada por defecto, tamaño original, sustitución y eliminación; chincheta compartida. |
| Página | Propiedades del documento: papel, dimensiones, orientación y márgenes en mm; sección actual o todo el documento, conservando las demás propiedades y agrupando el historial. |
| PDF | Exportación independiente desde el menú del documento, con selector nativo y trabajo en segundo plano sobre una copia del documento; conserva destino editable e historial. |
| Optimización | Release con ThinLTO, píxeles premultiplicados directos a GTK, búsqueda binaria de páginas y miniaturas agrupadas a 120 ms. |

Las funciones de formato son las expuestas por el motor, sin equivalencia completa con Microsoft Word.
Las tablas existentes y nuevas pueden editarse con los controles nativos. No hay todavía
tiradores de tamaño con ratón ni fórmulas. Las imágenes nuevas son en línea; no se añaden controles para hacerlas flotantes.

## Última entrega y pruebas

Código del último ejecutable validado: `7cd6debaa1ec55b4f44783966f2b22c56daad8c5`.
Los commits posteriores a esa revisión pueden ser exclusivamente documentación: comprobar el historial.

[CI completada correctamente](https://github.com/frandavid100/apexdesk-gnome/actions/runs/41137708372).
Artefacto de esa ejecución: `wordcraft-gnome-fase3d-linux`, ID `13018325422`.
Ejecutable actual con menús contextuales corregidos: 34.200.664 bytes.
El usuario recibió `wordcraft-gnome-fase3d.zip` con el ejecutable release y un LEEME.
Los artefactos de Actions caducan: si no están disponibles, recompilar esa revisión.

Medidas de la entrega optimizada anterior (antes de las tablas):
Ejecutable: 33.554.600 bytes frente a 92.516.752 del desarrollo anterior.
ZIP entregado: 15.642.305 bytes. Benchmark sobre el mismo runner y adaptador:
20 imágenes de páginas/miniaturas, desarrollo 1463,016 ms, release 21,464 ms.
Coinciden todos los píxeles comprobados; el cálculo de su checksum queda fuera del tiempo.
Es una muestra concreta, no una aceleración garantizada de toda la aplicación.
También se comparó una región de la página en las capturas anterior/actual sin diferencias.

Pasaron los tests de integración con el motor, de los adaptadores y de búsqueda de páginas frente al recorrido anterior;
la CI compiló y probó el ejecutable release en ventanas, pestañas y paneles nativos.
Las pruebas del visor verifican geometría de filas simples/dobles, alturas mixtas y páginas impares; márgenes simétricos de miniaturas, número dentro del botón, barras inferiores alineadas y título de una línea con cambios pendientes. La regresión de foco usa clics reales para abrir un menú, cerrarlo y situar el cursor en una página posterior sin alterar el scroll. Capturas revisadas en una columna y en vista doble al 100 %.
Los controles divididos usan AdwSplitButton plano. Se han revisado capturas en reposo, con puntero sobre el cuerpo y sobre el chevrón; un clic real abre la subpágina (SPLIT_SUBPAGE_OK). La regresión GTK comprueba el tipo nativo y la navegación por ambas mitades. Las miniaturas tienen esquinas de 6 px y borde fino como Papers, con recorte del contenido; capturas revisadas. Pendiente de prueba manual del usuario.
La regresión comprueba además ausencia de señales hide y conservación de la superficie al fijar/desfijar, y cambio exclusivo Estilo/Párrafo conservando la chincheta. Capturas de cabeceras únicas y botones de flecha con fondo nativo revisadas.
La prueba POPOVER_PIN_OK aplica acciones reales de Estilo/Párrafo/Texto/Tabla y comprueba que fijar permite mantenerlos abiertos al aplicar y enfocar el documento; desfijar conserva la subpágina y restaura el cierre. Fijar no cambia la revisión. La prueba espera fotogramas de GTK y exige is_mapped, para detectar el caso de un panel marcado como visible pero sin superficie dibujada. Se revisó además una captura tras aplicar un estilo fijado y varios segundos con el foco en el documento, así como el aspecto plano de la barra y los chevrones.
La prueba DOCUMENT_CHROME_OK verifica arranque vacío sin fixture, menú GtkPopoverMenu y sus cuatro acciones, historial sobre el documento activo y altura compartida de las barras. VIEW_POINTER_OK comprueba además el margen de 12 px y el número incluido en la miniatura sin regresión de foco. Capturas anchas, estrechas y oscuras comprobadas.
Las pruebas de navegación verifican niveles heredados/expresos, selección sin alterar historial, contraer/expandir secciones, conservación de Títulos al cerrar la búsqueda y acciones nativas de pestañas, historial, ayuda y atajos.
Las pruebas de búsqueda verifican reemplazo total literal y vacío, mayúsculas y reversión completa con un solo deshacer, además de texto Unicode, mayúsculas, sustituciones literales y vacías con deshacer, tablas, varias páginas, conservación de selección, rechazo de resultados antiguos e independencia de pestañas. Las pruebas de Párrafo recorren botones, subpáginas y vuelta atrás, aplicando formato sin abrir ventanas adicionales. Se han revisado capturas anchas, estrechas y oscuras.
Las pruebas del cursor verifican movimientos consecutivos, fin de animación con el reloj de GTK, ausencia de cambios de historial, movimiento reducido y arranque del panel sin alternar su visibilidad.
Las pruebas de estilos verifican previsualización sin cambios, gestión completa,
validación de nombres y el recorrido real de botones/subpáginas sin ventanas adicionales.

**Confirmación del usuario:** el desplegable de estilos funciona y su distribución le gusta; solicita quitar la negrita general. Se mantienen ¶/a para tipo con explicaciones; el estilo actual se distingue con fondo de selección en vez de ✓.
**Corrección solicitada por el usuario:** salto a la primera página al cerrar un menú y pulsar el texto. El visor usa un GtkViewport explícito con scroll-to-focus desactivado; desplazar el cursor con teclado sigue usando la geometría del motor.
**Pendiente de confirmación del usuario:** chincheta sin remapeo, sustitución exclusiva con estado compartido, cabecera única y botones de subopciones nativos; barras inferiores alineadas con sombras nativas, miniaturas de 12 px incluyendo número, menú de documento y retirada de botones/ejemplo; selector Páginas/Títulos, títulos contraíbles, menú principal nativo, vista de todas las pestañas y últimos ajustes de búsqueda/reemplazo total y fundido; cabecera simplificada, márgenes de miniaturas, páginas dobles y corrección de foco; búsqueda lateral, rediseño de Párrafo y fondo del estilo activo; movimiento suave del cursor, arranque del panel sin animación, tablas, formato avanzado y última optimización en su GNOME OS Preview 51.
No interpretar el éxito de la CI como una prueba manual del usuario.

## Límites y comprobación pendiente

Probar documentos reales, especialmente guardar/reabrir DOCX y ODT con formato complejo.
La fidelidad depende de los importadores/exportadores de WordCraft. La prueba ampliada
de formato avanzado cubre DOCX; no extrapolar esa cobertura íntegra a ODT.
Probar texto con tildes/IME, selecciones entre páginas, deshacer, recuperación y varias pestañas.
La lista de estilos se actualiza tras crear, renombrar o eliminar. La muestra se renderiza en una sesión aislada y no modifica el documento.
Las pruebas de tablas cubren estructura y texto en DOCX/ODT, sombreado y anchos,
selecciones vacías, combinar/dividir, historial, varias páginas y teclado real.
Probar manualmente tablas importadas complejas y los controles en ventana estrecha.

## Mapa del código

- `src/gnome_document_tools.rs`: Imagen, diálogo de propiedades y selector de exportación PDF.
- `src/pictures.rs`, `src/document_properties.rs`, `src/pdf_export.rs`: adaptadores de imágenes, sección/documento y PDF con estado editable preservado.
- `src/bin/gnome.rs`: widgets GTK, selección visual, entrada y lienzo Fixed de páginas.
- `src/gnome_view_tests.rs`: regresión GTK de foco/clic tras menús, vista doble y márgenes de miniaturas.
- `src/gnome_host.rs`: pestañas, ventanas, preferencias y recuperación.
- `src/navigation.rs`, `src/gnome_navigation.rs`: niveles de esquema y selector/lista de títulos.
- `src/gnome_menus.rs`: menú GMenu con acciones de ventana, vista general de pestañas, ayuda y atajos nativos.
- `src/gnome_format_panels.rs`: paneles de texto y funciones comunes de formato.
- `src/gnome_paragraph.rs`: iconos y subpáginas de Párrafo.
- `src/gnome_search.rs`, `src/search.rs`: búsqueda lateral y adaptador del matcher/selección/reemplazo del motor.
- `src/caret_motion.rs`: interpolación visual aislada, sin modificar selección, documento ni historial.
- `src/gnome_popover_pin.rs`: cabeceras, chincheta, reapertura no modal sin reiniciar subpáginas y chevrón común.
- `src/gnome_styles.rs`: lista, gestión y formularios dentro del desplegable de estilos.
- `src/styles.rs`: comandos de estilos y previsualización aislada.
- `src/gnome_tables.rs`: inserción, menú de tabla y propiedades.
- `src/gnome_context.rs`: menús contextuales nativos de texto/tablas/imágenes, selección por clic derecho y portapapeles enriquecido compartido entre documentos.
- `src/tables.rs`: navegación entre celdas sin ensuciar el documento ni el historial.
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





## Prioridad de MVP acordada

Completar el documento mínimo: imágenes (insertar, conservar proporción al redimensionar y eliminar), configuración de tamaño/orientación/márgenes y exportación PDF. Después, distribución instalable; combinar pruebas de documentos reales con primeros usuarios y mantenimiento. El rediseño completo de Texto no bloquea este orden. Flathub requiere colaboración humana para el manifiesto y la solicitud conforme a su política de IA revisada; no presentar la publicación como garantizada.


## Imágenes, propiedades y PDF: entrega validada

El usuario ha confirmado que le gusta el resultado de los botones divididos nativos y las miniaturas redondeadas. Autoriza conjuntamente Imagen en barra inferior, Propiedades del documento y Exportar a PDF en menú del documento. La implementación usa API públicas de WordCraft y no cambia las dependencias. CI 41127358053 completada correctamente, con cargo test --locked y compilación release. Las regresiones verifican imágenes y página en DOCX/ODT, proporción, tamaño original, sustitución, borrado y deshacer; cambios de sección/documento agrupados y formato pendiente conservado; errores y cambios nulos; destino, estado e historial al exportar. IMAGE_POINTER_READY/DOCUMENT_TOOLS_OK verifican clic real, controles de imagen fijados y diálogo de propiedades. PDF renderizado y revisado visualmente: una página A4 horizontal, texto extraíble e imagen incrustada. Capturas ancha/estrecha y propiedades revisadas. No es una confirmación manual del usuario; pendiente de sus documentos reales y del selector nativo en su equipo.



## Menús contextuales: trabajo autorizado

El usuario autoriza activar el clic derecho en texto, tablas e imágenes antes de preparar la aplicación instalable. Se sustituye el menú de tablas personalizado por GtkPopoverMenu/GMenu. Texto ofrece portapapeles, selección y acceso a Estilo/Párrafo/Texto; no repite Deshacer/Rehacer; tablas añaden submenús de inserción, selección y eliminación, combinar celdas y propiedades/dividir; imágenes ofrecen tamaño, tamaño original, sustitución y eliminación. Pulsar dentro de una selección de texto o celdas la conserva; fuera sitúa el cursor o selecciona la imagen. Menú/Mayús+F10 funciona para los tres contextos. Copiar/cortar/pegar y sus atajos comparten un portapapeles enriquecido que mantiene tablas, imágenes y formato entre documentos mientras GTK conserva el proveedor original; al copiar desde otra aplicación se usa el nuevo texto. No hay cambios en el motor ni en las dependencias.

El usuario ha autorizado retirar Tabla e Imagen de la barra y mover su inserción a Añadir en el menú del documento. Se aplica en la revisión correctiva posterior; se mantienen todos sus controles desde el contexto. Próxima prioridad acordada: distribución instalable después de estos ajustes de menús.

Validación de esta entrega: CI 41132689848 completada correctamente sobre 737a89cddaaa3a66b126d87c4da9d3ac72fad7ee; cargo test --locked y compilación release. CONTEXT_UI_OK verifica clic secundario real con xdotool, conservación de selección y revisión, menús GMenu nativos, copiar/pegar/deshacer, selección de columna, selección de imagen sin cambios, pegado de imagen en otro documento con deshacer y sustitución por texto de un propietario externo del portapapeles. También pasan POPOVER_PIN_OK, VIEW_POINTER_OK y DOCUMENT_TOOLS_OK. Captura del menú contextual de texto revisada. Pendiente de prueba manual del usuario en GNOME OS y con sus documentos reales.



## Corrección tras prueba del usuario: activación contextual y Añadir

El usuario informa de que las filas no realizan acciones. Causa verificada en GtkModelButton: el controlador nativo cierra el popover antes de ejecutar la acción asociada. El cierre del host desmontaba el menú inmediatamente, perdiendo su jerarquía y sus acciones. El desmontaje se difiere al siguiente ciclo idle, conservando el grupo hasta finalizar la activación. La prueba anterior activaba la acción directamente y no reproducía el recorrido de una fila nativa; se corrige para pulsar Copiar con xdotool y activar las demás filas mediante GtkWidget.activate/GtkModelButton, nunca mediante Popover.activate_action.

Aspecto según el código de Nautilus: GtkPopoverMenu sin punta, alineado al inicio y con aceleradores nativos. Sin Deshacer/Rehacer en el contexto. Menú del documento con Añadir → Tabla/Imagen. Barra normal solo Estilo/Párrafo/Texto, además del historial existente. Las opciones de tabla y tamaño de imagen siguen disponibles como paneles contextuales con la misma chincheta; se anclan al punto del clic y siguen usando el grupo exclusivo. Los botones antiguos solo se construyen como anclas visibles en diagnósticos internos que prueban los paneles anteriores, no en el arranque normal.

La regresión busca filas solo en la página visible del GtkStack nativo: GtkPopoverMenu conserva subpáginas ocultas con títulos y etiquetas repetidos, que no representan el menú que ve el usuario. La activación se hace sobre la fila nativa de esa página.

Los paneles contextuales de tabla/imagen permanecen como hijos del lienzo al reconstruir páginas o abrir el documento. clear_canvas elimina las páginas, no los GtkPopover: desmontarlos durante opened dejaba sus superficies sin una ventana nativa. La prueba normal (sin anclas de diagnóstico en barra) cubre esa apertura y conservación tras reconstrucción.

Entrega correctiva validada: código 7cd6debaa1ec55b4f44783966f2b22c56daad8c5; CI 41137708372 completada correctamente. cargo test --locked y compilación release superados. CONTEXT_UI_OK incluye clic real en Copiar, filas nativas para Cortar/Pegar/Seleccionar todo, apertura de Estilo/Párrafo/Texto, submenú visible de selección de columna y paneles normales de tabla/imagen tras abrir documentos. DOCUMENT_CHROME_OK verifica Añadir → Tabla/Imagen y ausencia de sus botones en la barra normal. También pasan POPOVER_PIN_OK, VIEW_POINTER_OK y DOCUMENT_TOOLS_OK. Capturas de contexto sin punta y menú de documento con Añadir revisadas. Pendiente de confirmación manual del usuario en su GNOME OS.



## Trabajo actual autorizado: barra adaptable y contexto contenido

El usuario autoriza conjuntamente mover los cinco controles a la cabecera con iconos, recuperar la barra inferior cuando no caben, animar tamaño/navegación como Añadir y abrir Estilo/Párrafo/Texto dentro del propio contexto. Implementación en revisión, aún sin sustituir la descarga validada anterior. Código de integración feb387055fabed2bda11ccb0d1e8dc8a1cb1fa0f (comprobar commits posteriores), CI 41143877597 en curso.

`gnome_adaptive.rs` mueve las mismas dos cajas de widgets: historial a la izquierda y formato a la derecha. Se usa el ancho real del viewport, no un tick continuo; histéresis 640/680 px para evitar oscilaciones. Al recolocar un panel abierto se conserva la subpágina y la chincheta; se captura antes de ocultar la barra. No modifica documento, selección ni historial. Iconos vectoriales con nombres accesibles y tooltips.

Los tres menús usan GtkStack con interpolate-size y SlideLeftRight, respetando el movimiento reducido. Texto conserva todos los comandos avanzados, ahora en páginas internas (caracteres, color/efectos y anotaciones), con selector de color nativo y entrada RGB opcional.

El contexto usa submenús nativos GMenu y custom slots oficiales de GtkPopoverMenu para alojar temporalmente los árboles de widgets existentes. No abre el popover de la barra. Un préstamo restaura cada árbol al cerrar; las acciones y estados siguen compartidos. Se oculta únicamente la cabecera atrás nativa duplicada para conservar una sola fila atrás/título/chincheta. Revalidar esa detección de GtkStack/GtkModelButton al actualizar GTK. El host contextual mantiene autohide=false desde antes de mapearse: la chincheta no remapea la superficie. No hay cambios en WordCraft ni dependencias.

Las pruebas ADAPTIVE_UI_OK y CONTEXT_UI_OK ya pasaron en la revisión 3c225058025de0622bc5fcd5ff3d6aa7c37e05c2: estrechar/ensanchar con panel fijado, subpágina conservada, préstamo/restauración de controles, cambio exclusivo del contexto a la barra, navegación de Texto y movimiento reducido. La regresión de foco se ajusta para pulsar fuera del nuevo popup de cabecera, que antes cubría el punto de prueba a la derecha de la página. Se conserva la comprobación de scroll y selección. La revisión actual conserva la captura síncrona del panel antes de que GTK desmonte el ancla al redimensionar, añade estilos de fuente/estados idénticos en contexto y foco de teclado explícito para el host no modal. Aún sin sustituir la descarga anterior.
