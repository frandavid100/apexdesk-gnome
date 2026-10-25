//! Motores GPL reutilizables y modelo de lectura independiente de la GUI.
use apex_doc::{Document, Paragraph, Pos, Selection};
use apex_io::{Format, Loaded};
use std::path::Path;

pub const FORMATS: [Format; 2] = [Format::Docx, Format::Odt];

#[derive(Clone, Debug)]
pub struct Summary {
    pub format: String,
    pub paragraphs: usize,
    pub bold_spans: usize,
    pub title: String,
}

/// Abrir con el motor de formatos de ApexDesk.
pub fn open_document(path: &Path) -> Result<(Document, Format), String> {
    let (loaded, fmt) = apex_io::open(path).map_err(|e| e.to_string())?;
    match (loaded, fmt) {
        (Loaded::Doc(doc), fmt @ (Format::Docx | Format::Odt)) => Ok((*doc, fmt)),
        _ => Err("Se esperaba un archivo DOCX u ODT".into()),
    }
}

pub fn summary(doc: &Document, fmt: Format) -> Summary {
    let bold_spans = doc
        .paragraphs
        .iter()
        .flat_map(|p| p.spans())
        .filter(|(_, _, a)| a.bold)
        .count();
    Summary {
        format: format!("{fmt:?}"),
        paragraphs: doc.paragraphs.len(),
        bold_spans,
        title: doc.meta.title.clone(),
    }
}

/// Editar una copia; nunca sobrescribir el original o un destino existente.
pub fn edit_copy(source: &Path, destination: &Path, from: &str, to: &str) -> Result<usize, String> {
    if from.is_empty() {
        return Err("La búsqueda no puede estar vacía".into());
    }
    let source_abs = source.canonicalize().map_err(|e| e.to_string())?;
    let destination_parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let filename = destination.file_name().ok_or("Falta el nombre del destino")?;
    let dest_abs = destination_parent.canonicalize().map_err(|e| e.to_string())?.join(filename);
    if source_abs == dest_abs {
        return Err("El archivo de origen debe quedar intacto".into());
    }
    if destination.symlink_metadata().is_ok() {
        return Err("El destino ya existe y no debe sobrescribirse".into());
    }

    let (mut doc, fmt) = open_document(source)?;
    let extension = match fmt {
        Format::Docx => "docx",
        Format::Odt => "odt",
        _ => unreachable!(),
    };
    if !destination
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .eq_ignore_ascii_case(extension)
    {
        return Err(format!("El destino debe tener extensión .{extension}"));
    }

    let count = doc.replace_all(from, to, true);
    if count == 0 {
        return Err("No se encontró el texto, así que no se creó la copia".into());
    }
    apex_io::save_doc(&doc, destination, fmt).map_err(|e| e.to_string())?;
    let (saved, saved_fmt) = open_document(destination)?;
    if saved_fmt != fmt || saved.find_all(to, true).is_empty() {
        return Err("Error: no se verificó el documento reabierto".into());
    }
    Ok(count)
}

pub fn sample_document() -> Document {
    let mut doc = Document::from_paragraphs(vec![
        Paragraph::new("Cabecera sin modificar"),
        Paragraph::new("Texto de ejemplo para comprobar el formato."),
    ]);
    doc.meta.title = "ApexDesk GNOME - prueba de motores".to_string();
    let selection = Selection {
        anchor: Pos::new(1, 0),
        cursor: Pos::new(1, "Texto".len()),
    };
    doc.format_range(selection, &|a| a.bold = true);
    doc
}

pub fn assert_sample_invariants(doc: &Document, edited: bool) -> Result<(), String> {
    if doc.paragraphs.len() != 2 {
        return Err("El número de párrafos cambió".into());
    }
    if doc.paragraphs[0].text != "Cabecera sin modificar" {
        return Err("Se modificó la cabecera sin solicitarlo".into());
    }
    let expected = if edited {
        "Texto de prueba para comprobar el formato."
    } else {
        "Texto de ejemplo para comprobar el formato."
    };
    if doc.paragraphs[1].text != expected {
        return Err(format!("Contenido inesperado: {:?}", doc.paragraphs[1].text));
    }
    // El formato puede estar definido en un estilo de carácter nominal
    // (frecuente en ODT) y no como atributo directo del run.
    let bold = doc.paragraphs[1].spans().any(|(s, e, a)| {
        if s != 0 || e < 5 {
            return false;
        }
        a.bold
            || a.char_style
                .as_deref()
                .and_then(|id| doc.styles.by_id(id))
                .and_then(|style| style.bold)
                .unwrap_or(false)
    });
    if !bold {
        return Err("Se perdió el tramo inicial en negrita".into());
    }
    Ok(())
}

/// Construir los dos tipos de documentos, editar y reabrir copias.
pub fn run_smoke(dir: &Path) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut report = Vec::new();
    for fmt in FORMATS {
        let ext = match fmt {
            Format::Docx => "docx",
            Format::Odt => "odt",
            _ => unreachable!(),
        };
        let source = dir.join(format!("original.{ext}"));
        let destination = dir.join(format!("editado.{ext}"));
        if source.exists() || destination.exists() {
            return Err(format!("Los archivos de prueba ya existen: {ext}"));
        }
        apex_io::save_doc(&sample_document(), &source, fmt).map_err(|e| e.to_string())?;
        let before_bytes = std::fs::read(&source).map_err(|e| e.to_string())?;
        let (initial, detected) = open_document(&source)?;
        if detected != fmt {
            return Err("Formato detectado incorrecto".into());
        }
        assert_sample_invariants(&initial, false)?;
        if edit_copy(&source, &destination, "ejemplo", "prueba")? != 1 {
            return Err("Esperábamos una sustitución".into());
        }
        let (changed, _) = open_document(&destination)?;
        assert_sample_invariants(&changed, true)?;
        if std::fs::read(&source).map_err(|e| e.to_string())? != before_bytes {
            return Err("Se alteró el archivo original".into());
        }
        let s = summary(&changed, fmt);
        report.push(format!(
            "OK {ext}: {} párrafos, {} tramos en negrita; original intacto",
            s.paragraphs, s.bold_spans
        ));
    }
    Ok(report)
}

pub mod preview;
