// Shared chrome for the four editing popovers. No document state is stored here.
fn format_chevron()->gtk::Widget {
    let a=gtk::DrawingArea::new();a.set_content_width(16);a.set_content_height(16);a.set_halign(gtk::Align::Center);a.set_valign(gtk::Align::Center);
    a.set_draw_func(|w,c,_,_|{let f=w.color();c.set_source_rgba(f.red()as f64,f.green()as f64,f.blue()as f64,f.alpha()as f64);c.set_line_width(1.6);c.set_line_cap(gtk::cairo::LineCap::Round);c.set_line_join(gtk::cairo::LineJoin::Round);c.move_to(4.5,6.5);c.line_to(8.0,10.0);c.line_to(11.5,6.5);let _=c.stroke();});a.upcast()
}
fn format_menu_label(menu:&gtk::MenuButton,name:&str){
    menu.add_css_class("flat");let row=gtk::Box::new(gtk::Orientation::Horizontal,6);row.append(&gtk::Label::new(Some(name)));row.append(&format_chevron());menu.set_child(Some(&row));
}
fn close_format_popover(pop:&gtk::Popover){if pop.is_autohide(){pop.popdown();}}
// GTK unrealizes an open popover when autohide changes. Reopen synchronously,
// guarding show handlers so pinning does not reset the current subpage/form.
fn format_popover_header(pop:&gtk::Popover,root:&gtk::Box,title:&str)->Rc<Cell<bool>>{
    let changing=Rc::new(Cell::new(false));let row=gtk::Box::new(gtk::Orientation::Horizontal,6);
    let label=gtk::Label::new(Some(title));label.set_xalign(0.0);label.set_hexpand(true);label.add_css_class("heading");row.append(&label);
    let pin=gtk::ToggleButton::new();pin.add_css_class("flat");pin.set_widget_name("format-pin");pin.set_tooltip_text(Some("Mantener abierto"));
    let icon=gtk::DrawingArea::new();icon.set_content_width(18);icon.set_content_height(18);icon.set_halign(gtk::Align::Center);icon.set_valign(gtk::Align::Center);
    icon.set_draw_func(|w,c,_,_|{let f=w.color();c.set_source_rgba(f.red()as f64,f.green()as f64,f.blue()as f64,f.alpha()as f64);c.set_line_width(1.4);c.set_line_join(gtk::cairo::LineJoin::Round);c.move_to(5.0,2.5);c.line_to(13.0,2.5);c.move_to(6.0,2.5);c.line_to(6.0,7.5);c.line_to(3.5,11.0);c.line_to(14.5,11.0);c.line_to(12.0,7.5);c.line_to(12.0,2.5);c.move_to(9.0,11.0);c.line_to(9.0,16.0);let _=c.stroke();});pin.set_child(Some(&icon));row.append(&pin);root.append(&row);root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let weak=pop.downgrade();let guard=changing.clone();pin.connect_toggled(move|b|{if let Some(p)=weak.upgrade(){let visible=p.is_visible();guard.set(true);p.set_autohide(!b.is_active());if visible{p.popup();}guard.set(false);b.set_tooltip_text(Some(if b.is_active(){"Dejar de mantener abierto"}else{"Mantener abierto"}));}});
    let keys=gtk::EventControllerKey::new();let weak=pop.downgrade();keys.connect_key_pressed(move|_,key,_,_|{if key==gdk::Key::Escape{if let Some(p)=weak.upgrade(){p.popdown();}glib::Propagation::Stop}else{glib::Propagation::Proceed}});pop.add_controller(keys);changing
}

fn popover_pin(pop:&gtk::Popover)->gtk::ToggleButton{style_widget(&pop.child().unwrap(),"format-pin").unwrap().downcast().unwrap()}
fn pin_ui_test(ui:&Rc<Ui>,sm:&gtk::MenuButton,s:&Rc<StylePopover>,pm:&gtk::MenuButton,p:&Rc<ParagraphPopover>,tm:&gtk::MenuButton,tp:&gtk::Popover,tb:&gtk::MenuButton,app:&Application){
    sm.popup();let sp=sm.popover().unwrap();let pin=popover_pin(&sp);let rev=ui.current.borrow().as_ref().unwrap().session.rev();
    pin.set_active(true);assert!(sp.is_visible());assert!(!sp.is_autohide());assert_eq!(rev,ui.current.borrow().as_ref().unwrap().session.rev());
    style_click(s,"style-apply-Heading1");assert!(sp.is_visible(),"pinned style must survive apply/focus");
    style_click(s,"style-manage-Heading1");pin.set_active(false);assert!(sp.is_visible());assert_eq!(s.stack.visible_child_name().as_deref(),Some("detail"),"unpin must preserve subpage");
    s.main();style_click(s,"style-apply-Normal");assert!(!sp.is_visible(),"unpin restores dismissal");
    pm.popup();let pp=pm.popover().unwrap();popover_pin(&pp).set_active(true);p.execute("para.align",serde_json::json!({"value":"center"}));assert!(pp.is_visible());assert_eq!(selection_paragraph(ui)["align"],"center");
    p.page("spacing");popover_pin(&pp).set_active(false);assert_eq!(p.stack.visible_child_name().as_deref(),Some("detail"));p.execute("para.align",serde_json::json!({"value":"left"}));assert!(!pp.is_visible());
    tm.popup();popover_pin(tp).set_active(true);ui.format_bold.set_active(!ui.format_bold.is_active());assert!(tp.is_visible(),"pinned text must survive toggle");
    popover_pin(tp).set_active(false);ui.size_input.set_value(18.0);style_widget(&tp.child().unwrap(),"text-apply-size").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();assert!(!tp.is_visible());
    format_execute(ui,"insert.table",serde_json::json!({"rows":2,"cols":2})).unwrap();tb.popup();let table=tb.popover().unwrap();popover_pin(&table).set_active(true);
    style_widget(&table.child().unwrap(),"table-action-table.insertRowBelow").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();assert!(table.is_visible());
    style_widget(&table.child().unwrap(),"table-delete").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();assert!(table.is_visible());assert!(!style_widget(&table.child().unwrap(),"table-context").unwrap().is_visible());
    popover_pin(&table).set_active(false);close_format_popover(&table);assert!(!table.is_visible());assert_eq!(app.windows().len(),1);
    eprintln!("POPOVER_PIN_OK");app.quit();
}
