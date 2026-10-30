# Hoja de ruta

Actualizada: 30 de octubre de 2026. Estado técnico en [PROJECT_STATUS.md](PROJECT_STATUS.md).
Las fases siguientes son propuestas; no implican autorización para implementarlas todas.

## Ahora: validar la última entrega

Recoger la prueba del usuario de formato avanzado y compilación optimizada en GNOME OS.
Corregir los fallos que aparezcan antes de acumular funciones nuevas.

## Siguiente ampliación propuesta: tablas

Auditar primero los comandos públicos y el comportamiento de selección en celdas de
la revisión fijada de WordCraft. Reutilizar esos comandos desde una interfaz GNOME.

Alcance propuesto: insertar tabla con filas/columnas; añadir y eliminar filas/columnas;
combinar/dividir celdas cuando lo admita el motor; ajustar anchos, alineación, bordes y
sombreado. Revisar navegación con Tab/Shift+Tab y edición de texto dentro de celdas.
Ubicación de controles por decidir con el usuario; priorizar controles contextuales compactos.

Criterios de aceptación: operaciones reversibles con Deshacer/Rehacer, cursor y selección
coherentes, guardado/reapertura en DOCX y ODT, tablas de varias páginas y ventana estrecha.
Identificar y documentar límites del motor; no simular funciones que no estén soportadas.

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
