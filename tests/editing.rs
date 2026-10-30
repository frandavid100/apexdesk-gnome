use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::editing::{apply, Edit};
use wordcraft_gnome::selection::{select_all, move_caret};

fn body(s: &Session) -> String { s.doc.plain_text(StoryRef::Body) }

#[test]
fn inserting_unicode_text_and_undoing_is_lossless() {
    let mut s = Session::new(Document::from_text("Hola"));
    move_caret(&mut s, "docEnd", false).unwrap();
    assert!(apply(&mut s, Edit::Insert(" ñáé🙂".into())).unwrap());
    assert_eq!(body(&s), "Hola ñáé🙂");
    apply(&mut s, Edit::Undo).unwrap();
    assert_eq!(body(&s), "Hola");
    apply(&mut s, Edit::Redo).unwrap();
    assert_eq!(body(&s), "Hola ñáé🙂");
}

#[test]
fn selection_replacement_and_deletion() {
    let mut s = Session::new(Document::from_text("Texto original"));
    select_all(&mut s).unwrap();
    apply(&mut s, Edit::Insert("Nuevo".into())).unwrap();
    assert_eq!(body(&s), "Nuevo");
    apply(&mut s, Edit::Backspace).unwrap();
    assert_eq!(body(&s), "Nuev");
    move_caret(&mut s, "docStart", false).unwrap();
    apply(&mut s, Edit::Delete).unwrap();
    assert_eq!(body(&s), "uev");
}

#[test]
fn paragraph_break_and_clipboard_paste_change_layout() {
    let mut s = Session::new(Document::from_text("Inicio"));
    move_caret(&mut s, "docEnd", false).unwrap();
    apply(&mut s, Edit::Paragraph).unwrap();
    apply(&mut s, Edit::Paste("Segunda línea".into())).unwrap();
    assert!(body(&s).contains("Inicio"));
    assert!(body(&s).contains("Segunda línea"));
    assert!(s.layout().pages.len() >= 1);
}

#[test]
fn empty_input_is_a_noop() {
    let mut s = Session::new(Document::from_text("Hola"));
    assert!(!apply(&mut s, Edit::Insert(String::new())).unwrap());
    assert!(!apply(&mut s, Edit::Paste(String::new())).unwrap());
}

#[test]
fn lengthy_edit_changes_pagination_without_losing_content() {
    let mut s = Session::new(Document::from_text("Inicio"));
    let before = s.layout().pages.len();
    move_caret(&mut s, "docEnd", false).unwrap();
    let added = ("Texto de prueba suficientemente largo para ocupar una línea entera. ".repeat(3) + "\n").repeat(90);
    apply(&mut s, Edit::Paste(added)).unwrap();
    let after = s.layout().pages.len();
    assert!(after > before, "a large paste should grow the document beyond one page");
    assert!(body(&s).contains("Texto de prueba"));
}
