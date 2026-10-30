//! Table integration through WordCraft's public commands and selection model.
use serde_json::json;
use wordcraft_engine::Session;

pub fn in_table(session: &Session) -> bool { session.sel.focus.path.cell().is_some() }

/// Return whether navigation changed content (Tab in the last cell adds a row).
pub fn navigate(session: &mut Session, backwards: bool, literal: bool) -> Result<bool,String> {
    let rev=session.rev();
    let command=if literal { "text.insertTab" } else if backwards { "text.backTab" } else { "text.tab" };
    session.run(command,&json!({})).map_err(|e|e.to_string())?;
    Ok(session.rev()!=rev)
}

pub fn selected_cells(session:&Session)->usize {
    if !in_table(session) { return 0; }
    wordcraft_engine::cmd::table::selected_cells(session).len()
}
