// Papers-like bottom selector and a heading hierarchy sourced from WordCraft.
fn sidebar_icon(kind:&str)->gtk::Widget{
    let area=gtk::DrawingArea::new();area.set_content_width(18);area.set_content_height(18);area.set_halign(gtk::Align::Center);area.set_valign(gtk::Align::Center);
    let kind=kind.to_string();area.set_draw_func(move|w,cr,_,_|{let c=w.color();cr.set_source_rgba(c.red()as f64,c.green()as f64,c.blue()as f64,c.alpha()as f64);cr.set_line_width(1.4);
        if kind=="pages"{for(x,y)in[(2.0,2.0),(10.0,2.0),(2.0,10.0),(10.0,10.0)]{cr.rectangle(x,y,5.0,5.0);}let _=cr.stroke();}
        else{for(x,y,end)in[(2.0,3.0,16.0),(6.0,8.0,16.0),(6.0,13.0,13.0)]{cr.move_to(x,y);cr.line_to(end,y);}cr.move_to(2.0,6.0);cr.line_to(2.0,15.0);let _=cr.stroke();}
    });area.upcast()
}
struct OutlinePanel{
    ui:std::rc::Weak<Ui>,list:gtk::ListBox,status:gtk::Label,
    collapsed:RefCell<std::collections::HashSet<String>>,positions:RefCell<Vec<Pos>>,pending:Cell<bool>,
}
impl OutlinePanel{
    fn schedule(self:&Rc<Self>){if self.pending.replace(true){return;}let weak=Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(160),move||{if let Some(s)=weak.upgrade(){s.pending.set(false);s.refresh();}});
    }
    fn refresh(self:&Rc<Self>){
        let Some(ui)=self.ui.upgrade().filter(|u|!u.closed.get())else{return;};
        let (headings,focus)={let value=ui.current.borrow();let Some(d)=value.as_ref()else{return;};(wordcraft_gnome::navigation::headings(&d.session.doc),d.session.sel.focus.clone())};
        while let Some(child)=self.list.first_child(){self.list.remove(&child);}
        self.status.set_visible(headings.is_empty());self.positions.borrow_mut().clear();
        let mut hidden:Option<u8>=None;let current=headings.iter().rposition(|h|h.pos<=focus);let mut selected=None;
        for(i,h)in headings.iter().enumerate(){
            if let Some(level)=hidden{if h.level>level{continue;}hidden=None;}
            let has_children=headings.get(i+1).is_some_and(|n|n.level>h.level);let key=format!("{:?}:{}",h.pos.path,h.text);
            let collapsed=self.collapsed.borrow().contains(&key);if collapsed&&has_children{hidden=Some(h.level);}
            let row=gtk::ListBoxRow::new();row.set_widget_name(&format!("outline-heading-{i}"));
            let content=gtk::Box::new(gtk::Orientation::Horizontal,2);content.set_margin_start(6+i32::from(h.level.min(8))*12);content.set_margin_end(8);content.set_margin_top(3);content.set_margin_bottom(3);
            let arrow=gtk::Button::with_label(if collapsed{"›"}else{"⌄"});arrow.add_css_class("flat");arrow.add_css_class("outline-expander");arrow.set_visible(has_children);arrow.set_tooltip_text(Some(if collapsed{"Expandir sección"}else{"Contraer sección"}));content.append(&arrow);
            let label=gtk::Label::new(Some(&h.text));label.set_xalign(0.0);label.set_hexpand(true);label.set_ellipsize(gtk::pango::EllipsizeMode::End);label.set_max_width_chars(1);label.set_tooltip_text(Some(&h.text));if h.level==0{label.add_css_class("heading");}content.append(&label);row.set_child(Some(&content));self.list.append(&row);
            let weak=Rc::downgrade(self);arrow.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){let mut keys=s.collapsed.borrow_mut();if !keys.remove(&key){keys.insert(key.clone());}drop(keys);s.refresh();}});
            self.positions.borrow_mut().push(h.pos.clone());
            if current==Some(i){selected=Some(row);}
        }
        if let Some(row)=selected{self.list.select_row(Some(&row));}
    }
    fn goto(&self,pos:&Pos){
        let Some(ui)=self.ui.upgrade()else{return;};let result={let mut value=ui.current.borrow_mut();value.as_mut().map(|d|selection::set_caret(&mut d.session,pos,false))};
        if let Some(Err(e))=result{ui.toasts.add_toast(Toast::new(&e));return;}
        if ui.sidebar.is_collapsed(){ui.sidebar.set_show_sidebar(false);}
        refresh_selection(&ui);caret_ensure_visible(&ui);ui.document_box.grab_focus();
    }
}
fn install_outline(ui:&Rc<Ui>,pages:&gtk::ToggleButton,titles:&gtk::ToggleButton)->Rc<OutlinePanel>{
    let box_=gtk::Box::new(gtk::Orientation::Vertical,6);let status=gtk::Label::new(Some("No hay títulos en este documento"));status.set_wrap(true);status.set_margin_start(12);status.set_margin_end(12);status.set_margin_top(18);status.add_css_class("dim-label");box_.append(&status);
    let list=gtk::ListBox::new();list.set_activate_on_single_click(true);list.add_css_class("navigation-sidebar");box_.append(&list);
    let scroll=gtk::ScrolledWindow::new();scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_vexpand(true);scroll.set_child(Some(&box_));ui.navigation_pages.add_named(&scroll,Some("outline"));
    let s=Rc::new(OutlinePanel{ui:Rc::downgrade(ui),list,status,collapsed:RefCell::new(Default::default()),positions:RefCell::new(Vec::new()),pending:Cell::new(false)});
    let weak=Rc::downgrade(&s);s.list.connect_row_activated(move|_,row|{if let Some(s)=weak.upgrade(){let pos=s.positions.borrow().get(row.index()as usize).cloned();if let Some(pos)=pos{s.goto(&pos);}}});
    let weak=Rc::downgrade(&s);titles.connect_toggled(move|b|{if b.is_active(){if let Some(s)=weak.upgrade(){if let Some(ui)=s.ui.upgrade(){ui.navigation_pages.set_visible_child_name("outline");}s.refresh();}}});
    let weak=Rc::downgrade(ui);pages.connect_toggled(move|b|{if b.is_active(){if let Some(ui)=weak.upgrade(){ui.navigation_pages.set_visible_child_name("thumbnails");refresh_thumbnails(&ui);}}});s
}

fn navigation_ui_test(ui:&Rc<Ui>,s:&Rc<OutlinePanel>,titles:&gtk::ToggleButton,host:&Rc<Workspace>,app:&Application){
    let rev=ui.current.borrow().as_ref().unwrap().session.rev();titles.set_active(true);s.refresh();
    assert_eq!(ui.navigation_pages.visible_child_name().as_deref(),Some("outline"));assert!(!s.positions.borrow().is_empty());
    let row=s.list.row_at_index(0).unwrap();row.emit_by_name::<()>("activate",&[]);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel.focus,s.positions.borrow()[0]);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev,"navigation must not edit");
    let search=ui.search.borrow().as_ref().unwrap().clone();search.show(false);search.toggle.set_active(false);
    assert_eq!(ui.sidebar_pages.visible_child_name().as_deref(),Some("navigation"));assert_eq!(ui.navigation_pages.visible_child_name().as_deref(),Some("outline"));
    assert_eq!(ui.sidebar_pages.transition_type(),gtk::StackTransitionType::Crossfade);
    apply_user_edit(ui,Edit::Insert("x".into()));refresh_main_actions(host);assert!(host.window.lookup_action("undo").unwrap().is_enabled());
    gio::prelude::ActionGroupExt::activate_action(&host.window,"undo",None);refresh_main_actions(host);assert!(host.window.lookup_action("redo").unwrap().is_enabled());
    let count=host.tabs.n_pages();gio::prelude::ActionGroupExt::activate_action(&host.window,"new-tab",None);assert_eq!(host.tabs.n_pages(),count+1);
    gio::prelude::ActionGroupExt::activate_action(&host.window,"tab-overview",None);assert!(host.overview.is_open());host.overview.set_open(false);
    show_keyboard_shortcuts(&host.window);show_editor_help(&host.window);
    eprintln!("SIDEBAR_NAVIGATION_OK");app.quit();
}
