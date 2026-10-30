# Hoja de ruta

Actualizada: 30 de octubre de 2026. Estado técnico en [PROJECT_STATUS.md](PROJECT_STATUS.md).
Las fases siguientes son propuestas; no implican autorización para implementarlas todas.

## Ahora: mejorar los desplegables y validar la última entrega

Recoger la prueba del usuario de formato avanzado y compilación optimizada en GNOME OS.
Corregir los fallos que aparezcan antes de acumular funciones nuevas.

## Estilos: rediseño autorizado

El desplegable incluye todos los estilos de párrafo/texto del documento,
búsqueda, aplicación por nombre y botón derecho para abrir una subpágina
de gestión. Previsualización, actualizar, renombrar, eliminar y crear quedan
dentro del propio desplegable. La fila superior vuelve a la página anterior.
Orden acordado de la barra: Estilo → Párrafo → Texto.

La revisión del resto de desplegables queda pendiente de diseño con el usuario.
No continuar todavía con imágenes por haber quedado después de esta revisión.

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

## Después: imágenes

Insertar desde selector nativo; conservar proporción al redimensionar; eliminar y configurar
las modalidades de ajuste/alineación que exponga WordCraft. Validar historial y guardado.

## Más adelante: página y entrega como aplicación

Configuración de tamaño/orientación/márgenes y saltos; exportación PDF e impresión.
Revisar primero las capacidades del motor y las API nativas relevantes de GNOME.
Empaquetado instalable (por ejemplo Flatpak), icono, archivo desktop, asociación DOCX/ODT,
metadatos y una versión identificable que sustituya el nombre provisional «fase 3D».

## Mantenimiento continuo

Compatibilidad con documentos reales, accesibilidad/teclado, rendimiento medido en archivos
largos y actualización controlada del motor según UPDATING_WORDCRAFT.md.
No modificar el motor por comodidad de la interfaz ni distribuir binarios de desarrollo.
