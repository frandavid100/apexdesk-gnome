//! Styles shared by the native popover and integration tests.
use serde_json::json;
use wordcraft_doc::{Document,styles::{Style,StyleKind}};
use wordcraft_engine::Session;
use crate::{PageImage,render_page_premultiplied};

pub fn list(session:&Session)->Vec<Style>{
    let mut styles:Vec<_>=session.doc.styles.styles.iter().filter(|s|matches!(s.kind,StyleKind::Paragraph|StyleKind::Character)).cloned().collect();
    styles.sort_by_key(|s|(s.id!="Normal",s.priority.unwrap_or(100),s.name.to_lowercase()));styles
}
pub fn current(session:&Session)->Option<String>{
    session.caret_props().style.or_else(||session.doc.para_at(&session.sel.focus).and_then(|p|p.props.style.clone())).or_else(||Some("Normal".into()))
}
#[derive(Clone,Debug)]
pub enum Edit{Apply(String),Update(String),Rename(String,String),Delete(String),Create(String)}
pub fn edit(session:&mut Session,edit:Edit)->Result<Option<String>,String>{
    let name_ok=|name:&str,except:Option<&str>|->Result<String,String>{
        let name=name.trim();if name.is_empty(){return Err("Escribe un nombre para el estilo".into());}
        if session.doc.styles.styles.iter().any(|s|Some(s.id.as_str())!=except && (s.name.eq_ignore_ascii_case(name)||s.id.eq_ignore_ascii_case(name))){return Err("Ya existe un estilo con ese nombre".into());}
        Ok(name.to_owned())
    };
    let (cmd,args)=match edit{
        Edit::Apply(id)=>("para.style",json!({"style":id})),
        Edit::Update(id)=>("styles.updateToMatch",json!({"style":id})),
        Edit::Delete(id)=>("styles.delete",json!({"style":id})),
        Edit::Rename(id,name)=>{let name=name_ok(&name,Some(&id))?;("styles.modify",json!({"style":id,"name":name}))},
        Edit::Create(name)=>{let name=name_ok(&name,None)?;
            let based=session.doc.para_at(&session.sel.focus).and_then(|p|p.props.style.clone()).unwrap_or_else(||"Normal".into());
            ("styles.create",json!({"name":name,"fromSelection":true,"basedOn":based}))},
    };
    let result=session.run(cmd,&args).map_err(|e|e.to_string())?;
    Ok(result["id"].as_str().map(str::to_owned))
}

/// Render a real style in an isolated sample: source selection, revision,
/// history and dirty state cannot be affected by a preview.
pub fn preview(source:&Session,id:&str)->Result<PageImage,String>{
    if !list(source).iter().any(|s|s.id==id){return Err("El estilo ya no existe".into());}
    let mut doc=Document::from_text("Aa Bb Cc · Texto de ejemplo");doc.styles=source.doc.styles.clone();
    let mut sample=Session::new(doc);sample.view.proofing=false;
    sample.run("select.all",&json!({"whole":true})).map_err(|e|e.to_string())?;
    sample.run("para.style",&json!({"style":id})).map_err(|e|e.to_string())?;
    let image=render_page_premultiplied(&mut sample,0,0.65)?;
    let mut x0=image.width;let mut y0=image.height;let mut x1=0;let mut y1=0;
    for y in 0..image.height{for x in 0..image.width{let i=((y*image.width+x)*4)as usize;let p=&image.rgba[i..i+4];
        if p[3]>0 && (p[0]<245||p[1]<245||p[2]<245){x0=x0.min(x);x1=x1.max(x);y0=y0.min(y);y1=y1.max(y);}
    }}
    if x0>x1{return Ok(PageImage{width:1,height:1,rgba:vec![255;4],total_pages:1});}
    x0=x0.saturating_sub(10);y0=y0.saturating_sub(10);x1=(x1+11).min(image.width);y1=(y1+11).min(image.height).min(y0+140);
    let w=x1-x0;let h=y1-y0;let mut rgba=Vec::with_capacity((w*h*4)as usize);
    for y in y0..y1 {let i=((y*image.width+x0)*4)as usize;rgba.extend_from_slice(&image.rgba[i..i+(w*4)as usize]);}
    Ok(PageImage{width:w,height:h,rgba,total_pages:1})
}
