use serde_json::json;
use wordcraft_doc::{Document,StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{styles::{self,Edit},selection};
fn session()->Session {Session::new(Document::from_text("Texto de prueba"))}
#[test]
fn full_style_list_and_preview_are_non_destructive(){
 let mut s=session();s.run("select.all",&json!({"whole":true})).unwrap();s.run("format.bold",&json!({"value":true})).unwrap();s.run("edit.undo",&json!({})).unwrap();
 let rev=s.rev();let sel=s.sel.clone();let dirty=s.dirty;let doc=s.doc.clone();let redo=s.can_redo();
 let all=styles::list(&s);assert!(all.len()>5);assert!(all.iter().any(|st|st.id=="Heading9"));
 let normal=styles::preview(&s,"Normal").unwrap();let heading=styles::preview(&s,"Heading1").unwrap();
 assert!(!normal.rgba.is_empty());assert_ne!(normal.rgba,heading.rgba);
 assert!(s.doc.same_content(&doc));assert_eq!(s.rev(),rev);assert_eq!(s.sel,sel);assert_eq!(s.dirty,dirty);assert_eq!(s.can_redo(),redo);
}
#[test]
fn create_rename_update_delete_and_history_use_the_same_popover_adapter(){
 let mut s=session();selection::select_all(&mut s).unwrap();
 styles::edit(&mut s,Edit::Apply("Heading1".into())).unwrap();
 let id=styles::edit(&mut s,Edit::Create("Mi estilo".into())).unwrap().unwrap();
 assert!(styles::list(&s).iter().any(|st|st.id==id));assert_eq!(styles::current(&s),Some(id.clone()));
 styles::edit(&mut s,Edit::Rename(id.clone(),"Renombrado".into())).unwrap();assert!(s.doc.styles.find("Renombrado").is_some());
 styles::edit(&mut s,Edit::Update(id.clone())).unwrap();styles::edit(&mut s,Edit::Delete(id.clone())).unwrap();assert!(s.doc.styles.get(&id).is_none());
 s.run("edit.undo",&json!({})).unwrap();assert!(s.doc.styles.get(&id).is_some());
 assert_eq!(s.doc.plain_text(StoryRef::Body),"Texto de prueba");
}
#[test]
fn duplicate_empty_names_and_builtin_deletion_leave_document_unchanged(){
 let mut s=session();let doc=s.doc.clone();let rev=s.rev();
 assert!(styles::edit(&mut s,Edit::Create(" ".into())).is_err());assert!(styles::edit(&mut s,Edit::Create("normal".into())).is_err());
 assert!(styles::edit(&mut s,Edit::Rename("Heading1".into(),"Normal".into())).is_err());assert!(styles::edit(&mut s,Edit::Delete("Normal".into())).is_err());
 assert!(s.doc.same_content(&doc));assert_eq!(s.rev(),rev);
}
#[test]
fn character_styles_apply_to_selected_text(){
 let mut s=session();selection::select_all(&mut s).unwrap();let st=styles::list(&s).into_iter().find(|st|st.kind==wordcraft_doc::styles::StyleKind::Character&&st.id!="DefaultParagraphFont").unwrap();
 styles::edit(&mut s,Edit::Apply(st.id.clone())).unwrap();assert_eq!(s.doc.para_at(&s.sel.anchor).unwrap().props_of_char(0).style.as_deref(),Some(st.id.as_str()));
}
