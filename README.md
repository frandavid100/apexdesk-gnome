# WordCraft GNOME — fase 3D

Para retomar el desarrollo: [estado actual](PROJECT_STATUS.md),
[hoja de ruta](ROADMAP.md) e [instrucciones para agentes](AGENTS.md).
Rama de trabajo: `wordcraft-saving`.

Guardar (Ctrl+S), Guardar como (Ctrl+Shift+S), DOCX y ODT mediante el motor WordCraft.
La primera operación propone una copia y protege el archivo abierto originalmente.
El nombre debe incluir .docx o .odt; cambiar la extensión elige el formato.
Los guardados siguientes actualizan la copia. El selector nativo confirma sobrescrituras.
Al cerrar un documento modificado, ofrece Cancelar, Descartar o Guardar.
Abrir un archivo conserva el documento actual y añade otra pestaña.
Ctrl+N crea una pestaña vacía; Ctrl+W cierra la pestaña actual.
Los controles de formato van de lo general a lo específico: Estilo, Párrafo y Texto. Se muestran como iconos a la derecha de la cabecera; Deshacer/Rehacer quedan a la izquierda. Si la ventana se estrecha, los cinco pasan a la barra inferior.
Junto a + y −, el tercer botón abre 100 % y Ajustar al ancho.
La navegación entre páginas se realiza mediante scroll o miniaturas.
Cancelar el selector o fallar el guardado mantiene abierto el documento con sus cambios.

Incluye las funciones de edición y formato de las fases 3B y 3C.
Las pruebas verifican contenido Unicode, negrita, tamaño y alineación después de
guardar y reabrir ambos formatos, además de guardados repetidos y fallos de escritura.
La fidelidad de documentos complejos depende del motor WordCraft; prueba con copias.

```sh
chmod +x wordcraft-gnome
./wordcraft-gnome
```

Documentación revisada:
https://docs.gtk.org/gtk4/method.FileDialog.save.html
https://developer.gnome.org/hig/patterns/feedback/dialogs.html

## Edición básica ampliada

Deshacer/Rehacer reflejan su disponibilidad tanto en la cabecera como en la disposición inferior. El historial también está en el menú nativo del documento, junto a Guardar y Guardar como.
Texto incluye el selector nativo de familias instaladas y tamaño libre de
1 a 1638 puntos, con decimales (botón Aplicar), además de B/I/U y tamaños rápidos.
Párrafo agrupa iconos en tres filas: alineación; listas y sangrías; interlineado,
sombreado y bordes. Las flechas abren subpáginas con vuelta atrás fija, y
Más opciones conserva los ajustes menos frecuentes dentro del desplegable.
Atajos: Ctrl+M / Ctrl+Shift+M para sangría y Ctrl+Shift+L para viñetas.
Abrir y Nueva ventana pertenecen al menú general encima del panel lateral;
el mismo menú sigue disponible cuando el panel está oculto.
El fondo del documento usa el color de ventana de Adwaita, como la cabecera
plana. Las hojas permanecen blancas.

## Pestañas, autoguardado y búsqueda

Pestaña nueva en el menú principal y Ctrl+N/Ctrl+T crean un documento vacío en otra pestaña. Ctrl+W cierra la pestaña;
Nueva ventana usa Ctrl+Shift+N. Abrir reutiliza una pestaña vacía o añade otra.
El panel de miniaturas es más estrecho y se abre por defecto.

Preferencias (menú general): desactivado, archivo de recuperación (predeterminado,
10 segundos, trabajador de WordCraft) o propio archivo (30 segundos). Este último
modo permite actualizar también el archivo abierto originalmente; los documentos
sin nombre usan recuperación hasta que se guarden por primera vez.
La preferencia persiste y se aplica también a las otras pestañas y ventanas.
Recuperar documentos permite recuperar copias de una ejecución anterior.

La lupa alterna miniaturas y búsqueda en el panel izquierdo (Ctrl+F / Ctrl+H).
Cada resultado muestra contexto y página; al pulsarlo selecciona la coincidencia.
Los tres puntos junto al campo permiten distinguir mayúsculas y activar Reemplazar.
Este modo añade un segundo campo y un botón de reemplazo por resultado.
Las sustituciones son literales y se pueden deshacer; cada pestaña conserva su búsqueda.

Véase UPDATING_WORDCRAFT.md para la separación de capas y la actualización del motor.

## Compilación optimizada

Para distribuir o usar a diario:

```sh
cargo build --release --locked --features gui --bin wordcraft-gnome
./target/release/wordcraft-gnome
```

El perfil de producción optimiza también el motor, usa ThinLTO y elimina
símbolos. La interfaz reutiliza los píxeles nativos del renderizador, busca
páginas mediante búsqueda binaria y agrupa las actualizaciones de miniaturas.
Las pruebas de interfaz se ejecutan sobre este mismo ejecutable de producción.

## Tablas

Añadir > Tabla, en el menú del documento, abre una cuadrícula y tamaños personalizados. Dentro
de una tabla, el clic derecho o Mayús+F10 ofrece selección de celdas/filas/columnas,
inserción y eliminación, combinación y propiedades. Más opciones de tabla abre
el panel completo con división de tabla y los demás controles.

Tab avanza entre celdas y crea una fila al final; Mayús+Tab retrocede.
Arrastrar entre celdas selecciona un rectángulo, incluidas celdas vacías.
Las propiedades incluyen anchos, alturas, autoajuste, alineación, bordes,
sombreado, división de celdas y opciones de cabecera/paginación.

## Desplegable de estilos

Estilo muestra todos los estilos de párrafo y de texto del documento, con
búsqueda y el estilo actual marcado con fondo de selección. Pulsar el nombre aplica el estilo; el
botón derecho abre su gestión dentro del mismo desplegable. La fila superior
permite volver. Incluye una muestra renderizada por WordCraft, actualizar
según la selección, renombrar y eliminar estilos personalizados. Nuevo estilo
abre un formulario interno para crearlo desde la selección. Los cambios se
reflejan al momento; estas operaciones no abren ventanas adicionales.

## Disposición de páginas y cabecera

El visor coloca dos páginas consecutivas por fila cuando caben al zoom actual;
al estrechar la ventana vuelve a una columna. Conserva las hojas blancas y
el fondo de Adwaita. Las miniaturas tienen 12 px de relleno simétrico alrededor de imagen y número, ambos dentro del botón.
La cabecera muestra una sola línea, con ● antes del nombre si hay cambios
pendientes; no incluye subtítulo ni contador de páginas. La vuelta del foco
desde los menús al documento conserva el desplazamiento del visor.

## Navegación lateral y menú principal

El selector inferior Páginas/Títulos sigue el aspecto de Papers. Títulos muestra
la jerarquía de niveles de esquema del documento, con numeración, secciones que
se pueden contraer y navegación al texto. La lupa alterna búsqueda/navegación
con un fundido breve y recuerda la vista anterior. No muestra un contador de
coincidencias. El botón junto al segundo campo reemplaza todos los resultados
en un solo paso de deshacer; cada fila mantiene el reemplazo individual.

El menú principal usa GMenu/GtkPopoverMenu, como Nautilus, con atajos a la derecha
y acciones de ventana/aplicación. Incluye ventana y pestaña
nuevas, todas las pestañas, abrir, preferencias, atajos, ayuda y acerca de.
Recuperar documentos está en el menú principal. No hay opción de ejemplo. La vista de todas
las pestañas es el AdwTabOverview nativo. Se mantienen Ctrl+N para pestaña nueva
y Ctrl+Mayús+N para ventana nueva; Ctrl+T también abre una pestaña.


## Barras inferiores y menú de documento

El selector Páginas/Títulos y la barra de edición tienen la misma altura,
compartida mediante GtkSizeGroup. Ambos usan AdwToolbarView y sus sombras de
contenido desplazado. El menú de los tres puntos del documento usa GMenu y
GtkPopoverMenu, con Deshacer/Rehacer y Guardar/Guardar como, con atajos y
estado del historial del documento activo. La cabecera no tiene botones de
pestaña nueva ni guardar. El arranque normal abre un documento vacío.
El documento usado en CI es una fixture interna (--test-document), sin entrada
en los menús ni ejemplo que deba abrirse para usar el editor.


## Fijar desplegables

Estilo, Párrafo, Texto y los paneles contextuales de Tabla e Imagen comparten la chincheta y solo uno puede estar
abierto. Abrir otro sustituye al anterior y conserva la fijación. Fijar/desfijar
no oculta ni recrea el panel. Al fijarlo se mantiene abierto al aplicar opciones
y seguir escribiendo; sin fijar se cierra al aplicar o pulsar fuera.
Las subpáginas tienen una sola cabecera con vuelta atrás, nombre y chincheta.
Los botones de subopciones de Párrafo tienen fondo y esquinas nativos como
Nautilus, con chevrones pequeños hacia abajo.


Los botones divididos de Párrafo usan AdwSplitButton plano: fondo compartido y resaltado independiente de cada mitad como Nautilus. Las miniaturas tienen esquinas de 6 px como Papers.


## Imágenes, propiedades y PDF

Añadir > Imagen, en el menú del documento, permite insertar una imagen en línea. Se selecciona con un clic y el menú contextual permite abrir su panel para cambiar el tamaño en milímetros. Mantener proporción está activado por defecto; también permite restaurar tamaño original, sustituir y eliminar. Comparte la chincheta con los demás desplegables.
El menú del documento incluye Propiedades del documento (papel, orientación y márgenes; sección actual o todo el documento) y Exportar a PDF. La exportación trabaja en segundo plano y no cambia el archivo que se sigue editando y guardando. Los selectores requieren el sufijo del formato para confirmar sobrescrituras sobre el destino exacto.



El clic derecho sobre texto, tablas e imágenes abre un menú contextual nativo. Conserva la selección si se pulsa dentro de ella; fuera sitúa el cursor o selecciona la imagen correspondiente. Menú o Mayús+F10 abre las opciones de la selección actual. Copiar y pegar dentro de la aplicación conserva el formato, las tablas y las imágenes, también entre documentos. El texto procedente de otras aplicaciones sigue admitiéndose.



Tabla e Imagen se insertan desde Añadir en el menú del documento (tres puntos). Sus opciones de edición están en el clic derecho. El menú contextual no repite Deshacer/Rehacer; conserva los accesos al portapapeles y al formato con el aspecto nativo de Nautilus.



Estilo, Párrafo y Texto se abren también como submenús dentro del clic derecho, con su propia vuelta al contexto; no abren los desplegables de la cabecera. Comparten controles, estados y chincheta. Las transiciones animan el tamaño como Añadir y respetan el movimiento reducido. Las opciones avanzadas de Texto se distribuyen en subpáginas del mismo menú.
