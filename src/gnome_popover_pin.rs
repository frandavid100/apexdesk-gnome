// Shared chrome for the four editing popovers. No document state is stored here.
fn format_chevron()->gtk::Widget {
    let a=gtk::DrawingArea::new();a.set_content_width(16);a.set_content_height(16);a.set_halign(gtk::Align::Center);a.set_valign(gtk::Align::Center);
    a.set_draw_func(|w,c,_,_|{let f=w.color();c.set_source_rgba(f.red()as f64,f.green()as f64,f.blue()as f64,f.alpha()as f64);c.set_line_width(1.6);c.set_line_cap(gtk::cairo::LineCap::Round);c.set_line_join(gtk::cairo::LineJoin::Round);c.move_to(4.5,6.5);c.line_to(8.0,10.0);c.line_to(11.5,6.5);let _=c.stroke();});a.upcast()
}
fn format_menu_label(menu:&gtk::MenuButton,name:&str){
    menu.add_css_class("flat");let row=gtk::Box::new(gtk::Orientation::Horizontal,6);row.append(&gtk::Label::new(Some(name)));row.append(&format_chevron());menu.set_child(Some(&row));
}
type FormatBack=Rc<RefCell<Option<Rc<dyn Fn()>>>>;
struct FormatHeader {button:gtk::Button,label:gtk::Label,arrow:gtk::Image,pin:gtk::ToggleButton,back:FormatBack,title:String}
impl FormatHeader {
    fn home(&self){self.label.set_text(&self.title);self.arrow.set_visible(false);self.button.set_can_target(false);self.button.set_focusable(false);self.back.borrow_mut().take();}
    fn back(&self,label:&str,callback:impl Fn()+'static){self.label.set_text(label);self.arrow.set_visible(true);self.button.set_can_target(true);self.button.set_focusable(true);self.back.replace(Some(Rc::new(callback)));}
}
#[derive(Default)]
struct FormatPopoverGroup {pinned:Cell<bool>,entries:RefCell<Vec<(glib::WeakRef<gtk::Popover>,Rc<FormatHeader>)>>}
impl FormatPopoverGroup {
    fn pin(&self,on:bool){if self.pinned.replace(on)==on{return;}for(_,h)in self.entries.borrow().iter(){h.pin.set_active(on);h.pin.set_tooltip_text(Some(if on{"Dejar de mantener abierto"}else{"Mantener abierto"}));}}
    fn show(&self,pop:&gtk::Popover){for(weak,h)in self.entries.borrow().iter(){if let Some(p)=weak.upgrade(){if p!=*pop{p.popdown();}h.pin.set_active(self.pinned.get());}}}
    fn outside(&self,gesture:&gtk::GestureClick,window:&ApplicationWindow,x:f64,y:f64){
        if self.pinned.get(){return;}let surface=gesture.current_event().and_then(|e|e.surface());
        for(weak,_)in self.entries.borrow().iter(){if let Some(p)=weak.upgrade(){if !p.is_visible()||surface==p.surface(){continue;}
            let anchor=p.parent().filter(|w|w.is::<gtk::MenuButton>()).and_then(|w|w.compute_bounds(window));
            if anchor.is_some_and(|r|x>=r.x()as f64&&y>=r.y()as f64&&x<=(r.x()+r.width())as f64&&y<=(r.y()+r.height())as f64){continue;}p.popdown();
        }}
    }
}
fn close_format_popover(pop:&gtk::Popover){if !popover_pin(pop).is_active(){pop.popdown();}}
fn format_popover_header(pop:&gtk::Popover,root:&gtk::Box,title:&str,group:&Rc<FormatPopoverGroup>)->Rc<FormatHeader>{
    // Modality never changes on a mapped popup: pinning cannot blink/remap it.
    pop.set_autohide(false);
    let row=gtk::Box::new(gtk::Orientation::Horizontal,6);let button=gtk::Button::new();button.add_css_class("flat");button.set_hexpand(true);
    let content=gtk::Box::new(gtk::Orientation::Horizontal,6);let arrow=gtk::Image::from_icon_name("go-previous-symbolic");content.append(&arrow);
    let label=gtk::Label::new(Some(title));label.set_xalign(0.0);label.set_hexpand(true);label.set_ellipsize(gtk::pango::EllipsizeMode::End);label.add_css_class("heading");content.append(&label);button.set_child(Some(&content));row.append(&button);
    let pin=gtk::ToggleButton::new();pin.add_css_class("flat");pin.set_widget_name("format-pin");pin.set_tooltip_text(Some("Mantener abierto"));
    let icon=gtk::DrawingArea::new();icon.set_content_width(18);icon.set_content_height(18);icon.set_halign(gtk::Align::Center);icon.set_valign(gtk::Align::Center);
    icon.set_draw_func(|w,c,_,_|{let f=w.color();c.set_source_rgba(f.red()as f64,f.green()as f64,f.blue()as f64,f.alpha()as f64);c.set_line_width(1.4);c.set_line_join(gtk::cairo::LineJoin::Round);c.move_to(5.0,2.5);c.line_to(13.0,2.5);c.move_to(6.0,2.5);c.line_to(6.0,7.5);c.line_to(3.5,11.0);c.line_to(14.5,11.0);c.line_to(12.0,7.5);c.line_to(12.0,2.5);c.move_to(9.0,11.0);c.line_to(9.0,16.0);let _=c.stroke();});pin.set_child(Some(&icon));row.append(&pin);root.append(&row);root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let back:FormatBack=Rc::new(RefCell::new(None));let action=back.clone();button.connect_clicked(move |_|{let f=action.borrow().clone();if let Some(f)=f{f();}});
    let h=Rc::new(FormatHeader{button,label,arrow,pin:pin.clone(),back,title:title.into()});h.home();group.entries.borrow_mut().push((pop.downgrade(),h.clone()));
    let weak=Rc::downgrade(group);pin.connect_toggled(move|b|{if let Some(g)=weak.upgrade(){g.pin(b.is_active());}});
    let weak=Rc::downgrade(group);pop.connect_show(move|p|{if let Some(g)=weak.upgrade(){g.show(p);}});
    let keys=gtk::EventControllerKey::new();let weak=pop.downgrade();keys.connect_key_pressed(move|_,key,_,_|{if key==gdk::Key::Escape{if let Some(p)=weak.upgrade(){p.popdown();}glib::Propagation::Stop}else{glib::Propagation::Proceed}});pop.add_controller(keys);h
}

fn popover_pin(pop:&gtk::Popover)->gtk::ToggleButton{style_widget(&pop.child().unwrap(),"format-pin").unwrap().downcast().unwrap()}
struct PinTest {
    ui:Rc<Ui>,sm:gtk::MenuButton,s:Rc<StylePopover>,pm:gtk::MenuButton,p:Rc<ParagraphPopover>,tm:gtk::MenuButton,tp:gtk::Popover,tb:gtk::MenuButton,app:Application,
}
fn pin_ui_test(ui:&Rc<Ui>,sm:&gtk::MenuButton,s:&Rc<StylePopover>,pm:&gtk::MenuButton,p:&Rc<ParagraphPopover>,tm:&gtk::MenuButton,tp:&gtk::Popover,tb:&gtk::MenuButton,app:&Application){
    pin_test_step(Rc::new(PinTest{ui:ui.clone(),sm:sm.clone(),s:s.clone(),pm:pm.clone(),p:p.clone(),tm:tm.clone(),tp:tp.clone(),tb:tb.clone(),app:app.clone()}),0);
}
fn pin_test_step(t:Rc<PinTest>,step:u8){
    let ui=&t.ui;
    match step {
        0=>{t.sm.popup();let sp=t.sm.popover().unwrap();let rev=ui.current.borrow().as_ref().unwrap().session.rev();let surface=sp.surface();let hides=Rc::new(Cell::new(0));let n=hides.clone();sp.connect_hide(move |_|n.set(n.get()+1));popover_pin(&sp).set_active(true);popover_pin(&sp).set_active(false);popover_pin(&sp).set_active(true);assert_eq!(hides.get(),0,"pinning must never hide the popup");assert_eq!(surface,sp.surface());assert!(!sp.is_autohide());assert_eq!(rev,ui.current.borrow().as_ref().unwrap().session.rev());style_click(&t.s,"style-apply-Heading1");},
        1=>{let sp=t.sm.popover().unwrap();assert!(sp.is_mapped(),"pinned style must stay drawn after apply and focus");t.pm.popup();let pp=t.pm.popover().unwrap();assert!(!sp.is_visible());assert!(popover_pin(&pp).is_active(),"pin must carry to replacement panel");t.sm.popup();assert!(!pp.is_visible());assert!(popover_pin(&sp).is_active());style_click(&t.s,"style-manage-Heading1");popover_pin(&sp).set_active(false);assert_eq!(t.s.stack.visible_child_name().as_deref(),Some("detail"),"unpin must preserve subpage");},
        2=>{assert!(t.sm.popover().unwrap().is_mapped(),"unpin must remap the popup");t.s.main();style_click(&t.s,"style-apply-Normal");assert!(!t.sm.popover().unwrap().is_visible());t.pm.popup();let pp=t.pm.popover().unwrap();popover_pin(&pp).set_active(true);t.p.execute("para.align",serde_json::json!({"value":"center"}));assert_eq!(selection_paragraph(ui)["align"],"center");},
        3=>{let pp=t.pm.popover().unwrap();assert!(pp.is_mapped(),"pinned paragraph must stay drawn");t.p.page("spacing");popover_pin(&pp).set_active(false);assert_eq!(t.p.stack.visible_child_name().as_deref(),Some("detail"));t.p.execute("para.align",serde_json::json!({"value":"left"}));assert!(!pp.is_visible());t.tm.popup();popover_pin(&t.tp).set_active(true);ui.format_bold.set_active(!ui.format_bold.is_active());},
        4=>{assert!(t.tp.is_mapped(),"pinned text must stay drawn after toggling bold");popover_pin(&t.tp).set_active(false);ui.size_input.set_value(18.0);style_widget(&t.tp.child().unwrap(),"text-apply-size").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();assert!(!t.tp.is_visible());format_execute(ui,"insert.table",serde_json::json!({"rows":2,"cols":2})).unwrap();t.tb.popup();let table=t.tb.popover().unwrap();popover_pin(&table).set_active(true);style_widget(&table.child().unwrap(),"table-action-table.insertRowBelow").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();},
        5=>{let table=t.tb.popover().unwrap();assert!(table.is_mapped(),"pinned table must stay drawn");style_widget(&table.child().unwrap(),"table-delete").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();assert!(!style_widget(&table.child().unwrap(),"table-context").unwrap().is_visible());},
        _=>{let table=t.tb.popover().unwrap();assert!(table.is_mapped(),"deleting table must not hide pinned popup");popover_pin(&table).set_active(false);close_format_popover(&table);assert!(!table.is_visible());assert_eq!(t.app.windows().len(),1);eprintln!("POPOVER_PIN_OK");t.app.quit();return;}
    }
    glib::timeout_add_local_once(std::time::Duration::from_millis(220),move||pin_test_step(t,step+1));
}


