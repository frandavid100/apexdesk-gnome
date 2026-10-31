// Optional corpus audit; not a test or release gate.
use std::{path::Path,fs};
use serde_json::{json,Value};
use wordcraft_doc::{Block,Blocks,Document,StoryRef};
use wordcraft_engine::Session;
fn blocks(doc:&Document,bs:&Blocks)->Vec<Value>{
 bs.iter().map(|b|match &**b{
  Block::Para(p)=>{
   let mut offset=0;let runs:Vec<Value>=p.runs.iter().map(|r|{
    let end=(offset+r.len).min(p.text.len());let text=p.text.get(offset..end).unwrap_or("").to_string();offset=end;
    json!({"text":text,"format":doc.styles.resolve_char(p.props.style.as_deref(),&r.props)})
   }).collect();
   json!({"kind":"para","text":p.text,"runs":runs,"format":doc.styles.resolve_para(&p.props),"objects":p.objects})
  },
  Block::Table(t)=>json!({"kind":"table","grid":t.grid,"props":t.props,"rows":t.rows.iter().map(|r|json!({"props":r.props,"cells":r.cells.iter().map(|c|json!({"props":c.props,"blocks":blocks(doc,&c.blocks)})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
 }).collect()
}
fn state(s:&mut Session)->Value{
 let page_count=s.layout().pages.len();
 json!({"pages":page_count,"body":blocks(&s.doc,&s.doc.body),"parts":s.doc.parts.iter().map(|(id,p)|json!({"id":id,"kind":p.kind,"text":s.doc.plain_text(StoryRef::Part(*id)),"blocks":blocks(&s.doc,&p.blocks)})).collect::<Vec<_>>(),
 "body_text":s.doc.plain_text(StoryRef::Body),"model":s.doc})
}
fn pictures(session:&mut Session,out:&Path,label:&str)->Result<(),Box<dyn std::error::Error>>{
 for page in 1..=session.layout().pages.len().min(2){
  let path=out.join(format!("{label}-{page}.png"));
  session.run("file.exportPng",&json!({"path":path.to_str().unwrap(),"page":page,"scale":1.0}))?;
 }Ok(())
}
fn run()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let input=Path::new(&args[1]);let out=Path::new(&args[2]);fs::create_dir_all(out)?;
 let mut s=wordcraft_gnome::open_document(input)?;let before=state(&mut s);
 fs::write(out.join("before.json"),serde_json::to_vec(&before)?)?;
 pictures(&mut s,out,"before")?;
 let ext=input.extension().unwrap().to_str().unwrap();let output=out.join(format!("saved.{ext}"));
 wordcraft_gnome::saving::save(&mut s,&output)?;
 let mut reopened=wordcraft_gnome::open_document(&output)?;let after=state(&mut reopened);
 fs::write(out.join("after.json"),serde_json::to_vec(&after)?)?;
 pictures(&mut reopened,out,"after")?;
 println!("AUDIT_FILE_OK");Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");std::process::exit(1);}}
