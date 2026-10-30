//! Inline picture operations through the public engine commands.
use std::path::Path;
use serde_json::json;
use wordcraft_doc::{Pos, para::InlineObject};
use wordcraft_engine::{Session, cmd::objects};

pub fn selected(s:&Session)->Option<(Pos,f32,f32)> {
    let (pos,obj)=if s.sel.is_collapsed(){objects::selected(s)?}else{
        let (p,o)=objects::object_selection(s)?;(p,o.clone())
    };
    match obj {InlineObject::Image{w,h,..}=>Some((pos,w,h)),_=>None}
}
pub fn select(s:&mut Session,pos:&Pos)->Result<(),String>{
    if !matches!(s.doc.para_at(pos).and_then(|p|p.object_at(pos.off)),Some(InlineObject::Image{..})) {return Err("Selecciona una imagen".into());}
    crate::selection::select_range(s,pos,&Pos{off:pos.off+wordcraft_doc::para::OBJ.len_utf8(),..pos.clone()})
}
pub fn insert(s:&mut Session,path:&Path)->Result<(),String>{
    let p=path.to_str().ok_or("La ruta no es UTF-8")?;
    let snapshot=s.edit_snapshot();
    s.run("insert.picture",&json!({"path":p})).map_err(|e|e.to_string())?;
    let Some((p,_,_))=selected(s)else{s.restore(snapshot);return Err("No se admite insertar una imagen dentro de un lienzo de dibujo".into());};select(s,&p)
}
pub fn replace(s:&mut Session,path:&Path)->Result<(),String>{
    selected(s).ok_or("Selecciona una imagen")?;
    s.run("picture.change",&json!({"path":path.to_str().ok_or("La ruta no es UTF-8")?})).map_err(|e|e.to_string())?;Ok(())
}
pub fn resize(s:&mut Session,width:f32,height:f32,lock:bool,width_changed:bool)->Result<(),String>{
    selected(s).ok_or("Selecciona una imagen")?;
    if !width.is_finite()||!height.is_finite()||!(1.0..=4000.0).contains(&width)||!(1.0..=4000.0).contains(&height){return Err("El tamaño debe estar entre 1 y 4000 puntos".into());}
    // The engine preserves proportions when only one dimension is supplied.
    let args=if !lock{json!({"width":width,"height":height,"lockAspect":false})}else if width_changed{json!({"width":width})}else{json!({"height":height})};
    s.run("picture.size",&args).map_err(|e|e.to_string())?;Ok(())
}
pub fn original(s:&mut Session)->Result<(),String>{
    selected(s).ok_or("Selecciona una imagen")?;s.run("picture.size",&json!({"original":true})).map_err(|e|e.to_string())?;Ok(())
}
pub fn delete(s:&mut Session)->Result<(),String>{
    let (p,_,_)=selected(s).ok_or("Selecciona una imagen")?;select(s,&p)?;s.run("text.delete",&json!({})).map_err(|e|e.to_string())?;Ok(())
}
