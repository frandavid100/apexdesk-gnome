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
