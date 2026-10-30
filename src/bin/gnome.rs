//! Visor nativo GNOME para WordCraft: páginas continuas, zoom y miniaturas.
//! El contenido es solo lectura; no modifica los documentos.
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView, WindowTitle};
use gtk::{gdk, gio, glib};
use std::{cell::RefCell, collections::HashSet, path::Path, rc::Rc};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, render_page, sample_session};
use wordcraft_gnome::viewer_layout::{self, THUMB_SCALE};

const APP_ID: &str = "io.github.frandavid100.WordCraftGnome";
const STEPS: &[f32] = &[0.5, 0.67, 0.75, 0.85, 1.0, 1.25, 1.5, 1.75, 2.0];

struct Current {
    session: Session,
    sizes: Vec<(f32, f32)>,
    tops: Vec<f64>,
    zoom: f32,
    pages: Vec<gtk::Picture>,
    thumbs: Vec<gtk::Picture>,
    thumb_buttons: Vec<gtk::Button>,
    painted_pages: HashSet<usize>,
    painted_thumbs: HashSet<usize>,
    active: usize,
}

struct Ui {
    current: RefCell<Option<Current>>,
    document_box: gtk::Box,
    thumbnails_box: gtk::Box,
    scroll: gtk::ScrolledWindow,
    thumbnails_scroll: gtk::ScrolledWindow,
    sidebar: gtk::Revealer,
    sidebar_button: gtk::ToggleButton,
    page_label: gtk::Label,
    zoom_label: gtk::Label,
    zoom_out: gtk::Button,
    zoom_in: gtk::Button,
    previous: gtk::Button,
    next: gtk::Button,
    stack: gtk::Stack,
    title: WindowTitle,
    toasts: ToastOverlay,
}

fn texture(img: wordcraft_gnome::PageImage) -> gdk::MemoryTexture {
    let bytes = glib::Bytes::from_owned(img.rgba);
    gdk::MemoryTexture::new(
        img.width as i32, img.height as i32,
        gdk::MemoryFormat::R8g8b8a8, &bytes, img.width as usize * 4,
    )
}

fn clear_box(parent: &gtk::Box) {
    while let Some(child) = parent.first_child() {
        parent.remove(&child);
    }
}

fn document_page_picture() -> gtk::Picture {
    let image = gtk::Picture::new();
    image.set_can_shrink(false);
    image.set_content_fit(gtk::ContentFit::Fill);
    image.add_css_class("document-sheet");
    image
}

fn update_status(ui: &Rc<Ui>) {
    let current = ui.current.borrow();
    if let Some(doc) = current.as_ref() {
        ui.page_label.set_text(&format!("{} de {}", doc.active + 1, doc.sizes.len()));
        ui.previous.set_sensitive(doc.active > 0);
        ui.next.set_sensitive(doc.active + 1 < doc.sizes.len());
        ui.zoom_label.set_text(&format!("{:.0} %", doc.zoom * 100.0));
        ui.zoom_out.set_sensitive(doc.zoom > viewer_layout::MIN_ZOOM);
        ui.zoom_in.set_sensitive(doc.zoom < viewer_layout::MAX_ZOOM);
        for (i, button) in doc.thumb_buttons.iter().enumerate() {
            if i == doc.active { button.add_css_class("selected-thumb"); }
            else { button.remove_css_class("selected-thumb"); }
        }
    } else {
        ui.page_label.set_text("Sin documento");
        ui.zoom_label.set_text("100 %");
    }
}

/// Renderizado bajo demanda: solo hojas cercanas al área visible.
fn refresh_pages(ui: &Rc<Ui>) {
    let adj = ui.scroll.vadjustment();
    let top = adj.value();
    let bottom = top + adj.page_size().max(600.0);
    let mut errors = Vec::new();
    if let Ok(mut borrowed) = ui.current.try_borrow_mut() {
        if let Some(doc) = borrowed.as_mut() {
            let viewport = top + adj.page_size().max(600.0) * 0.3;
            doc.active = viewer_layout::page_at(&doc.tops, &doc.sizes, doc.zoom, viewport);
            let visible = viewer_layout::visible_indices(&doc.tops, &doc.sizes, doc.zoom, top, bottom);
            let keep: HashSet<usize> = visible.iter().copied().collect();
            let old: Vec<usize> = doc.painted_pages.difference(&keep).copied().collect();
            for index in old {
                doc.pages[index].set_paintable(None::<&gdk::Paintable>);
                doc.painted_pages.remove(&index);
            }
            for index in visible {
                if !doc.painted_pages.contains(&index) {
                    match render_page(&mut doc.session, index, doc.zoom) {
                        Ok(img) => {
                            doc.pages[index].set_paintable(Some(&texture(img)));
                            doc.painted_pages.insert(index);
                        }
                        Err(error) => errors.push(error),
                    }
                }
            }
        }
    }
    for e in errors { ui.toasts.add_toast(Toast::new(&e)); }
    update_status(ui);
}

/// Renderizar miniaturas solo alrededor de la zona desplazada del panel.
fn refresh_thumbnails(ui: &Rc<Ui>) {
    if !ui.sidebar.reveals_child() { return; }
    let adj = ui.thumbnails_scroll.vadjustment();
    let top = adj.value();
    let bottom = top + adj.page_size().max(600.0);
    if let Ok(mut borrowed) = ui.current.try_borrow_mut() {
        if let Some(doc) = borrowed.as_mut() {
            let mut offset = 12.0;
            for i in 0..doc.sizes.len() {
                let (_, h) = doc.sizes[i];
                let height = (f64::from(h) * f64::from(THUMB_SCALE)).ceil() + 38.0;
                let is_visible = offset <= bottom + 250.0 && offset + height >= top - 250.0;
                if is_visible && !doc.painted_thumbs.contains(&i) {
                    if let Ok(image) = render_page(&mut doc.session, i, THUMB_SCALE) {
                        doc.thumbs[i].set_paintable(Some(&texture(image)));
                        doc.painted_thumbs.insert(i);
                    }
                } else if !is_visible && doc.painted_thumbs.remove(&i) {
                    doc.thumbs[i].set_paintable(None::<&gdk::Paintable>);
                }
                offset += height;
            }
        }
    }
}

fn jump_to(ui: &Rc<Ui>, index: usize) {
    let target = {
        let borrowed = ui.current.borrow();
        borrowed.as_ref().and_then(|doc| doc.tops.get(index).copied())
    };
    if let Some(top) = target {
        let adj = ui.scroll.vadjustment();
        let max = (adj.upper() - adj.page_size()).max(0.0);
        adj.set_value((top - 10.0).max(0.0).min(max));
        refresh_pages(ui);
    }
}

fn zoom_to(ui: &Rc<Ui>, zoom: f32) {
    let zoom = viewer_layout::zoom_clamped(zoom);
    let current_page;
    {
        let mut borrowed = ui.current.borrow_mut();
        let Some(doc) = borrowed.as_mut() else { return };
        current_page = doc.active;
        if (zoom - doc.zoom).abs() < 0.005 { return; }
        doc.zoom = zoom;
        doc.tops = viewer_layout::page_tops(&doc.sizes, zoom);
        for (i, picture) in doc.pages.iter().enumerate() {
            picture.set_paintable(None::<&gdk::Paintable>);
            picture.set_size_request(
                (doc.sizes[i].0 * zoom).ceil() as i32,
                (doc.sizes[i].1 * zoom).ceil() as i32,
            );
        }
        doc.painted_pages.clear();
    }
    update_status(ui);
    let ui = ui.clone();
    // Ejecutarlo después de que GTK recalcule las dimensiones de las páginas.
    glib::idle_add_local_once(move || jump_to(&ui, current_page));
}

fn zoom_step(ui: &Rc<Ui>, positive: bool) {
    let value = ui.current.borrow().as_ref().map(|d| d.zoom);
    let Some(zoom) = value else { return };
    let next = if positive {
        STEPS.iter().copied().find(|&v| v > zoom + 0.005).unwrap_or(2.0)
    } else {
        STEPS.iter().copied().rev().find(|&v| v < zoom - 0.005).unwrap_or(0.5)
    };
    zoom_to(ui, next);
}

fn fit_width(ui: &Rc<Ui>) {
    let max_width = ui.current.borrow().as_ref().and_then(|d| {
        d.sizes.iter().map(|x| x.0).reduce(f32::max)
    });
    let Some(width) = max_width else { return };
    let available = (ui.scroll.allocated_width() as f32 - 54.0).max(250.0);
    zoom_to(ui, available / width);
}

fn opened(ui: &Rc<Ui>, mut session: Session, label: &str) {
    let layout = session.layout();
    let sizes: Vec<(f32,f32)> = layout.pages.iter().map(|p| (p.w, p.h)).collect();
    if sizes.is_empty() {
        ui.toasts.add_toast(Toast::new("El documento no contiene páginas"));
        return;
    }
    let zoom = 1.0;
    let tops = viewer_layout::page_tops(&sizes, zoom);
    clear_box(&ui.document_box);
    clear_box(&ui.thumbnails_box);

    let mut pages = Vec::with_capacity(sizes.len());
    let mut thumbs = Vec::with_capacity(sizes.len());
    let mut thumb_buttons = Vec::with_capacity(sizes.len());

    for (i, &(width, height)) in sizes.iter().enumerate() {
        let picture = document_page_picture();
        picture.set_size_request(width.ceil() as i32, height.ceil() as i32);
        picture.set_halign(gtk::Align::Center);
        picture.set_margin_top(11);
        picture.set_margin_bottom(11);
        ui.document_box.append(&picture);
        pages.push(picture);

        let thumb = gtk::Picture::new();
        thumb.set_can_shrink(false);
        thumb.set_content_fit(gtk::ContentFit::Fill);
        thumb.set_size_request(
            (width * THUMB_SCALE).ceil() as i32,
            (height * THUMB_SCALE).ceil() as i32,
        );
        thumb.add_css_class("thumbnail-sheet");
        let number = gtk::Label::new(Some(&(i + 1).to_string()));
        number.add_css_class("dim-label");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
        content.set_halign(gtk::Align::Center);
        content.append(&thumb);
        content.append(&number);
        let button = gtk::Button::new();
        button.set_child(Some(&content));
        button.add_css_class("flat");
        button.add_css_class("thumbnail-button");
        button.set_margin_top(7);
        button.set_margin_bottom(7);
        button.set_halign(gtk::Align::Center);
        {
            let ui = ui.clone();
            button.connect_clicked(move |_| jump_to(&ui, i));
        }
        ui.thumbnails_box.append(&button);
        thumbs.push(thumb);
        thumb_buttons.push(button);
    }

    ui.current.replace(Some(Current {
        session, sizes, tops, zoom, pages, thumbs, thumb_buttons,
        painted_pages: HashSet::new(),
        painted_thumbs: HashSet::new(),
        active: 0,
    }));
    ui.title.set_title(label);
    ui.title.set_subtitle("WordCraft · solo lectura");
    ui.stack.set_visible_child_name("pages");
    ui.scroll.vadjustment().set_value(0.0);
    ui.thumbnails_scroll.vadjustment().set_value(0.0);
    update_status(ui);
    refresh_pages(ui);
    refresh_thumbnails(ui);
    let ui = ui.clone();
    glib::idle_add_local_once(move || {
        refresh_pages(&ui);
        refresh_thumbnails(&ui);
    });
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data("
      .document-scroll { background: @view_bg_color; }
      .document-sheet { background: #ffffff; box-shadow: 0 1px 5px alpha(black, 0.19); }
      .thumbnail-strip { background: @sidebar_bg_color; }
      .thumbnail-sheet { background: white; box-shadow: 0 1px 4px alpha(black, 0.16); }
      .thumbnail-button { border-radius: 10px; padding: 9px; }
      .thumbnail-button.selected-thumb { background: alpha(@accent_bg_color, 0.15); }
      .zoom-value { font-feature-settings: 'tnum'; }
    ");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn make_window(app: &Application, sample_on_start: bool, sidebar_on_start: bool) {
    install_css();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("WordCraft GNOME")
        .default_width(1140)
        .default_height(840)
        .build();

    let header = HeaderBar::new();
    let title = WindowTitle::new("WordCraft GNOME", "Visor de documentos");
    header.set_title_widget(Some(&title));

    let open_button = gtk::Button::from_icon_name("document-open-symbolic");
    open_button.set_tooltip_text(Some("Abrir DOCX u ODT"));
    header.pack_start(&open_button);
    let example_button = gtk::Button::with_label("Ejemplo");
    example_button.set_tooltip_text(Some("Abrir documento de muestra"));
    header.pack_start(&example_button);

    let sidebar_button = gtk::ToggleButton::new();
    sidebar_button.set_icon_name("view-sidebar-symbolic");
    sidebar_button.set_tooltip_text(Some("Mostrar u ocultar miniaturas"));
    header.pack_start(&sidebar_button);

    let previous = gtk::Button::from_icon_name("go-previous-symbolic");
    previous.set_tooltip_text(Some("Página anterior"));
    previous.set_sensitive(false);
    let next = gtk::Button::from_icon_name("go-next-symbolic");
    next.set_tooltip_text(Some("Página siguiente"));
    next.set_sensitive(false);
    let page_label = gtk::Label::new(Some("Sin documento"));
    let nav = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    nav.append(&previous);
    nav.append(&page_label);
    nav.append(&next);

    let zoom_out = gtk::Button::from_icon_name("zoom-out-symbolic");
    zoom_out.set_tooltip_text(Some("Reducir zoom (Ctrl+-)"));
    let zoom_in = gtk::Button::from_icon_name("zoom-in-symbolic");
    zoom_in.set_tooltip_text(Some("Ampliar zoom (Ctrl++)"));
    let zoom_label = gtk::Label::new(Some("100 %"));
    zoom_label.set_width_chars(6);
    zoom_label.add_css_class("zoom-value");
    let fit_button = gtk::Button::from_icon_name("zoom-fit-best-symbolic");
    fit_button.set_tooltip_text(Some("Ajustar al ancho disponible (Ctrl+0)"));
    let zoom_controls = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    zoom_controls.append(&zoom_out);
    zoom_controls.append(&zoom_label);
    zoom_controls.append(&zoom_in);
    zoom_controls.append(&fit_button);
    header.pack_end(&zoom_controls);
    header.pack_end(&nav);

    let stack = gtk::Stack::new();
    let empty = StatusPage::builder()
        .icon_name("x-office-document-symbolic")
        .title("Abre un documento")
        .description("DOCX y ODT con páginas continuas, zoom y miniaturas.")
        .build();
    stack.add_named(&empty, Some("empty"));

    let document_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    document_box.set_halign(gtk::Align::Center);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add_css_class("document-scroll");
    scroll.set_child(Some(&document_box));

    let thumbnails_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    thumbnails_box.set_halign(gtk::Align::Fill);
    let thumbnails_scroll = gtk::ScrolledWindow::new();
    thumbnails_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    thumbnails_scroll.set_child(Some(&thumbnails_box));
    thumbnails_scroll.set_width_request(190);
    thumbnails_scroll.add_css_class("thumbnail-strip");
    let sidebar = gtk::Revealer::new();
    sidebar.set_transition_type(gtk::RevealerTransitionType::SlideRight);
    sidebar.set_reveal_child(false);
    sidebar.set_child(Some(&thumbnails_scroll));
    let workspace = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    workspace.append(&sidebar);
    workspace.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    workspace.append(&scroll);
    stack.add_named(&workspace, Some("pages"));
    stack.set_visible_child_name("empty");

    let toasts = ToastOverlay::new();
    toasts.set_child(Some(&stack));
    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&toasts));
    let note = gtk::Label::new(Some("Solo lectura · Desplazamiento continuo · Ctrl + rueda para ampliar"));
    note.add_css_class("dim-label");
    note.set_margin_top(6);
    note.set_margin_bottom(6);
    toolbar.add_bottom_bar(&note);
    window.set_content(Some(&toolbar));

    let ui = Rc::new(Ui {
        current: RefCell::new(None),
        document_box, thumbnails_box, scroll, thumbnails_scroll,
        sidebar, sidebar_button,
        page_label, zoom_label, zoom_out, zoom_in, previous, next, stack, title, toasts,
    });

    {
        let ui = ui.clone();
        example_button.connect_clicked(move |_| opened(&ui, sample_session(), "Documento de ejemplo"));
    }
    {
        let ui = ui.clone();
        ui.sidebar_button.connect_toggled({
            let ui = ui.clone();
            move |button| {
                ui.sidebar.set_reveal_child(button.is_active());
                if button.is_active() {
                    let ui = ui.clone();
                    glib::idle_add_local_once(move || refresh_thumbnails(&ui));
                }
            }
        });
    }
    {
        let ui = ui.clone();
        ui.scroll.vadjustment().connect_value_changed(move |_| refresh_pages(&ui));
    }
    {
        let ui = ui.clone();
        ui.thumbnails_scroll.vadjustment().connect_value_changed(move |_| refresh_thumbnails(&ui));
    }
    {
        let ui = ui.clone();
        ui.previous.connect_clicked({
            let ui = ui.clone();
            move |_| {
                let page = ui.current.borrow().as_ref().map(|d| d.active.saturating_sub(1));
                if let Some(page) = page { jump_to(&ui, page); }
            }
        });
    }
    {
        let ui = ui.clone();
        ui.next.connect_clicked({
            let ui = ui.clone();
            move |_| {
                let page = ui.current.borrow().as_ref().map(|d| d.active + 1);
                if let Some(page) = page { jump_to(&ui, page); }
            }
        });
    }
    {
        let ui = ui.clone();
        ui.zoom_out.connect_clicked({
            let ui = ui.clone();
            move |_| zoom_step(&ui, false)
        });
    }
    {
        let ui = ui.clone();
        ui.zoom_in.connect_clicked({
            let ui = ui.clone();
            move |_| zoom_step(&ui, true)
        });
    }
    {
        let ui = ui.clone();
        fit_button.connect_clicked(move |_| fit_width(&ui));
    }
    {
        let ui = ui.clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, modifiers| {
            if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                return glib::Propagation::Proceed;
            }
            match key {
                gdk::Key::plus | gdk::Key::equal | gdk::Key::KP_Add => zoom_step(&ui, true),
                gdk::Key::minus | gdk::Key::KP_Subtract => zoom_step(&ui, false),
                gdk::Key::_0 | gdk::Key::KP_0 => fit_width(&ui),
                _ => return glib::Propagation::Proceed,
            }
            glib::Propagation::Stop
        });
        window.add_controller(controller);
    }
    {
        let ui = ui.clone();
        let controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        controller.connect_scroll(move |ctrl, _, dy| {
            if ctrl.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                zoom_step(&ui, dy < 0.0);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        ui.scroll.add_controller(controller);
    }
    {
        let ui = ui.clone();
        let parent = window.clone();
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
            let ui = ui.clone();
            dialog.open(Some(&parent), None::<&gio::Cancellable>, move |result| {
                let Ok(file) = result else { return };
                let Some(path) = file.path() else {
                    ui.toasts.add_toast(Toast::new("Solo se admiten archivos locales"));
                    return;
                };
                match open_document(Path::new(&path)) {
                    Ok(session) => {
                        let label = path.file_name().map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Documento".to_string());
                        opened(&ui, session, &label);
                    }
                    Err(err) => ui.toasts.add_toast(Toast::new(&err)),
                }
            });
        });
    }

    if sample_on_start { opened(&ui, sample_session(), "Documento de ejemplo"); }
    if sidebar_on_start { ui.sidebar_button.set_active(true); }
    window.present();
}

fn main() {
    let sample_on_start = std::env::args().any(|arg| arg == "--sample");
    let sidebar_on_start = std::env::args().any(|arg| arg == "--sidebar");
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| make_window(app, sample_on_start, sidebar_on_start));
    app.run_with_args(&["wordcraft-gnome"]);
}
