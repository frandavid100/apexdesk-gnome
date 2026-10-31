use std::{sync::Arc,path::PathBuf};
use wordcraft_doc::{Document,Block,Blocks,Paragraph,Part,PartKind,StoryRef,Table,Row,Cell,InlineObject};
use wordcraft_doc::section::{HeaderSet,SectionProps};
use wordcraft_engine::Session;
fn paragraph(text:&str)->Arc<Block>{Arc::new(Block::Para(Paragraph::with_text(text,Default::default())))}
fn table(header:bool,nested:bool)->Arc<Block>{
 let mut rows=vec![Row{cells:vec![Cell::with_text("First")],..Default::default()},Row{cells:vec![Cell::with_text("Second")],..Default::default()}];
 rows[0].props.header=header;
 if nested{rows[1].cells[0].blocks=vec![table(false,false)];}
 let mut t=Table{grid:vec![120.0],rows,..Default::default()};t.props.look.header_row=true;Arc::new(Block::Table(t))
}
fn roundtrip(d:&Document)->Document{
 let before=d.clone();let bytes=wordcraft_engine::io::save_bytes("test.odt",d).unwrap();
 assert_eq!(*d,before,"export must not alter the live document");
 wordcraft_engine::io::open_bytes("test.odt",&bytes).unwrap()
}
fn count_tables(bs:&Blocks)->usize{bs.iter().map(|b|match &**b{Block::Para(_)=>0,Block::Table(t)=>1+t.rows.iter().flat_map(|r|&r.cells).map(|c|count_tables(&c.blocks)).sum::<usize>()}).sum()}
fn id_text(d:&Document,id:Option<u32>)->String{d.plain_text(StoryRef::Part(id.expect("page part preserved")))}
#[test]
fn all_six_page_parts_keep_text_format_nested_tables_and_images(){
 let mut d=Document::from_text("Body");d.settings.even_odd_headers=true;d.last_section.title_page=true;
 for id in 1..=6{let mut p=Paragraph::with_text(&format!("Slot {id}"),Default::default());p.runs[0].props.bold=Some(true);d.parts.insert(id,Part{kind:if id<=3{PartKind::Header}else{PartKind::Footer},blocks:vec![Arc::new(Block::Para(p))],body:Default::default()});}
 d.last_section.headers=HeaderSet{default:Some(1),even:Some(2),first:Some(3)};
 d.last_section.footers=HeaderSet{default:Some(4),even:Some(5),first:Some(6)};
 d.parts.get_mut(&4).unwrap().blocks.push(table(false,true));
 let png:Vec<u8>=vec![137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,0,0,0,3,0,0,0,3,8,2,0,0,0,217,74,34,232,0,0,0,23,73,68,65,84,120,156,99,188,35,34,194,192,192,192,192,192,192,196,0,3,8,22,0,29,176,1,10,58,105,173,217,0,0,0,0,73,69,78,68,174,66,96,130];d.media.insert("logo.png".into(),Arc::new(png.clone()));
 let mut picture=Paragraph::new();picture.text="\u{fffc}".into();picture.runs.push(wordcraft_doc::Run{len:3,props:Default::default()});
 picture.objects.push(InlineObject::Image{media:"logo.png".into(),w:12.0,h:12.0,alt:"Header logo".into(),float:Default::default(),crop:[0.0;4],ole:None});
 d.parts.get_mut(&1).unwrap().blocks.push(Arc::new(Block::Para(picture)));
 let out=roundtrip(&d);assert!(out.media.values().any(|b|b.as_slice()==png.as_slice()));let h=out.last_section.headers;let f=out.last_section.footers;
 for(id,slot)in [(1,h.default),(2,h.even),(3,h.first),(4,f.default),(5,f.even),(6,f.first)]{
  assert!(id_text(&out,slot).contains(&format!("Slot {id}")));
  let part=&out.parts[&slot.unwrap()];let p=part.blocks[0].as_para().unwrap();
  assert!(out.styles.resolve_char(p.props.style.as_deref(),&p.runs[0].props).bold);
 }
 assert!(out.last_section.title_page);assert!(out.settings.even_odd_headers);
 assert_eq!(count_tables(&out.parts[&f.default.unwrap()].blocks),2);
}
#[test]
fn repeating_header_is_independent_of_first_row_styling(){
 for repeated in [false,true]{
  let mut d=Document::from_text("Body");d.body.push(table(repeated,true));let out=roundtrip(&d);
  let t=out.body.iter().find_map(|b|b.as_table()).unwrap();
  assert_eq!(t.rows[0].props.header,repeated);
  let nested=t.rows[1].cells[0].blocks.iter().find_map(|b|b.as_table()).unwrap();
  assert!(!nested.rows[0].props.header);
 }
}
#[test]
fn linked_headers_survive_section_boundaries_without_new_body_text(){
 let mut d=Document::from_text("First section\nSecond section");
 let mut first=SectionProps::default();first.headers.default=Some(1);
 if let Block::Para(p)=Arc::make_mut(&mut d.body[0]){p.section=Some(Box::new(first));}
 d.parts.insert(1,Part{kind:PartKind::Header,blocks:vec![paragraph("Shared header")],body:Default::default()});
 let out=roundtrip(&d);assert_eq!(out.plain_text(StoryRef::Body),d.plain_text(StoryRef::Body));
 for(_,s)in out.sections(){assert_eq!(id_text(&out,s.headers.default),"Shared header");}
}
#[test]
fn empty_documents_do_not_gain_page_parts(){
 let out=roundtrip(&Document::new());assert!(out.parts.is_empty());assert!(out.last_section.headers.default.is_none());assert!(out.last_section.footers.default.is_none());
}
fn temp()->PathBuf{let p=std::env::temp_dir().join(format!("leaf-odt-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));std::fs::create_dir_all(&p).unwrap();p}
#[test]
fn save_failure_and_success_preserve_selection_and_history(){
 let root=temp();let mut s=Session::new(Document::from_text("Original"));
 s.run("text.insert",&serde_json::json!({"text":"Edited "})).unwrap();
 let sel=s.sel.clone();let depth=s.undo_depth();let doc=s.doc.clone();let rev=s.rev();
 std::fs::write(root.join("not-a-directory"),b"keep").unwrap();
 assert!(wordcraft_gnome::saving::save(&mut s,&root.join("not-a-directory/fail.odt")).is_err());
 assert_eq!(s.doc,doc);assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth);assert_eq!(s.rev(),rev);assert!(s.dirty);assert!(s.path.is_none());
 let path=root.join("saved.odt");wordcraft_gnome::saving::save(&mut s,&path).unwrap();
 assert_eq!(s.path.as_ref(),Some(&path));assert!(!s.dirty);assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth);assert_eq!(s.rev(),rev);
 std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dormant_variants_do_not_enable_first_or_even_page_modes(){
 let mut d=Document::from_text("Body");
 d.last_section.headers=HeaderSet{default:Some(1),first:Some(2),even:Some(3)};
 for (id,text) in [(1,"Default"),(2,"Dormant first"),(3,"Dormant even")]{
  d.parts.insert(id,Part{kind:PartKind::Header,blocks:vec![paragraph(text)],body:Default::default()});
 }
 let out=roundtrip(&d);assert!(!out.last_section.title_page);assert!(!out.settings.even_odd_headers);
 assert_eq!(id_text(&out,out.last_section.headers.default),"Default");
}

#[test]
fn unused_page_style_parts_survive_without_becoming_visible(){
 let mut d=Document::from_text("Body");
 d.parts.insert(42,Part{kind:PartKind::Footer,blocks:vec![paragraph("Unused footer")],body:Default::default()});
 let out=roundtrip(&d);
 assert!(out.parts.keys().any(|id|out.plain_text(StoryRef::Part(*id))=="Unused footer"));
 assert!(out.last_section.footers.default.map(|id|out.plain_text(StoryRef::Part(id)).is_empty()).unwrap_or(true));
}
