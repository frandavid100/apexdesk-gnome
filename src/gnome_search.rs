// Per-tab sidebar search; weak callbacks never keep a closed document alive.
struct SearchPanel {
    ui: std::rc::Weak<Ui>,
    toggle: gtk::ToggleButton,
    query: gtk::SearchEntry,
    replacement: gtk::Entry,
    replacement_row: gtk::Box,
    replace_all: gtk::Button,
    _actions_width: gtk::SizeGroup,
    match_case: gtk::CheckButton,
    replace_mode: gtk::CheckButton,
    list: gtk::ListBox,
    status: gtk::Label,
    hits: RefCell<Vec<wordcraft_gnome::search::Hit>>,
    pending: Cell<bool>,
}
impl SearchPanel {
    fn show(&self, replace: bool) {
        if replace {self.replace_mode.set_active(true);}
        self.toggle.set_active(true);
        if let Some(ui)=self.ui.upgrade(){ui.sidebar.set_show_sidebar(true);ui.sidebar_pages.set_visible_child_name("search");}
        self.query.grab_focus();self.query.select_region(0,-1);
    }
    fn schedule(self:&Rc<Self>) {
        if self.pending.replace(true){return;}
        let weak=Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(160),move||{
            if let Some(s)=weak.upgrade(){s.pending.set(false);s.refresh();}
        });
    }
    fn refresh(self:&Rc<Self>) {
        let Some(ui)=self.ui.upgrade().filter(|u|!u.closed.get())else{return;};
        let result={let mut state=ui.current.borrow_mut();state.as_mut().map(|d|wordcraft_gnome::search::query(&mut d.session,&self.query.text(),self.match_case.is_active()))};
        let (hits,error)=match result {Some(Ok(h))=>(h,None),Some(Err(e))=>(Vec::new(),Some(e)),None=>(Vec::new(),None)};
        while let Some(child)=self.list.first_child(){self.list.remove(&child);}
        ui.search_highlights.borrow_mut().clear();
        if self.toggle.is_active(){*ui.search_highlights.borrow_mut()=hits.iter().flat_map(|h|h.rects.clone()).collect();}
        self.status.set_visible(hits.is_empty());
        self.status.set_text(error.as_deref().unwrap_or(if self.query.text().is_empty(){"Escribe para buscar"}else{"No hay coincidencias"}));
        self.replace_all.set_sensitive(!hits.is_empty());
        for (index,hit) in hits.iter().enumerate(){
            let row=gtk::ListBoxRow::new();row.set_widget_name(&format!("search-hit-{index}"));
            let content=gtk::Box::new(gtk::Orientation::Horizontal,6);content.set_margin_top(1);content.set_margin_bottom(1);content.set_margin_start(8);content.set_margin_end(6);
            let snippet=gtk::Label::new(None);snippet.set_xalign(0.0);snippet.set_hexpand(true);snippet.set_ellipsize(gtk::pango::EllipsizeMode::End);snippet.set_max_width_chars(1);
            let escaped=|s:&str|glib::markup_escape_text(&s.replace('\n'," ")).to_string();
            let before=hit.before.chars().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<String>();
            let before=if before.len()<hit.before.len(){format!("…{before}")}else{before};
            snippet.set_markup(&format!("{}<b>{}</b>{}",escaped(&before),escaped(&hit.matched),escaped(&hit.after)));
            snippet.set_tooltip_text(Some(&format!("{}{}{}",hit.before,hit.matched,hit.after)));content.append(&snippet);
            let page=gtk::Label::new(Some(&hit.page.map(|p|(p+1).to_string()).unwrap_or_else(||"—".into())));page.add_css_class("dim-label");page.set_tooltip_text(Some("Página"));content.append(&page);
            if self.replace_mode.is_active(){let b=gtk::Button::new();b.set_child(Some(&paragraph_icon("replace")));b.add_css_class("flat");b.add_css_class("search-replace");b.set_tooltip_text(Some("Reemplazar esta coincidencia"));b.set_widget_name(&format!("search-replace-{index}"));accessible_name(&b,"Reemplazar esta coincidencia");content.append(&b);
                let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.replace_hit(index);}});
            }
            row.set_child(Some(&content));self.list.append(&row);
        }
        *self.hits.borrow_mut()=hits;
        refresh_selection(&ui);
    }
    fn activate(&self,index:usize){
        let Some(ui)=self.ui.upgrade()else{return;};let Some(hit)=self.hits.borrow().get(index).cloned()else{return;};
        let result={let mut state=ui.current.borrow_mut();state.as_mut().map(|d|{let r=wordcraft_gnome::search::select(&mut d.session,&hit);if r.is_ok(){d.session.page_hint=hit.page.unwrap_or(0);}r})};
        if let Some(Err(e))=result{ui.toasts.add_toast(Toast::new(&e));return;}
        refresh_selection(&ui);caret_ensure_visible(&ui);
        if let Some(row)=self.list.row_at_index(index as i32){self.list.select_row(Some(&row));}
    }
    fn replace_all(self:&Rc<Self>){
        let Some(ui)=self.ui.upgrade()else{return;};
        let result={let mut value=ui.current.borrow_mut();value.as_mut().map(|d|wordcraft_gnome::search::replace_all(&mut d.session,&self.query.text(),&self.replacement.text(),self.match_case.is_active()))};
        match result{Some(Ok(n))if n>0=>after_content_change(&ui),Some(Err(e))=>ui.toasts.add_toast(Toast::new(&e)),_=>{}}
        self.refresh();ui.document_box.grab_focus();
    }
    fn replace_hit(self:&Rc<Self>,index:usize){
        let Some(ui)=self.ui.upgrade()else{return;};let Some(hit)=self.hits.borrow().get(index).cloned()else{return;};
        let result={let mut state=ui.current.borrow_mut();state.as_mut().map(|d|wordcraft_gnome::search::replace(&mut d.session,&hit,&self.replacement.text()))};
        match result{Some(Ok(()))=>after_content_change(&ui),Some(Err(e))=>ui.toasts.add_toast(Toast::new(&e)),None=>{}}
        self.refresh();
        let count=self.hits.borrow().len();if count>0{self.activate(index.min(count-1));}
        ui.document_box.grab_focus();
    }
}
fn install_search_panel(ui:&Rc<Ui>,toggle:&gtk::ToggleButton,compact:&gtk::Button)->Rc<SearchPanel>{
    let root=gtk::Box::new(gtk::Orientation::Vertical,6);root.set_margin_top(6);
    let query=gtk::SearchEntry::new();query.set_placeholder_text(Some("Buscar en el documento"));query.set_hexpand(true);query.set_width_chars(1);query.set_widget_name("document-search");accessible_name(&query,"Buscar en el documento");
    let line=gtk::Box::new(gtk::Orientation::Horizontal,4);line.set_margin_start(6);line.set_margin_end(6);line.append(&query);
    let options=gtk::MenuButton::new();options.set_icon_name("view-more-symbolic");accessible_name(&options,"Opciones de búsqueda");options.add_css_class("flat");options.add_css_class("search-options");options.set_tooltip_text(Some("Opciones de búsqueda"));line.append(&options);root.append(&line);
    let pop=gtk::Popover::new();let choices=gtk::Box::new(gtk::Orientation::Vertical,8);choices.set_margin_top(8);choices.set_margin_bottom(8);choices.set_margin_start(8);choices.set_margin_end(8);
    let case=gtk::CheckButton::with_label("Distinguir mayúsculas");let replace=gtk::CheckButton::with_label("Reemplazar");choices.append(&case);choices.append(&replace);pop.set_child(Some(&choices));options.set_popover(Some(&pop));
    let replacement=gtk::Entry::new();replacement.set_placeholder_text(Some("Reemplazar por"));replacement.set_hexpand(true);replacement.set_width_chars(1);accessible_name(&replacement,"Reemplazar por");
    let replacement_row=gtk::Box::new(gtk::Orientation::Horizontal,4);replacement_row.set_margin_start(6);replacement_row.set_margin_end(6);replacement_row.append(&replacement);
    let replace_all=gtk::Button::new();accessible_name(&replace_all,"Reemplazar todos los resultados");replace_all.set_child(Some(&paragraph_icon("replace")));replace_all.add_css_class("flat");replace_all.set_tooltip_text(Some("Reemplazar todos los resultados"));replace_all.set_widget_name("search-replace-all");replacement_row.append(&replace_all);replacement_row.set_visible(false);root.append(&replacement_row);
    let actions_width=gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);actions_width.add_widget(&options);actions_width.add_widget(&replace_all);
    let status=gtk::Label::new(Some("Escribe para buscar"));status.add_css_class("dim-label");status.set_wrap(true);root.append(&status);
    let list=gtk::ListBox::new();list.set_activate_on_single_click(true);list.add_css_class("navigation-sidebar");
    let scroll=gtk::ScrolledWindow::new();scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_vexpand(true);scroll.set_child(Some(&list));root.append(&scroll);
    ui.sidebar_pages.add_named(&root,Some("search"));
    let s=Rc::new(SearchPanel{ui:Rc::downgrade(ui),toggle:toggle.clone(),query:query.clone(),replacement:replacement.clone(),replacement_row,replace_all:replace_all.clone(),_actions_width:actions_width,match_case:case.clone(),replace_mode:replace.clone(),list:list.clone(),status,hits:RefCell::new(Vec::new()),pending:Cell::new(false)});
    let weak=Rc::downgrade(&s);replace_all.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.replace_all();}});
    let weak=Rc::downgrade(&s);query.connect_search_changed(move |_|{if let Some(s)=weak.upgrade(){s.refresh();}});
    let weak=Rc::downgrade(&s);case.connect_toggled(move |_|{if let Some(s)=weak.upgrade(){s.refresh();}});
    let weak=Rc::downgrade(&s);replace.connect_toggled(move |b|{if let Some(s)=weak.upgrade(){s.replacement_row.set_visible(b.is_active());s.refresh();}});
    let weak=Rc::downgrade(&s);toggle.connect_toggled(move|b|{if let Some(s)=weak.upgrade(){if let Some(ui)=s.ui.upgrade(){
        ui.sidebar_pages.set_visible_child_name(if b.is_active(){"search"}else{"navigation"});
        if b.is_active(){ui.sidebar.set_show_sidebar(true);s.query.grab_focus();s.refresh();}else{ui.search_highlights.borrow_mut().clear();refresh_selection(&ui);}
    }}});
    let weak=Rc::downgrade(&s);compact.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.show(false);}});
    let weak=Rc::downgrade(&s);list.connect_row_activated(move|_,r|{if let Some(s)=weak.upgrade(){s.activate(r.index() as usize);}});
    let weak=Rc::downgrade(&s);query.connect_activate(move |_|{if let Some(s)=weak.upgrade(){let next=s.list.selected_row().map(|r|r.index() as usize+1).unwrap_or(0);let n=s.hits.borrow().len();if n>0{s.activate(next%n);}}});
    let weak=Rc::downgrade(&s);query.connect_stop_search(move |_|{if let Some(s)=weak.upgrade(){s.toggle.set_active(false);if let Some(ui)=s.ui.upgrade(){ui.document_box.grab_focus();}}});
    s
}
fn show_search(ui:&Rc<Ui>,_parent:&ApplicationWindow,replace:bool){if let Some(s)=ui.search.borrow().as_ref(){s.show(replace);}}

fn search_ui_test(ui:&Rc<Ui>,s:&Rc<SearchPanel>,app:&Application,host:&Rc<Workspace>){
    opened(ui,Session::new(wordcraft_doc::Document::from_text("Hola hola hola\nSecond Hola")),"Prueba de búsqueda");
    let selection=ui.current.borrow().as_ref().unwrap().session.sel.clone();
    s.show(true);s.query.set_text("hola");s.refresh();assert_eq!(s.hits.borrow().len(),4);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel,selection);
    assert!(!ui.search_highlights.borrow().is_empty());
    s.match_case.set_active(true);assert_eq!(s.hits.borrow().len(),2);
    s.match_case.set_active(false);s.replacement.set_text("Adiós");
    style_widget(&s.list,"search-replace-1").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc.plain_text(wordcraft_doc::StoryRef::Body),"Hola Adiós hola\nSecond Hola");
    apply_user_edit(ui,Edit::Undo);s.refresh();assert_eq!(s.hits.borrow().len(),4);
    s.replacement.set_text("");s.replace_hit(0);assert_eq!(s.hits.borrow().len(),3);
    apply_user_edit(ui,Edit::Undo);s.refresh();assert_eq!(s.hits.borrow().len(),4);
    s.replacement.set_text("$1 ^p");s.replace_all.emit_clicked();assert!(s.hits.borrow().is_empty());
    apply_user_edit(ui,Edit::Undo);s.refresh();assert_eq!(s.hits.borrow().len(),4);assert!(!s.status.is_visible());
    s.toggle.set_active(false);assert!(ui.search_highlights.borrow().is_empty());s.show(false);
    let original=host.tabs.selected_page().unwrap();blank_tab(app,host);
    let new=host.docs.borrow().last().unwrap().1.clone();
    assert!(new.search.borrow().as_ref().unwrap().query.text().is_empty());
    host.tabs.set_selected_page(&original);assert_eq!(s.query.text(),"hola");
    assert_eq!(app.windows().len(),1);eprintln!("SEARCH_SIDEBAR_OK");app.quit();
}
