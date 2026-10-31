# Hoja de ruta

Actualizada: 31 de octubre de 2026. Estado técnico en [PROJECT_STATUS.md](PROJECT_STATUS.md).
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








## Ajuste previo a la instalación: clic derecho

Autorizados e implementados menús contextuales nativos para texto, tablas e imágenes, con conservación de selección y portapapeles enriquecido entre documentos. Validación de entrega en PROJECT_STATUS.md. Antes de retirar botones inferiores, resolver la propuesta del usuario: Tabla/Imagen dejarían la barra y su inserción pasaría a Añadir en el menú del documento; la gestión seguiría accesible desde el contexto. El usuario la autoriza después de probar el contexto: se retiran los dos botones, se incorpora Añadir y se conservan los controles desde el clic derecho.



Corrección prioritaria autorizada: conservar las acciones al cerrar una fila nativa, aspecto de contexto como Nautilus sin punta, retirar historial redundante del contexto y mover inserción de Tabla/Imagen a Añadir. Validar activación real de filas, no solo invocación directa de acciones, antes de distribuir.

Entrega correctiva validada automáticamente en CI 41137708372, código 7cd6debaa1ec55b4f44783966f2b22c56daad8c5. Contexto nativo funcional, sin historial redundante; Tabla/Imagen fuera de la barra, inserción en Añadir y paneles preservados al reconstruir el documento. Pendiente de prueba manual del usuario.



## Ajuste autorizado: barra adaptable y submenús contenidos

Implementados y validados: controles con iconos en cabecera cuando hay espacio; mismos controles abajo en estrecho. Estilo/Párrafo/Texto se gestionan también dentro del contexto, con vuelta atrás y chincheta, compartiendo widgets y acciones. Transición nativa de deslizamiento y tamaño en los tres. Texto avanzado pasa a subpáginas. Validación completa en CI 41144295677, código c744dac3f553b0304e631dd80dde7fbe3669e1cd, con capturas revisadas. Pendiente de prueba manual del usuario.


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
