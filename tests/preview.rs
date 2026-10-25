use apex_doc::{Document, ParaKind, Paragraph, Pos, Selection};
use apexdesk_gnome_phase1::preview::{BlockKind, from_document};
use apexdesk_gnome_phase1::open_document;

#[test]
fn preview_preserves_direct_bold_and_italic() {
    let mut doc = Document::from_paragraphs(vec![Paragraph::new("Hola mundo")]);
    let sel = Selection { anchor: Pos::new(0, 0), cursor: Pos::new(0, 4) };
    doc.format_range(sel, &|a| { a.bold = true; a.italic = true; });
    let p = from_document(&doc);
    assert_eq!(p.paragraphs.len(), 1);
    assert_eq!(p.paragraphs[0].spans[0].text, "Hola");
    assert!(p.paragraphs[0].spans[0].bold);
    assert!(p.paragraphs[0].spans[0].italic);
    assert!(!p.paragraphs[0].spans[1].bold);
}

#[test]
fn preview_marks_headings() {
    let mut doc = Document::from_paragraphs(vec![Paragraph::new("Título")]);
    doc.paragraphs[0].style.kind = ParaKind::Heading(2);
    let preview = from_document(&doc);
    assert_eq!(preview.paragraphs[0].kind, BlockKind::Heading(2));
}

#[test]
fn preview_uses_named_odt_character_styles() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/externo.odt");
    let (doc, _) = open_document(&fixture).unwrap();
    let p = from_document(&doc);
    assert_eq!(p.paragraphs[1].spans[0].text, "Texto");
    assert!(p.paragraphs[1].spans[0].bold, "Negrita del estilo importado");
}
