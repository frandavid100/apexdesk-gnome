//! Editor experimental GNOME para WordCraft: motor y maquetación reutilizados.
//! La fase 3D guarda copias DOCX/ODT mediante los comandos de WordCraft.
use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay, ToolbarView, WindowTitle};
use glib::value::ToValue;
use gtk::{gdk, gio, glib};
use std::{cell::{Cell, RefCell}, collections::HashSet, path::{Path, PathBuf}, rc::Rc};
use wordcraft_doc::Pos;
use wordcraft_gnome::selection::{self, SelectionGeometry};
use wordcraft_gnome::editing::{self, Edit};
use wordcraft_gnome::formatting::{self, Format};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, render_page_premultiplied as render_page, sample_session};
use wordcraft_gnome::viewer_layout::{self, THUMB_SCALE};

const APP_ID: &str = "io.github.frandavid100.WordCraftGnome.PapersPreview";
const STEPS: &[f32] = &[0.5, 0.67, 0.75, 0.85, 1.0, 1.25, 1.5, 1.75, 2.0];

struct Current {
    session: Session,
    sizes: Vec<(f32, f32)>,
    arrangement: viewer_layout::Arrangement,
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
    status_active: Option<usize>,
}

struct Ui {
    current: RefCell<Option<Current>>,
    document_box: gtk::Fixed,
    thumbnails_box: gtk::Box,
    scroll: gtk::ScrolledWindow,
    thumbnails_scroll: gtk::ScrolledWindow,
    sidebar: adw::OverlaySplitView,
    sidebar_button: gtk::ToggleButton,
    sidebar_pages: gtk::Stack,
    search: RefCell<Option<Rc<SearchPanel>>>,
    paragraph: RefCell<Option<Rc<ParagraphPopover>>>,
    search_highlights: Rc<RefCell<Vec<selection::Rect>>>,
    document_name: RefCell<String>,
    layout_pending: Cell<bool>,
    viewport_width: Cell<i32>,
    zoom_label: gtk::Label,
    zoom_out: gtk::Button,
    zoom_in: gtk::Button,
    stack: gtk::Stack,
    title: WindowTitle,
    toasts: ToastOverlay,
    selection_data: Rc<RefCell<SelectionGeometry>>,
    selection_zoom: Rc<Cell<f32>>,
    caret_motion: Rc<RefCell<wordcraft_gnome::caret_motion::CaretMotion>>,
    caret_tick_active: Cell<bool>,
    drag_anchor: RefCell<Option<Pos>>,
    dirty: Cell<bool>,
    thumbnail_refresh_pending: Cell<bool>,
    closing_confirmed: Cell<bool>,
    tab_page: RefCell<Option<adw::TabPage>>,
    closed: Cell<bool>,
    restoring: Cell<bool>,
    autosave_error: RefCell<Option<String>>,
    pending_tab_close: RefCell<Option<(adw::TabView,adw::TabPage)>>,
    save_path: RefCell<Option<PathBuf>>,
    source_path: RefCell<Option<PathBuf>>,
    format_toolbar: gtk::ScrolledWindow,
    format_bold: gtk::ToggleButton,
    format_italic: gtk::ToggleButton,
    format_underline: gtk::ToggleButton,
    format_syncing: Cell<bool>,
    undo_button: gtk::Button,
    redo_button: gtk::Button,
    font_button: gtk::Button,
    size_input: gtk::SpinButton,
}

fn texture(img: wordcraft_gnome::PageImage) -> gdk::MemoryTexture {
    let bytes = glib::Bytes::from_owned(img.rgba);
    gdk::MemoryTexture::new(
        img.width as i32, img.height as i32,
        gdk::MemoryFormat::R8g8b8a8Premultiplied, &bytes, img.width as usize * 4,
    )
}


fn refresh_selection(ui: &Rc<Ui>) {
    let snapshot = {
        let Ok(mut value) = ui.current.try_borrow_mut() else { return };
        let Some(doc) = value.as_mut() else { return };
        selection::geometry(&mut doc.session)
    };
    let now = ui.document_box.frame_clock().map(|c| c.frame_time()).unwrap_or(0);
    let animate = ui.document_box.is_mapped()
        && ui.document_box.settings().property::<bool>("gtk-enable-animations")
        && snapshot.highlights.is_empty() && ui.drag_anchor.borrow().is_none();
    ui.caret_motion.borrow_mut().retarget(snapshot.caret.clone(), now, animate);
    *ui.selection_data.borrow_mut() = snapshot;
    if ui.caret_motion.borrow().is_moving() && !ui.caret_tick_active.replace(true) {
        let weak = Rc::downgrade(ui);
        ui.document_box.add_tick_callback(move |widget, clock| {
            let Some(ui) = weak.upgrade() else { return glib::ControlFlow::Break };
            let enabled = widget.settings().property::<bool>("gtk-enable-animations");
            let moving = {
                let mut motion = ui.caret_motion.borrow_mut();
                if !enabled || ui.closed.get() { motion.finish(); }
                motion.advance(clock.frame_time());
                motion.is_moving()
            };
            if let Some(doc) = ui.current.borrow().as_ref() {
                for area in &doc.selection_areas { area.queue_draw(); }
            }
            if moving { glib::ControlFlow::Continue } else {
                ui.caret_tick_active.set(false);
                glib::ControlFlow::Break
            }
        });
    }
    if let Ok(value) = ui.current.try_borrow() {
        if let Some(doc) = value.as_ref() {
            for area in &doc.selection_areas { area.queue_draw(); }
        }
    }
    refresh_format_controls(ui);
}

/// Reflect the actual WordCraft formatting state, rather than leaving B/I/U
/// buttons visually disconnected from the selected text.
fn refresh_format_controls(ui: &Rc<Ui>) {
    let state = {
        let Ok(mut current) = ui.current.try_borrow_mut() else { return };
        current.as_mut()
            .and_then(|doc| doc.session.run("format.state", &serde_json::json!({})).ok())
    };
    if let Some(doc) = ui.current.borrow().as_ref() {
        ui.undo_button.set_sensitive(doc.session.can_undo());
        ui.redo_button.set_sensitive(doc.session.can_redo());
    }
    let Some(state) = state else { return };
    ui.format_syncing.set(true);
    for (button, property) in [
        (&ui.format_bold, "bold"),
        (&ui.format_italic, "italic"),
        (&ui.format_underline, "underline"),
    ] {
        let active = state.get(property).and_then(|x| x.as_bool()).unwrap_or(false);
        button.set_active(active);
    }
    if let Some(font) = state["font"].as_str() {
        ui.font_button.set_label(font);
    } else { ui.font_button.set_label("Varias fuentes"); }
    if let Some(size) = state["size"].as_f64() { ui.size_input.set_value(size); }
    else { ui.size_input.set_text(""); }
    ui.format_syncing.set(false);
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


/// Reflow using the existing WordCraft session. Rebuild the page widgets only
/// when the pagination or the dimensions really changed.
fn after_content_change(ui: &Rc<Ui>) {
    let (rebuild, old_zoom, fit_width_mode, old_scroll) = {
        let mut state = ui.current.borrow_mut();
        let Some(doc) = state.as_mut() else { return };
        let layout = doc.session.layout();
        let sizes: Vec<(f32, f32)> = layout.pages.iter().map(|p| (p.w, p.h)).collect();
        let needs_rebuild = sizes != doc.sizes;
        if !needs_rebuild {
            for index in doc.painted_pages.drain() {
                doc.pages[index].set_paintable(None::<&gdk::Paintable>);
            }
        }
        (needs_rebuild, doc.zoom, doc.fit_to_width, ui.scroll.vadjustment().value())
    };
    if rebuild {
        let prior = ui.current.borrow_mut().take();
        if let Some(previous) = prior {
            let name = ui.document_name.borrow().clone();
            let saved_path = ui.save_path.borrow().clone();
            let source_path = ui.source_path.borrow().clone();
            opened(ui, previous.session, &name);
            ui.save_path.replace(saved_path);
            ui.source_path.replace(source_path);
            if !fit_width_mode {
                if let Some(doc) = ui.current.borrow_mut().as_mut() {
                    doc.fit_to_width = false;
                }
                zoom_to(ui, old_zoom);
            }
            let ui = ui.clone();
            glib::idle_add_local_once(move || {
                if fit_width_mode { fit_width(&ui); }
                let vadj = ui.scroll.vadjustment();
                vadj.set_value(old_scroll.min((vadj.upper()-vadj.page_size()).max(0.0)));
                refresh_pages(&ui);
                refresh_thumbnails(&ui);
                refresh_selection(&ui);
                caret_ensure_visible(&ui);
                ui.document_box.grab_focus();
            });
        }
    } else {
        refresh_pages(ui);
        schedule_thumbnail_refresh(ui);
        refresh_selection(ui);
        let ui = ui.clone();
        glib::idle_add_local_once(move || caret_ensure_visible(&ui));
    }
    if let Some(s)=ui.search.borrow().as_ref(){s.schedule();}
    ui.dirty.set(true);

    sync_tab(ui);
}

fn apply_user_edit(ui: &Rc<Ui>, edit: Edit) {
    let changed = {
        let mut value = ui.current.borrow_mut();
        value.as_mut().map(|doc| editing::apply(&mut doc.session, edit))
    };
    match changed {
        Some(Ok(true)) => {
            let first_edit = !ui.dirty.get();
            after_content_change(ui);
            if first_edit {
                ui.toasts.add_toast(Toast::new(
                    "Cambios pendientes. Usa Guardar para conservarlos en una copia.",
                ));
            }
        }
        Some(Ok(false)) | None => {}
        Some(Err(err)) => ui.toasts.add_toast(Toast::new(&err)),
    }
}



/// Apply formatting through WordCraft. A collapsed selection may change the
/// pending formatting of future typing without mutating existing paragraphs.
fn apply_user_format(ui: &Rc<Ui>, action: Format) {
    apply_format_operation(ui, move |session| formatting::apply(session, action));
}

fn apply_format_operation(ui: &Rc<Ui>, operation: impl FnOnce(&mut Session) -> Result<(), String>) {
    let result = {
        let mut state = ui.current.borrow_mut();
        state.as_mut().map(|doc| {
            let old_rev = doc.session.rev();
            operation(&mut doc.session)
                .map(|_| doc.session.rev() != old_rev)
        })
    };
    match result {
        Some(Ok(true)) => {
            let first = !ui.dirty.get();
            after_content_change(ui);
            if first {
                ui.toasts.add_toast(Toast::new(
                    "Formato modificado. Usa Guardar para conservar los cambios.",
                ));
            }
        },
        Some(Ok(false)) => refresh_selection(ui),
        Some(Err(err)) => ui.toasts.add_toast(Toast::new(&err)),
        None => {},
    }
    ui.document_box.grab_focus();
}

fn ask_discard(ui: &Rc<Ui>, parent: &ApplicationWindow, action: impl FnOnce() + 'static) {
    if !ui.dirty.get() { action(); return; }
    let dialog = adw::MessageDialog::new(Some(parent), Some("¿Guardar los cambios?"),
        Some("Si continúas sin guardar, se perderán los cambios del documento."));
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("discard", "Descartar");
    dialog.add_response("save", "Guardar");
    dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("save"));
    dialog.set_close_response("cancel");
    let once = Rc::new(RefCell::new(Some(action)));
    let ui = ui.clone();
    let parent = parent.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "discard" {
            if let Some(doc)=ui.current.borrow_mut().as_mut(){doc.session.discard_recovery_copy();doc.session.finish_recovery();}
            if let Some(action) = once.borrow_mut().take() { action(); }
        } else if response == "save" {
            let once = once.clone();
            save_document(&ui, &parent, false, Box::new(move || {
                if let Some(action) = once.borrow_mut().take() { action(); }
            }));
        } else {cancel_tab_close(&ui);}
    });
    dialog.present();
}

fn save_to(ui: &Rc<Ui>, path: &Path) -> bool {
    // Protect the originally opened file during this experimental phase.
    let original = ui.source_path.borrow().clone();
    if wordcraft_gnome::preferences::autosave_mode()!=2 && original.as_ref().is_some_and(|p| p == path ||
        p.canonicalize().ok().zip(path.canonicalize().ok()).is_some_and(|(a,b)| a == b)) {
        ui.toasts.add_toast(Toast::new("Elige otro nombre para proteger el documento original."));
        return false;
    }
    let result = {
        let mut state = ui.current.borrow_mut();
        let Some(doc) = state.as_mut() else { return false };
        wordcraft_gnome::saving::save(&mut doc.session, path)
    };
    match result {
        Ok(()) => {
            ui.save_path.replace(Some(path.to_path_buf()));
            ui.dirty.set(false);
            ui.document_name.replace(path.file_name().unwrap_or_default().to_string_lossy().into_owned());
            if let Some(doc)=ui.current.borrow_mut().as_mut(){doc.session.finish_recovery();}
            sync_tab(ui);
            ui.toasts.add_toast(Toast::new("Documento guardado"));
            true
        }
        Err(e) => { ui.toasts.add_toast(Toast::new(&format!("No se ha guardado: {e}"))); false }
    }
}

fn save_document(ui: &Rc<Ui>, parent: &ApplicationWindow, save_as: bool, done: Box<dyn FnOnce()>) {
    if ui.current.borrow().is_none() {cancel_tab_close(ui); return;}
    let existing = ui.save_path.borrow().clone();
    if !save_as {
        if let Some(path) = existing { if save_to(ui, &path) { done(); } else {cancel_tab_close(ui);} return; }
    }
    let dialog = gtk::FileDialog::new();
    dialog.set_title("Guardar una copia — DOCX u ODT");
    dialog.set_modal(true);
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Documentos (.docx, .odt)"));
    filter.add_pattern("*.docx"); filter.add_pattern("*.odt");
    let filters = gio::ListStore::new::<gtk::FileFilter>(); filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let source = existing.or_else(|| ui.source_path.borrow().clone());
    let name = source.as_ref().and_then(|p| p.file_stem()).map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Documento".into());
    let ext = source.as_ref().and_then(|p| p.extension()).and_then(|x| x.to_str()).unwrap_or("docx");
    dialog.set_initial_name(Some(&format!("{name}-copia.{ext}")));
    if let Some(folder) = source.as_ref().and_then(|p| p.parent()) {
        dialog.set_initial_folder(Some(&gio::File::for_path(folder)));
    }
    let ui = ui.clone();
    dialog.save(Some(parent), None::<&gio::Cancellable>, move |result| {
        match result {
            Ok(file) => {
                let Some(path) = file.path() else {
                    ui.toasts.add_toast(Toast::new("Elige un archivo local")); cancel_tab_close(&ui); return;
                };
                // Require an extension so GTK's overwrite confirmation covers the actual target.
                if path.extension().is_none() {
                    ui.toasts.add_toast(Toast::new("Incluye .docx o .odt en el nombre del archivo")); cancel_tab_close(&ui); return;
                }
                if save_to(&ui, &path) { done(); } else {cancel_tab_close(&ui);}
            }
            Err(e) if e.matches(gtk::DialogError::Dismissed) || e.matches(gtk::DialogError::Cancelled) => {cancel_tab_close(&ui);},
            Err(e) => {ui.toasts.add_toast(Toast::new(&format!("No se pudo abrir el guardado: {e}")));cancel_tab_close(&ui);},
        }
    });
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
    let Ok(mut current) = ui.current.try_borrow_mut() else { return };
    if let Some(doc) = current.as_mut() {
        ui.zoom_label.set_text(&format!("{:.0} %", doc.zoom * 100.0));
        ui.zoom_out.set_sensitive(doc.zoom > viewer_layout::MIN_ZOOM);
        ui.zoom_in.set_sensitive(doc.zoom < viewer_layout::MAX_ZOOM);
        if doc.status_active != Some(doc.active) {
            if let Some(previous)=doc.status_active {
                doc.thumb_buttons[previous].remove_css_class("selected-thumb");
            }
            doc.thumb_buttons[doc.active].add_css_class("selected-thumb");
            doc.status_active=Some(doc.active);
        }
    } else {
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
            doc.active = doc.arrangement.page_at(ui.scroll.hadjustment().value()+ui.scroll.hadjustment().page_size()*0.3,viewport);
            let visible = doc.arrangement.visible(top,bottom);
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
                let (start,end)=doc.thumb_buttons[i].compute_bounds(&ui.thumbnails_box).map(|r|(r.y()as f64,(r.y()+r.height())as f64)).filter(|(_,end)|*end>0.0).unwrap_or((offset,offset+height));
                let is_visible = start <= bottom + 250.0 && end >= top - 250.0;
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
        borrowed.as_ref().and_then(|doc| doc.arrangement.pages.get(index).map(|p|p.y))
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
        let changed=(zoom - doc.zoom).abs()>=0.005;
        if !changed {drop(borrowed);layout_pages(ui);return;}
        doc.zoom = zoom;
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
    layout_pages(ui);
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

fn viewport_width(ui:&Ui)->f64 {
    let split=ui.sidebar.allocated_width() as f64;
    let side=if ui.sidebar.shows_sidebar()&&!ui.sidebar.is_collapsed(){ui.sidebar.sidebar().map(|w|w.allocated_width()as f64).unwrap_or(0.0)}else{0.0};
    (ui.scroll.allocated_width()as f64).min((split-side).max(0.0)).max(250.0)
}
fn layout_pages(ui:&Rc<Ui>){
    let width=viewport_width(ui);
    let Ok(mut value)=ui.current.try_borrow_mut()else{return;};let Some(doc)=value.as_mut()else{return;};
    let next=viewer_layout::arrange(&doc.sizes,doc.zoom,width);
    for(frame,p)in doc.page_frames.iter().zip(&next.pages){ui.document_box.move_(frame,p.x,p.y);}
    ui.document_box.set_size_request(next.width.ceil()as i32,next.height.ceil()as i32);
    doc.arrangement=next;
}
fn fit_width(ui:&Rc<Ui>){
    let width=viewport_width(ui);
    let zoom={let mut value=ui.current.borrow_mut();let Some(doc)=value.as_mut()else{return;};
        doc.fit_to_width=true;
        let natural=viewer_layout::arrange(&doc.sizes,1.0,width);
        let maximum=doc.sizes.chunks(natural.columns).map(|c|c.iter().map(|s|s.0 as f64).sum::<f64>()).fold(1.0,f64::max);
        ((width-viewer_layout::PAGE_GAP*(natural.columns+1)as f64-2.0)/maximum)as f32
    };
    zoom_to(ui,zoom);
}
fn schedule_layout(ui:&Rc<Ui>){
    if ui.layout_pending.replace(true){return;}
    let weak=Rc::downgrade(ui);glib::timeout_add_local_once(std::time::Duration::from_millis(80),move||{
        let Some(ui)=weak.upgrade().filter(|u|!u.closed.get())else{return;};ui.layout_pending.set(false);
        let fit=ui.current.borrow().as_ref().is_some_and(|d|d.fit_to_width);
        if fit {fit_width(&ui);}else{layout_pages(&ui);refresh_pages(&ui);}
    });
}
fn clear_canvas(canvas:&gtk::Fixed){while let Some(child)=canvas.first_child(){canvas.remove(&child);}}

fn opened(ui: &Rc<Ui>, mut session: Session, label: &str) {
    if !session.recovery_ready() {
        if let Err(e)=session.enable_recovery(wordcraft_gnome::preferences::recovery_root(),std::sync::Arc::new(||{})) {
            ui.toasts.add_toast(Toast::new(&format!("Recuperación: {e}")));
        }
    }
    let layout = session.layout();
    let sizes: Vec<(f32,f32)> = layout.pages.iter().map(|p| (p.w, p.h)).collect();
    if sizes.is_empty() {
        ui.toasts.add_toast(Toast::new("El documento no contiene páginas"));
        return;
    }
    let zoom = 1.0;
    let arrangement=viewer_layout::arrange(&sizes,zoom,viewport_width(ui));
    ui.caret_motion.borrow_mut().retarget(None, 0, false);
    ui.search_highlights.borrow_mut().clear();
    if let Some(s)=ui.search.borrow().as_ref(){s.schedule();}
    clear_canvas(&ui.document_box);
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
        frame.set_valign(gtk::Align::Start);
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
            let motion = ui.caret_motion.clone();
            let search = ui.search_highlights.clone();
            area.set_draw_func(move |_, cr, _, _| {
                let selection = data.borrow();
                let z = f64::from(scale.get());
                for r in search.borrow().iter().filter(|r| r.page == i) {
                    cr.set_source_rgba(0.94, 0.57, 0.12, 0.35);
                    cr.rectangle(r.x*z, r.y*z, r.width*z, r.height*z);let _=cr.fill();
                }
                for r in selection.highlights.iter().filter(|r| r.page == i) {
                    cr.set_source_rgba(0.20, 0.44, 0.88, 0.30);
                    cr.rectangle(r.x*z, r.y*z, r.width*z, r.height*z);
                    let _ = cr.fill();
                }
                if let Some(r) = motion.borrow().position().filter(|r| r.page == i) {
                    cr.set_source_rgba(0.08, 0.30, 0.74, 1.0);
                    cr.rectangle(r.x*z, r.y*z, (r.width*z).max(1.5), (r.height*z).max(2.0));
                    let _ = cr.fill();
                }
            });
        }
        frame.add_overlay(&area);
        let position=&arrangement.pages[i];
        ui.document_box.put(&frame,position.x,position.y);
        pages.push(picture);
        page_frames.push(frame);
        selection_areas.push(area);

        let thumb = gtk::Picture::new();
        thumb.set_can_shrink(false);
        thumb.set_content_fit(gtk::ContentFit::Fill);
        thumb.set_halign(gtk::Align::Center);
        thumb.set_hexpand(false);thumb.set_vexpand(false);
        thumb.set_size_request(
            (width * THUMB_SCALE).ceil() as i32,
            (height * THUMB_SCALE).ceil() as i32,
        );
        thumb.add_css_class("thumbnail-sheet");
        let number = gtk::Label::new(Some(&(i + 1).to_string()));
        number.add_css_class("dim-label");
        let content=gtk::Box::new(gtk::Orientation::Vertical,5);content.set_halign(gtk::Align::Center);content.set_vexpand(false);
        let button=gtk::Button::new();button.set_child(Some(&thumb));button.add_css_class("flat");button.add_css_class("thumbnail-button");
        button.set_halign(gtk::Align::Center);button.set_valign(gtk::Align::Start);button.set_vexpand(false);thumb.set_valign(gtk::Align::Center);
        content.append(&button);content.append(&number);content.set_margin_top(7);content.set_margin_bottom(14);
        {
            let ui = ui.clone();
            button.connect_clicked(move |_| jump_to(&ui, i));
        }
        ui.thumbnails_box.append(&content);
        thumbs.push(thumb);
        thumb_buttons.push(button);
    }

    ui.current.replace(Some(Current {
        session, sizes, arrangement, zoom, fit_to_width: true,
        pages, page_frames, selection_areas, thumbs, thumb_buttons,
        painted_pages: HashSet::new(),
        painted_thumbs: HashSet::new(),
        active: 0, status_active: None,
    }));
    *ui.drag_anchor.borrow_mut() = None;
    ui.selection_zoom.set(zoom);
    ui.document_name.replace(label.to_owned());
    ui.dirty.set(false);
    sync_tab(ui);
    ui.save_path.replace(None);
    ui.source_path.replace(ui.current.borrow().as_ref().and_then(|d| d.session.path.clone()));

    ui.format_toolbar.set_visible(true);
    ui.stack.set_visible_child_name("pages");
    layout_pages(ui);
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
      .document-scroll { background: @window_bg_color; }
      .document-sheet { background: white; box-shadow: 0 1px 4px alpha(black, 0.15); }
      .thumbnail-sheet { background: white; box-shadow: 0 1px 3px alpha(black, 0.16); }
      .thumbnail-button { padding: 8px; min-width: 0; min-height: 0; }
      .thumbnail-button.selected-thumb { background: alpha(@window_fg_color, 0.09); }
      button.search-replace { min-width: 24px; min-height: 24px; padding: 0; }
      .style-popover button, .format-popover button { font-weight: normal; }
      .style-row.current-style { background: alpha(@window_fg_color, 0.14); border-radius: 6px; }
      .format-popover button:checked { background: alpha(@window_fg_color, 0.14); }
      .page-indicator, .zoom-value { font-feature-settings: 'tnum'; }
    ");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

include!("../gnome_host.rs");
include!("../gnome_format_panels.rs");
include!("../gnome_tables.rs");
include!("../gnome_styles.rs");
include!("../gnome_search.rs");
include!("../gnome_paragraph.rs");
include!("../gnome_view_tests.rs");

fn build_tab(
    app: &Application,
    host: &Rc<Workspace>,
    sample_on_start: bool,
    sidebar_on_start: bool,
    narrow_on_start: bool,
    dark_on_start: bool,
    selection_demo: bool,
    editing_demo: bool,
    formatting_demo: bool,
    initial_document: Option<(Session, String)>,
) {
    let initial_loaded = initial_document.is_some();
    if dark_on_start {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    }
    install_css();
    let window = host.window.clone();
    // Composición oficial: panel adaptable y dos páginas de barras nativas.
    let side_header = HeaderBar::new();
    let side_title = gtk::Label::new(Some("Editor de documentos"));
    side_title.add_css_class("heading");side_title.set_ellipsize(gtk::pango::EllipsizeMode::End);side_title.set_max_width_chars(16);
    side_header.set_title_widget(Some(&side_title));
    let search_toggle=gtk::ToggleButton::new();search_toggle.set_icon_name("edit-find-symbolic");search_toggle.set_tooltip_text(Some("Buscar en el documento (Ctrl+F)"));side_header.pack_start(&search_toggle);

    // Menú de aplicación, independiente del menú de documento.
    let example_button = gtk::Button::with_label("Abrir ejemplo");
    let open_button = gtk::Button::with_label("Abrir… (Ctrl+O)");
    let new_window_button = gtk::Button::with_label("Nueva ventana (Ctrl+Shift+N)");
    let app_popover = gtk::Popover::new();
    let app_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    app_content.set_margin_top(12);
    app_content.set_margin_bottom(12);
    app_content.set_margin_start(12);
    app_content.set_margin_end(12);
    app_content.append(&open_button);
    app_content.append(&new_window_button);
    app_content.append(&example_button);
    let preferences_button=gtk::Button::with_label("Preferencias");
    let recover_button=gtk::Button::with_label("Recuperar documentos…");
    app_content.append(&recover_button);app_content.append(&preferences_button);
    app_popover.set_child(Some(&app_content));
    let app_menu = gtk::MenuButton::new();
    app_menu.set_icon_name("open-menu-symbolic");
    app_menu.set_tooltip_text(Some("Menú de la aplicación"));
    app_menu.set_popover(Some(&app_popover));
    side_header.pack_end(&app_menu);

    let header = HeaderBar::new();
    let title = WindowTitle::new("WordCraft GNOME", "");
    header.set_title_widget(Some(&title));

    let sidebar_button = gtk::ToggleButton::new();
    sidebar_button.set_icon_name("sidebar-show-symbolic");
    sidebar_button.set_tooltip_text(Some("Mostrar u ocultar miniaturas (F9)"));
    header.pack_start(&sidebar_button);
    let compact_search=gtk::Button::from_icon_name("edit-find-symbolic");compact_search.set_tooltip_text(Some("Buscar (Ctrl+F)"));header.pack_start(&compact_search);
    let new_document=gtk::Button::with_label("+");
    new_document.add_css_class("flat");
    new_document.set_tooltip_text(Some("Documento nuevo en otra pestaña (Ctrl+N)"));
    header.pack_start(&new_document);

    let save_button = gtk::Button::from_icon_name("document-save-symbolic");
    save_button.set_tooltip_text(Some("Guardar (Ctrl+S)"));
    header.pack_start(&save_button);
    let save_as_button = gtk::Button::with_label("Guardar como…");



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
    menu_content.append(&save_as_button);


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
    let compact_open = gtk::Button::with_label("Abrir… (Ctrl+O)");
    let compact_new = gtk::Button::with_label("Nueva ventana (Ctrl+Shift+N)");
    compact_content.append(&compact_open); compact_content.append(&compact_new);
    compact_content.append(&compact_example_button);
    let compact_preferences=gtk::Button::with_label("Preferencias");
    let compact_recover=gtk::Button::with_label("Recuperar documentos…");
    compact_content.append(&compact_recover);compact_content.append(&compact_preferences);
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


    let stack = gtk::Stack::new();
    let empty = StatusPage::builder()
        .icon_name("x-office-document-symbolic")
        .title("Abre un documento")
        .description("DOCX y ODT con desplazamiento continuo y miniaturas")
        .build();
    stack.add_named(&empty, Some("empty"));

    let document_box = gtk::Fixed::new();
    document_box.set_valign(gtk::Align::Start);
    document_box.set_halign(gtk::Align::Center);
    document_box.set_focusable(true);
    document_box.set_cursor_from_name(Some("text"));
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add_css_class("document-scroll");
    // The canvas is taller than the viewport. Restoring focus must not scroll
    // its whole allocation into view before pointer hit testing runs.
    let viewport=gtk::Viewport::new(None::<&gtk::Adjustment>,None::<&gtk::Adjustment>);
    viewport.set_scroll_to_focus(false);viewport.set_child(Some(&document_box));scroll.set_child(Some(&viewport));

    let thumbnails_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    thumbnails_box.set_valign(gtk::Align::Start);
    let thumbnails_scroll = gtk::ScrolledWindow::new();
    thumbnails_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    thumbnails_scroll.set_vexpand(true);
    thumbnails_scroll.set_child(Some(&thumbnails_box));

    let sidebar_toolbar = ToolbarView::new();
    sidebar_toolbar.add_top_bar(&side_header);
    let sidebar_pages=gtk::Stack::new();sidebar_pages.add_named(&thumbnails_scroll,Some("thumbnails"));
    sidebar_toolbar.set_content(Some(&sidebar_pages));

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
    let zoom_popover = gtk::Popover::new();
    let zoom_options = gtk::Box::new(gtk::Orientation::Vertical, 6);
    zoom_options.set_margin_top(12); zoom_options.set_margin_bottom(12);
    zoom_options.set_margin_start(12); zoom_options.set_margin_end(12);
    zoom_options.append(&zoom_label);
    zoom_options.append(&normal_zoom);
    zoom_options.append(&fit_button);
    zoom_popover.set_child(Some(&zoom_options));
    let zoom_menu = gtk::MenuButton::new();
    zoom_menu.set_icon_name("view-more-symbolic");
    zoom_menu.set_tooltip_text(Some("Opciones de zoom"));
    zoom_menu.add_css_class("osd"); zoom_menu.add_css_class("circular");
    zoom_menu.set_popover(Some(&zoom_popover));
    zoom_menu.set_direction(gtk::ArrowType::Up);
    zoom_buttons.append(&zoom_menu);
    let canvas_overlay = gtk::Overlay::new();
    canvas_overlay.set_child(Some(&scroll));
    canvas_overlay.add_overlay(&zoom_buttons);
    stack.add_named(&canvas_overlay, Some("pages"));
    stack.set_visible_child_name("empty");

    let toasts = ToastOverlay::new();
    toasts.set_child(Some(&stack));
    // A bottom native toolbar for document editing, horizontally scrollable
    // on small windows. It lives in AdwToolbarView, never over the pages.
    let format_bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    format_bar.set_margin_start(10);
    format_bar.set_margin_end(10);
    format_bar.set_margin_top(5);
    format_bar.set_margin_bottom(5);
    let undo_button = gtk::Button::from_icon_name("edit-undo-symbolic");
    let redo_button = gtk::Button::from_icon_name("edit-redo-symbolic");
    let icons = gtk::IconTheme::for_display(&gtk::prelude::WidgetExt::display(&window));
    if !icons.has_icon("edit-undo-symbolic") { undo_button.set_label("Deshacer"); }
    if !icons.has_icon("edit-redo-symbolic") { redo_button.set_label("Rehacer"); }
    undo_button.set_tooltip_text(Some("Deshacer (Ctrl+Z)"));
    redo_button.set_tooltip_text(Some("Rehacer (Ctrl+Shift+Z)"));
    undo_button.set_sensitive(false); redo_button.set_sensitive(false);
    format_bar.append(&undo_button); format_bar.append(&redo_button);
    format_bar.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    let bold_button = gtk::ToggleButton::with_label("B");
    bold_button.set_tooltip_text(Some("Negrita (Ctrl+B)"));
    bold_button.add_css_class("flat");
    let italic_button = gtk::ToggleButton::with_label("I");
    italic_button.set_tooltip_text(Some("Cursiva (Ctrl+I)"));
    italic_button.add_css_class("flat");
    let underline_button = gtk::ToggleButton::with_label("U");
    underline_button.set_tooltip_text(Some("Subrayado (Ctrl+U)"));
    underline_button.add_css_class("flat");
    let text_popover = gtk::Popover::new();
    let text_options = gtk::Box::new(gtk::Orientation::Vertical, 12);
    text_options.set_margin_top(12); text_options.set_margin_bottom(12);
    text_options.set_margin_start(12); text_options.set_margin_end(12);
    let inline_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    inline_buttons.append(&bold_button); inline_buttons.append(&italic_button);
    inline_buttons.append(&underline_button);
    text_options.append(&inline_buttons);
    let font_label = gtk::Label::new(Some("Fuente")); font_label.set_halign(gtk::Align::Start);
    text_options.append(&font_label);
    let font_dialog = gtk::FontDialog::new(); font_dialog.set_title("Elegir fuente");
    let font_button = gtk::Button::with_label("Elegir fuente…");
    font_button.set_tooltip_text(Some("Elegir una familia tipográfica instalada"));
    text_options.append(&font_button);
    let size_label = gtk::Label::new(Some("Tamaño de letra"));
    size_label.set_halign(gtk::Align::Start);
    text_options.append(&size_label);
    let custom_size_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let size_input = gtk::SpinButton::with_range(1.0, 1638.0, 0.5);
    size_input.set_digits(1); size_input.set_value(12.0); size_input.set_hexpand(true);
    size_input.set_tooltip_text(Some("Tamaño en puntos, también con decimales"));
    let apply_size = gtk::Button::with_label("Aplicar");
    custom_size_row.append(&size_input); custom_size_row.append(&apply_size);
    text_options.append(&custom_size_row);
    let sizes = gtk::Grid::new();
    sizes.set_row_spacing(6); sizes.set_column_spacing(6);
    text_options.append(&sizes);
    let more_text=gtk::Button::with_label("Más opciones de texto…");
    text_options.append(&more_text);
    text_popover.set_child(Some(&text_options));
    let text_menu = gtk::MenuButton::new();
    text_menu.set_label("Texto");
    text_menu.set_tooltip_text(Some("Negrita, cursiva, subrayado y tamaño de letra"));
    text_menu.set_popover(Some(&text_popover));
    text_menu.set_direction(gtk::ArrowType::Up);

    let mut format_actions: Vec<(gtk::Button, Format, gtk::Popover)> = Vec::new();
    let style_menu=gtk::MenuButton::new();style_menu.set_label("Estilo");style_menu.set_direction(gtk::ArrowType::Up);style_menu.set_tooltip_text(Some("Aplicar y gestionar estilos"));
    for (index, size) in [10u8, 11, 12, 14, 16, 18, 24, 36].iter().enumerate() {
        let button = gtk::Button::with_label(&format!("{size} pt"));
        sizes.attach(&button, (index % 4) as i32, (index / 4) as i32, 1, 1);
        format_actions.push((button, Format::FontSize(*size), text_popover.clone()));
    }
    let align_menu=gtk::MenuButton::new();align_menu.set_label("Párrafo");align_menu.set_direction(gtk::ArrowType::Up);align_menu.set_tooltip_text(Some("Formato de párrafo"));
    format_bar.append(&style_menu);
    format_bar.append(&align_menu);
    format_bar.append(&text_menu);
    let format_toolbar = gtk::ScrolledWindow::new();
    format_toolbar.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    format_toolbar.set_child(Some(&format_bar));
    format_toolbar.add_css_class("toolbar");
    format_toolbar.set_visible(false);

    let document_toolbar = ToolbarView::new();
    document_toolbar.add_top_bar(&header);
    let tab_bar = adw::TabBar::new(); tab_bar.set_view(Some(&host.tabs));
    tab_bar.set_autohide(true); document_toolbar.add_top_bar(&tab_bar);
    document_toolbar.add_bottom_bar(&format_toolbar);
    document_toolbar.set_content(Some(&toasts));

    let sidebar = adw::OverlaySplitView::new();
    sidebar.set_sidebar(Some(&sidebar_toolbar));
    sidebar.set_content(Some(&document_toolbar));
    sidebar.set_collapsed(narrow_on_start);
    sidebar.set_show_sidebar(sidebar_on_start || !narrow_on_start);
    sidebar_button.set_active(sidebar_on_start || !narrow_on_start);
    compact_app_menu.set_visible(!(sidebar_on_start || !narrow_on_start));
    compact_search.set_visible(!(sidebar_on_start || !narrow_on_start));
    sidebar.set_min_sidebar_width(190.0);
    sidebar.set_max_sidebar_width(240.0);
    sidebar.set_sidebar_width_fraction(0.21);
    sidebar.set_pin_sidebar(true);
    let tab_page = host.tabs.append(&sidebar);
    host.tabs.set_selected_page(&tab_page);

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
        caret_motion: Rc::new(RefCell::new(Default::default())),
        caret_tick_active: Cell::new(false),
        drag_anchor: RefCell::new(None),
        dirty: Cell::new(false), thumbnail_refresh_pending: Cell::new(false),
        closing_confirmed: Cell::new(false),
        tab_page: RefCell::new(Some(tab_page.clone())), closed: Cell::new(false),
        restoring: Cell::new(false), autosave_error: RefCell::new(None),
        pending_tab_close: RefCell::new(None),
        save_path: RefCell::new(None),
        source_path: RefCell::new(None),
        format_toolbar,
        format_bold: bold_button.clone(),
        format_italic: italic_button.clone(),
        format_underline: underline_button.clone(),
        format_syncing: Cell::new(false),
        undo_button: undo_button.clone(), redo_button: redo_button.clone(),
        font_button: font_button.clone(), size_input: size_input.clone(),
        document_box, thumbnails_box, scroll, thumbnails_scroll,
        sidebar, sidebar_button, sidebar_pages,
        search: RefCell::new(None), paragraph: RefCell::new(None), search_highlights: Rc::new(RefCell::new(Vec::new())),
        document_name:RefCell::new(String::new()),layout_pending:Cell::new(false),viewport_width:Cell::new(0),
        zoom_label, zoom_out, zoom_in, stack, title, toasts,
    });


    {let ui=ui.clone();let parent=window.clone();let pop=text_popover.clone();more_text.connect_clicked(move |_|{pop.popdown();show_text_options(&ui,&parent);});}
    let paragraph_popover=install_paragraph_popover(&ui,&align_menu);ui.paragraph.replace(Some(paragraph_popover.clone()));
    if std::env::args().any(|a| a == "--motion-ui-test") {
        assert!(ui.sidebar.shows_sidebar(), "sidebar must be open before mapping");
        let changes = Rc::new(Cell::new(0));
        let count = changes.clone();
        ui.sidebar.connect_show_sidebar_notify(move |_| count.set(count.get() + 1));
        let test_ui = ui.clone(); let test_app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || {
            assert_eq!(changes.get(), 0, "startup must not toggle the sidebar");
            assert!(test_ui.sidebar.shows_sidebar());
            let settings = test_ui.document_box.settings();
            let old = settings.property::<bool>("gtk-enable-animations");
            settings.set_property("gtk-enable-animations", true);
            {
                let mut state = test_ui.current.borrow_mut();
                state.as_mut().unwrap().session.run("caret.docStart", &serde_json::json!({})).unwrap();
            }
            test_ui.caret_motion.borrow_mut().retarget(None, 0, false);
            refresh_selection(&test_ui);
            let revision = test_ui.current.borrow().as_ref().unwrap().session.rev();
            selected_command(&test_ui, "right", false);
            assert!(test_ui.caret_motion.borrow().is_moving());
            assert_ne!(test_ui.caret_motion.borrow().position(), test_ui.selection_data.borrow().caret.as_ref());
            let next = test_ui.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
                assert!(!next.caret_motion.borrow().is_moving(), "frame clock must finish the motion");
                assert_eq!(next.caret_motion.borrow().position(), next.selection_data.borrow().caret.as_ref());
                settings.set_property("gtk-enable-animations", false);
                selected_command(&next, "right", false);
                assert!(!next.caret_motion.borrow().is_moving());
                assert_eq!(next.caret_motion.borrow().position(), next.selection_data.borrow().caret.as_ref());
                assert_eq!(revision, next.current.borrow().as_ref().unwrap().session.rev());
                settings.set_property("gtk-enable-animations", old);
                eprintln!("MOTION_UI_OK"); test_app.quit();
            });
        });
    }
    let search_panel=install_search_panel(&ui,&search_toggle,&compact_search);ui.search.replace(Some(search_panel.clone()));
    let style_popover=install_style_popover(&ui,&style_menu);
    let table_menu=install_table_controls(&ui,&window,&format_bar);
    host.docs.borrow_mut().push((tab_page,ui.clone()));
    install_autosave(&ui);
    {
        let app=app.clone();let host=host.clone();
        new_document.connect_clicked(move |_| blank_tab(&app,&host));
    }
    for button in [preferences_button,compact_preferences] {
        let parent=window.clone();let pop1=app_popover.clone();let pop2=compact_app_popover.clone();
        button.connect_clicked(move |_| {pop1.popdown();pop2.popdown();show_preferences(&parent);});
    }
    for button in [recover_button,compact_recover] {
        let ui=ui.clone();let parent=window.clone();let app=app.clone();let host=host.clone();
        button.connect_clicked(move |_| show_recovery(&ui,&parent,&app,&host));
    }
    for (button, edit) in [(undo_button, Edit::Undo), (redo_button, Edit::Redo)] {
        let ui = ui.clone();
        button.connect_clicked(move |_| {
            apply_user_edit(&ui, edit.clone()); ui.document_box.grab_focus();
        });
    }
    {
        let ui = ui.clone(); let popover = text_popover.clone();
        let parent = window.clone();
        font_button.connect_clicked(move |_| {
            popover.popdown();
            let ui = ui.clone();
            font_dialog.choose_family(Some(&parent), None::<&gtk::pango::FontFamily>,
                None::<&gio::Cancellable>, move |result| {
                    match result {
                        Ok(family) => {
                            let name = family.name();
                            apply_format_operation(&ui, |session| formatting::font(session, &name));
                        }
                        Err(e) if e.matches(gtk::DialogError::Dismissed) || e.matches(gtk::DialogError::Cancelled) => {},
                        Err(e) => ui.toasts.add_toast(Toast::new(&format!("No se pudo elegir fuente: {e}"))),
                    }
                });
        });
    }
    {
        let ui = ui.clone(); let popover = text_popover.clone();
        let input = size_input.clone();
        apply_size.connect_clicked(move |_| {
            input.update();
            let size = input.value(); popover.popdown();
            apply_format_operation(&ui, |session| formatting::size(session, size));
        });
    }
    for (proxy, target, popover) in [
        (compact_open, open_button.clone(), compact_app_popover.clone()),
        (compact_new, new_window_button.clone(), compact_app_popover.clone()),
    ] {
        proxy.connect_clicked(move |_| { popover.popdown(); target.emit_clicked(); });
    }

    for (button, save_as) in [(save_button, false), (save_as_button, true)] {
        let ui = ui.clone();
        let parent = window.clone();
        button.connect_clicked(move |_| save_document(&ui, &parent, save_as, Box::new(|| {})));
    }

    {
        let app = app.clone();
        let popover = app_popover.clone();
        new_window_button.connect_clicked(move |_| {
            popover.popdown();
            make_window(&app, false, false, false, false, false, false, false, None);
        });
    }

    // All formatting commands preserve the selection and return keyboard
    // focus to the editable document.
    for (button, action) in [
        (bold_button, Format::Bold),
        (italic_button, Format::Italic),
        (underline_button, Format::Underline),
    ] {
        let ui = ui.clone();
        let popover = text_popover.clone();
        button.connect_toggled(move |_| {
            if !ui.format_syncing.get() {
                popover.popdown();
                apply_user_format(&ui, action);
            }
        });
    }
    for (button, action, popover) in format_actions {
        let ui = ui.clone();
        button.connect_clicked(move |_| {
            popover.popdown();
            apply_user_format(&ui, action);
        });
    }

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
        let parent = window.clone();
        let app_popover = app_popover.clone();
        example_button.connect_clicked(move |_| {
            app_popover.popdown();
            let next = ui.clone();
            ask_discard(&ui, &parent, move || {
                opened(&next, sample_session(), "Documento de ejemplo");
            });
        });
    }
    {
        let ui = ui.clone();
        let parent = window.clone();
        let compact_app_popover = compact_app_popover.clone();
        compact_example_button.connect_clicked(move |_| {
            compact_app_popover.popdown();
            let next = ui.clone();
            ask_discard(&ui, &parent, move || {
                opened(&next, sample_session(), "Documento de ejemplo");
            });
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
        let compact_search=compact_search.clone();
        split.connect_show_sidebar_notify(move |split| {
            let visible = split.shows_sidebar();
            compact_menu.set_visible(!visible);compact_search.set_visible(!visible);
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
    {let weak=Rc::downgrade(&ui);ui.scroll.hadjustment().connect_changed(move |a|{if let Some(ui)=weak.upgrade(){let width=a.page_size().round()as i32;if width>0&&ui.viewport_width.replace(width)!=width{schedule_layout(&ui);}}});}
    {
        let ui = ui.clone();
        ui.thumbnails_scroll.vadjustment().connect_value_changed(move |_| refresh_thumbnails(&ui));
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
        let menu = zoom_popover.clone();
        fit_button.connect_clicked(move |_| {
            fit_width(&ui);
            menu.popdown();
        });
    }
    {
        let ui = ui.clone();
        let menu = zoom_popover.clone();
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
        let save_parent = window.clone();
        let key_host=host.clone();let key_app=app.clone();
        let new_window_button = new_window_button.clone();
        let controller = gtk::EventControllerKey::new();
        // GTK supplies dead-key composition, accents and input methods.
        // Committed text enters WordCraft; ordinary shortcuts remain in
        // the key controller instead of being converted into characters.
        let input = gtk::IMMulticontext::new();
        input.set_client_widget(Some(&ui.document_box));
        controller.set_im_context(Some(&input));
        {
            let ui = ui.clone();
            input.connect_commit(move |_, text| {
                if ui.document_box.has_focus() && !text.is_empty() {
                    apply_user_edit(&ui, Edit::Insert(text.to_owned()));
                }
            });
        }
        {
            let input = input.clone();
            ui.document_box.connect_notify_local(Some("has-focus"), move |widget, _| {
                if widget.has_focus() { input.focus_in(); } else { input.focus_out(); }
            });
        }
        let key_target=ui.sidebar.clone();
        controller.connect_key_pressed(move |_, key, _, modifiers| {
            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
            let control = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            if ui.document_box.has_focus() {
                if key==gdk::Key::Menu || (shift && key==gdk::Key::F10) {
                    let y=ui.scroll.vadjustment().value() as i32+40;
                    show_table_context_at(&ui,&save_parent,60,y);
                    return glib::Propagation::Stop;
                }
                if (key==gdk::Key::Tab || key==gdk::Key::ISO_Left_Tab) && !control && !modifiers.contains(gdk::ModifierType::ALT_MASK) {
                    let inside=ui.current.borrow().as_ref().is_some_and(|d|wordcraft_gnome::tables::in_table(&d.session));
                    if inside {
                        table_navigate(&ui,shift || key==gdk::Key::ISO_Left_Tab,control);
                        return glib::Propagation::Stop;
                    }
                }
                // Commands which change the document are resolved by
                // WordCraft, including replacement of selected text.
                if !control && !modifiers.contains(gdk::ModifierType::ALT_MASK) {
                    let change = match key {
                        gdk::Key::Return | gdk::Key::KP_Enter => {
                            Some(if shift { Edit::LineBreak } else { Edit::Paragraph })
                        }
                        gdk::Key::BackSpace => Some(Edit::Backspace),
                        gdk::Key::Delete | gdk::Key::KP_Delete => Some(Edit::Delete),
                        _ => None,
                    };
                    if let Some(edit) = change {
                        apply_user_edit(&ui, edit);
                        return glib::Propagation::Stop;
                    }
                }
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
            if key == gdk::Key::n || key == gdk::Key::N {
                if shift {new_window_button.emit_clicked();} else {blank_tab(&key_app,&key_host);}
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::w || key == gdk::Key::W {
                if let Some(page)=key_host.tabs.selected_page(){key_host.tabs.close_page(&page);}
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::f || key == gdk::Key::F || key == gdk::Key::h || key == gdk::Key::H {
                show_search(&ui,&save_parent,key==gdk::Key::h || key==gdk::Key::H);
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::s || key == gdk::Key::S {
                save_document(&ui, &save_parent, shift, Box::new(|| {}));
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::o || key == gdk::Key::O {
                open_button.emit_clicked();
                return glib::Propagation::Stop;
            }
            if ui.document_box.has_focus() {
                let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);
                let format_shortcut = if alt {
                    match key {
                        gdk::Key::_1 | gdk::Key::KP_1 => Some(Format::Heading1),
                        gdk::Key::_2 | gdk::Key::KP_2 => Some(Format::Heading2),
                        gdk::Key::_3 | gdk::Key::KP_3 => Some(Format::Heading3),
                        _ => None,
                    }
                } else if shift {
                    match key {
                        gdk::Key::n | gdk::Key::N => Some(Format::Normal),
                        gdk::Key::m | gdk::Key::M => Some(Format::Outdent),
                        gdk::Key::l | gdk::Key::L => Some(Format::Bullets),
                        _ => None,
                    }
                } else {
                    match key {
                        gdk::Key::b | gdk::Key::B => Some(Format::Bold),
                        gdk::Key::i | gdk::Key::I => Some(Format::Italic),
                        gdk::Key::u | gdk::Key::U => Some(Format::Underline),
                        gdk::Key::l | gdk::Key::L => Some(Format::AlignLeft),
                        gdk::Key::e | gdk::Key::E => Some(Format::AlignCenter),
                        gdk::Key::r | gdk::Key::R => Some(Format::AlignRight),
                        gdk::Key::j | gdk::Key::J => Some(Format::Justify),
                        gdk::Key::m | gdk::Key::M => Some(Format::Indent),
                        _ => None,
                    }
                };
                if let Some(action) = format_shortcut {
                    apply_user_format(&ui, action);
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::z || key == gdk::Key::Z {
                    apply_user_edit(&ui, if shift { Edit::Redo } else { Edit::Undo });
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::y || key == gdk::Key::Y {
                    apply_user_edit(&ui, Edit::Redo);
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::v || key == gdk::Key::V {
                    let ui = ui.clone();
                    if let Some(display) = gdk::Display::default() {
                        display.clipboard().read_text_async(
                            None::<&gio::Cancellable>,
                            move |result| {
                                if let Ok(Some(text)) = result {
                                    if ui.document_box.has_focus() {
                                        apply_user_edit(&ui, Edit::Paste(text.to_string()));
                                    }
                                }
                            },
                        );
                    }
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::x || key == gdk::Key::X {
                    let has_selected = ui.current.borrow().as_ref()
                        .is_some_and(|doc| !doc.session.sel.is_collapsed());
                    if has_selected {
                        copy_selection(&ui);
                        apply_user_edit(&ui, Edit::Delete);
                    }
                    return glib::Propagation::Stop;
                }
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
        key_target.add_controller(controller);
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
        let app = app.clone();
        let host=host.clone();
        let popover = app_popover.clone();
        open_button.connect_clicked(move |_| {
            popover.popdown();
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
            let app = app.clone();
            let host=host.clone();
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
                        if ui.current.borrow().as_ref().is_none_or(|doc|doc.session.is_untouched_blank()) {
                            opened(&ui, session, &label);
                        } else {
                            build_tab(&app,&host,false,true,false,false,false,false,false,Some((session,label)));
                        }
                    }
                    Err(err) => ui.toasts.add_toast(Toast::new(&err)),
                }
            });
        });
    }

    if let Some((session, label)) = initial_document {
        opened(&ui, session, &label);
    } else if sample_on_start { opened(&ui, sample_session(), "Documento de ejemplo"); }
    else {opened(&ui,Session::new(wordcraft_doc::Document::new()),"Documento nuevo");}
    window.present();
    install_table_context_menu(&ui,&window);
    if std::env::args().any(|a|a=="--table-keyboard-test") {
        let u=ui.clone();let app=app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||{
            opened(&u,Session::new(wordcraft_doc::Document::new()),"Prueba de tablas");
            table_command(&u,"insert.table",serde_json::json!({"rows":1,"cols":2}));
            eprintln!("TABLE_KEYBOARD_READY");
            glib::timeout_add_local_once(std::time::Duration::from_secs(6),move||{
            {let s=u.current.borrow();let d=s.as_ref().unwrap();let (tp,r,c)=d.session.sel.focus.path.cell().expect("keyboard stayed in table");
                let table=d.session.doc.table(d.session.sel.focus.story,&tp).unwrap();
                assert_eq!(table.rows.len(),2,"Tab at the end must add a row");
                assert_eq!((r,c),(0,1),"Shift+Tab must return to the previous cell");
                assert!(d.session.doc.plain_text(wordcraft_doc::StoryRef::Body).contains("GNOME"),"keyboard input reaches the cell");}
            eprintln!("TABLE_KEYBOARD_OK");app.quit();
            });
        });
    }
    if std::env::args().any(|a|a=="--table-menu-demo"||a=="--table-properties-demo"||a=="--table-insert-demo") {
        let u=ui.clone();let w=window.clone();let m=table_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||{
            if std::env::args().any(|a|a=="--table-insert-demo"){show_insert_table(&u,&w);return;}
            if let Some(d)=u.current.borrow_mut().as_mut(){
                d.session.run("caret.docEnd",&serde_json::json!({})).unwrap();
                d.session.run("insert.table",&serde_json::json!({"rows":3,"cols":3})).unwrap();
                d.session.run("text.insert",&serde_json::json!({"text":"Tabla de prueba"})).unwrap();
            }
            after_content_change(&u);caret_ensure_visible(&u);
            if std::env::args().any(|a|a=="--table-properties-demo"){show_table_properties(&u,&w);}else{m.popup();}
        });
    }
    if std::env::args().any(|a|a=="--paragraph-demo"||a=="--paragraph-spacing-demo") {let m=align_menu.clone();let s=paragraph_popover.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||{m.popup();if std::env::args().any(|a|a=="--paragraph-spacing-demo"){s.page("spacing");}});}
    if std::env::args().any(|a|a=="--paragraph-ui-test") {let s=paragraph_popover.clone();let m=align_menu.clone();let app=app.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||paragraph_ui_test(&s,&m,&app));}
    if std::env::args().any(|a|a=="--search-ui-test") {let s=search_panel.clone();let ui=ui.clone();let app=app.clone();let host=host.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||search_ui_test(&ui,&s,&app,&host));}
    if std::env::args().any(|a|a=="--view-pointer-test"){let u=ui.clone();let m=style_menu.clone();let w=window.clone();let a=app.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1300),move||view_pointer_test(&u,&m,&w,&a));}
    if std::env::args().any(|a|a=="--spreads-demo"){window.set_default_size(1800,950);let u=ui.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1300),move||{if let Some(d)=u.current.borrow_mut().as_mut(){d.fit_to_width=false;}zoom_to(&u,1.0);});}
    if std::env::args().any(|a|a=="--advanced-text-demo") {let w=window.clone();let u=ui.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||show_text_options(&u,&w));}
    if std::env::args().any(|a|a=="--advanced-paragraph-demo") {let w=window.clone();let u=ui.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||show_paragraph_options(&u,&w));}
    if std::env::args().any(|a|a=="--styles-ui-test") {let s=style_popover.clone();let m=style_menu.clone();let a=app.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||style_ui_test(&s,&m,&a));}
    if std::env::args().any(|a|a=="--styles-demo"||a=="--style-details-demo"||a=="--style-new-demo") {
        let menu=style_menu.clone();let s=style_popover.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||{
            menu.popup();if std::env::args().any(|a|a=="--style-details-demo"){s.show_detail("Heading1");}
            if std::env::args().any(|a|a=="--style-new-demo"){s.show_form(None);}
        });
    }
    if std::env::args().any(|a|a=="--preferences-demo") {let w=window.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||show_preferences(&w));}
    if std::env::args().any(|a|a=="--search-demo") {let s=search_panel.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1100),move||{s.show(true);s.query.set_text("the");s.replacement.set_text("THE");s.refresh();});}
    {
        let ui=ui.clone();let app=app.clone();let host=host.clone();let parent=window.clone();
        glib::timeout_add_local_once(std::time::Duration::from_secs(3),move || {
            let records=ui.current.borrow().as_ref().map(|d|d.session.recovery_documents()).unwrap_or_default();
            if !records.is_empty() {
                let toast=Toast::new("Hay documentos pendientes de recuperar");toast.set_button_label(Some("Recuperar"));
                let next=ui.clone();toast.connect_button_clicked(move |_|show_recovery(&next,&parent,&app,&host));
                ui.toasts.add_toast(toast);
            }
        });
    }
    if std::env::args().any(|a| a == "--text-menu-demo") {
        let menu = text_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || menu.popup());
    }
    if std::env::args().any(|a| a == "--zoom-menu-demo") {
        let menu = zoom_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || menu.popup());
    }
    if sample_on_start || initial_loaded {
        let ui = ui.clone();
        glib::idle_add_local_once(move || fit_width(&ui));
    }
    if formatting_demo {
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(950), move || {
            {
                let mut borrowed = ui.current.borrow_mut();
                if let Some(doc) = borrowed.as_mut() {
                    let _ = doc.session.run("select.text", &serde_json::json!({
                        "text": "Why share a studio?"
                    }));
                }
            }
            apply_user_format(&ui, Format::Heading1);
            apply_user_format(&ui, Format::AlignCenter);
        });
    }
    if editing_demo {
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            {
                let mut value = ui.current.borrow_mut();
                if let Some(doc) = value.as_mut() {
                    let _ = doc.session.run("select.text", &serde_json::json!({
                        "text": "The Open Studio Handbook"
                    }));
                }
            }
            apply_user_edit(&ui, Edit::Insert("Documento editable — fase 3B".to_string()));
        });
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
        println!("WordCraft GNOME · guardado DOCX y ODT · fase 3D");
        return;
    }
    let sample_on_start = std::env::args().any(|arg| arg == "--sample");
    let sidebar_on_start = std::env::args().any(|arg| arg == "--sidebar");
    let narrow_on_start = std::env::args().any(|arg| arg == "--narrow");
    let dark_on_start = std::env::args().any(|arg| arg == "--dark");
    let selection_demo = std::env::args().any(|arg| arg == "--select-demo");
    let editing_demo = std::env::args().any(|arg| arg == "--edit-demo");
    let formatting_demo = std::env::args().any(|arg| arg == "--format-demo");
    // Permite probar versiones simultáneas sin activar una ventana anterior por D-Bus.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        if std::env::args().any(|a| a == "--multiwindow-test") {
            let first = make_window(app, false, false, false, false, false, false, false,
                Some((sample_session(), "Primero".into())));
            let _second = make_window(app, false, false, false, false, false, false, false,
                Some((sample_session(), "Segundo".into())));
            let app = app.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
                if app.windows().len() != 2 { std::process::exit(1); }
                first.close();
                glib::timeout_add_local_once(std::time::Duration::from_millis(150),move || {
                    if app.windows().len() != 1 { std::process::exit(1); }
                    eprintln!("MULTIWINDOW_OK");app.quit();
                });
            });
            return;
        }
        make_window(app, sample_on_start, sidebar_on_start, narrow_on_start, dark_on_start, selection_demo, editing_demo, formatting_demo, None);
    });
    app.run_with_args(&["wordcraft-gnome"]);
}

/// Coalesce thumbnail rasterization while typing; keep the last preview until
/// the next refresh so the sidebar does not flash on every keystroke.
fn schedule_thumbnail_refresh(ui: &Rc<Ui>) {
    if ui.thumbnail_refresh_pending.replace(true) { return; }
    let weak=Rc::downgrade(ui);
    glib::timeout_add_local_once(std::time::Duration::from_millis(120),move || {
        let Some(ui)=weak.upgrade() else { return; };
        ui.thumbnail_refresh_pending.set(false);
        if ui.closed.get() { return; }
        if let Some(doc)=ui.current.borrow_mut().as_mut() {
            for index in doc.painted_thumbs.drain() {
                doc.thumbs[index].set_paintable(None::<&gdk::Paintable>);
            }
        }
        refresh_thumbnails(&ui);
    });
}
