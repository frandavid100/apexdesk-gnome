use wordcraft_doc::{Document,StoryRef};
use wordcraft_engine::Session;
use serde_json::json;
#[test]
fn literal_search_replace_is_undoable() {
    let mut s=Session::new(Document::from_text("Hola ñ Hola"));
    let found=s.run("edit.find",&json!({"text":"Hola","special":false})).unwrap();
    assert_eq!(found["count"],2);
    s.run("edit.replaceAll",&json!({"text":"Hola","with":"Adiós","special":false})).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body),"Adiós ñ Adiós");
    s.run("edit.undo",&json!({})).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body),"Hola ñ Hola");
}
#[test]
fn engine_recovery_keeps_an_unsaved_document_and_discards_on_request() {
    let root=std::env::temp_dir().join(format!("gnome-recovery-{}",std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut s=Session::new(Document::new());
    s.enable_recovery(root.clone(),std::sync::Arc::new(||{})).unwrap();
    s.run("text.insert",&json!({"text":"Recuperar ñ"})).unwrap();
    let end=std::time::Instant::now()+std::time::Duration::from_secs(15);
    loop {
        s.recovery_tick();
        let records=std::fs::read_dir(&root).unwrap().filter_map(Result::ok).filter(|x|x.path().extension().is_some_and(|e|e=="json")).count();
        if s.recovery_ready() && !s.recovery_busy() && records>0 {break;}
        assert!(std::time::Instant::now()<end,"no recovery snapshot");
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    s.discard_recovery_copy();s.finish_recovery();
    drop(s);std::fs::remove_dir_all(root).unwrap();
}
