//! Mutaciones del documento ejecutadas exclusivamente por WordCraft.
//! No usamos GtkTextBuffer para remaquetar DOCX.
use serde_json::json;
use wordcraft_engine::Session;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    Insert(String),
    Paragraph,
    LineBreak,
    Backspace,
    Delete,
    Paste(String),
    Undo,
    Redo,
}

/// Devuelve true solo cuando cambia la revisión (no al editar en solo lectura).
/// El documento real no se escribe en disco hasta implementar el guardado.
pub fn apply(session: &mut Session, edit: Edit) -> Result<bool, String> {
    let rev = session.rev();
    let (command, args) = match edit {
        Edit::Insert(text) if !text.is_empty() =>
            ("text.insert", json!({"text": text})),
        Edit::Insert(_) => return Ok(false),
        Edit::Paragraph => ("text.newParagraph", json!({})),
        Edit::LineBreak => ("text.lineBreak", json!({})),
        Edit::Backspace => ("text.backspace", json!({})),
        Edit::Delete => ("text.delete", json!({})),
        Edit::Paste(text) if !text.is_empty() =>
            ("edit.pasteText", json!({"text": text})),
        Edit::Paste(_) => return Ok(false),
        Edit::Undo => ("edit.undo", json!({})),
        Edit::Redo => ("edit.redo", json!({})),
    };
    session.run(command, &args).map_err(|error| error.to_string())?;
    Ok(session.rev() != rev)
}
