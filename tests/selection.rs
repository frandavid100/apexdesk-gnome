use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::selection::{hit, set_caret, select_range, move_caret, select_all, copy_text, geometry};

#[test]
fn coordinates_reach_real_caret_and_selection() {
    let mut s = Session::new(Document::from_text("Alfa beta gamma."));
    let layout = s.layout();
    let page = layout.pages.first().expect("one page");
    let x = page.w / 2.0;
    let y = page.h / 2.0;
    let p = hit(&mut s, 0, x, y).expect("hit inside document");
    set_caret(&mut s, &p, false).unwrap();
    assert_eq!(s.sel.anchor, p);
    assert_eq!(s.sel.focus, p);
    assert!(geometry(&mut s).caret.is_some());
    move_caret(&mut s, "docStart", false).unwrap();
    let start = s.sel.focus.clone();
    move_caret(&mut s, "right", true).unwrap();
    assert_ne!(start, s.sel.focus);
    assert!(!geometry(&mut s).highlights.is_empty());
}

#[test]
fn selection_copies_actual_text_without_modifying_document() {
    let mut s = Session::new(Document::from_text("Alfa beta gamma."));
    let original = s.doc.plain_text(StoryRef::Body);
    select_all(&mut s).unwrap();
    assert!(geometry(&mut s).highlights.len() > 0);
    assert_eq!(copy_text(&mut s).unwrap(), original);
    assert_eq!(s.doc.plain_text(StoryRef::Body), original);
}

#[test]
fn arbitrary_anchor_focus_support_reverse_selection() {
    let mut s = Session::new(Document::from_text("Uno dos tres"));
    select_all(&mut s).unwrap();
    let anchor = s.sel.anchor.clone();
    let focus = s.sel.focus.clone();
    select_range(&mut s, &focus, &anchor).unwrap();
    assert_eq!(s.sel.anchor, focus);
    assert_eq!(s.sel.focus, anchor);
    assert!(geometry(&mut s).highlights.len() > 0);
}

#[test]
fn out_of_bounds_coordinates_cannot_corrupt_selection() {
    let mut s = Session::new(Document::from_text("Hola"));
    assert!(hit(&mut s, 99_999, 5.0, 5.0).is_none());
    assert!(hit(&mut s, 0, f32::NAN, 5.0).is_none());
    assert!(move_caret(&mut s, "invalid", false).is_err());
}

#[test]
fn selection_can_cross_wordcraft_pages() {
    let mut session = wordcraft_gnome::sample_session();
    let layout = session.layout();
    assert!(layout.pages.len() >= 2, "sample document should span pages");
    let p0 = &layout.pages[0];
    let p1 = &layout.pages[1];
    let a = hit(&mut session, 0, p0.w / 2.0, p0.h / 2.0).unwrap();
    let b = hit(&mut session, 1, p1.w / 2.0, p1.h / 2.0).unwrap();
    select_range(&mut session, &a, &b).unwrap();
    let selection = geometry(&mut session);
    assert!(selection.highlights.iter().any(|r| r.page == 0),
        "selection should highlight page 1");
    assert!(selection.highlights.iter().any(|r| r.page == 1),
        "selection should highlight page 2");
}
