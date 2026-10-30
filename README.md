# WordCraft GNOME — fase 3D

Guardar (Ctrl+S), Guardar como (Ctrl+Shift+S), DOCX y ODT mediante el motor WordCraft.
La primera operación propone una copia y protege el archivo abierto originalmente.
El nombre debe incluir .docx o .odt; cambiar la extensión elige el formato.
Los guardados siguientes actualizan la copia. El selector nativo confirma sobrescrituras.
Al cerrar un documento modificado, ofrece Cancelar, Descartar o Guardar.
Abrir un archivo conserva el documento actual y abre otra ventana independiente.
Ctrl+N abre una ventana vacía; Ctrl+W cierra la ventana actual.
La barra inferior agrupa formato y tamaños en Texto, junto a Estilo y Alineación.
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
