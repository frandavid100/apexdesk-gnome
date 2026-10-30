//! PDF is an export: it never becomes the editable document's save destination.
use std::path::Path;
use wordcraft_engine::Session;
pub fn export(s:&mut Session,path:&Path)->Result<(),String>{
    if !path.extension().and_then(|x|x.to_str()).is_some_and(|x|x.eq_ignore_ascii_case("pdf")){return Err("Incluye .pdf en el nombre del archivo".into());}
    s.run("file.exportPdf",&serde_json::json!({"path":path.to_str().ok_or("La ruta no es UTF-8")?,"markup":false})).map_err(|e|e.to_string())?;Ok(())
}
