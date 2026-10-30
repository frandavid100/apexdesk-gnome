//! Visor nativo GNOME para WordCraft: páginas continuas, zoom y miniaturas.
//! El contenido es solo lectura; no modifica los documentos.
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView, WindowTitle};
use glib::value::ToValue;
use gtk::{gdk, gio, glib};
use std::{cell::RefCell, collections::HashSet, path::Path, rc::Rc};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, render_page, sample_session};
use wordcraft_gnome::viewer_layout::{self, THUMB_SCALE};

const APP_ID: &str = "io.github.frandavid100.WordCraftGnome.PapersPreview";
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
    sidebar: adw::OverlaySplitView,
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
    // Un cambio de tamaño puede emitir una señal de scroll de forma reentrante.
    let Ok(current) = ui.current.try_borrow() else { return };
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
    if !ui.sidebar.shows_sidebar() { return; }
    let adj = ui.thumbnails_scroll.vadjustment();
    let top = adj.value();
    let bottom = top + adj.page_size().max(600.0);
    if let Ok(mut borrowed) = ui.current.try_borrow_mut() {
        if let Some(doc) = borrowed.as_mut() {
            let mut offset = 12.0;
            for i in 0..doc.sizes.len() {
                let (_, h) = doc.sizes[i];
                let height = (f64::from(h) * f64::from(THUMB_SCALE)).ceil() + 59.0;
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
        if ui.sidebar.is_collapsed() { ui.sidebar.set_show_sidebar(false); }
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
        thumb.set_halign(gtk::Align::Center);
        thumb.set_hexpand(false);
        thumb.set_size_request(
            (width * THUMB_SCALE).ceil() as i32,
            (height * THUMB_SCALE).ceil() as i32,
        );
        thumb.add_css_class("thumbnail-sheet");
        let number = gtk::Label::new(Some(&(i + 1).to_string()));
        number.add_css_class("dim-label");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
        content.set_halign(gtk::Align::Center);
        // El ancho de las hojas pequeñas no debe crecer hasta ocupar
        // casi todo el panel lateral.
        let thumb_clamp = adw::Clamp::new();
        thumb_clamp.set_maximum_size(185);
        thumb_clamp.set_child(Some(&thumb));
        content.append(&thumb_clamp);
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
    ui.title.set_subtitle("WordCraft · visor Papers · solo lectura");
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
    // Adwaita suministra color, espaciados y estados; CSS mínimo para papel.
    provider.load_from_data("
      .document-scroll { background: @view_bg_color; }
      .document-sheet { background: white; box-shadow: 0 1px 4px alpha(black, 0.15); }
      .thumbnail-sheet { background: white; box-shadow: 0 1px 3px alpha(black, 0.16); }
      .thumbnail-button.selected-thumb { background: alpha(@window_fg_color, 0.09); }
      .page-indicator, .zoom-value { font-feature-settings: 'tnum'; }
    ");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn make_window(
    app: &Application,
    sample_on_start: bool,
    sidebar_on_start: bool,
    narrow_on_start: bool,
    dark_on_start: bool,
) {
    if dark_on_start {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    }
    install_css();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("WordCraft GNOME")
        .default_width(if narrow_on_start { 640 } else { 1140 })
        .default_height(if narrow_on_start { 600 } else { 840 })
        .build();

    // Composición oficial: panel adaptable y dos páginas de barras nativas.
    let side_header = HeaderBar::new();
    let side_title = gtk::Label::new(Some("Visor de documentos"));
    side_title.add_css_class("heading");
    side_header.set_title_widget(Some(&side_title));

    // Menú de aplicación, independiente del menú de documento.
    let example_button = gtk::Button::with_label("Abrir ejemplo");
    let app_popover = gtk::Popover::new();
    let app_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    app_content.set_margin_top(12);
    app_content.set_margin_bottom(12);
    app_content.set_margin_start(12);
    app_content.set_margin_end(12);
    app_content.append(&example_button);
    app_popover.set_child(Some(&app_content));
    let app_menu = gtk::MenuButton::new();
    app_menu.set_icon_name("open-menu-symbolic");
    app_menu.set_tooltip_text(Some("Menú de la aplicación"));
    app_menu.set_popover(Some(&app_popover));
    side_header.pack_end(&app_menu);

    let header = HeaderBar::new();
    let title = WindowTitle::new("WordCraft GNOME", "Solo lectura");
    header.set_title_widget(Some(&title));

    let sidebar_button = gtk::ToggleButton::new();
    sidebar_button.set_icon_name("sidebar-show-symbolic");
    sidebar_button.set_tooltip_text(Some("Mostrar u ocultar miniaturas (F9)"));
    header.pack_start(&sidebar_button);

    let open_button = gtk::Button::from_icon_name("document-open-symbolic");
    open_button.set_tooltip_text(Some("Abrir documento (Ctrl+O)"));
    header.pack_start(&open_button);

    let page_label = gtk::Label::new(Some("Sin documento"));
    page_label.add_css_class("page-indicator");

    let previous = gtk::Button::with_label("Página anterior");
    previous.set_sensitive(false);
    let next = gtk::Button::with_label("Página siguiente");
    next.set_sensitive(false);
    let zoom_out = gtk::Button::from_icon_name("zoom-out-symbolic");
    zoom_out.set_tooltip_text(Some("Reducir zoom (Ctrl+-)"));
    let zoom_in = gtk::Button::from_icon_name("zoom-in-symbolic");
    zoom_in.set_tooltip_text(Some("Ampliar zoom (Ctrl++)"));
    let zoom_label = gtk::Label::new(Some("100 %"));
    zoom_label.add_css_class("zoom-value");
    let fit_button = gtk::Button::with_label("Ajustar al ancho");
    let normal_zoom = gtk::Button::with_label("Tamaño real (100 %)");
    let menu = gtk::Popover::new();
    let menu_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    menu_content.set_margin_top(12);
    menu_content.set_margin_bottom(12);
    menu_content.set_margin_start(12);
    menu_content.set_margin_end(12);
    menu_content.append(&zoom_label);
    menu_content.append(&fit_button);
    menu_content.append(&normal_zoom);
    menu_content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    menu_content.append(&previous);
    menu_content.append(&next);
    menu.set_child(Some(&menu_content));
    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("view-more-symbolic");
    menu_button.set_tooltip_text(Some("Opciones del documento"));
    menu_button.set_popover(Some(&menu));
    header.pack_end(&menu_button);
    header.pack_end(&page_label);

    let stack = gtk::Stack::new();
    let empty = StatusPage::builder()
        .icon_name("x-office-document-symbolic")
        .title("Abre un documento")
        .description("DOCX y ODT con desplazamiento continuo y miniaturas")
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
    let thumbnails_scroll = gtk::ScrolledWindow::new();
    thumbnails_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    thumbnails_scroll.set_vexpand(true);
    thumbnails_scroll.set_child(Some(&thumbnails_box));

    let sidebar_toolbar = ToolbarView::new();
    sidebar_toolbar.add_top_bar(&side_header);
    sidebar_toolbar.set_content(Some(&thumbnails_scroll));

    let zoom_buttons = gtk::Box::new(gtk::Orientation::Vertical, 8);
    zoom_buttons.set_halign(gtk::Align::End);
    zoom_buttons.set_valign(gtk::Align::End);
    zoom_buttons.set_margin_end(16);
    zoom_buttons.set_margin_bottom(16);
    zoom_buttons.add_css_class("osd");
    zoom_in.add_css_class("circular");
    zoom_out.add_css_class("circular");
    zoom_in.add_css_class("flat");
    zoom_out.add_css_class("flat");
    zoom_buttons.append(&zoom_in);
    zoom_buttons.append(&zoom_out);
    let canvas_overlay = gtk::Overlay::new();
    canvas_overlay.set_child(Some(&scroll));
    canvas_overlay.add_overlay(&zoom_buttons);
    stack.add_named(&canvas_overlay, Some("pages"));
    stack.set_visible_child_name("empty");

    let toasts = ToastOverlay::new();
    toasts.set_child(Some(&stack));
    let document_toolbar = ToolbarView::new();
    document_toolbar.add_top_bar(&header);
    document_toolbar.set_content(Some(&toasts));

    let sidebar = adw::OverlaySplitView::new();
    sidebar.set_sidebar(Some(&sidebar_toolbar));
    sidebar.set_content(Some(&document_toolbar));
    sidebar.set_show_sidebar(false);
    sidebar.set_min_sidebar_width(240.0);
    sidebar.set_max_sidebar_width(310.0);
    sidebar.set_sidebar_width_fraction(0.27);
    sidebar.set_pin_sidebar(true);
    window.set_content(Some(&sidebar));

    // En estrecho la barra lateral se superpone sobre la página.
    let narrow = adw::Breakpoint::new(
        adw::BreakpointCondition::parse("max-width: 760sp")
            .expect("condición de ancho válida"),
    );
    narrow.add_setter(&sidebar, "collapsed", Some(&true.to_value()));
    window.add_breakpoint(narrow);
    window.set_size_request(360, 420);

    let ui = Rc::new(Ui {
        current: RefCell::new(None),
        document_box, thumbnails_box, scroll, thumbnails_scroll,
        sidebar, sidebar_button,
        page_label, zoom_label, zoom_out, zoom_in, previous, next, stack, title, toasts,
    });

    {
        let ui = ui.clone();
        let app_popover = app_popover.clone();
        example_button.connect_clicked(move |_| {
            opened(&ui, sample_session(), "Documento de ejemplo");
            app_popover.popdown();
        });
    }
    {
        let ui = ui.clone();
        ui.sidebar_button.connect_toggled({
            let ui = ui.clone();
            move |button| ui.sidebar.set_show_sidebar(button.is_active())
        });
    }
    {
        let ui = ui.clone();
        let split = ui.sidebar.clone();
        split.connect_show_sidebar_notify(move |split| {
            let visible = split.shows_sidebar();
            if ui.sidebar_button.is_active() != visible {
                ui.sidebar_button.set_active(visible);
            }
            if visible {
                let ui = ui.clone();
                glib::idle_add_local_once(move || refresh_thumbnails(&ui));
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
        let menu = menu.clone();
        fit_button.connect_clicked(move |_| {
            fit_width(&ui);
            menu.popdown();
        });
    }
    {
        let ui = ui.clone();
        let menu = menu.clone();
        normal_zoom.connect_clicked(move |_| {
            zoom_to(&ui, 1.0);
            menu.popdown();
        });
    }
    {
        let ui = ui.clone();
        let open_button = open_button.clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, modifiers| {
            if key == gdk::Key::F9 {
                ui.sidebar_button.set_active(!ui.sidebar_button.is_active());
                return glib::Propagation::Stop;
            }
            if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                return glib::Propagation::Proceed;
            }
            if key == gdk::Key::o || key == gdk::Key::O {
                open_button.emit_clicked();
                return glib::Propagation::Stop;
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
        let scroll = ui.scroll.clone();
        let controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        controller.connect_scroll(move |ctrl, _, dy| {
            if ctrl.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                zoom_step(&ui, dy < 0.0);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        scroll.add_controller(controller);
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
    if sample_on_start {
        let ui = ui.clone();
        glib::idle_add_local_once(move || fit_width(&ui));
    }
}

fn main() {
    if std::env::args().any(|arg| arg == "--version") {
        println!("WordCraft GNOME · interfaz Adwaita adaptativa · edición 2026-10-30.3");
        return;
    }
    let sample_on_start = std::env::args().any(|arg| arg == "--sample");
    let sidebar_on_start = std::env::args().any(|arg| arg == "--sidebar");
    let narrow_on_start = std::env::args().any(|arg| arg == "--narrow");
    let dark_on_start = std::env::args().any(|arg| arg == "--dark");
    // Permite probar versiones simultáneas sin activar una ventana anterior por D-Bus.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        make_window(app, sample_on_start, sidebar_on_start, narrow_on_start, dark_on_start)
    });
    app.run_with_args(&["wordcraft-gnome"]);
}
