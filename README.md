# ApexDesk GNOME — Fase 1

Esta etapa comprueba si los motores originales de ApexDesk pueden reutilizarse
para editar archivos DOCX/ODT **sin egui ni eframe**. Todavía **no hay interfaz gráfica**.

## Advertencia de licencia

El código fuente actual de ApexDesk está publicado bajo **PolyForm Strict**,
que no permite desarrollar derivados sin autorización. Este experimento usa
**únicamente la revisión anterior GPL-3.0-or-later**, fijada al commit
[`f35e6fa`](https://github.com/apexdesk/apexdesk/commit/f35e6fa10470156a4e55febdf9f8bc2920031b82).
No se utilizan cambios posteriores.

Las condiciones GPL-3.0-or-later se aplican a esta integración.

## Pruebas automáticas

GitHub Actions debe ejecutar el flujo
[`Fase 1 - motores ApexDesk sin GUI`](.github/workflows/rust-check.yml).

El flujo realiza:

1. Generación de un DOCX externo con python-docx y un ODT con odfpy.
2. Comprobación estructural de los archivos de prueba.
3. Compilación con Cargo y tres pruebas de integración.
4. Detección de dependencias transitivas de egui/eframe.
5. Apertura, edición, guardado en **copia** y reapertura de documentos.
6. Publicación de documentos de salida como artefactos.

La prueba comprueba dos párrafos, negrita inicial, título y que los archivos
de origen permanecen intactos. **No valida fidelidad visual** ni tablas,
imágenes, paginación o compatibilidad general con Office.

## Ejecutar localmente

Requiere Git, Rust estable y Python con `python-docx` y `odfpy`:

```sh
python scripts/make_fixtures.py
python scripts/verify_fixtures.py
cargo generate-lockfile
cargo test --locked
bash scripts/check-dependencies.sh
cargo run --locked -- smoke target/phase1-smoke
```

## Estado

Repositorio de investigación. Los tests Rust **no pueden considerarse
superados** hasta que conste una ejecución exitosa de GitHub Actions.
