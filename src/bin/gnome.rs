//! Primera interfaz GTK4/libadwaita reutilizando directamente WordCraft.
//! No es un editor; muestra páginas reales maquetadas y rasterizadas por WordCraft.
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView, WindowTitle};
use gtk::{gdk, gio, glib};
use std::{cell::RefCell, path::Path, rc::Rc};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, render_page, sample_session};

const APP_ID: &str = "io.github.frandavid100.WordCraftGnome";

struct Current {
    session: Session,
    page: usize,
}

struct Ui {
    current: RefCell<Option<Current>>,
    picture: gtk::Picture,
    page_label: gtk::Label,
    previous: gtk::Button,
    next: gtk::Button,
    stack: gtk::Stack,
    title: WindowTitle,
    toasts: ToastOverlay,
}

fn draw(ui: &Rc<Ui>) {
    let result = {
        let mut current = ui.current.borrow_mut();
        match current.as_mut() {
            Some(doc) => render_page(&mut doc.session, doc.page, 1.25)
                .map(|image| (image, doc.page)),
            None => return,
        }
    };
    match result {
        Ok((image, index)) => {
            let bytes = glib::Bytes::from_owned(image.rgba);
            let texture = gdk::MemoryTexture::new(
                image.width as i32, image.height as i32,
                gdk::MemoryFormat::R8g8b8a8, &bytes,
                image.width as usize * 4,
            );
            ui.picture.set_paintable(Some(&texture));
            ui.page_label.set_text(&format!("Página {} de {}", index + 1, image.total_pages));
            ui.previous.set_sensitive(index > 0);
            ui.next.set_sensitive(index + 1 < image.total_pages);
            ui.stack.set_visible_child_name("pages");
        }
        Err(error) => ui.toasts.add_toast(Toast::new(&error)),
    }
}

fn opened(ui: &Rc<Ui>, session: Session, label: &str) {
    ui.current.replace(Some(Current { session, page: 0 }));
    ui.title.set_title(label);
    ui.title.set_subtitle("WordCraft · vista preliminar de solo lectura");
    draw(ui);
}

fn make_window(app: &Application, sample_on_start: bool) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("WordCraft GNOME")
        .default_width(1000)
        .default_height(800)
        .build();

    let header = HeaderBar::new();
    let title = WindowTitle::new("WordCraft GNOME", "Procesador de textos · prueba de integración");
    header.set_title_widget(Some(&title));

    let open_button = gtk::Button::with_label("Abrir");
    open_button.set_icon_name("document-open-symbolic");
    open_button.set_tooltip_text(Some("Abrir DOCX u ODT"));
    header.pack_start(&open_button);

    let example_button = gtk::Button::with_label("Ejemplo");
    example_button.set_tooltip_text(Some("Documento de muestra de WordCraft"));
    header.pack_start(&example_button);

    let previous = gtk::Button::from_icon_name("go-previous-symbolic");
    previous.set_tooltip_text(Some("Página anterior"));
    previous.set_sensitive(false);
    let next = gtk::Button::from_icon_name("go-next-symbolic");
    next.set_tooltip_text(Some("Página siguiente"));
    next.set_sensitive(false);
    let page_label = gtk::Label::new(Some("Sin documento"));
    let nav = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    nav.append(&previous);
    nav.append(&page_label);
    nav.append(&next);
    header.pack_end(&nav);

    let stack = gtk::Stack::new();
    let empty = StatusPage::builder()
        .icon_name("x-office-document-symbolic")
        .title("Abre un documento")
        .description("La vista muestra páginas auténticas generadas por WordCraft.")
        .build();
    stack.add_named(&empty, Some("empty"));

    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_halign(gtk::Align::Center);
    picture.set_valign(gtk::Align::Start);
    picture.set_margin_top(24);
    picture.set_margin_bottom(24);
    picture.set_margin_start(24);
    picture.set_margin_end(24);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&picture));
    stack.add_named(&scroll, Some("pages"));
    stack.set_visible_child_name("empty");

    let toasts = ToastOverlay::new();
    toasts.set_child(Some(&stack));
    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&toasts));
    let note = gtk::Label::new(Some("Solo lectura · Las páginas se maquetan con WordCraft, no con GTK TextView"));
    note.add_css_class("dim-label");
    note.set_margin_top(8);
    note.set_margin_bottom(8);
    toolbar.add_bottom_bar(&note);
    window.set_content(Some(&toolbar));

    let ui = Rc::new(Ui {
        current: RefCell::new(None), picture, page_label, previous, next, stack, title, toasts,
    });

    {
        let ui = ui.clone();
        example_button.connect_clicked(move |_| opened(&ui, sample_session(), "Documento de ejemplo"));
    }
    {
        let ui = ui.clone();
        ui.previous.connect_clicked({
            let ui = ui.clone();
            move |_| {
                if let Some(doc) = ui.current.borrow_mut().as_mut() { doc.page = doc.page.saturating_sub(1); }
                draw(&ui);
            }
        });
    }
    {
        let ui = ui.clone();
        ui.next.connect_clicked({
            let ui = ui.clone();
            move |_| {
                if let Some(doc) = ui.current.borrow_mut().as_mut() {
                    doc.page = doc.page.saturating_add(1);
                }
                draw(&ui);
            }
        });
    }
    {
        let ui = ui.clone();
        let parent = window.clone();
        open_button.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("Abrir documento WordCraft");
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Documentos DOCX y ODT"));
            filter.add_pattern("*.docx");
            filter.add_pattern("*.odt");
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));
            let ui = ui.clone();
            dialog.open(Some(&parent), None::<&gio::Cancellable>, move |result| {
                let Ok(file) = result else { return };
                let Some(path) = file.path() else {
                    ui.toasts.add_toast(Toast::new("Por ahora solo se admiten archivos locales"));
                    return;
                };
                match open_document(Path::new(&path)) {
                    Ok(session) => {
                        let label = path.file_name().map(|x| x.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Documento".to_string());
                        opened(&ui, session, &label);
                    }
                    Err(err) => ui.toasts.add_toast(Toast::new(&err)),
                }
            });
        });
    }

    if sample_on_start {
        opened(&ui, sample_session(), "Documento de ejemplo");
    }
    window.present();
}

fn main() {
    let sample_on_start = std::env::args().any(|arg| arg == "--sample");
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| make_window(app, sample_on_start));
    app.run();
}
