//! Native formatting commands routed to the WordCraft engine.
use serde_json::json;
use wordcraft_engine::Session;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Bold, Italic, Underline,
    Normal, Heading1, Heading2, Heading3,
    AlignLeft, AlignCenter, AlignRight, Justify,
    FontSize(u8),
}

pub fn apply(session: &mut Session, action: Format) -> Result<(), String> {
    let (name, params) = match action {
        Format::Bold => ("format.bold", json!({})),
        Format::Italic => ("format.italic", json!({})),
        Format::Underline => ("format.underline", json!({})),
        Format::Normal => ("para.normal", json!({})),
        Format::Heading1 => ("para.heading1", json!({})),
        Format::Heading2 => ("para.heading2", json!({})),
        Format::Heading3 => ("para.heading3", json!({})),
        Format::AlignLeft => ("para.alignLeft", json!({})),
        Format::AlignCenter => ("para.alignCenter", json!({})),
        Format::AlignRight => ("para.alignRight", json!({})),
        Format::Justify => ("para.justify", json!({})),
        Format::FontSize(size) if (6..=96).contains(&size) =>
            ("format.size", json!({"size": size})),
        Format::FontSize(_) => return Err("Tamaño de fuente fuera del rango permitido (6–96 puntos)".into()),
    };
    session.run(name, &params).map_err(|err| err.to_string())?;
    Ok(())
}
