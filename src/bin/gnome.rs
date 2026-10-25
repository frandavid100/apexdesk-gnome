//! Primera interfaz GNOME: lector DOCX/ODT, sin edición ni paginación.
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView, WindowTitle};
use apexdesk_gnome_phase1::open_document;
use apexdesk_gnome_phase1::preview::{BlockKind, DocumentPreview, from_document};
use gtk::{gio, pango};
use gtk::prelude::*;
use std::path::Path;

const APP_ID: &str = "io.github.frandavid100.ApexDeskGnome";

struct TextTags {
    bold: gtk::TextTag,
    italic: gtk::TextTag,
    underline: gtk::TextTag,
    strike: gtk::TextTag,
    title: gtk::TextTag,
    h1: gtk::TextTag,
    h2: gtk::TextTag,
    h3: gtk::TextTag,
}

impl TextTags {
    fn new(buffer: &gtk::TextBuffer) -> Self {
        Self {
            bold: buffer.create_tag(Some("bold"), &[("weight", &700i32)]).unwrap(),
            italic: buffer.create_tag(Some("italic"), &[("style", &pango::Style::Italic)]).unwrap(),
            underline: buffer.create_tag(Some("underline"), &[("underline", &pango::Underline::Single)]).unwrap(),
            strike: buffer.create_tag(Some("strike"), &[("strikethrough", &true)]).unwrap(),
            title: buffer.create_tag(Some("title"), &[("scale", &1.7f64), ("weight", &700i32)]).unwrap(),
            h1: buffer.create_tag(Some("h1"), &[("scale", &1.4f64), ("weight", &700i32)]).unwrap(),
            h2: buffer.create_tag(Some("h2"), &[("scale", &1.22f64), ("weight", &700i32)]).unwrap(),
            h3: buffer.create_tag(Some("h3"), &[("scale", &1.1f64), ("weight", &700i32)]).unwrap(),
        }
    }
}

fn show_document(buffer: &gtk::TextBuffer, tags: &TextTags, preview: &DocumentPreview) {
    buffer.set_text("");
    let mut iter = buffer.end_iter();
    for (i, paragraph) in preview.paragraphs.iter().enumerate() {
        if i > 0 {
            buffer.insert(&mut iter, "\n");
        }
        for span in &paragraph.spans {
            let mut active: Vec<&gtk::TextTag> = Vec::new();
            match paragraph.kind {
                BlockKind::Title => active.push(&tags.title),
                BlockKind::Heading(1) => active.push(&tags.h1),
                BlockKind::Heading(2) => active.push(&tags.h2),
                BlockKind::Heading(_) => active.push(&tags.h3),
                BlockKind::Normal => {}
            }
            if span.bold { active.push(&tags.bold); }
            if span.italic { active.push(&tags.italic); }
            if span.underline { active.push(&tags.underline); }
            if span.strike { active.push(&tags.strike); }
            buffer.insert_with_tags(&mut iter, &span.text, &active);
        }
    }
}

fn make_window(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("ApexDesk GNOME")
        .default_width(900)
        .default_height(680)
        .build();

    let header = HeaderBar::new();
    let title = WindowTitle::new("ApexDesk GNOME", "Lector de documentos · fase 2");
    header.set_title_widget(Some(&title));

    let open_button = gtk::Button::with_label("Abrir");
    open_button.set_icon_name("document-open-symbolic");
    open_button.set_tooltip_text(Some("Abrir un documento DOCX u ODT"));
    header.pack_start(&open_button);

    let stack = gtk::Stack::new();
    let empty = StatusPage::builder()
        .icon_name("document-open-symbolic")
        .title("Abre un documento")
        .description("Puedes leer documentos DOCX y ODT sin modificar los archivos originales.")
        .build();
    stack.add_named(&empty, Some("empty"));

    let view = gtk::TextView::new();
    view.set_editable(false);
    view.set_cursor_visible(false);
    view.set_wrap_mode(gtk::WrapMode::WordChar);
    view.set_left_margin(36);
    view.set_right_margin(36);
    view.set_top_margin(32);
    view.set_bottom_margin(32);
    let buffer = view.buffer();
    let tags = std::rc::Rc::new(TextTags::new(&buffer));

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&view));
    stack.add_named(&scroll, Some("document"));
    stack.set_visible_child_name("empty");

    let toast_overlay = ToastOverlay::new();
    toast_overlay.set_child(Some(&stack));

    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&toast_overlay));
    let notice = gtk::Label::new(Some(
        "Vista previa parcial · Solo lectura · Sin maquetación de páginas, tablas ni imágenes",
    ));
    notice.add_css_class("dim-label");
    notice.set_margin_top(8);
    notice.set_margin_bottom(8);
    notice.set_wrap(true);
    toolbar.add_bottom_bar(&notice);

    window.set_content(Some(&toolbar));

    let dialog_window = window.clone();
    open_button.connect_clicked(move |_| {
        let dialog = gtk::FileDialog::new();
        dialog.set_title("Abrir documento");
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Documentos DOCX y ODT"));
        filter.add_pattern("*.docx");
        filter.add_pattern("*.odt");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        let window = dialog_window.clone();
        let stack = stack.clone();
        let title = title.clone();
        let buffer = buffer.clone();
        let tags = tags.clone();
        let toast_overlay = toast_overlay.clone();
        dialog.open(Some(&window), None::<&gio::Cancellable>, move |result| {
            let Ok(file) = result else { return }; // Cancelar no es un error.
            let Some(path) = file.path() else {
                toast_overlay.add_toast(Toast::new("Solo se admiten archivos locales por ahora"));
                return;
            };
            match open_document(Path::new(&path)) {
                Ok((document, _)) => {
                    let preview = from_document(&document);
                    show_document(&buffer, &tags, &preview);
                    let filename = path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Documento".into());
                    title.set_title(&filename);
                    title.set_subtitle("Solo lectura · vista preliminar");
                    stack.set_visible_child_name("document");
                }
                Err(e) => toast_overlay.add_toast(Toast::new(&format!("No se pudo abrir: {e}"))),
            }
        });
    });

    window.present();
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(make_window);
    app.run();
}
