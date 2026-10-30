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
La barra inferior va de lo general a lo específico: Estilo, Párrafo y Texto.
Junto a + y −, el tercer botón abre 100 % y Ajustar al ancho.
La navegación entre páginas se realiza mediante scroll o miniaturas.
Cancelar el selector o fallar el guardado mantiene abierto el documento con sus cambios.

Incluye las funciones de edición y formato de las fases 3B y 3C.
Las pruebas verifican contenido Unicode, negrita, tamaño y alineación después de
guardar y reabrir ambos formatos, además de guardados repetidos y fallos de escritura.
La fidelidad de documentos complejos depende del motor WordCraft; prueba con copias.

```sh
chmod +x wordcraft-gnome
./wordcraft-gnome --sample --sidebar
```

Documentación revisada:
https://docs.gtk.org/gtk4/method.FileDialog.save.html
https://developer.gnome.org/hig/patterns/feedback/dialogs.html

## Edición básica ampliada

La barra inferior empieza con Deshacer/Rehacer y refleja su disponibilidad.
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

+ y Ctrl+N crean un documento nuevo en una pestaña. Ctrl+W cierra la pestaña;
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

Tabla > Insertar tabla abre una cuadrícula y tamaños personalizados. Dentro
de una tabla, el mismo menú ofrece selección de celdas/filas/columnas, inserción
y eliminación, combinación, división de tabla y propiedades. También hay
menú contextual con clic derecho o Mayús+F10.

Tab avanza entre celdas y crea una fila al final; Mayús+Tab retrocede.
El menú contextual permite insertar una tabulación literal dentro de la celda.
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
el fondo de Adwaita. Las miniaturas tienen relleno simétrico y el número debajo.
La cabecera muestra una sola línea, con ● antes del nombre si hay cambios
pendientes; no incluye subtítulo ni contador de páginas. La vuelta del foco
desde los menús al documento conserva el desplazamiento del visor.
