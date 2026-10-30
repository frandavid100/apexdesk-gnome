//! Table integration through WordCraft's public commands and selection model.
use serde_json::json;
use wordcraft_doc::Pos;
use wordcraft_engine::Session;

pub fn in_table(session: &Session) -> bool { session.sel.focus.path.cell().is_some() }

/// Return whether navigation changed content (Tab in the last cell adds a row).
/// In this engine revision text.tab is a mutating command even when it only
/// moves the selection. Pure caret commands avoid dirtying the document and
/// adding navigation-only undo steps; structural changes still use text.tab.
pub fn navigate(session: &mut Session, backwards: bool, literal: bool) -> Result<bool,String> {
    if !literal && let Some((tp,r,c))=session.sel.focus.path.cell() {
        let t=session.doc.table(session.sel.focus.story,&tp).ok_or("No hay tabla en esa posición")?;
        let row=t.rows.get(r).ok_or("No existe la fila")?;
        let target=if backwards {
            if c>0 {Some((r,c-1))}else if r>0 {Some((r-1,t.rows[r-1].cells.len().saturating_sub(1)))}else{return Ok(false);}
        } else if c+1<row.cells.len(){Some((r,c+1))}else if r+1<t.rows.len(){Some((r+1,0))}else{None};
        if let Some((nr,nc))=target {
            let mut prefix=tp.0.clone();prefix.extend([nr as u32,nc as u32]);
            let paths:Vec<_>=session.doc.para_paths(session.sel.focus.story).into_iter().filter(|p|p.0.starts_with(&prefix)).collect();
            let first=paths.first().ok_or("La celda no contiene un párrafo")?;
            let last=if backwards {first}else{paths.last().unwrap()};
            let anchor=Pos{story:session.sel.focus.story,path:first.clone(),off:0};
            let focus=Pos{story:anchor.story,path:last.clone(),off:if backwards{0}else{session.doc.para(anchor.story,last).map_or(0,|p|p.len())}};
            session.run("select.range",&json!({"anchor":anchor,"focus":focus})).map_err(|e|e.to_string())?;
            return Ok(false);
        }
    }
    let rev=session.rev();
    let command=if literal { "text.insertTab" } else if backwards { "text.backTab" } else { "text.tab" };
    session.run(command,&json!({})).map_err(|e|e.to_string())?;
    Ok(session.rev()!=rev)
}

pub fn selected_cells(session:&Session)->usize {
    if !in_table(session) { return 0; }
    wordcraft_engine::cmd::table::selected_cells(session).len()
}
