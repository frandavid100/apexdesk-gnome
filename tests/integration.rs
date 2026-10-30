use wordcraft_doc::Document;
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, render_page, sample_session};

#[test]
fn sample_is_paginated_and_rendered() {
    let mut session = sample_session();
    let image = render_page(&mut session, 0, 1.0).expect("WordCraft genera una página");
    assert!(image.total_pages >= 1);
    assert!(image.width > 200);
    assert!(image.height > 200);
    assert_eq!(image.rgba.len(), image.width as usize * image.height as usize * 4);
    assert!(image.rgba.chunks_exact(4).any(|px| px[0] < 170 || px[1] < 170 || px[2] < 170), "Hay contenido visible");
    assert!(render_page(&mut session, image.total_pages, 1.0).is_err());
}

#[test]
fn docx_engine_round_trip() {
    let doc = Document::from_text("Hola, WordCraft GNOME");
    let path = std::env::temp_dir().join(format!("wc-gnome-{}-{}.docx", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    wordcraft_engine::io::save_path(&path, &doc).expect("Guardar DOCX con WordCraft");
    let mut session = open_document(&path).expect("Abrir DOCX con WordCraft");
    assert!(session.doc.plain_text(wordcraft_doc::StoryRef::Body).contains("Hola, WordCraft GNOME"));
    assert!(render_page(&mut session, 0, 0.8).is_ok());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn reject_unsupported_file_types() {
    assert!(open_document(std::path::Path::new("no-existe.pptx")).is_err());
    let mut session = Session::new(Document::from_text("Prueba"));
    assert!(render_page(&mut session, 0, f32::NAN).is_ok());
}

#[test]
fn thumbnails_render_at_their_actual_small_scale() {
    let mut session = sample_session();
    let full = render_page(&mut session, 0, 1.0).unwrap();
    let thumb = render_page(&mut session, 0, 0.28).unwrap();
    // A 28% thumbnail must not silently become a 40% page.
    assert!(thumb.width as f32 <= full.width as f32 * 0.30);
    assert!(thumb.height as f32 <= full.height as f32 * 0.30);
}
