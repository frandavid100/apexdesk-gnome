# Estado de Leaf

Actualizado: 31 de octubre de 2026. Punto de entrada para continuar desde otra conversación.

## En curso: auditoría GTK de propiedad y memoria autorizada

Medición inicial en b7b5b7f792856fc5ca02f1f3b7769ad96320bd3a, CI 41234617745: tras cerrar 15 documentos, 15 Ui y 6 Workspace permanecen vivos; cuerpos de documento e imágenes de página sí se liberan. RSS tras malloc_trim: 188684 → 212208 → 235676 KiB en tres tandas. Corrección en curso: propietarios explícitos de ventanas, referencias débiles en señales, temporizadores detenidos al cerrar, CSS registrado una vez y liberación de portapapeles obsoleto. Pendientes prueba estricta posterior, comparación y paquete; no presentar mejora medida hasta validar.

## Última entrega: alpha.5 — teclado, controles accesibles e impresión (31-10-2026)

Publicada: https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.5
Etiqueta/paquete: `3bb562d04129d668261b6aafb25833628d801480`. Código nativo validado: `b1b42907f2059353518480899a3d4d56beb2996f`; igualdad de src/Cargo/tests/vendor/patches comprobada antes de reutilizar su artefacto.
Flatpak x86_64: **10.151.304 bytes**; SHA256 `0a9d56137e03d2d622e3f224b6ac866851ece7a2766c18e4d750d3dc350ef34d`.

Imprimir… en el menú del documento y Ctrl+P usan GtkPrintDialog y el portal nativo. PDF vectorial temporal privado desde una copia del documento en segundo plano, sin guardar ni cambiar el documento vivo, selección, destino o historial. Control de solicitudes duplicadas, cancelación al cerrar pestaña y limpieza al finalizar. No hay librería nueva ni cambio del motor; compilación desde código requiere GTK 4.14+, runtime Flatpak GNOME 49 ya compatible.

F6/Mayús+F6 recorren documento/panel/formato; F10 abre el menú principal. Etiquetas explícitas en iconos, campos, subopciones, estilos, tablas y números; Escape de formato devuelve foco al ancla. Acerca de y --version comparten versión. No equivale a accesibilidad completa: el lienzo aún no expone texto/selección completos a Orca.

[Informe de implementación, validación y límites](docs/KEYBOARD_PRINTING.md). CI nativa [41225574820](https://github.com/frandavid100/leaf/actions/runs/41225574820), job **124608903617**: cargo test --locked, release GTK, teclas reales, entrega real de descriptor PDF a portal de prueba (éxito/cancelación/error), regresiones gráficas e HiDPI correctos. No se repitió el corpus de 24 archivos. Artefacto nativo 13051823942, digest GitHub ca176fa5dc604a53fdc85dfd349aa071c85649090b10b066fb2305f047e12534.

Paquete [41226912994](https://github.com/frandavid100/leaf/actions/runs/41226912994), job **124612866903**: actualizado desde alpha.4, diálogo real de impresión abierto/cancelado, tres respuestas del portal simulado dentro del sandbox con PDF comprobado, lanzador, archivo externo y persistencia/reinstalación. Artefacto final 13051049778, digest GitHub a63e32c3e3c1bc1a57b5b183f0f06fe302abc17b2a2a724bc1bf415a6e47460e. No se descargaron/verificaron independientemente los artefactos ni se revisó visualmente la captura final tras quedar indisponible el entorno local; resultados automáticos y fuentes permanecen en GitHub.

Detalle observado del backend GTK 1.15.1 de CI: Escape/cierre de ventana devuelve respuesta 2 (fallo), mientras Cancelar devuelve 1. La prueba final usa Cancelar (Alt+C); no se silencian errores reales. En ese backend Escape puede mostrar un error de impresión; usar Cancelar. Impresora física, Orca y contraste alto pendientes de prueba manual. PDF mantiene limitaciones del exportador upstream, incluida advertencia de metadatos Suspects de Poppler. Ver informe; no atribuir a CI una confirmación manual.

Los primeros intentos fallaron antes de publicar por una referencia del test GTK, el número antiguo de opciones del menú, la diferencia Escape/Cancelar del portal y una herramienta ausente en el entorno de prueba del paquete. Corregidos; solo se distribuye la validación final.

## Entrega anterior: alpha.4 — conservación ODT (31-10-2026)

Publicada: https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.4
Fuente/etiqueta: `d5f969801fb8f069396922cb6603ff4cfa68b35c`.
Flatpak x86_64: **10.133.160 bytes**; SHA256 `300b3e778fbfcffcbd5bc8d9a601bba7fa37a8e1b655bd3c6d969b78493e4707`.

Corregida la pérdida de cabeceras/pies ODT (incluye tablas anidadas, estilos automáticos y medios), conservadas partes de estilos de página no usados, y eliminada la conversión de primera fila visual a cabecera repetida. Excepción explícita al motor antes sin cambios: snapshot aislado de wordcraft-formats con Cargo patch, misma revisión upstream, licencias y diff en patches/. Los demás crates del motor/renderizador no se modifican. UPDATING_WORDCRAFT explica cómo actualizar o retirar el parche.

[Informe de seguimiento](docs/ODT_PRESERVATION_2026-10-31.md) y [datos permanentes](docs/audits/2026-10-31-odt-fix-results.json). Segunda ejecución de los mismos 24 archivos LibreOffice (14 DOCX/10 ODT): sin pérdidas de tokens importados, mismas firmas medidas DOCX; ODT conserva texto/formato medido/medios/objetos, tablas y partes. Quedan redondeos y diferencias menores de raster. El caso de 7 pies conserva 7; el pie anidado conserva 2 tablas. Revisión visual de cuatro PDF exportados por LibreOffice confirma texto de cabecera/pie y tablas. No es validación universal ni comparación completa contra originales en LibreOffice.

CI [41219266814](https://github.com/frandavid100/leaf/actions/runs/41219266814): job nativo **124590219910** y paquete **124593334957** correctos. Pasan cargo test --locked, 7 regresiones nuevas, compilación release GTK, regresiones gráficas e HiDPI. Flatpak supera actualización desde alpha.3, lanzador, portal, datos persistentes/reinstalación y pruebas de interfaz dentro del sandbox. Artefacto nativo 13049921141 (digest verificado af934ca273e37a417346f17ff0b0096fbe816a37fe93b407d10837c302880081); artefacto de paquete 13049433666.

La auditoría anterior queda como evidencia histórica en docs/CORPUS_AUDIT_2026-10-31.md. Sus avisos aplican hasta alpha.3. Auditoría manual, no obligatoria por versión; el marcador [odt-audit] solo activa la comprobación puntual solicitada en este arreglo. Pendiente prueba manual del usuario; usar copias. Limitaciones de importación, estilos de página y columnas ODT desiguales siguen descritas. Variantes desactivadas conservan contenido en páginas maestras no usadas, sin retener su asociación de ranura inactiva.

## Comprobación puntual inicial — histórico alpha.3

La primera auditoría completó 24 aperturas/guardados/reaperturas y descubrió pérdidas en 3 ODT y cabeceras repetidas erróneas en otros 2. Sus datos no se sustituyen. Los fallos de conservación que motivaron la advertencia se han corregido y vuelto a comprobar en alpha.4; no extender esa conclusión a documentos arbitrarios.

## Entrega anterior: alpha.3 — escalado HiDPI (31-10-2026)

Publicada: https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.3
Fuente/etiqueta: `45037abc0ce0e323ea03f8f2c20218eda08c77a2`.
Flatpak x86_64: 10120440 bytes; SHA256 `1ed5b96ba13a43c71e5b6f1101b106fced359a537305c5d52eb3aa6b82863268`.

La resolución de páginas y miniaturas combina zoom y escala GTK. Un GdkPaintable con dimensiones lógicas independientes evita que la textura HiDPI agrande páginas, miniaturas o coordenadas. El cambio de scale-factor invalida ambas cachés mediante un callback idle con referencia débil. Se retira el límite de 2 píxeles/punto del adaptador, conservando el límite de asignación de 12 millones de píxeles/página. Páginas excepcionalmente grandes o densidades extremas pueden superar ese límite; no se ha implementado renderizado por teselas. Motor y dependencias sin modificaciones.

CI 41210344397: job nativo 124563840489 y paquete 124565963986 correctos. Cargo test --locked, release, regresiones GTK anteriores y HIDPI_UI_OK a GDK_SCALE=1 y 2 con zoom 1 y 2; se verifican dimensiones de textura, geometría lógica e identidad del documento tras invalidación. Capturas vista-hidpi-1/2.png; detalle al 200% revisado visualmente, texto nítido. Flatpak: prueba HiDPI al 200%, actualización desde alpha.2, lanzador, archivo externo por portal y persistencia/reinstalación. El primer intento de compilación falló por nombres de constantes de gtk-rs 0.11; ajustados a la versión fijada 0.10 antes de distribuir.

Pendiente: confirmación del usuario de Reddit, pruebas reales Wayland multimonitor y escala fraccionaria. La invalidación se ha probado forzando una caché obsoleta; no se ha simulado un traslado real entre monitores.

## Entrega anterior: alpha.2 — columnas e iconos (31-10-2026)

- Publicada: https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.2
- Flatpak x86_64: 10,113,936 bytes; SHA256 `6a90c8fc6e371586f786193cc8662009254157be39bb5d1d996215b3179dedd2`.
- Fuente etiquetada/empaquetada: `aeab3e4d31115597dd739ced269c2cf8e1b7c7eb`. Código nativo idéntico al validado en `a13bc6c1a2221035190eb500abbdb0a8683fe30b`; el segundo commit solo corrige preparación de pruebas de paquete/documentación.
- Párrafo → Columnas: 1–12, iguales/asimétricas/personalizadas, separación en mm y línea opcional. Alcance explícito: sección, documento, selección de texto o desde el cursor. Se prepara el cambio y se aplica en un paso de deshacer; volver sin aplicar no modifica el documento.
- Se conserva el motor público sin cambios. DOCX conserva anchuras desiguales; el exportador ODT fijado las iguala (mantiene número, separación y línea). Se explica en el desplegable.
- Estilo/Párrafo/Texto usan iconos simbólicos GTK con tamaño/alineación nativos.
- CI nativa 41192206555, job 124511105404: cargo test --locked, compilación release, pruebas GTK incluyendo COLUMNS_UI_OK y capturas. Captura de Columnas revisada visualmente.
- CI paquete 41192998882, job 124513448571: actualización desde alpha.1, datos persistentes, pruebas de columnas/estilos/cabecera en sandbox, lanzador, apertura por portal y reinstalación. Primera prueba de paquete falló por iniciar el portal fuera de la sesión gráfica; corregida sin cambiar el ejecutable. No se publicó ese intento fallido.
- Pendiente: prueba manual del usuario en GNOME OS/documentos reales.
- Investigación y prioridades propuestas: [docs/FEATURE_PRIORITIES.md](docs/FEATURE_PRIORITIES.md). No confundir frecuencia histórica de comandos con importancia ni con demanda actual. No se ha añadido telemetría.

## Cómo retomar

Repositorio: https://github.com/frandavid100/leaf
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
- Orden de formato: Estilo → Párrafo → Texto, en la cabecera o en la barra inferior según el ancho útil del documento. Tabla e Imagen se insertan desde Añadir en el menú del documento y se editan con clic derecho. Estilos y Párrafo se gestionan en desplegables con subpáginas internas; Texto agrupa sus opciones avanzadas en subpáginas internas.
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

Código del último ejecutable validado: `497cd6c5aad7a71f4b580e933f4c3e505b9126c5`. Primera prerelease Flatpak y validación detallada en la sección final de este documento.

Entrega histórica del ZIP anterior: `c744dac3f553b0304e631dd80dde7fbe3669e1cd`.
Los commits posteriores a esa revisión pueden ser exclusivamente documentación: comprobar el historial.

[CI completada correctamente](https://github.com/frandavid100/leaf/actions/runs/41144295677).
Artefacto de esa ejecución: `wordcraft-gnome-fase3d-linux`, ID `13020294789`.
Ejecutable actual con barra adaptable y contexto contenido: 34.258.488 bytes.
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

- `src/gnome_adaptive.rs`: reubicación de los mismos controles, iconos accesibles y subpáginas avanzadas de Texto.
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



## Barra adaptable y contexto contenido: entrega validada

El usuario autoriza conjuntamente mover los cinco controles a la cabecera con iconos, recuperar la barra inferior cuando no caben, animar tamaño/navegación como Añadir y abrir Estilo/Párrafo/Texto dentro del propio contexto. Implementación validada en c744dac3f553b0304e631dd80dde7fbe3669e1cd, CI 41144295677 completada correctamente.

`gnome_adaptive.rs` mueve las mismas dos cajas de widgets: historial a la izquierda y formato a la derecha. Se usa el ancho real del viewport, no un tick continuo; histéresis 640/680 px para evitar oscilaciones. Al recolocar un panel abierto se conserva la subpágina y la chincheta; se captura antes de ocultar la barra. No modifica documento, selección ni historial. Iconos vectoriales con nombres accesibles y tooltips.

Los tres menús usan GtkStack con interpolate-size y SlideLeftRight, respetando el movimiento reducido. Texto conserva todos los comandos avanzados, ahora en páginas internas (caracteres, color/efectos y anotaciones), con selector de color nativo y entrada RGB opcional.

El contexto usa submenús nativos GMenu y custom slots oficiales de GtkPopoverMenu para alojar temporalmente los árboles de widgets existentes. No abre el popover de la barra. Un préstamo restaura cada árbol al cerrar; las acciones y estados siguen compartidos. Se oculta únicamente la cabecera atrás nativa duplicada para conservar una sola fila atrás/título/chincheta. Revalidar esa detección de GtkStack/GtkModelButton al actualizar GTK. El host contextual mantiene autohide=false desde antes de mapearse: la chincheta no remapea la superficie. No hay cambios en WordCraft ni dependencias.

Las pruebas ADAPTIVE_UI_OK y CONTEXT_UI_OK ya pasaron en la revisión 3c225058025de0622bc5fcd5ff3d6aa7c37e05c2: estrechar/ensanchar con panel fijado, subpágina conservada, préstamo/restauración de controles, cambio exclusivo del contexto a la barra, navegación de Texto y movimiento reducido. La regresión de foco se ajusta para pulsar fuera del nuevo popup de cabecera, que antes cubría el punto de prueba a la derecha de la página. Se conserva la comprobación de scroll y selección. La revisión actual conserva la captura síncrona del panel antes de que GTK desmonte el ancla al redimensionar, añade estilos de fuente/estados idénticos en contexto y foco de teclado explícito para el host no modal. La validación completa de la revisión final se registra a continuación.


Validación final: cargo test --locked y compilación release superados; CI 41144295677 sobre c744dac3f553b0304e631dd80dde7fbe3669e1cd. ADAPTIVE_NARROW_OK / ADAPTIVE_WIDE_OK / ADAPTIVE_UI_OK verifican controles únicos, cambio de barra, panel fijado realmente mapeado y subpágina conservada, selección/revisión sin cambios, préstamo/restauración de widgets, cambio del contexto a otro panel y movimiento reducido. CONTEXT_UI_OK comprueba clic real en Copiar, foco en el menú al abrirlo y en el editor al cerrarlo, filas nativas de portapapeles y los tres submenús contenidos. VIEW_POINTER_OK, DOCUMENT_CHROME_OK, POPOVER_PIN_OK y DOCUMENT_TOOLS_OK también pasan. Revisadas capturas ancha/estrecha, los tres contextos con cabecera única y Texto avanzado. Pendiente de prueba manual del usuario en GNOME OS; no interpretar estas pruebas como confirmación manual.


## Leaf: identidad visual — 31 de octubre de 2026

Nombre elegido por el usuario: **Leaf**. Descripción: «Crea y edita documentos» / «Create and edit documents». WordCraft continúa siendo el motor sin modificaciones.

Iconos originales en `data/icons/hicolor/scalable/apps/leaf.svg` (128 × 128) y `data/icons/hicolor/symbolic/apps/leaf-symbolic.svg` (16 × 16). SVG editable: hoja escrita, pliegue verde, paleta GNOME y perfil inferior de 4 px alineado en y=116; sin sombra exterior. XML validado y renders revisados a 16/32/64/128/512 px, sobre fondos claros/oscuros. No se ha recompilado ni sustituido el ejecutable: esta entrega añade únicamente identidad y documentación. Integración con lanzador/instalador pendiente.

Repositorio renombrado manualmente por el usuario a `frandavid100/leaf` y verificado mediante GitHub (mismo ID de repositorio: 1432905461). Enlaces de continuidad y CI actualizados. La rama de desarrollo sigue siendo `wordcraft-saving`; `main` permanece como rama predeterminada. No se ha recompilado ni regenerado el ZIP por este cambio de documentación.


## Preparación de pruebas públicas — 31 de octubre de 2026

Preparados README público en inglés, guía de pruebas de 15 minutos, registro vacío de resultados reales, guía de contribución, formularios de fallo/compatibilidad (inglés o español), declaración del alcance de IA, lista de requisitos para la primera release y borradores de notas/Reddit. La guía española previa se conserva en docs/USER_GUIDE_ES.md. Capturas reales sin retoques de CI 41144295677, con revisión visual y procedencia/digest registrados en docs/screenshots/README.md. No se inventan resultados manuales ni descarga de una release inexistente.

La portada de main se ha actualizado para dirigir al código y documentos de wordcraft-saving, sin sustituir su código antiguo; las plantillas están también en main para que GitHub pueda ofrecerlas. Commit de portada y formularios: c4b0be9c869dca4fe44141edee4bf4321b5c149a. No cambia la rama predeterminada ni se publica en Reddit. No se ha creado una release, un paquete Flatpak ni un manifiesto. El instalable requiere integrar marca/lanzador/metadatos, empaquetar y validar en GNOME OS antes de anunciarlo. docs/RELEASE_CHECKLIST.md recoge el trabajo pendiente y la necesidad de colaboración humana para un manifiesto destinado a Flathub conforme a su política actual.

Esta entrega es documental y de capturas: no cambia el ejecutable ni exige recompilarlo. Pendiente antes de difusión: completar y probar el instalable, registrar resultados manuales, actualizar capturas con la marca Leaf, publicar prerelease y revisar el borrador con su enlace real.

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
