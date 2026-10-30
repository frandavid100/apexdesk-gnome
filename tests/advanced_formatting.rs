use wordcraft_doc::{Document,StoryRef};
use wordcraft_engine::Session;
use serde_json::json;
use wordcraft_gnome::selection::select_all;
fn session(text:&str)->Session{let mut s=Session::new(Document::from_text(text));select_all(&mut s).unwrap();s}
#[test]
fn character_effects_and_colors_survive_docx_round_trip(){
    let mut s=session("Alpha Beta");
    for(cmd,args)in[
        ("format.strikethrough",json!({"value":true})),("format.superscript",json!({"value":true})),
        ("format.underline",json!({"style":"wave"})),("format.underlineColor",json!({"color":"CC2200"})),
        ("format.color",json!({"color":"336699"})),("format.highlight",json!({"color":"yellow"})),
        ("format.shadow",json!({"value":true})),("format.changeCase",json!({"mode":"upper"})),
    ]{s.run(cmd,&args).unwrap();}
    let path=std::env::temp_dir().join(format!("gnome-advanced-{}.docx",std::process::id()));
    wordcraft_gnome::saving::save(&mut s,&path).unwrap();
    let mut r=wordcraft_gnome::open_document(&path).unwrap();select_all(&mut r).unwrap();
    let state=r.run("format.state",&json!({})).unwrap();
    assert_eq!(state["strike"],true);assert_eq!(state["superscript"],true);
    assert_eq!(state["color"],"336699");assert_eq!(state["underlineColor"],"CC2200");
    assert_eq!(state["highlight"],"FFFF00");assert_eq!(r.doc.plain_text(StoryRef::Body),"ALPHA BETA");
    std::fs::remove_file(path).unwrap();
}
#[test]
fn paragraph_controls_preserve_text_and_sort_is_undoable(){
    let mut s=session("zeta\nalfa");
    s.run("para.sort",&json!({"descending":false})).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body),"alfa\nzeta");
    s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc.plain_text(StoryRef::Body),"zeta\nalfa");
    select_all(&mut s).unwrap();
    for(cmd,args)in[
        ("para.indents",json!({"left":36,"right":12,"firstLine":-18})),
        ("para.spacing",json!({"before":6,"after":12})),("para.lineSpacing",json!({"exactly":18})),
        ("para.borders",json!({"kind":"outside","width":1,"color":"112233"})),
        ("para.shading",json!({"color":"DDEEFF"})),("para.keepNext",json!({"value":true})),
        ("para.tabs",json!({"set":[{"pos":72,"align":"left"}]})),
        ("para.multilevel",json!({"kind":"legal"})),("para.listLevel",json!({"level":1})),
    ]{s.run(cmd,&args).unwrap();}
    assert_eq!(s.doc.plain_text(StoryRef::Body),"zeta\nalfa");assert!(!s.layout().pages.is_empty());
    let p=s.doc.para_at(&s.sel.focus).unwrap();let p=s.doc.styles.resolve_para(&p.props);
    assert_eq!(p.space_before,6.0);assert_eq!(p.space_after,12.0);assert!(p.keep_next);
    assert!(p.numbering.is_some());
}
#[test]
fn styles_can_be_created_renamed_updated_deleted_and_undone(){
    let mut s=session("Estilo propio");s.run("format.bold",&json!({"value":true})).unwrap();
    let id=s.run("styles.create",&json!({"name":"Prueba GNOME","fromSelection":true})).unwrap()["id"].as_str().unwrap().to_string();
    s.run("styles.modify",&json!({"style":id,"name":"Renombrado"})).unwrap();
    s.run("styles.updateToMatch",&json!({"style":id})).unwrap();
    assert!(s.doc.styles.find("Renombrado").is_some());
    s.run("styles.delete",&json!({"style":id})).unwrap();assert!(s.doc.styles.find("Renombrado").is_none());
    s.run("edit.undo",&json!({})).unwrap();assert!(s.doc.styles.find("Renombrado").is_some());
    assert_eq!(s.doc.plain_text(StoryRef::Body),"Estilo propio");
}
#[test]
fn formatting_marks_change_pixels_without_editing_the_document(){
    let mut s=session("Uno dos\nTres");let text=s.doc.plain_text(StoryRef::Body);let rev=s.rev();let dirty=s.dirty;
    let before=wordcraft_gnome::render_page(&mut s,0,1.0).unwrap();
    s.run("view.marks",&json!({"value":true})).unwrap();
    let after=wordcraft_gnome::render_page(&mut s,0,1.0).unwrap();
    assert_ne!(before.rgba,after.rgba);assert_eq!(s.rev(),rev);assert_eq!(s.dirty,dirty);assert_eq!(s.doc.plain_text(StoryRef::Body),text);
}
#[test]
fn phonetic_guide_and_enclosure_commands_are_real_document_edits(){
    let mut s=session("AZ");s.run("format.enclose",&json!({"shape":"circle"})).unwrap();
    s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc.plain_text(StoryRef::Body),"AZ");
    select_all(&mut s).unwrap();s.run("format.phonetic",&json!({"ruby":"a zeta","size":8,"offset":0,"align":"center"})).unwrap();
    assert!(!s.layout().pages.is_empty());s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc.plain_text(StoryRef::Body),"AZ");
}
