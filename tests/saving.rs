use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{saving, formatting::{self, Format}, selection};
use serde_json::json;

#[test]
fn edited_formatted_documents_reopen_as_docx_and_odt() {
    let dir = std::env::temp_dir().join(format!("wordcraft-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for ext in ["docx", "odt"] {
        let mut s = Session::new(Document::from_text("Hola ñáé"));
        selection::select_all(&mut s).unwrap();
        formatting::apply(&mut s, Format::Bold).unwrap();
        formatting::apply(&mut s, Format::FontSize(18)).unwrap();
        formatting::apply(&mut s, Format::AlignCenter).unwrap();
        let path = dir.join(format!("copia.{ext}"));
        saving::save(&mut s, &path).unwrap();
        assert!(!s.dirty);
        let mut reopened = wordcraft_gnome::open_document(&path).unwrap();
        assert_eq!(reopened.doc.plain_text(StoryRef::Body), "Hola ñáé");
        selection::select_all(&mut reopened).unwrap();
        let state = reopened.run("format.state", &json!({})).unwrap();
        assert_eq!(state["bold"], true);
        assert_eq!(state["size"], 18.0);
        let para = reopened.doc.para_at(&reopened.sel.focus).unwrap();
        assert_eq!(reopened.doc.styles.resolve_para(&para.props).align,
            wordcraft_doc::props::Align::Center);
        assert!(!reopened.layout().pages.is_empty());
        // A subsequent save must update the same copy.
        reopened.run("text.insert", &json!({"text":"Segunda versión"})).unwrap();
        saving::save(&mut reopened, &path).unwrap();
        let again = wordcraft_gnome::open_document(&path).unwrap();
        assert_eq!(again.doc.plain_text(StoryRef::Body), "Segunda versión");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_save_does_not_clear_pending_changes_or_change_the_path() {
    let mut s = Session::new(Document::from_text("Original"));
    s.run("text.insert", &json!({"text":"Cambio"})).unwrap();
    let previous = s.path.clone();
    let dir = std::env::temp_dir().join(format!("missing-wordcraft-{}", std::process::id()));
    assert!(saving::save(&mut s, &dir.join("missing/copy.docx")).is_err());
    assert!(s.dirty);
    assert_eq!(s.path, previous);
    assert!(saving::save(&mut s, std::path::Path::new("unexpected.pdf")).is_err());
}
