//! Prueba automática del motor de WordCraft, sin GTK ni interfaz.
use std::path::Path;
use wordcraft_gnome::{open_document, render_page, sample_session};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut session = if args.first().map(String::as_str) == Some("--file") {
        let path = args.get(1).ok_or("Uso: wordcraft-gnome-smoke [--file documento.docx] [--png salida.png]")?;
        open_document(Path::new(path))?
    } else {
        sample_session()
    };
    let page = render_page(&mut session, 0, 1.0)?;
    println!("OK: WordCraft generó {} páginas; la primera: {} × {} RGBA",
        page.total_pages, page.width, page.height);
    if let Some(position) = args.iter().position(|s| s == "--png") {
        let output = args.get(position + 1).ok_or("Falta la ruta después de --png")?;
        let layout = session.layout();
        let first = layout.pages.first().ok_or("El documento está vacío")?;
        let image = wordcraft_render::render_page(&session.doc, first, 1.0, &Default::default());
        let png = image.to_png();
        if png.is_empty() {
            return Err("No se pudo codificar la imagen PNG".into());
        }
        std::fs::write(output, png).map_err(|e| e.to_string())?;
        println!("PNG guardado: {output}");
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("ERROR: {err}");
        std::process::exit(1);
    }
}
