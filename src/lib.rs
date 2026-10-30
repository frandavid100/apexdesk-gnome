//! Integración headless entre WordCraft y nuestra interfaz GNOME.
//! La lógica de apertura, paginación y renderizado no utiliza GTK ni egui.

use std::path::Path;
use wordcraft_doc::Document;
use wordcraft_engine::Session;

/// La vista contiene pixels RGBA en vez de widgets egui.
pub struct PageImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub total_pages: usize,
}

/// Documento de demostración de WordCraft: incluye texto, títulos y tabla.
pub fn sample_session() -> Session {
    Session::new(wordcraft_engine::sample::sample_document())
}

/// Utiliza el comando file.open, el mismo que emplea WordCraft.
pub fn open_document(path: &Path) -> Result<Session, String> {
    if !path.is_file() {
        return Err("El archivo no existe o no es un fichero normal".into());
    }
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !["docx", "odt"].iter().any(|ext| extension.eq_ignore_ascii_case(ext)) {
        return Err("Este prototipo admite únicamente DOCX y ODT".into());
    }
    let name = path.to_str().ok_or("La ruta del archivo no se puede representar como UTF-8")?;
    let mut session = Session::new(Document::new());
    session.run("file.open", &serde_json::json!({"path": name}))
        .map_err(|e| format!("WordCraft no pudo abrir el archivo: {e}"))?;
    Ok(session)
}

/// Rasterización fiel de una página, con el propio maquetador de WordCraft.
/// La escala se limita para evitar imágenes enormes en esta prueba inicial.
pub fn render_page(session: &mut Session, page_index: usize, scale: f32) -> Result<PageImage, String> {
    render_page_pixels(session, page_index, scale, false)
}

/// Native GTK texture data. Retains the renderer's premultiplied RGBA buffer
/// instead of copying and unpremultiplying every pixel on the UI thread.
pub fn render_page_premultiplied(session: &mut Session, page_index: usize, scale: f32) -> Result<PageImage, String> {
    render_page_pixels(session, page_index, scale, true)
}

fn render_page_pixels(session: &mut Session, page_index: usize, scale: f32, premultiplied: bool) -> Result<PageImage, String> {
    let scale = if scale.is_finite() { scale.clamp(0.15, 2.0) } else { 1.0 };
    let mut options=wordcraft_render::RenderOptions::default();
    options.display.marks=session.view.marks;
    let layout = session.layout();
    let count = layout.pages.len();
    let page = layout.pages.get(page_index).ok_or("La página solicitada no existe")?;
    // Es una prueba funcional, no un visor para archivos gigantes.
    let width = (page.w * scale).ceil();
    let height = (page.h * scale).ceil();
    if !width.is_finite() || !height.is_finite() || width * height > 12_000_000.0 {
        return Err("La página excede el límite de tamaño de esta vista preliminar".into());
    }
    let output = wordcraft_render::render_page(&session.doc, page, scale, &options);
    let rgba = if premultiplied { output.pixels } else { output.to_straight() };
    if output.width == 0 || output.height == 0 ||
        rgba.len() != output.width as usize * output.height as usize * 4 {
        return Err("WordCraft devolvió una imagen incompleta".into());
    }
    Ok(PageImage { width: output.width, height: output.height, rgba, total_pages: count })
}

pub mod viewer_layout;

pub mod selection;

pub mod editing;

pub mod formatting;

pub mod saving;

pub mod preferences;

pub mod tables;

pub mod styles;

pub mod caret_motion;

pub mod search;
