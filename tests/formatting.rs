use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::formatting::{apply, Format};
use wordcraft_gnome::selection::{select_all, move_caret};

fn text(s: &Session) -> String { s.doc.plain_text(StoryRef::Body) }

#[test]
fn inline_formatting_keeps_content_and_works_with_undo() {
    let mut s = Session::new(Document::from_text("Texto de ejemplo"));
    select_all(&mut s).unwrap();
    for action in [Format::Bold, Format::Italic, Format::Underline, Format::FontSize(18)] {
        apply(&mut s, action).unwrap();
        assert_eq!(text(&s), "Texto de ejemplo");
    }
    s.run("edit.undo", &serde_json::json!({})).unwrap();
    assert_eq!(text(&s), "Texto de ejemplo");
}

#[test]
fn paragraph_styles_and_alignment_work_in_real_wordcraft() {
    let mut s = Session::new(Document::from_text("Primer párrafo"));
    move_caret(&mut s, "docStart", false).unwrap();
    for action in [Format::Heading1, Format::Heading2, Format::Heading3,
                   Format::Normal, Format::AlignLeft, Format::AlignCenter,
                   Format::AlignRight, Format::Justify] {
        apply(&mut s, action).unwrap();
        assert_eq!(text(&s), "Primer párrafo");
        assert!(!s.layout().pages.is_empty());
    }
}

#[test]
fn selected_document_remains_editable_after_formatting() {
    let mut s = Session::new(Document::from_text("Hola"));
    select_all(&mut s).unwrap();
    apply(&mut s, Format::Bold).unwrap();
    s.run("text.insert", &serde_json::json!({"text":"Adiós"})).unwrap();
    assert_eq!(text(&s), "Adiós");
}

#[test]
fn invalid_font_sizes_are_rejected() {
    let mut s = Session::new(Document::from_text("Hola"));
    assert!(apply(&mut s, Format::FontSize(0)).is_err());
    assert!(apply(&mut s, Format::FontSize(100)).is_err());
}

#[test]
fn formatting_state_is_observable_and_undoable() {
    let mut s = Session::new(Document::from_text("Texto de ejemplo"));
    select_all(&mut s).unwrap();
    let state = |s: &mut Session| s.run("format.state", &serde_json::json!({})).unwrap();
    assert_eq!(state(&mut s)["bold"], false);
    apply(&mut s, Format::Bold).unwrap();
    assert_eq!(state(&mut s)["bold"], true);
    apply(&mut s, Format::Italic).unwrap();
    assert_eq!(state(&mut s)["italic"], true);
    apply(&mut s, Format::Underline).unwrap();
    assert_eq!(state(&mut s)["underline"], true);
    s.run("edit.undo", &serde_json::json!({})).unwrap();
    assert_eq!(state(&mut s)["underline"], false);
    assert_eq!(text(&s), "Texto de ejemplo");
}

#[test]
fn font_fractional_size_and_lists_survive_save_and_reopen() {
    let mut s = Session::new(Document::from_text("Primero\nSegundo"));
    select_all(&mut s).unwrap();
    wordcraft_gnome::formatting::font(&mut s, "DejaVu Sans").unwrap();
    wordcraft_gnome::formatting::size(&mut s, 13.5).unwrap();
    apply(&mut s, Format::Bullets).unwrap();
    apply(&mut s, Format::Indent).unwrap();
    apply(&mut s, Format::Outdent).unwrap();
    let path = std::env::temp_dir().join(format!("wordcraft-basic-{}.docx", std::process::id()));
    wordcraft_gnome::saving::save(&mut s, &path).unwrap();
    let mut reopened = wordcraft_gnome::open_document(&path).unwrap();
    select_all(&mut reopened).unwrap();
    let state = reopened.run("format.state", &serde_json::json!({})).unwrap();
    assert_eq!(state["font"], "DejaVu Sans");
    assert_eq!(state["size"], 13.5);
    let para = reopened.doc.para_at(&reopened.sel.focus).unwrap();
    assert!(reopened.doc.styles.resolve_para(&para.props).numbering.is_some());
    assert_eq!(text(&reopened), "Primero\nSegundo");
    apply(&mut reopened, Format::Numbering).unwrap();
    assert!(reopened.can_undo());
    reopened.run("edit.undo", &serde_json::json!({})).unwrap();
    assert!(reopened.can_redo());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_free_sizes_and_empty_fonts_are_rejected_without_edits() {
    let mut s = Session::new(Document::from_text("Conservar"));
    let rev = s.rev();
    for size in [f64::NAN, f64::INFINITY, 0.0, -4.0, 2000.0] {
        assert!(wordcraft_gnome::formatting::size(&mut s, size).is_err());
    }
    assert!(wordcraft_gnome::formatting::font(&mut s, "  ").is_err());
    assert_eq!(s.rev(), rev);
    assert_eq!(text(&s), "Conservar");
}
