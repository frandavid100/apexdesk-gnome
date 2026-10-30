//! Native formatting commands routed to the WordCraft engine.
use serde_json::json;
use wordcraft_engine::Session;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Bold, Italic, Underline,
    Normal, NoSpacing, Heading1, Heading2, Heading3,
    AlignLeft, AlignCenter, AlignRight, Justify,
    FontSize(u8),
    Bullets, Numbering, Indent, Outdent,
}

pub fn apply(session: &mut Session, action: Format) -> Result<(), String> {
    let (name, params) = match action {
        Format::Bullets => ("para.bullets", json!({})),
        Format::Numbering => ("para.numbering", json!({})),
        Format::Indent => ("para.indent", json!({})),
        Format::Outdent => ("para.outdent", json!({})),
        Format::Bold => ("format.bold", json!({})),
        Format::Italic => ("format.italic", json!({})),
        Format::Underline => ("format.underline", json!({})),
        Format::NoSpacing => ("para.style", json!({"style":"NoSpacing"})),
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

/// A point size may include decimals; reject non-finite input before dispatch.
pub fn size(session: &mut Session, points: f64) -> Result<(), String> {
    if !points.is_finite() || !(1.0..=1638.0).contains(&points) {
        return Err("El tamaño debe estar entre 1 y 1638 puntos".into());
    }
    session.run("format.size", &json!({"size":points})).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn font(session: &mut Session, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() { return Err("Elige una familia tipográfica".into()); }
    session.run("format.font", &json!({"name":name})).map_err(|e| e.to_string())?;
    Ok(())
}
