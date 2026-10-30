use serde_json::json;
use wordcraft_engine::Session;
use wordcraft_doc::{Document,Pos};
use wordcraft_gnome::{pictures,document_properties::{self,PageSetup},pdf_export,saving,open_document};
fn folder()->std::path::PathBuf{let p=std::env::temp_dir().join(format!("wordcraft-tools-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir_all(&p).unwrap();p}
fn fixture(p:&std::path::Path)->std::path::PathBuf{let f=p.join("imagen.png");std::fs::write(&f,wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64")).unwrap()).unwrap();f}
#[test]
fn inline_picture_history_selection_and_roundtrips(){
    let dir=folder();let file=fixture(&dir);let mut s=Session::new(Document::from_text("Antes y después"));
    s.run("caret.set",&json!({"pos":Pos::body(0,6)})).unwrap();let depth=s.undo_depth();
    pictures::insert(&mut s,&file).unwrap();let (pos,w,h)=pictures::selected(&s).unwrap();assert!((w/h-2.0).abs()<0.01);
    assert!(matches!(wordcraft_engine::cmd::objects::object_selection(&s).unwrap().1,wordcraft_doc::para::InlineObject::Image{float,..}if float.wrap==wordcraft_doc::para::Wrap::Inline));
    assert_eq!(s.undo_depth(),depth+1);let rev=s.rev();pictures::select(&mut s,&pos).unwrap();assert_eq!(s.rev(),rev);
    pictures::resize(&mut s,w*2.0,h,true,true).unwrap();let (_,nw,nh)=pictures::selected(&s).unwrap();assert!((nw/nh-2.0).abs()<0.01);
    s.run("edit.undo",&json!({})).unwrap();assert_eq!(pictures::selected(&s).unwrap().1,w);
    s.run("edit.redo",&json!({})).unwrap();pictures::original(&mut s).unwrap();assert_eq!(pictures::selected(&s).unwrap().1,120.0);
    pictures::resize(&mut s,90.0,60.0,false,true).unwrap();pictures::replace(&mut s,&file).unwrap();assert_eq!(pictures::selected(&s).unwrap().1,90.0);
    for ext in ["docx","odt"]{
        let path=dir.join(format!("copia.{ext}"));saving::save(&mut s,&path).unwrap();let mut reopened=open_document(&path).unwrap();reopened.run("select.objects",&json!({"index":0})).unwrap();
        let (_,rw,rh)=pictures::selected(&reopened).unwrap();assert!((rw-90.0).abs()<0.2&& (rh-60.0).abs()<0.2);assert!(!reopened.doc.media.is_empty());
    }
    pictures::delete(&mut s).unwrap();assert!(pictures::selected(&s).is_none());s.run("edit.undo",&json!({})).unwrap();assert!(pictures::selected(&s).is_some());
    s.run("select.all",&json!({})).unwrap();let doc=s.doc.clone();assert!(pictures::delete(&mut s).is_err());assert_eq!(s.doc,doc);
    let bad=dir.join("rota.png");std::fs::write(&bad,b"not an image").unwrap();let depth=s.undo_depth();assert!(pictures::insert(&mut s,&bad).is_err());assert_eq!(s.doc,doc);assert_eq!(s.undo_depth(),depth);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn page_setup_preserves_sections_and_groups_undo(){
    let mut s=Session::new(Document::from_text("Primera\nSegunda"));s.run("caret.set",&json!({"pos":Pos::body(1,0)})).unwrap();s.run("layout.break",&json!({"kind":"nextPage"})).unwrap();
    s.run("layout.columns",&json!({"count":2,"space":20})).unwrap();assert!(s.doc.sections().len()>1);
    let sections:Vec<_>=s.doc.sections().into_iter().map(|(_,p)|p.clone()).collect();let sel=s.sel.clone();let doc=s.doc.clone();let depth=s.undo_depth();
    let setup=PageSetup{width:841.89,height:595.28,top:36.0,bottom:42.0,left:45.0,right:48.0};
    assert!(document_properties::apply(&mut s,setup,true).unwrap());assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth+1);
    for ((_,p),old) in s.doc.sections().into_iter().zip(&sections){assert_eq!(p.columns,old.columns);assert_eq!(p.header,old.header);assert!(p.landscape);assert_eq!(p.margin_top,36.0);}
    let applied=s.doc.clone();s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc,doc);assert_eq!(s.sel,sel);s.run("edit.redo",&json!({})).unwrap();assert_eq!(s.doc,applied);
    let invalid=PageSetup{left:900.0,..setup};let depth=s.undo_depth();assert!(document_properties::apply(&mut s,invalid,true).is_err());assert_eq!(s.doc,applied);assert_eq!(s.undo_depth(),depth);
    assert!(!document_properties::apply(&mut s,setup,true).unwrap());assert_eq!(s.undo_depth(),depth);
    let before=s.doc.sections()[0].1.clone();let changed=PageSetup{top:60.0,..setup};document_properties::apply(&mut s,changed,false).unwrap();assert_eq!(s.doc.sections()[0].1,&before);
}
#[test]
fn pdf_export_preserves_editable_destination_dirty_selection_and_history(){
    let dir=folder();let img=fixture(&dir);let mut s=Session::new(Document::from_text("Documento mínimo con imagen"));pictures::insert(&mut s,&img).unwrap();
    let path=dir.join("original.docx");saving::save(&mut s,&path).unwrap();pictures::resize(&mut s,90.0,60.0,false,true).unwrap();
    let (destination,doc,sel,dirty,depth,rev)=(s.path.clone(),s.doc.clone(),s.sel.clone(),s.dirty,s.undo_depth(),s.rev());
    let pdf=dir.join("salida.pdf");pdf_export::export(&mut s,&pdf).unwrap();let data=std::fs::read(&pdf).unwrap();assert!(data.starts_with(b"%PDF-"));assert!(data.len()>1000);
    assert_eq!(s.path,destination);assert_eq!(s.doc,doc);assert_eq!(s.sel,sel);assert_eq!(s.dirty,dirty);assert_eq!(s.undo_depth(),depth);assert_eq!(s.rev(),rev);
    assert!(pdf_export::export(&mut s,&dir.join("no.docx")).is_err());assert!(!dir.join("no.docx").exists());assert!(pdf_export::export(&mut s,&dir.join("missing/out.pdf")).is_err());
    for ext in ["docx","odt"]{let setup=PageSetup{width:841.89,height:595.28,top:36.0,bottom:36.0,left:40.0,right:40.0};document_properties::apply(&mut s,setup,true).unwrap();let file=dir.join(format!("pagina.{ext}"));saving::save(&mut s,&file).unwrap();let reopened=open_document(&file).unwrap();let p=PageSetup::current(&reopened);assert!((p.width-setup.width).abs()<0.2&& (p.left-40.0).abs()<0.2);}
    std::fs::remove_dir_all(dir).unwrap();
}
