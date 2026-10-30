//! Bridge entre la selección de WordCraft y la representación GTK.
//! El código es independiente de egui y de GTK.
use wordcraft_doc::{Pos, StoryRef};
use wordcraft_engine::Session;
use serde_json::json;

#[derive(Clone, Debug, PartialEq)]
pub struct Rect {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectionGeometry {
    pub highlights: Vec<Rect>,
    pub caret: Option<Rect>,
}

pub fn hit(session: &mut Session, page: usize, x: f32, y: f32) -> Option<Pos> {
    if !x.is_finite() || !y.is_finite() { return None; }
    session.layout().hit(page, x, y, StoryRef::Body)
}

/// Use WordCraft's caret command rather than modifying selection state from GTK.
pub fn set_caret(session: &mut Session, pos: &Pos, extend: bool) -> Result<(), String> {
    session.run("caret.set", &json!({"pos": pos, "extend": extend}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn select_range(session: &mut Session, anchor: &Pos, focus: &Pos) -> Result<(), String> {
    if anchor.story == focus.story && anchor.path.cell().is_some() && anchor.path.cell() != focus.path.cell() {
        set_caret(session,anchor,false)?;
        return set_caret(session,focus,true);
    }
    session.run("select.range", &json!({"anchor": anchor, "focus": focus}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn move_caret(session: &mut Session, action: &str, extend: bool) -> Result<(), String> {
    let id = match action {
        "left" => "caret.left", "right" => "caret.right",
        "up" => "caret.up", "down" => "caret.down",
        "home" => "caret.home", "end" => "caret.end",
        "docStart" => "caret.docStart", "docEnd" => "caret.docEnd",
        "wordLeft" => "caret.wordLeft", "wordRight" => "caret.wordRight",
        _ => return Err(format!("Acción de navegación desconocida: {action}")),
    };
    session.run(id, &json!({"extend": extend})).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn select_all(session: &mut Session) -> Result<(), String> {
    session.run("select.all", &json!({"whole": true})).map_err(|e| e.to_string())?;
    Ok(())
}

/// In addition to plain text, WordCraft also provides HTML and MathML.
pub fn copy_text(session: &mut Session) -> Result<String, String> {
    let copied = session.run("edit.copy", &json!({})).map_err(|e| e.to_string())?;
    Ok(copied.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_owned())
}

pub fn geometry(session: &mut Session) -> SelectionGeometry {
    let lay = session.layout();
    let selection = session.sel.clone();
    let mut out = SelectionGeometry::default();
    if let Some((pos,wordcraft_doc::para::InlineObject::Image{..}))=wordcraft_engine::cmd::objects::object_selection(session){
        if let Some(hit)=lay.object(&pos,session.page_hint){let r=hit.bounds();out.highlights.push(Rect{page:hit.page,x:r.x as f64,y:r.y as f64,width:r.w as f64,height:r.h as f64});return out;}
    }
    if let Some(range)=session.cell_range() {
        let cells=session.doc.table(range.story,&range.table).map(|t|range.cells(t)).unwrap_or_default();
        for (page, p) in lay.pages.iter().enumerate() {
            for item in &p.items {
                if let wordcraft_engine::layout::Placed::Cell {rect,table,row,cell,story,covered,..}=item {
                    if !covered && *story==range.story && *table==range.table && cells.contains(&(*row,*cell)) {
                        out.highlights.push(Rect {page,x:rect.x as f64,y:rect.y as f64,width:rect.w as f64,height:rect.h as f64});
                    }
                }
            }
        }
        return out;
    }
    if !selection.is_collapsed() {
        for (page, rect) in lay.selection_rects(&session.doc, &selection.anchor, &selection.focus, session.page_hint) {
            out.highlights.push(Rect {
                page, x: rect.x as f64, y: rect.y as f64,
                width: rect.w as f64, height: rect.h as f64,
            });
        }
    } else if let Some(c) = lay.caret_on(&selection.focus, session.page_hint) {
        out.caret = Some(Rect {
            page: c.page, x: c.x as f64, y: c.top as f64,
            width: c.width.max(1.15) as f64, height: c.height.max(1.0) as f64,
        });
    }
    out
}

