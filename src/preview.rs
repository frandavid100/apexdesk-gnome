//! Proyección de lectura, independiente de GTK y del editor egui.
use apex_doc::{Document, ParaKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Normal,
    Title,
    Heading(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewParagraph {
    pub kind: BlockKind,
    pub spans: Vec<TextSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentPreview {
    pub title: String,
    pub paragraphs: Vec<PreviewParagraph>,
}

/// No estamos reconstruyendo un documento WYSIWYG: aún no interpretamos
/// tablas, imágenes, marcas de página ni geometría. Conservar las referencias
/// a estilos de carácter evita perder formato al importar ODT de terceros.
pub fn from_document(doc: &Document) -> DocumentPreview {
    let mut paragraphs = Vec::with_capacity(doc.paragraphs.len());
    for p in &doc.paragraphs {
        let kind = match p.style.kind {
            ParaKind::Title => BlockKind::Title,
            ParaKind::Heading(level) => BlockKind::Heading(level),
            _ => BlockKind::Normal,
        };
        let spans = p
            .spans()
            .filter_map(|(start, end, attrs)| {
                let text = p.text.get(start..end)?;
                let named = attrs
                    .char_style
                    .as_deref()
                    .and_then(|id| doc.styles.by_id(id));
                let paragraph = doc
                    .styles
                    .custom(p.style.custom)
                    .or_else(|| doc.styles.builtin(p.style.kind));

                let property = |direct: bool, inherited: Option<bool>, base: Option<bool>| {
                    direct || inherited.or(base).unwrap_or(false)
                };
                Some(TextSpan {
                    text: text.to_owned(),
                    bold: property(
                        attrs.bold,
                        named.and_then(|s| s.bold),
                        paragraph.and_then(|s| s.bold),
                    ),
                    italic: property(
                        attrs.italic,
                        named.and_then(|s| s.italic),
                        paragraph.and_then(|s| s.italic),
                    ),
                    underline: property(
                        attrs.underline,
                        named.and_then(|s| s.underline),
                        paragraph.and_then(|s| s.underline),
                    ),
                    strike: attrs.strike,
                })
            })
            .collect();
        paragraphs.push(PreviewParagraph { kind, spans });
    }
    DocumentPreview {
        title: doc.meta.title.clone(),
        paragraphs,
    }
}
