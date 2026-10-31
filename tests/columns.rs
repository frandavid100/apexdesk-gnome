use serde_json::json;
use wordcraft_doc::{Document,Pos,StoryRef};
use wordcraft_engine::{Session,cmd::page};
use wordcraft_gnome::{document_properties::{self,ColumnSetup,ColumnKind,ColumnScope},saving,open_document};
fn equal(count:u32)->ColumnSetup{ColumnSetup{count,space:18.0,separator:true,kind:ColumnKind::Equal,widths:Vec::new()}}
#[test]
fn selected_columns_preserve_text_pending_format_and_one_undo(){
    let mut s=Session::new(Document::from_text("Antes\nalpha beta gamma\nDespués"));
    s.run("select.range",&json!({"anchor":Pos::body(1,6),"focus":Pos::body(1,10)})).unwrap();
    let selected=s.selected_text();let before=s.doc.clone();let depth=s.undo_depth();
    assert!(document_properties::apply_columns(&mut s,&equal(2),ColumnScope::Selection).unwrap());
    assert_eq!(s.selected_text(),selected);assert_eq!(s.undo_depth(),depth+1);
    let counts:Vec<_>=s.doc.sections().iter().map(|(_,p)|p.columns.count).collect();assert_eq!(counts,vec![1,2,1]);
    let after=s.doc.clone();let sel=s.sel.clone();let depth=s.undo_depth();let rev=s.rev();
    let current=ColumnSetup::current(&s);assert!(!document_properties::apply_columns(&mut s,&current,ColumnScope::Selection).unwrap());assert_eq!(s.undo_depth(),depth);assert_eq!(s.rev(),rev);
    s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc,before);assert_eq!(s.selected_text(),selected);
    s.run("edit.redo",&json!({})).unwrap();assert_eq!(s.doc,after);assert_eq!(s.sel,sel);
    s.run("caret.docEnd",&json!({})).unwrap();s.run("format.bold",&json!({"value":true})).unwrap();let pending=s.pending.clone();assert!(pending.is_some());
    document_properties::apply_columns(&mut s,&equal(3),ColumnScope::Section).unwrap();assert_eq!(s.pending,pending);
}
#[test]
fn scope_is_explicit_and_invalid_layout_is_atomic(){
    let mut s=Session::new(Document::from_text("First\nSecond\nThird"));s.run("caret.set",&json!({"pos":Pos::body(1,0)})).unwrap();s.run("layout.break",&json!({"kind":"nextPage"})).unwrap();
    let original_sections=s.doc.sections().len();let other=s.doc.sections()[0].1.clone();
    document_properties::apply_columns(&mut s,&equal(2),ColumnScope::Section).unwrap();assert_eq!(s.doc.sections().len(),original_sections);assert_eq!(s.doc.sections()[0].1,&other);
    document_properties::apply_columns(&mut s,&equal(3),ColumnScope::Document).unwrap();assert!(s.doc.sections().iter().all(|(_,p)|p.columns.count==3));
    let before=s.doc.clone();let sel=s.sel.clone();let depth=s.undo_depth();let rev=s.rev();
    let invalid=ColumnSetup{space:300.0,..equal(12)};assert!(document_properties::apply_columns(&mut s,&invalid,ColumnScope::Document).is_err());
    assert_eq!(s.doc,before);assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth);assert_eq!(s.rev(),rev);
    assert!(document_properties::apply_columns(&mut s,&equal(2),ColumnScope::Selection).is_err());assert_eq!(s.doc,before);
    s.run("caret.docEnd",&json!({})).unwrap();let before=s.doc.clone();let depth=s.undo_depth();document_properties::apply_columns(&mut s,&equal(2),ColumnScope::Forward).unwrap();assert_eq!(s.doc.sections().len(),original_sections+1);assert_eq!(s.undo_depth(),depth+1);s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc,before);
}
#[test]
fn custom_columns_and_separator_roundtrip_in_docx_and_odt(){
    let root=std::env::temp_dir().join(format!("leaf-columns-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir_all(&root).unwrap();
    for kind in [ColumnKind::Equal,ColumnKind::Left,ColumnKind::Right,ColumnKind::Custom]{
        let mut s=Session::new(Document::from_text(&"Texto para comprobar las columnas y su conservación. ".repeat(120)));
        let tw=page::sect(&s).text_width();let setup=ColumnSetup{count:2,space:18.0,separator:true,kind,widths:vec![((tw-18.0)*0.4,18.0),((tw-18.0)*0.6,0.0)]};
        document_properties::apply_columns(&mut s,&setup,ColumnScope::Document).unwrap();let expected=page::sect(&s).columns;let text=s.doc.plain_text(StoryRef::Body);let layout=s.layout();assert!(!layout.pages.is_empty());
        for ext in ["docx","odt"]{
            let path=root.join(format!("{:?}.{ext}",kind));saving::save(&mut s,&path).unwrap();let reopened=open_document(&path).unwrap();let got=page::sect(&reopened).columns;
            assert_eq!(got.count,expected.count,"{kind:?} {ext}");assert_eq!(got.separator,expected.separator,"{kind:?} {ext}");assert!((got.space-expected.space).abs()<0.2,"{kind:?} {ext}");assert_eq!(reopened.doc.plain_text(StoryRef::Body),text);
            if ext=="docx"{assert_eq!(got.widths.len(),expected.widths.len(),"{kind:?} {ext}");for((w,g),(x,y))in got.widths.iter().zip(&expected.widths){assert!((w-x).abs()<0.2&&(g-y).abs()<0.2,"{kind:?} {ext}");}}else{assert!(got.widths.is_empty(),"upstream ODT export supports equal widths only");}
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
