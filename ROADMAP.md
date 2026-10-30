# Hoja de ruta

Actualizada: 30 de octubre de 2026. Estado técnico en [PROJECT_STATUS.md](PROJECT_STATUS.md).
Las fases siguientes son propuestas; no implican autorización para implementarlas todas.

## Ahora: imágenes, propiedades de documento y exportación PDF

Validado automáticamente en CI (41094124112), pendiente de prueba del usuario: se han aplicado márgenes simétricos de miniaturas, cabecera de una línea sin contador de páginas, disposición automática de dos páginas y corrección del salto al devolver el foco desde los menús al documento.

Implementados: búsqueda con opciones sin fondo, reemplazar todo en un paso de deshacer, sin contador y con fundido; selector Páginas/Títulos como Papers y títulos jerárquicos contraíbles; menú principal GMenu/GtkPopoverMenu como Nautilus, acciones de ventana y vista general nativa de pestañas. Validación y revisión de entrega en PROJECT_STATUS.md. Pendiente de prueba del usuario.

Últimos ajustes solicitados: igualar las barras inferiores y sus sombras nativas; ampliar el marco de miniaturas e incluir el número; mover el historial a un menú de documento nativo junto a Guardar/Guardar como; quitar los botones de pestaña nueva/guardar de la cabecera y la opción de ejemplo. Arranque normal con documento vacío.

Revisión solicitada tras probar la chincheta: eliminar el parpadeo al fijar/desfijar, sustituir el panel anterior al abrir otro manteniendo la chincheta, unificar vuelta atrás/título/chincheta y cambiar el botón completo de subopciones al aspecto de Nautilus. Grupo exclusivo por pestaña con modalidad GTK constante.

Cabeceras con nombre y chincheta en los cuatro desplegables de edición. La chincheta permite aplicar varias opciones sin cerrar el panel; al desfijar se recupera el cierre normal. Chevrones pequeños como Nautilus en Párrafo/Texto y coherentes en los botones inferiores. No implica reorganizar todavía todas las opciones de Texto.

Último ajuste implementado y validado en CI 41122192743, aprobado por el usuario: usar AdwSplitButton plano como Nautilus, con resaltado conjunto y por mitad, conservando navegación interna; redondear las hojas de miniatura a 6 px como Papers.

Prioridad acordada del MVP: completar imágenes, configuración de página y exportación PDF; después aplicación instalable, y pruebas con usuarios/documentos reales junto al mantenimiento. El rediseño completo de Texto queda aplazado. La publicación en Flathub requiere participación humana para manifiesto y solicitud según la política de IA revisada.

Recoger la prueba del usuario de formato avanzado y compilación optimizada en GNOME OS.
Corregir los fallos que aparezcan antes de acumular funciones nuevas.

## Estilos: rediseño probado por el usuario; ajustes visuales

El desplegable incluye todos los estilos de párrafo/texto del documento,
búsqueda, aplicación por nombre y botón derecho para abrir una subpágina
de gestión. Previsualización, actualizar, renombrar, eliminar y crear quedan
dentro del propio desplegable. La fila superior vuelve a la página anterior.
Orden acordado de la barra: Estilo → Párrafo → Texto.
El usuario confirma la distribución; se aligera el peso tipográfico de los botones.
Antes de Párrafo se añade movimiento suave del cursor y se elimina la apertura diferida del panel lateral.

Se ha autorizado e implementado la búsqueda lateral tipo Papers y después el rediseño de Párrafo con tres filas de iconos y gestión en subpáginas. El estilo actual usa fondo de selección. Las pruebas automáticas y las capturas de la CI han pasado; pendiente de validación en el equipo del usuario.
La revisión de Texto queda pendiente de diseño con el usuario.
Se ha acordado priorizar imágenes/configuración de página/PDF sobre la revisión completa de Texto; esta última no bloquea el MVP.

## Tablas: implementadas, pendientes de prueba manual

El usuario autorizó esta ampliación. Incluye selector de tamaño, filas/columnas,
selección rectangular de celdas, combinar/dividir, anchos/alturas, alineación,
bordes, sombreado, cabeceras y división de tabla. Menú Tabla en la barra inferior,
propiedades en un panel nativo y menú contextual con clic derecho/Mayús+F10.

La estructura se modifica mediante los comandos de WordCraft. La navegación entre
celdas existentes usa selección pura para conservar el estado de cambios e historial.
Tab en la última celda añade una fila mediante el motor; Mayús+Tab retrocede.

Pruebas: historial, selecciones vacías, columnas independientes, combinar/dividir,
DOCX/ODT, varias páginas y teclado real en Xvfb. Revisar el resultado de CI y
la prueba del usuario antes de marcar esta entrega como validada manualmente.

Quedan para una ampliación posterior los tiradores de tamaño con ratón, las
fórmulas y los estilos completos de tabla; no forman parte de esta entrega.

## Imágenes, página y PDF: implementados y validados automáticamente

Imagen a la derecha de Tabla. Insertar imágenes en línea desde selector nativo; tamaño en mm y proporción por defecto; restaurar tamaño, sustituir y eliminar en el propio desplegable.
Propiedades del documento en los tres puntos: papel, orientación y márgenes, con alcance de sección/documento y un solo paso de deshacer. Exportar a PDF separado de Guardar como para conservar el destino editable; trabajo en segundo plano.
Pruebas: imágenes y página en DOCX/ODT, historial/errores, selección mediante clic real, panel fijado, diálogo y PDF. Validación completa de CI 41127358053 y revisión visual de paneles/PDF superadas; pendiente de prueba del usuario.

## Más adelante: página y entrega como aplicación

Tras validar esta entrega, preparar aplicación instalable. Impresión y ajustes avanzados de imagen quedan para otra ampliación.
Empaquetado instalable (por ejemplo Flatpak), icono, archivo desktop, asociación DOCX/ODT,
metadatos y una versión identificable que sustituya el nombre provisional «fase 3D».

## Mantenimiento continuo

Compatibilidad con documentos reales, accesibilidad/teclado, rendimiento medido en archivos
largos y actualización controlada del motor según UPDATING_WORDCRAFT.md.
No modificar el motor por comodidad de la interfaz ni distribuir binarios de desarrollo.





