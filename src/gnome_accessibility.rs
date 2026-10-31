// Native controls expose their purpose independently of tooltip timing/icons.
fn accessible_name(widget: &impl IsA<gtk::Accessible>, name: &str) {
    widget.update_property(&[gtk::accessible::Property::Label(name)]);
}
fn contains_focus(container: &impl IsA<gtk::Widget>, focus: &gtk::Widget) -> bool {
    focus == container.as_ref() || focus.is_ancestor(container)
}
fn focus_zone(ui: &Rc<Ui>, backwards: bool) {
    let Some(host) = ui.host.upgrade() else { return; };
    let focus = gtk::prelude::GtkWindowExt::focus(&host.window);
    let zone = focus.as_ref().map(|f| {
        if ui.format_buttons.borrow().iter().any(|m| contains_focus(m, f)) || contains_focus(&ui.undo_button,f) || contains_focus(&ui.redo_button,f) { 2 }
        else if ui.sidebar.sidebar().is_some_and(|side| contains_focus(&side,f)) { 1 }
        else { 0 }
    }).unwrap_or(0);
    match (zone + if backwards { 2 } else { 1 }) % 3 {
        0 => { ui.document_box.grab_focus(); },
        1 => {
            ui.sidebar_button.set_active(true);
            let ui = ui.clone();
            glib::idle_add_local_once(move || {
                if ui.search.borrow().as_ref().is_some_and(|s| s.toggle.is_active()) {
                    if let Some(s) = ui.search.borrow().as_ref() { s.query.grab_focus(); }
                } else if let Some(side) = ui.sidebar.sidebar() { side.child_focus(gtk::DirectionType::TabForward); }
            });
        },
        _ => { if let Some(button) = ui.format_buttons.borrow().first() { button.grab_focus(); } }
    }
}
fn install_accessibility(ui: &Rc<Ui>, main: &gtk::MenuButton, compact: &gtk::MenuButton) {
    accessible_name(&ui.document_box,"Contenido del documento");
    ui.document_box.update_property(&[gtk::accessible::Property::Description("F6 cambia entre documento, panel lateral y formato. Tab navega entre celdas de una tabla. Mayús+F10 abre el menú contextual.")]);
    for (w,name) in [(ui.sidebar_button.clone().upcast::<gtk::Widget>(),"Mostrar u ocultar el panel lateral"),
        (ui.zoom_in.clone().upcast(),"Ampliar zoom"),(ui.zoom_out.clone().upcast(),"Reducir zoom"),
        (ui.font_button.clone().upcast(),"Familia tipográfica"),(ui.size_input.clone().upcast(),"Tamaño de letra en puntos"),
        (main.clone().upcast(),"Menú principal"),(compact.clone().upcast(),"Menú principal")] { accessible_name(&w,name); }
    for (w,name) in [(&ui.format_bold,"Negrita"),(&ui.format_italic,"Cursiva"),(&ui.format_underline,"Subrayado")] { accessible_name(w,name); }
    let keys=gtk::EventControllerKey::new();keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak=Rc::downgrade(ui);let main=main.downgrade();let compact=compact.downgrade();
    keys.connect_key_pressed(move |_,key,_,mods| {
        if mods.intersects(gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK) { return glib::Propagation::Proceed; }
        let Some(ui)=weak.upgrade() else { return glib::Propagation::Proceed; };
        let shift=mods.contains(gdk::ModifierType::SHIFT_MASK);
        if key==gdk::Key::F6 { focus_zone(&ui,shift); return glib::Propagation::Stop; }
        if key==gdk::Key::F10 && !shift {
            if let Some(menu)=main.upgrade().filter(|m|m.is_mapped()).or_else(||compact.upgrade()) { menu.popup(); }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    ui.sidebar.add_controller(keys);
}

// Staged assertions driven by real xdotool keys in CI, not direct callbacks.
fn keyboard_ui_test(ui:&Rc<Ui>, parent:&ApplicationWindow, main:&gtk::MenuButton, app:&Application) {
    let ui=ui.clone();let parent=parent.clone();let main=main.clone();let app=app.clone();
    opened(&ui, Session::new(wordcraft_doc::Document::from_text("Keyboard test")), "Teclado");
    let document=ui.current.borrow().as_ref().unwrap().session.doc.clone();
    ui.document_box.grab_focus(); eprintln!("KEYBOARD_READY 0");
    let stage=Rc::new(Cell::new(0));let ticks=Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(150),move || {
        ticks.set(ticks.get()+1);assert!(ticks.get()<250,"keyboard test timed out");
        let marker=format!("keyboard-stage-{}",stage.get());
        if !Path::new(&marker).exists(){return glib::ControlFlow::Continue;}
        std::fs::remove_file(marker).unwrap();
        match stage.get() {
            0=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.sidebar.sidebar().is_some_and(|s|contains_focus(&s,&f)));},
            1=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.format_buttons.borrow().iter().any(|m|contains_focus(m,&f)));},
            2=>{assert!(ui.document_box.has_focus());},
            3=>{assert!(main.popover().unwrap().is_visible());},
            4=>{assert!(!main.popover().unwrap().is_visible());show_search(&ui,&parent,false);},
            5=>{assert!(ui.document_box.has_focus());assert!(!ui.search.borrow().as_ref().unwrap().toggle.is_active());},
            6=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.format_buttons.borrow().iter().any(|m|contains_focus(m,&f)));},
            7=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.sidebar.sidebar().is_some_and(|side|contains_focus(&side,&f)));},
            8=>{assert!(ui.document_box.has_focus());},
            _=>{assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,document);eprintln!("KEYBOARD_UI_OK");app.quit();return glib::ControlFlow::Break;}
        }
        stage.set(stage.get()+1);eprintln!("KEYBOARD_READY {}",stage.get());glib::ControlFlow::Continue
    });
}
