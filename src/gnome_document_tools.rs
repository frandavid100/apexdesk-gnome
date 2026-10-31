// Native document tools; engine adapters own all mutations.
use wordcraft_gnome::{pictures,document_properties::{self,PageSetup},pdf_export};
const PT_PER_MM:f64=72.0/25.4;
fn mm_input(value:f64,min:f64,max:f64)->gtk::SpinButton {
    let n=gtk::SpinButton::with_range(min,max,1.0);n.set_digits(2);n.set_value(value);n.set_width_chars(7);n.set_valign(gtk::Align::Center);n
}
fn document_identity(ui:&Ui)->Option<(u64,u64)>{ui.current.borrow().as_ref().map(|d|(d.session.document_id(),d.session.rev()))}
fn tools_change(ui:&Rc<Ui>,f:impl FnOnce(&mut Session)->Result<(),String>)->bool {
    let result={let mut state=ui.current.borrow_mut();let Some(d)=state.as_mut()else{return false;};let rev=d.session.rev();f(&mut d.session).map(|_|d.session.rev()!=rev)};
    match result {Ok(changed)=>{if changed{after_content_change(ui);}else{refresh_selection(ui);}true},Err(e)=>{ui.toasts.add_toast(Toast::new(&e));false}}
}
struct PicturePopover {
    ui:std::rc::Weak<Ui>,pop:gtk::Popover,context:gtk::Box,hint:gtk::Label,
    width:gtk::SpinButton,height:gtk::SpinButton,lock:gtk::CheckButton,
    syncing:Cell<bool>,width_changed:Cell<bool>,target:RefCell<Option<(Pos,f32,f32)>>,
}
impl PicturePopover {
    fn refresh(&self){
        let target=self.ui.upgrade().and_then(|ui|ui.current.borrow().as_ref().and_then(|d|pictures::selected(&d.session)));
        self.context.set_visible(target.is_some());self.hint.set_visible(target.is_none());
        if *self.target.borrow()==target{return;}
        self.syncing.set(true);
        if let Some((_,w,h))=&target{self.width.set_value(*w as f64/PT_PER_MM);self.height.set_value(*h as f64/PT_PER_MM);}
        self.syncing.set(false);self.target.replace(target);
    }
    fn execute(&self,f:impl FnOnce(&mut Session)->Result<(),String>){
        if let Some(ui)=self.ui.upgrade(){if tools_change(&ui,f){self.refresh();close_format_popover(&self.pop);ui.document_box.grab_focus();caret_ensure_visible(&ui);}}
    }
}
fn choose_picture(ui:&Rc<Ui>,parent:&ApplicationWindow,replace:bool){
    let identity=document_identity(ui);let target=ui.current.borrow().as_ref().and_then(|d|pictures::selected(&d.session));
    let dialog=gtk::FileDialog::new();dialog.set_modal(true);dialog.set_title(if replace{"Sustituir imagen"}else{"Insertar imagen"});
    let filter=gtk::FileFilter::new();filter.set_name(Some("Imágenes (PNG, JPEG, GIF, WebP, BMP, TIFF)"));
    for mime in ["image/png","image/jpeg","image/gif","image/webp","image/bmp","image/tiff"]{filter.add_mime_type(mime);}
    let filters=gio::ListStore::new::<gtk::FileFilter>();filters.append(&filter);dialog.set_filters(Some(&filters));
    let ui=ui.clone();dialog.open(Some(parent),None::<&gio::Cancellable>,glib::clone!(#[weak] ui, move|result|match result{
        Ok(file)=>{
            if ui.closed.get()||document_identity(&ui)!=identity{ui.toasts.add_toast(Toast::new("El documento ha cambiado; vuelve a elegir la imagen"));return;}
            let Some(path)=file.path()else{ui.toasts.add_toast(Toast::new("Elige un archivo local"));return;};
            if replace&&ui.current.borrow().as_ref().and_then(|d|pictures::selected(&d.session))!=target{return;}
            if tools_change(&ui,|s|if replace{pictures::replace(s,&path)}else{pictures::insert(s,&path)}){
                if let Some(p)=ui.picture.borrow().as_ref(){p.refresh();close_format_popover(&p.pop);}
                ui.document_box.grab_focus();caret_ensure_visible(&ui);
            }
        },
        Err(e)if e.matches(gtk::DialogError::Cancelled)||e.matches(gtk::DialogError::Dismissed)=>{},
        Err(e)=>ui.toasts.add_toast(Toast::new(&e.to_string())),
    }));
}
fn install_picture_controls(ui:&Rc<Ui>,parent:&ApplicationWindow,bar:&gtk::Box)->(gtk::MenuButton,Rc<PicturePopover>){
    let menu=gtk::MenuButton::new();format_menu_label(&menu,"Imagen");menu.set_direction(gtk::ArrowType::Up);bar.append(&menu);
    let pop=gtk::Popover::new();pop.add_css_class("format-popover");
    let root=gtk::Box::new(gtk::Orientation::Vertical,8);root.set_width_request(300);
    root.set_margin_top(8);root.set_margin_bottom(8);root.set_margin_start(8);root.set_margin_end(8);
    format_popover_header(&pop,&root,"Imagen",&ui.format_group);
    let insert=style_menu_button("Insertar imagen…");insert.set_widget_name("picture-insert");insert.set_visible(tool_diagnostics());root.append(&insert);
    let hint=gtk::Label::new(Some("Selecciona una imagen en el documento para modificarla."));hint.set_wrap(true);hint.set_max_width_chars(30);hint.add_css_class("dim-label");root.append(&hint);
    let context=gtk::Box::new(gtk::Orientation::Vertical,8);context.set_widget_name("picture-context");root.append(&context);
    context.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let width=mm_input(30.0,1.0/PT_PER_MM,4000.0/PT_PER_MM);width.set_widget_name("picture-width");
    let height=mm_input(15.0,1.0/PT_PER_MM,4000.0/PT_PER_MM);height.set_widget_name("picture-height");
    for(label,n)in[("Anchura (mm)",&width),("Altura (mm)",&height)]{let row=gtk::Box::new(gtk::Orientation::Horizontal,8);let l=gtk::Label::new(Some(label));l.set_xalign(0.0);l.set_hexpand(true);row.append(&l);row.append(n);context.append(&row);}
    let lock=gtk::CheckButton::with_label("Mantener proporción");lock.set_active(true);context.append(&lock);
    let apply=style_menu_button("Aplicar tamaño");apply.set_widget_name("picture-apply");context.append(&apply);
    let original=style_menu_button("Restaurar tamaño original");original.set_widget_name("picture-original");context.append(&original);
    let replace=style_menu_button("Sustituir imagen…");replace.set_widget_name("picture-replace");context.append(&replace);
    let delete=style_menu_button("Eliminar imagen");delete.set_widget_name("picture-delete");context.append(&delete);
    // Keep the header fixed when the window is short.
    let scroll=gtk::ScrolledWindow::new();scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_max_content_height(440);scroll.set_propagate_natural_height(true);
    let body=gtk::Box::new(gtk::Orientation::Vertical,8);root.remove(&insert);root.remove(&hint);root.remove(&context);body.append(&insert);body.append(&hint);body.append(&context);scroll.set_child(Some(&body));root.append(&scroll);
    pop.set_child(Some(&root));menu.set_popover(Some(&pop));
    let state=Rc::new(PicturePopover{ui:Rc::downgrade(ui),pop:pop.clone(),context,hint,width,height,lock,syncing:Cell::new(false),width_changed:Cell::new(true),target:RefCell::new(None)});
    for(is_width,n)in[(true,&state.width),(false,&state.height)]{let weak=Rc::downgrade(&state);n.connect_value_changed(move|n|{let Some(s)=weak.upgrade()else{return;};if s.syncing.get(){return;}s.width_changed.set(is_width);
        if s.lock.is_active(){if let Some((_,w,h))=s.target.borrow().as_ref(){s.syncing.set(true);if is_width{s.height.set_value(n.value()* *h as f64/ *w as f64);}else{s.width.set_value(n.value()* *w as f64/ *h as f64);}s.syncing.set(false);}}
    });}
    {let weak=Rc::downgrade(&state);apply.connect_clicked(glib::clone!(#[weak] ui, move |_|{if let Some(s)=weak.upgrade(){s.width.update();s.height.update();let (w,h,lock,by_width)=(s.width.value()as f32*PT_PER_MM as f32,s.height.value()as f32*PT_PER_MM as f32,s.lock.is_active(),s.width_changed.get());s.execute(move|session|pictures::resize(session,w,h,lock,by_width));}}));}
    {let weak=Rc::downgrade(&state);original.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.execute(pictures::original);}});}
    {let weak=Rc::downgrade(&state);delete.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.execute(pictures::delete);}});}
    for(b,replacing)in[(insert,false),(replace,true)]{let weak=Rc::downgrade(ui);let parent=parent.downgrade();b.connect_clicked(move |_|{if let(Some(ui),Some(parent))=(weak.upgrade(),parent.upgrade()){choose_picture(&ui,&parent,replacing);}});}
    {let weak=Rc::downgrade(&state);pop.connect_show(move |_|{if let Some(s)=weak.upgrade(){s.refresh();}});}
    state.refresh();(menu,state)
}

struct DocumentProperties {window:adw::Window,width:gtk::SpinButton,height:gtk::SpinButton,top:gtk::SpinButton,bottom:gtk::SpinButton,left:gtk::SpinButton,right:gtk::SpinButton,apply:gtk::Button}
fn properties_number(group:&adw::PreferencesGroup,label:&str,value:f32)->gtk::SpinButton{
    let row=adw::ActionRow::new();row.set_title(label);let n=mm_input(value as f64/PT_PER_MM,0.0,1584.0/PT_PER_MM);row.add_suffix(&n);row.set_activatable_widget(Some(&n));accessible_name(&n,label);group.add(&row);n
}
fn show_document_properties(ui:&Rc<Ui>,parent:&ApplicationWindow)->DocumentProperties {
    let state={let value=ui.current.borrow();PageSetup::current(&value.as_ref().unwrap().session)};let identity=document_identity(ui);
    let window=adw::Window::builder().title("Propiedades del documento").transient_for(parent).modal(true).default_width(480).default_height(650).build();
    let toolbar=ToolbarView::new();let header=HeaderBar::new();header.set_show_end_title_buttons(false);header.set_show_start_title_buttons(false);
    let cancel=gtk::Button::with_label("Cancelar");header.pack_start(&cancel);let apply=gtk::Button::with_label("Aplicar");apply.set_widget_name("document-properties-apply");apply.add_css_class("suggested-action");header.pack_end(&apply);toolbar.add_top_bar(&header);
    let page=adw::PreferencesPage::new();let paper=adw::PreferencesGroup::new();paper.set_title("Página");page.add(&paper);
    let sizes=adw::ComboRow::new();sizes.set_title("Tamaño de papel");sizes.set_model(Some(&gtk::StringList::new(&["Personalizado","A4","A5","A3","Carta","Legal"])));paper.add(&sizes);
    let orientation=adw::ComboRow::new();orientation.set_title("Orientación");orientation.set_model(Some(&gtk::StringList::new(&["Vertical","Horizontal"])));orientation.set_selected(if state.width>state.height{1}else{0});paper.add(&orientation);
    let width=properties_number(&paper,"Anchura (mm)",state.width);let height=properties_number(&paper,"Altura (mm)",state.height);
    let presets=[(210.0,297.0),(148.0,210.0),(297.0,420.0),(215.9,279.4),(215.9,355.6)];
    for(i,(w,h))in presets.iter().enumerate(){let (a,b)=if state.width>state.height{(*h,*w)}else{(*w,*h)};if (width.value()-a).abs()<0.1&&(height.value()-b).abs()<0.1{sizes.set_selected(i as u32+1);}}
    let margins=adw::PreferencesGroup::new();margins.set_title("Márgenes");page.add(&margins);
    let top=properties_number(&margins,"Superior (mm)",state.top);let bottom=properties_number(&margins,"Inferior (mm)",state.bottom);let left=properties_number(&margins,"Izquierdo (mm)",state.left);let right=properties_number(&margins,"Derecho (mm)",state.right);
    let scope=adw::ComboRow::new();scope.set_title("Aplicar a");scope.set_model(Some(&gtk::StringList::new(&["Todo el documento","Sección actual"])));
    let multi=ui.current.borrow().as_ref().is_some_and(|d|d.session.doc.sections().len()>1);scope.set_selected(if multi{1}else{0});scope.set_visible(multi);margins.add(&scope);
    let message=gtk::Label::new(None);message.set_wrap(true);message.add_css_class("error");let errors=adw::PreferencesGroup::new();errors.add(&message);page.add(&errors);message.set_visible(false);
    toolbar.set_content(Some(&page));window.set_content(Some(&toolbar));
    let syncing=Rc::new(Cell::new(false));
    {let width=width.downgrade();let height=height.downgrade();let orientation=orientation.downgrade();let syncing=syncing.clone();sizes.connect_selected_notify(move|r|{let Some(i)=r.selected().checked_sub(1)else{return;};let Some(&(a,b))=presets.get(i as usize)else{return;};let(Some(width),Some(height),Some(orientation))=(width.upgrade(),height.upgrade(),orientation.upgrade())else{return;};syncing.set(true);let (a,b)=if orientation.selected()==1{(b,a)}else{(a,b)};width.set_value(a);height.set_value(b);syncing.set(false);});}
    {let width=width.downgrade();let height=height.downgrade();let syncing=syncing.clone();orientation.connect_selected_notify(move|r|{let(Some(width),Some(height))=(width.upgrade(),height.upgrade())else{return;};let (w,h)=(width.value(),height.value());if (r.selected()==1&&w<h)||(r.selected()==0&&w>h){syncing.set(true);width.set_value(h);height.set_value(w);syncing.set(false);}});}
    for n in [&width,&height]{let sizes=sizes.downgrade();let orientation=orientation.downgrade();let width=width.downgrade();let height=height.downgrade();let syncing=syncing.clone();n.connect_value_changed(move |_|{if !syncing.get(){let(Some(sizes),Some(orientation),Some(width),Some(height))=(sizes.upgrade(),orientation.upgrade(),width.upgrade(),height.upgrade())else{return;};sizes.set_selected(0);orientation.set_selected(if width.value()>height.value(){1}else{0});}});}
    {let weak=window.downgrade();cancel.connect_clicked(move |_|{if let Some(w)=weak.upgrade(){w.close();}});}
    {let ui=ui.clone();let window=window.downgrade();let fields=[width.clone(),height.clone(),top.clone(),bottom.clone(),left.clone(),right.clone()];apply.connect_clicked(glib::clone!(#[weak] ui, move |_|{
        if ui.closed.get()||document_identity(&ui)!=identity{message.set_text("El documento ha cambiado. Cierra y vuelve a abrir las propiedades.");message.set_visible(true);return;}
        for n in &fields{n.update();}let values=fields.each_ref().map(|n|n.value()as f32*PT_PER_MM as f32);
        let setup=PageSetup{width:values[0],height:values[1],top:values[2],bottom:values[3],left:values[4],right:values[5]};
        let result={let mut value=ui.current.borrow_mut();document_properties::apply(&mut value.as_mut().unwrap().session,setup,scope.selected()==0)};
        match result {Ok(changed)=>{if changed{after_content_change(&ui);}if let Some(w)=window.upgrade(){w.close();}},Err(e)=>{message.set_text(&e);message.set_visible(true);}}
    }));}
    window.present();DocumentProperties{window,width,height,top,bottom,left,right,apply}
}
fn export_pdf_dialog(ui:&Rc<Ui>,parent:&ApplicationWindow){
    let dialog=gtk::FileDialog::new();dialog.set_title("Exportar a PDF");dialog.set_modal(true);
    let filter=gtk::FileFilter::new();filter.set_name(Some("Documento PDF (.pdf)"));filter.add_pattern("*.pdf");let filters=gio::ListStore::new::<gtk::FileFilter>();filters.append(&filter);dialog.set_filters(Some(&filters));
    let name=ui.document_name.borrow().clone();let stem=Path::new(&name).file_stem().unwrap_or_default().to_string_lossy();dialog.set_initial_name(Some(&format!("{stem}.pdf")));
    let identity=document_identity(ui);let ui=ui.clone();dialog.save(Some(parent),None::<&gio::Cancellable>,glib::clone!(#[weak] ui, move|result|match result{
        Ok(file)=>{
            if ui.closed.get()||document_identity(&ui)!=identity{ui.toasts.add_toast(Toast::new("El documento ha cambiado; vuelve a exportarlo"));return;}
            let Some(path)=file.path()else{ui.toasts.add_toast(Toast::new("Elige un archivo local"));return;};
            let doc=ui.current.borrow().as_ref().unwrap().session.doc.clone();
            ui.toasts.add_toast(Toast::new("Exportando PDF…"));let weak=Rc::downgrade(&ui);
            glib::MainContext::default().spawn_local(async move{
                let result=gio::spawn_blocking(move||pdf_export::export(&mut Session::new(doc),&path)).await;
                if let Some(ui)=weak.upgrade(){if !ui.closed.get(){let text=match result{Ok(Ok(()))=>"PDF exportado".to_string(),Ok(Err(e))=>e,Err(_)=>"No se pudo exportar el PDF".to_string()};ui.toasts.add_toast(Toast::new(&text));}}
            });
        },Err(e)if e.matches(gtk::DialogError::Cancelled)||e.matches(gtk::DialogError::Dismissed)=>{},Err(e)=>ui.toasts.add_toast(Toast::new(&e.to_string())),
    }));
}

fn document_tools_fixture(ui:&Rc<Ui>){
    let file=std::env::temp_dir().join(format!("wordcraft-picture-{}.png",std::process::id()));
    std::fs::write(&file,wordcraft_engine::cmd::insert::base64_decode(include_str!("../tests/fixtures/picture.b64")).unwrap()).unwrap();
    let mut session=Session::new(wordcraft_doc::Document::from_text("Documento con imagen\nUna imagen en línea conserva su lugar entre los párrafos.\n\nEl documento puede guardarse como DOCX u ODT y exportarse a PDF."));
    session.run("caret.set",&serde_json::json!({"pos":Pos::body(2,0)})).unwrap();pictures::insert(&mut session,&file).unwrap();pictures::resize(&mut session,240.0,120.0,false,true).unwrap();
    opened(ui,session,"Documento con imagen.docx");
}
fn document_tools_ui_test(ui:&Rc<Ui>,panel:&Rc<PicturePopover>,menu:&gtk::MenuButton,parent:&ApplicationWindow,app:&Application){
    let ui=ui.clone();let panel=panel.clone();let menu=menu.clone();let parent=parent.clone();let app=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(350),move||{
        let (px,py,rev)={let mut state=ui.current.borrow_mut();let d=state.as_mut().unwrap();let hit=d.session.layout().find_object(0,|_|true).unwrap();let frame=d.page_frames[hit.page].compute_bounds(&parent).unwrap();
            let point=(frame.x()as f64+(hit.rect.x+hit.rect.w/2.0)as f64*d.zoom as f64,frame.y()as f64+(hit.rect.y+hit.rect.h/2.0)as f64*d.zoom as f64,d.session.rev());
            d.session.run("caret.docStart",&serde_json::json!({})).unwrap();point};refresh_selection(&ui);
        eprintln!("IMAGE_POINTER_READY {} {}",px as i32,py as i32);
        glib::timeout_add_local_once(std::time::Duration::from_millis(1500),move||{
            {let state=ui.current.borrow();let d=state.as_ref().unwrap();assert!(wordcraft_engine::cmd::objects::object_selection(&d.session).is_some(),"real pointer must select the picture");assert_eq!(d.session.rev(),rev);}
            menu.popup();popover_pin(&panel.pop).set_active(true);panel.width.set_value(50.0);
            style_widget(&panel.pop.child().unwrap(),"picture-apply").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();
            {let state=ui.current.borrow();let (_,w,h)=pictures::selected(&state.as_ref().unwrap().session).unwrap();assert!((w/h-2.0).abs()<0.01);assert!((w as f64/PT_PER_MM-50.0).abs()<0.1);}
            style_widget(&panel.pop.child().unwrap(),"picture-original").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();
            assert_eq!(pictures::selected(&ui.current.borrow().as_ref().unwrap().session).unwrap().1,120.0);
            style_widget(&panel.pop.child().unwrap(),"picture-delete").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();assert!(!panel.context.is_visible());apply_user_edit(&ui,Edit::Undo);assert!(panel.context.is_visible());
            panel.pop.popdown();let props=show_document_properties(&ui,&parent);props.width.set_value(297.0);props.height.set_value(210.0);props.top.set_value(20.0);props.bottom.set_value(20.0);props.left.set_value(25.0);props.right.set_value(25.0);props.apply.emit_clicked();
            {let mut state=ui.current.borrow_mut();let d=state.as_mut().unwrap();let p=PageSetup::current(&d.session);assert!(p.width>p.height);let (id,dirty,depth,rev)=(d.session.document_id(),d.session.dirty,d.session.undo_depth(),d.session.rev());pdf_export::export(&mut d.session,Path::new("document-tools-test.pdf")).unwrap();assert_eq!((d.session.document_id(),d.session.dirty,d.session.undo_depth(),d.session.rev()),(id,dirty,depth,rev));}
            assert!(!props.window.is_visible());assert_eq!(app.windows().len(),1);eprintln!("DOCUMENT_TOOLS_OK");app.quit();
        });
    });
}


include!("gnome_columns.rs");
