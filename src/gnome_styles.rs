// One popover with list, detail and name-entry pages; no auxiliary windows.
struct StylePopover {
    ui: std::rc::Weak<Ui>,
    popover: glib::WeakRef<gtk::Popover>,
    stack: gtk::Stack,
    list: gtk::Box,
    search: gtk::SearchEntry,
    detail: gtk::Box,
    form: gtk::Box,
    message: gtk::Label,
}
impl StylePopover {
    fn report(&self,text:&str){self.message.set_text(text);self.message.set_visible(!text.is_empty());}
    fn run(&self,edit:wordcraft_gnome::styles::Edit)->Result<Option<String>,String>{
        let ui=self.ui.upgrade().ok_or("El documento está cerrado")?;
        let (result,changed)={let mut s=ui.current.borrow_mut();let d=s.as_mut().ok_or("No hay documento")?;
            let rev=d.session.rev();let result=wordcraft_gnome::styles::edit(&mut d.session,edit)?;(result,d.session.rev()!=rev)};
        if changed{after_content_change(&ui);}else{refresh_selection(&ui);}
        Ok(result)
    }
    fn refresh_list(self:&Rc<Self>){
        clear_box(&self.list);
        let Some(ui)=self.ui.upgrade()else{return;};
        let (styles,current)={let s=ui.current.borrow();let Some(d)=s.as_ref()else{return;};(wordcraft_gnome::styles::list(&d.session),wordcraft_gnome::styles::current(&d.session))};
        let query=self.search.text().to_lowercase();let mut count=0;
        for st in styles.into_iter().filter(|s|s.name.to_lowercase().contains(&query)||s.id.to_lowercase().contains(&query)){
            count+=1;
            let row=gtk::Box::new(gtk::Orientation::Horizontal,0);
            let apply=gtk::Button::new();apply.add_css_class("flat");apply.set_hexpand(true);apply.set_widget_name(&format!("style-apply-{}",st.id));
            let label=gtk::Label::new(Some(&st.name));label.set_xalign(0.0);label.set_hexpand(true);label.set_ellipsize(gtk::pango::EllipsizeMode::End);label.set_max_width_chars(28);
            let content=gtk::Box::new(gtk::Orientation::Horizontal,8);content.append(&label);
            let kind=gtk::Label::new(Some(if st.kind==wordcraft_doc::styles::StyleKind::Paragraph{"¶"}else{"a"}));kind.add_css_class("dim-label");content.append(&kind);
            if current.as_deref()==Some(st.id.as_str()){let check=gtk::Label::new(Some("✓"));content.append(&check);}
            apply.set_child(Some(&content));row.append(&apply);
            let manage=gtk::Button::with_label("›");manage.add_css_class("flat");manage.set_tooltip_text(Some(&format!("Gestionar «{}»",st.name)));manage.set_widget_name(&format!("style-manage-{}",st.id));row.append(&manage);
            let weak=Rc::downgrade(self);let id=st.id.clone();apply.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){
                match s.run(wordcraft_gnome::styles::Edit::Apply(id.clone())){Ok(_)=>{if let Some(p)=s.popover.upgrade(){p.popdown();}if let Some(ui)=s.ui.upgrade(){ui.document_box.grab_focus();caret_ensure_visible(&ui);}},Err(e)=>s.report(&e)}
            }});
            let weak=Rc::downgrade(self);manage.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){s.show_detail(&st.id);}});
            self.list.append(&row);
        }
        if count==0{let label=gtk::Label::new(Some("No se encontraron estilos"));label.add_css_class("dim-label");label.set_margin_top(18);label.set_margin_bottom(18);self.list.append(&label);}
    }
    fn main(self:&Rc<Self>){self.report("");self.refresh_list();self.stack.set_visible_child_name("list");}
    fn back(self:&Rc<Self>,container:&gtk::Box,label:&str,id:Option<String>){
        let b=gtk::Button::with_label(&format!("‹ {label}"));b.add_css_class("flat");b.set_widget_name("style-back");if let Some(l)=b.child().and_then(|c|c.downcast::<gtk::Label>().ok()){l.set_xalign(0.0);l.set_ellipsize(gtk::pango::EllipsizeMode::End);l.set_max_width_chars(28);}
        container.append(&b);container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let weak=Rc::downgrade(self);b.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){if let Some(id)=&id{s.show_detail(id);}else{s.main();}}});
    }
    fn style(self:&Rc<Self>,id:&str)->Option<wordcraft_doc::styles::Style>{
        let ui=self.ui.upgrade()?;let state=ui.current.borrow();state.as_ref()?.session.doc.styles.get(id).cloned()
    }
    fn action(self:&Rc<Self>,label:&str,name:&str,edit:wordcraft_gnome::styles::Edit,stay:Option<String>)->gtk::Button{
        let b=gtk::Button::with_label(label);b.add_css_class("flat");b.set_widget_name(name);
        let weak=Rc::downgrade(self);b.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){match s.run(edit.clone()){
            Ok(_)=>{if let Some(id)=&stay{s.show_detail(id);s.report("Estilo actualizado");}else{s.main();s.report("Estilo eliminado");}},Err(e)=>s.report(&e)
        }}});b
    }
    fn show_detail(self:&Rc<Self>,id:&str){
        let Some(st)=self.style(id)else{self.main();return;};self.report("");clear_box(&self.detail);self.back(&self.detail,"Estilos",None);
        let title=gtk::Label::new(Some(&st.name));title.set_xalign(0.0);title.set_wrap(true);title.set_max_width_chars(28);title.add_css_class("heading");title.set_margin_top(6);self.detail.append(&title);
        let kind=gtk::Label::new(Some(if st.kind==wordcraft_doc::styles::StyleKind::Paragraph{"Estilo de párrafo"}else{"Estilo de texto"}));kind.set_xalign(0.0);kind.add_css_class("dim-label");self.detail.append(&kind);
        if let Some(ui)=self.ui.upgrade(){let preview={let state=ui.current.borrow();state.as_ref().map(|d|wordcraft_gnome::styles::preview(&d.session,id))};
            if let Some(Ok(image))=preview{let picture=gtk::Picture::for_paintable(&texture(image));picture.set_content_fit(gtk::ContentFit::ScaleDown);picture.set_size_request(290,85);picture.add_css_class("document-sheet");picture.set_margin_top(8);picture.set_margin_bottom(8);picture.set_widget_name("style-preview");self.detail.append(&picture);}
        }
        self.detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let update=self.action("Actualizar según la selección","style-update",wordcraft_gnome::styles::Edit::Update(st.id.clone()),Some(st.id.clone()));self.detail.append(&update);
        let rename=gtk::Button::with_label("Renombrar…");rename.add_css_class("flat");rename.set_widget_name("style-rename");self.detail.append(&rename);
        let weak=Rc::downgrade(self);let style=st.clone();rename.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){s.show_form(Some(&style.id));}});
        let delete=self.action("Eliminar estilo","style-delete",wordcraft_gnome::styles::Edit::Delete(st.id.clone()),None);
        delete.set_sensitive(!st.builtin);if st.builtin{delete.set_tooltip_text(Some("Los estilos integrados no se pueden eliminar"));}self.detail.append(&delete);
        self.stack.set_visible_child_name("detail");
    }
    fn show_form(self:&Rc<Self>,id:Option<&str>){
        self.report("");clear_box(&self.form);let style=id.and_then(|id|self.style(id));
        self.back(&self.form,style.as_ref().map(|s|s.name.as_str()).unwrap_or("Estilos"),style.as_ref().map(|s|s.id.clone()));
        let title=gtk::Label::new(Some(if style.is_some(){"Renombrar estilo"}else{"Nuevo estilo"}));title.set_xalign(0.0);title.add_css_class("heading");title.set_margin_top(8);self.form.append(&title);
        let entry=gtk::Entry::new();entry.set_placeholder_text(Some("Nombre del estilo"));entry.set_widget_name("style-name");entry.set_hexpand(true);if let Some(st)=&style{entry.set_text(&st.name);}self.form.append(&entry);
        if style.is_none(){let hint=gtk::Label::new(Some("Se creará con el formato de la selección actual."));hint.set_wrap(true);hint.set_xalign(0.0);hint.add_css_class("dim-label");self.form.append(&hint);}
        let commit=gtk::Button::with_label(if style.is_some(){"Renombrar"}else{"Crear estilo"});commit.add_css_class("suggested-action");commit.set_widget_name("style-commit");self.form.append(&commit);
        let weak=Rc::downgrade(self);let input=entry.clone();commit.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){
            let edit=if let Some(st)=&style{wordcraft_gnome::styles::Edit::Rename(st.id.clone(),input.text().to_string())}else{wordcraft_gnome::styles::Edit::Create(input.text().to_string())};
            match s.run(edit){Ok(_)=>{s.search.set_text("");s.main();s.report("Estilo guardado");},Err(e)=>s.report(&e)}
        }});
        let b=commit.downgrade();entry.connect_activate(move|_|{if let Some(b)=b.upgrade(){b.emit_clicked();}});
        self.stack.set_visible_child_name("form");entry.grab_focus();entry.select_region(0,-1);
    }
}
fn install_style_popover(ui:&Rc<Ui>,menu:&gtk::MenuButton)->Rc<StylePopover>{
    let pop=gtk::Popover::new();let root=gtk::Box::new(gtk::Orientation::Vertical,6);root.set_width_request(330);
    root.set_margin_top(8);root.set_margin_bottom(8);root.set_margin_start(8);root.set_margin_end(8);
    let stack=gtk::Stack::new();stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);stack.set_vhomogeneous(false);
    let main=gtk::Box::new(gtk::Orientation::Vertical,6);let search=gtk::SearchEntry::new();search.set_placeholder_text(Some("Buscar estilos"));search.set_widget_name("style-search");main.append(&search);
    let list=gtk::Box::new(gtk::Orientation::Vertical,0);let scroll=gtk::ScrolledWindow::new();scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_min_content_height(240);scroll.set_max_content_height(340);scroll.set_propagate_natural_height(true);scroll.set_child(Some(&list));main.append(&scroll);
    main.append(&gtk::Separator::new(gtk::Orientation::Horizontal));let create=gtk::Button::with_label("Nuevo estilo…");create.set_widget_name("style-new");create.add_css_class("flat");main.append(&create);
    let detail=gtk::Box::new(gtk::Orientation::Vertical,6);let form=gtk::Box::new(gtk::Orientation::Vertical,8);
    stack.add_named(&main,Some("list"));stack.add_named(&detail,Some("detail"));stack.add_named(&form,Some("form"));root.append(&stack);
    let message=gtk::Label::new(None);message.set_wrap(true);message.set_max_width_chars(36);message.set_xalign(0.0);message.set_visible(false);root.append(&message);pop.set_child(Some(&root));
    let state=Rc::new(StylePopover{ui:Rc::downgrade(ui),popover:pop.downgrade(),stack,list,search:search.clone(),detail,form,message});
    let weak=Rc::downgrade(&state);search.connect_search_changed(move|_|{if let Some(s)=weak.upgrade(){s.refresh_list();}});
    let weak=Rc::downgrade(&state);create.connect_clicked(move|_|{if let Some(s)=weak.upgrade(){s.show_form(None);}});
    let s=state.clone();pop.connect_show(move|_|{s.main();});menu.set_popover(Some(&pop));state
}

// Exercise actual widget callbacks on every subpage without opening a dialog.
fn style_widget(root:&impl IsA<gtk::Widget>,name:&str)->Option<gtk::Widget>{
    let root=root.as_ref();if root.widget_name()==name{return Some(root.clone());}
    let mut child=root.first_child();while let Some(w)=child {if let Some(found)=style_widget(&w,name){return Some(found);}child=w.next_sibling();}None
}
fn style_click(s:&StylePopover,name:&str){style_widget(&s.stack,name).unwrap_or_else(||panic!("missing widget {name}")).downcast::<gtk::Button>().unwrap().emit_clicked();}
fn style_ui_test(s:&Rc<StylePopover>,menu:&gtk::MenuButton,app:&Application){
    menu.popup();let ui=s.ui.upgrade().unwrap();
    let revision=ui.current.borrow().as_ref().unwrap().session.rev();
    style_click(s,"style-manage-Heading1");assert_eq!(s.stack.visible_child_name().as_deref(),Some("detail"));
    assert!(style_widget(&s.detail,"style-preview").is_some());assert!(!style_widget(&s.detail,"style-delete").unwrap().is_sensitive());
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),revision);
    style_widget(&s.detail,"style-back").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();
    style_click(s,"style-new");let name=style_widget(&s.form,"style-name").unwrap().downcast::<gtk::Entry>().unwrap();name.set_text("Prueba del desplegable");style_click(s,"style-commit");
    let id=ui.current.borrow().as_ref().unwrap().session.doc.styles.find("Prueba del desplegable").unwrap().id.clone();
    assert!(style_widget(&s.list,&format!("style-manage-{id}")).is_some());
    style_click(s,&format!("style-manage-{id}"));style_click(s,"style-rename");style_widget(&s.form,"style-name").unwrap().downcast::<gtk::Entry>().unwrap().set_text("Renombrado en desplegable");style_click(s,"style-commit");
    assert!(ui.current.borrow().as_ref().unwrap().session.doc.styles.find("Renombrado en desplegable").is_some());
    style_click(s,&format!("style-manage-{id}"));style_click(s,"style-update");style_click(s,"style-delete");
    assert!(ui.current.borrow().as_ref().unwrap().session.doc.styles.get(&id).is_none());assert_eq!(s.stack.visible_child_name().as_deref(),Some("list"));
    s.search.set_text("Heading 9");s.refresh_list();assert!(style_widget(&s.list,"style-apply-Heading9").is_some());assert!(style_widget(&s.list,"style-apply-Normal").is_none());
    style_click(s,"style-apply-Heading9");assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc.para_at(&ui.current.borrow().as_ref().unwrap().session.sel.focus).unwrap().props.style.as_deref(),Some("Heading9"));
    assert_eq!(app.windows().len(),1,"style actions must stay inside the popover");eprintln!("STYLES_POPOVER_OK");app.quit();
}
