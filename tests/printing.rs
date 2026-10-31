use wordcraft_engine::Session;
use wordcraft_doc::Document;
use wordcraft_gnome::printing::PrintFile;

#[test]
fn printing_snapshots_unsaved_changes_and_cleans_private_files() {
    let mut session = Session::new(Document::from_text("Original"));
    session.run("text.insert", &serde_json::json!({"text":"Unsaved changes "})).unwrap();
    let document = session.doc.clone(); let selection = session.sel.clone();
    let (revision, history, dirty, path) = (session.rev(), session.undo_depth(), session.dirty, session.path.clone());
    let first = PrintFile::prepare(session.doc.clone()).unwrap();
    let second = PrintFile::prepare(session.doc.clone()).unwrap();
    assert_ne!(first.directory(), second.directory());
    assert!(std::fs::read(first.path()).unwrap().starts_with(b"%PDF-"));
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(std::fs::metadata(first.directory()).unwrap().permissions().mode() & 0o777, 0o700); }
    assert_eq!(session.doc, document); assert_eq!(session.sel, selection);
    assert_eq!((session.rev(), session.undo_depth(), session.dirty, session.path.clone()), (revision, history, dirty, path));
    let directory = first.directory().to_owned(); drop(first); assert!(!directory.exists());
    assert!(second.path().exists());
}
