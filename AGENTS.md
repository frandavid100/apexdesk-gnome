# Continuidad del proyecto

Antes de trabajar, leer `PROJECT_STATUS.md`, `ROADMAP.md` y `UPDATING_WORDCRAFT.md`.
La rama de desarrollo documentada es `wordcraft-saving`; comprobar rama, estado local
y commits remotos antes de editar. No descartar cambios locales preexistentes.

La interfaz es nativa GTK4/libadwaita. El documento, renderizado y operaciones de edición
pertenecen a WordCraft. Usar sus API y comandos públicos; mantener las dependencias fijadas
y los cambios de GNOME separados del motor. Consultar documentación oficial de GNOME
cuando se diseñen o cambien patrones de interfaz.

Para validar cambios funcionales, ejecutar los tests relevantes y compilar la interfaz.
La CI añade capturas y pruebas de pestañas/ventanas. Para distribuir:

```sh
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
```

No distribuir `target/debug` aunque se eliminen sus símbolos. Conservar panic unwinding:
el motor utiliza catch_unwind. Comparar rendimiento en condiciones equivalentes y
no extrapolar un benchmark de muestra a toda la aplicación.

Después de cada entrega o decisión importante, actualizar PROJECT_STATUS y ROADMAP con
lo implementado, validación, revisión y pendientes. Distinguir pruebas automáticas de
confirmaciones del usuario. Una propuesta de la hoja de ruta no es una tarea ya autorizada.
Los cambios solo de documentación no requieren compilar otra vez ni regenerar el ZIP.
