//! Editor experimental GNOME para WordCraft: motor y maquetación reutilizados.
//! La fase 3B modifica el documento en memoria, pero aún no lo guarda.
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView, WindowTitle};
use glib::value::ToValue;
use gtk::{gdk, gio, glib};
use std::{cell::{Cell, RefCell}, collections::HashSet, path::Path, rc::Rc};
use wordcraft_doc::Pos;
use wordcraft_gnome::selection::{self, SelectionGeometry};
use wordcraft_gnome::editing::{self, Edit};
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
    fit_to_width: bool,
    pages: Vec<gtk::Picture>,
    page_frames: Vec<gtk::Overlay>,
    selection_areas: Vec<gtk::DrawingArea>,
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
    selection_data: Rc<RefCell<SelectionGeometry>>,
    selection_zoom: Rc<Cell<f32>>,
    drag_anchor: RefCell<Option<Pos>>,
    dirty: Cell<bool>,
    closing_confirmed: Cell<bool>,
}

fn texture(img: wordcraft_gnome::PageImage) -> gdk::MemoryTexture {
    let bytes = glib::Bytes::from_owned(img.rgba);
    gdk::MemoryTexture::new(
        img.width as i32, img.height as i32,
        gdk::MemoryFormat::R8g8b8a8, &bytes, img.width as usize * 4,
    )
}


fn refresh_selection(ui: &Rc<Ui>) {
    let snapshot = {
        let Ok(mut value) = ui.current.try_borrow_mut() else { return };
        let Some(doc) = value.as_mut() else { return };
        selection::geometry(&mut doc.session)
    };
    *ui.selection_data.borrow_mut() = snapshot;
    if let Ok(value) = ui.current.try_borrow() {
        if let Some(doc) = value.as_ref() {
            for area in &doc.selection_areas { area.queue_draw(); }
        }
    }
}

/// Compute actual per-page widget bounds in GTK coordinates, then convert to
/// WordCraft points. This remains correct after scrolling, zooming and opening
/// or closing the sidebar.
fn pos_at_canvas(ui: &Rc<Ui>, x: f64, y: f64) -> Option<Pos> {
    let mut value = ui.current.borrow_mut();
    let doc = value.as_mut()?;
    let (page, rect, _) = doc.page_frames.iter().enumerate()
        .filter_map(|(i, frame)| {
            let rect = frame.compute_bounds(&ui.document_box)?;
            let min_x = f64::from(rect.x());
            let min_y = f64::from(rect.y());
            let max_x = min_x + f64::from(rect.width());
            let max_y = min_y + f64::from(rect.height());
            let dx = (min_x - x).max(0.0).max(x - max_x);
            let dy = (min_y - y).max(0.0).max(y - max_y);
            Some((i, rect, (dx * dx + dy * dy).sqrt()))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))?;
    let z = f64::from(doc.zoom);
    let (w, h) = doc.sizes[page];
    let px = ((x - f64::from(rect.x())) / z).clamp(0.0, f64::from(w)) as f32;
    let py = ((y - f64::from(rect.y())) / z).clamp(0.0, f64::from(h)) as f32;
    selection::hit(&mut doc.session, page, px, py)
}

fn caret_ensure_visible(ui: &Rc<Ui>) {
    let target = {
        let Ok(mut value) = ui.current.try_borrow_mut() else { return };
        let Some(doc) = value.as_mut() else { return };
        let layout = doc.session.layout();
        layout.caret_on(&doc.session.sel.focus, doc.session.page_hint)
            .and_then(|c| doc.page_frames.get(c.page)
                .and_then(|frame| frame.compute_bounds(&ui.document_box))
                .map(|r| f64::from(r.y()) + f64::from(c.top * doc.zoom)))
    };
    if let Some(y) = target {
        let adjustment = ui.scroll.vadjustment();
        let top = adjustment.value();
        let bottom = top + adjustment.page_size();
        if y < top + 24.0 {
            adjustment.set_value((y - 32.0).max(0.0));
        } else if y > bottom - 32.0 {
            adjustment.set_value((y - adjustment.page_size() + 60.0).max(0.0));
        }
    }
}

fn selected_command(ui: &Rc<Ui>, action: &str, extend: bool) {
    let result = {
        let mut value = ui.current.borrow_mut();
        value.as_mut().map(|doc| {
            if action == "select-all" {
                selection::select_all(&mut doc.session)
            } else {
                selection::move_caret(&mut doc.session, action, extend)
            }
        })
    };
    if let Some(Err(e)) = result { ui.toasts.add_toast(Toast::new(&e)); }
    refresh_selection(ui);
    caret_ensure_visible(ui);
}

fn copy_selection(ui: &Rc<Ui>) {
    let result = {
        let mut value = ui.current.borrow_mut();
        value.as_mut().map(|doc| selection::copy_text(&mut doc.session))
    };
    match result {
        Some(Ok(text)) if !text.is_empty() => {
            if let Some(display) = gdk::Display::default() {
                display.clipboard().set_text(&text);
            }
        }
        Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
        _ => (),
    }
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
            doc.page_frames[i].set_size_request(
                (doc.sizes[i].0 * zoom).ceil() as i32,
                (doc.sizes[i].1 * zoom).ceil() as i32,
            );
            picture.set_size_request(
                (doc.sizes[i].0 * zoom).ceil() as i32,
                (doc.sizes[i].1 * zoom).ceil() as i32,
            );
        }
        doc.painted_pages.clear();
    }
    ui.selection_zoom.set(zoom);
    refresh_selection(ui);
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
    if let Some(doc) = ui.current.borrow_mut().as_mut() {
        doc.fit_to_width = false;
    }
    zoom_to(ui, next);
}

fn refit_if_needed(ui: &Rc<Ui>) {
    let fit = ui.current.try_borrow().ok()
        .and_then(|s| s.as_ref().map(|d| d.fit_to_width))
        .unwrap_or(false);
    if fit { fit_width(ui); }
}

fn fit_width(ui: &Rc<Ui>) {
    let max_width = ui.current.borrow().as_ref().and_then(|d| {
        d.sizes.iter().map(|x| x.0).reduce(f32::max)
    });
    let Some(width) = max_width else { return };
    // El scroll puede conservar una anchura natural mayor que la del área
    // visible durante la animación de AdwOverlaySplitView. Tomamos también
    // el ancho real del split y descontamos el panel si está al lado.
    let split_width = ui.sidebar.allocated_width() as f32;
    let sidebar_width = if ui.sidebar.shows_sidebar() && !ui.sidebar.is_collapsed() {
        ui.sidebar.sidebar().map(|w| w.allocated_width() as f32).unwrap_or(0.0)
    } else { 0.0 };
    let layout_width = (split_width - sidebar_width).max(0.0);
    let available = (ui.scroll.allocated_width() as f32)
        .min(layout_width)
        .max(250.0) - 46.0;
    if let Some(doc) = ui.current.borrow_mut().as_mut() {
        doc.fit_to_width = true;
    }
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
    let mut page_frames = Vec::with_capacity(sizes.len());
    let mut selection_areas = Vec::with_capacity(sizes.len());
    let mut thumbs = Vec::with_capacity(sizes.len());
    let mut thumb_buttons = Vec::with_capacity(sizes.len());

    for (i, &(width, height)) in sizes.iter().enumerate() {
        let picture = document_page_picture();
        picture.set_size_request(width.ceil() as i32, height.ceil() as i32);
        picture.set_halign(gtk::Align::Center);
        picture.set_margin_top(11);
        picture.set_margin_bottom(11);
        let frame = gtk::Overlay::new();
        frame.set_size_request(width.ceil() as i32, height.ceil() as i32);
        frame.set_halign(gtk::Align::Center);
        frame.set_margin_top(11);
        frame.set_margin_bottom(11);
        picture.set_margin_top(0);
        picture.set_margin_bottom(0);
        frame.set_child(Some(&picture));
        let area = gtk::DrawingArea::new();
        area.set_can_target(false);
        area.set_halign(gtk::Align::Fill);
        area.set_valign(gtk::Align::Fill);
        {
            let data = ui.selection_data.clone();
            let scale = ui.selection_zoom.clone();
            area.set_draw_func(move |_, cr, _, _| {
                let selection = data.borrow();
                let z = f64::from(scale.get());
                for r in selection.highlights.iter().filter(|r| r.page == i) {
                    cr.set_source_rgba(0.20, 0.44, 0.88, 0.30);
                    cr.rectangle(r.x*z, r.y*z, r.width*z, r.height*z);
                    let _ = cr.fill();
                }
                if let Some(r) = selection.caret.as_ref().filter(|r| r.page == i) {
                    cr.set_source_rgba(0.08, 0.30, 0.74, 1.0);
                    cr.rectangle(r.x*z, r.y*z, (r.width*z).max(1.5), (r.height*z).max(2.0));
                    let _ = cr.fill();
                }
            });
        }
        frame.add_overlay(&area);
        ui.document_box.append(&frame);
        pages.push(picture);
        page_frames.push(frame);
        selection_areas.push(area);

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
        session, sizes, tops, zoom, fit_to_width: true,
        pages, page_frames, selection_areas, thumbs, thumb_buttons,
        painted_pages: HashSet::new(),
        painted_thumbs: HashSet::new(),
        active: 0,
    }));
    *ui.drag_anchor.borrow_mut() = None;
    ui.selection_zoom.set(zoom);
    ui.title.set_title(label);
    ui.dirty.set(false);
    ui.title.set_subtitle("WordCraft · edición experimental · sin guardar");
    ui.stack.set_visible_child_name("pages");
    ui.scroll.vadjustment().set_value(0.0);
    ui.thumbnails_scroll.vadjustment().set_value(0.0);
    update_status(ui);
    refresh_pages(ui);
    refresh_thumbnails(ui);
    refresh_selection(ui);
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
    selection_demo: bool,
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
    let title = WindowTitle::new("WordCraft GNOME", "Edición experimental · sin guardar");
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
    let zoom_out = gtk::Button::with_label("−");
    zoom_out.set_tooltip_text(Some("Reducir zoom (Ctrl+-)"));
    let zoom_in = gtk::Button::with_label("+");
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
    // Si el panel lateral está cerrado, su menú de aplicación debe seguir
    // accesible desde la cabecera principal.
    let compact_example_button = gtk::Button::with_label("Abrir ejemplo");
    let compact_app_popover = gtk::Popover::new();
    let compact_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    compact_content.set_margin_top(12);
    compact_content.set_margin_bottom(12);
    compact_content.set_margin_start(12);
    compact_content.set_margin_end(12);
    compact_content.append(&compact_example_button);
    compact_app_popover.set_child(Some(&compact_content));
    let compact_app_menu = gtk::MenuButton::new();
    compact_app_menu.set_icon_name("open-menu-symbolic");
    compact_app_menu.set_tooltip_text(Some("Menú de la aplicación"));
    compact_app_menu.set_popover(Some(&compact_app_popover));
    header.pack_end(&compact_app_menu);

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
    document_box.set_focusable(true);
    document_box.set_cursor_from_name(Some("text"));
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
    zoom_in.add_css_class("osd");
    zoom_out.add_css_class("osd");
    zoom_in.add_css_class("circular");
    zoom_out.add_css_class("circular");
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
        selection_data: Rc::new(RefCell::new(SelectionGeometry::default())),
        selection_zoom: Rc::new(Cell::new(1.0)),
        drag_anchor: RefCell::new(None),
        dirty: Cell::new(false),
        closing_confirmed: Cell::new(false),
        document_box, thumbnails_box, scroll, thumbnails_scroll,
        sidebar, sidebar_button,
        page_label, zoom_label, zoom_out, zoom_in, previous, next, stack, title, toasts,
    });


    // Click sets the caret, double-click selects a word and triple-click
    // selects a paragraph. Coordinates are relative to the document stack.
    {
        let click = gtk::GestureClick::new();
        click.set_button(gdk::BUTTON_PRIMARY);
        let document_box = ui.document_box.clone();
        let ui = ui.clone();
        click.connect_pressed(move |gesture, count, x, y| {
            let Some(pos) = pos_at_canvas(&ui, x, y) else { return };
            ui.document_box.grab_focus();
            let shift = gesture.current_event_state().contains(gdk::ModifierType::SHIFT_MASK);
            let result = {
                let mut value = ui.current.borrow_mut();
                let Some(doc) = value.as_mut() else { return };
                selection::set_caret(&mut doc.session, &pos, shift)
                    .and_then(|_| {
                        let cmd = if count == 2 { Some("select.word") }
                            else if count >= 3 { Some("select.paragraph") }
                            else { None };
                        if let Some(cmd) = cmd {
                            doc.session.run(cmd, &serde_json::json!({}))
                                .map_err(|e| e.to_string())?;
                        }
                        Ok(())
                    })
            };
            if let Err(e) = result { ui.toasts.add_toast(Toast::new(&e)); }
            refresh_selection(&ui);
        });
        document_box.add_controller(click);
    }
    // One drag gesture for the entire document, including page boundaries.
    {
        let drag = gtk::GestureDrag::new();
        drag.set_button(gdk::BUTTON_PRIMARY);
        {
            let ui = ui.clone();
            drag.connect_drag_begin(move |_, x, y| {
                *ui.drag_anchor.borrow_mut() = pos_at_canvas(&ui, x, y);
                ui.document_box.grab_focus();
            });
        }
        {
            let ui = ui.clone();
            drag.connect_drag_update(move |gesture, dx, dy| {
                let Some((sx, sy)) = gesture.start_point() else { return };
                let Some(anchor) = ui.drag_anchor.borrow().clone() else { return };
                let Some(focus) = pos_at_canvas(&ui, sx + dx, sy + dy) else { return };
                let result = {
                    let mut value = ui.current.borrow_mut();
                    value.as_mut().map(|doc| selection::select_range(
                        &mut doc.session, &anchor, &focus,
                    ))
                };
                if let Some(Err(e)) = result { ui.toasts.add_toast(Toast::new(&e)); }
                refresh_selection(&ui);
            });
        }
        {
            let ui = ui.clone();
            drag.connect_drag_end(move |_, _, _| {
                ui.drag_anchor.borrow_mut().take();
                refresh_selection(&ui);
            });
        }
        ui.document_box.add_controller(drag);
    }

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
        let compact_app_popover = compact_app_popover.clone();
        compact_example_button.connect_clicked(move |_| {
            opened(&ui, sample_session(), "Documento de ejemplo");
            compact_app_popover.popdown();
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
        let compact_menu = compact_app_menu.clone();
        split.connect_show_sidebar_notify(move |split| {
            let visible = split.shows_sidebar();
            compact_menu.set_visible(!visible);
            if ui.sidebar_button.is_active() != visible {
                ui.sidebar_button.set_active(visible);
            }
            if visible {
                let ui = ui.clone();
                glib::idle_add_local_once(move || refresh_thumbnails(&ui));
            }
            // Ajustar al ancho sigue siendo un modo, no un zoom fijo. Tras
            // la animación del panel el ancho disponible puede cambiar.
            let ui = ui.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
                refit_if_needed(&ui);
            });
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
            if let Some(doc) = ui.current.borrow_mut().as_mut() {
                doc.fit_to_width = false;
            }
            zoom_to(&ui, 1.0);
            menu.popdown();
        });
    }
    {
        let ui = ui.clone();
        let open_button = open_button.clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, modifiers| {
            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
            let control = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            if ui.document_box.has_focus() {
                let movement = match key {
                    gdk::Key::Left => Some(if control { "wordLeft" } else { "left" }),
                    gdk::Key::Right => Some(if control { "wordRight" } else { "right" }),
                    gdk::Key::Up => Some("up"),
                    gdk::Key::Down => Some("down"),
                    gdk::Key::Home => Some(if control { "docStart" } else { "home" }),
                    gdk::Key::End => Some(if control { "docEnd" } else { "end" }),
                    _ => None,
                };
                if let Some(action) = movement {
                    selected_command(&ui, action, shift);
                    return glib::Propagation::Stop;
                }
            }
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
            if ui.document_box.has_focus() {
                if key == gdk::Key::c || key == gdk::Key::C {
                    copy_selection(&ui);
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::a || key == gdk::Key::A {
                    selected_command(&ui, "select-all", false);
                    return glib::Propagation::Stop;
                }
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
    window.present();
    if sidebar_on_start {
        // Cuando se aplica AdwBreakpoint, el split puede cambiar por sí mismo
        // la visibilidad del panel; abrimos la miniatura tras estabilizarlo.
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
            ui.sidebar.set_show_sidebar(true);
            ui.sidebar_button.set_active(true);
            refresh_thumbnails(&ui);
            let ui2 = ui.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(420), move || {
                refit_if_needed(&ui2);
            });
        });
    }
    if sample_on_start {
        let ui = ui.clone();
        glib::idle_add_local_once(move || fit_width(&ui));
    }
    if selection_demo {
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(650), move || {
            {
                let mut value = ui.current.borrow_mut();
                if let Some(doc) = value.as_mut() {
                    let _ = doc.session.run("select.text", &serde_json::json!({
                        "text": "The Open Studio Handbook"
                    }));
                }
            }
            refresh_selection(&ui);
        });
    }
}

fn main() {
    if std::env::args().any(|arg| arg == "--version") {
        println!("WordCraft GNOME · escritura y edición · fase 3B");
        return;
    }
    let sample_on_start = std::env::args().any(|arg| arg == "--sample");
    let sidebar_on_start = std::env::args().any(|arg| arg == "--sidebar");
    let narrow_on_start = std::env::args().any(|arg| arg == "--narrow");
    let dark_on_start = std::env::args().any(|arg| arg == "--dark");
    let selection_demo = std::env::args().any(|arg| arg == "--select-demo");
    // Permite probar versiones simultáneas sin activar una ventana anterior por D-Bus.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        make_window(app, sample_on_start, sidebar_on_start, narrow_on_start, dark_on_start, selection_demo)
    });
    app.run_with_args(&["wordcraft-gnome"]);
}
