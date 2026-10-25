use apex_io::Format;
use apexdesk_gnome_phase1::{
    assert_sample_invariants, edit_copy, open_document, run_smoke, sample_document,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "apexdesk-gnome-phase1-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn docx_and_odt_open_edit_save_and_preserve_formatting() {
    let dir = temp_dir();
    let report = run_smoke(&dir).expect("integración con los motores ApexDesk");
    assert_eq!(report.len(), 2);
    for ext in ["docx", "odt"] {
        let (original, _) = open_document(&dir.join(format!("original.{ext}"))).unwrap();
        let (edited, _) = open_document(&dir.join(format!("editado.{ext}"))).unwrap();
        assert_sample_invariants(&original, false).unwrap();
        assert_sample_invariants(&edited, true).unwrap();
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_overwrite_empty_search_and_wrong_extension() {
    let dir = temp_dir();
    let input = dir.join("input.docx");
    apex_io::save_doc(&sample_document(), &input, Format::Docx).unwrap();
    assert!(edit_copy(&input, &input, "ejemplo", "prueba").is_err());
    assert!(edit_copy(&input, &dir.join("copy.docx"), "", "x").is_err());
    assert!(edit_copy(&input, &dir.join("copy.docx"), "inexistente", "x").is_err());
    assert!(!dir.join("copy.docx").exists());
    assert!(edit_copy(&input, &dir.join("copy.odt"), "ejemplo", "x").is_err());
    let existing = dir.join("existing.docx");
    std::fs::write(&existing, b"NO SOBRESCRIBIR").unwrap();
    assert!(edit_copy(&input, &existing, "ejemplo", "x").is_err());
    assert_eq!(std::fs::read(&existing).unwrap(), b"NO SOBRESCRIBIR");
    std::fs::remove_dir_all(dir).unwrap();
}

/// Archivos de entrada generados mediante python-docx/odfpy (NO ApexDesk).
#[test]
fn foreign_files_preserve_basic_formatting() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let dir = temp_dir();
    for ext in ["docx", "odt"] {
        let input = fixtures.join(format!("externo.{ext}"));
        let output = dir.join(format!("salida.{ext}"));
        let original_bytes = std::fs::read(&input).unwrap();
        let (loaded, _) = open_document(&input).unwrap();
        assert_sample_invariants(&loaded, false).unwrap_or_else(|e| {
            let spans: Vec<_> = loaded.paragraphs[1].spans()
                .map(|(start, end, attrs)| (start, end, attrs.bold))
                .collect();
            panic!("Error leyendo {ext}: {e}; tramos = {spans:?}");
        });
        assert_eq!(loaded.meta.title, "Documento externo de prueba");
        assert_eq!(edit_copy(&input, &output, "ejemplo", "prueba").unwrap(), 1);
        let (edited, _) = open_document(&output).unwrap();
        assert_sample_invariants(&edited, true).unwrap();
        assert_eq!(edited.meta.title, loaded.meta.title);
        assert_eq!(std::fs::read(&input).unwrap(), original_bytes);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
