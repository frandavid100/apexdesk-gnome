// Native context menus. Selection and editing remain public engine commands.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum DocumentContext { Text, Table, Picture }

// Share the engine fragment between tabs only while GTK still owns this exact
// clipboard provider. Never paste an old fragment after another app copies.
thread_local! {
    static RICH_CLIPBOARD: RefCell<Option<(gdk::ContentProvider,wordcraft_doc::edit::Fragment)>> = const { RefCell::new(None) };
}
fn context_copy(ui:&Rc<Ui>)->bool {
    let result={let mut value=ui.current.borrow_mut();let Some(d)=value.as_mut()else{return false;};
        if d.session.sel.is_collapsed(){return false;}
        d.session.run("edit.copy",&serde_json::json!({})).map(|v|(v,d.session.clipboard.clone()))};
    let (value,fragment)=match result{Ok(v)=>v,Err(e)=>{ui.toasts.add_toast(Toast::new(&e.to_string()));return false;}};
    let text=value["text"].as_str().unwrap_or_default();
    let mut providers=vec![gdk::ContentProvider::for_value(&text.to_value())];
    if let Some(html)=value["html"].as_str().filter(|h|!h.is_empty()){
        providers.push(gdk::ContentProvider::for_bytes("text/html",&glib::Bytes::from_owned(html.as_bytes().to_vec())));
    }
    let provider=gdk::ContentProvider::new_union(&providers);
    if let Err(e)=ui.document_box.clipboard().set_content(Some(&provider)){ui.toasts.add_toast(Toast::new(&e.to_string()));return false;}
    RICH_CLIPBOARD.with(|c|c.replace(fragment.map(|f|(provider,f))));true
}
fn context_paste(ui:&Rc<Ui>){
    let clipboard=ui.document_box.clipboard();
    let fragment=RICH_CLIPBOARD.with(|c|c.borrow().as_ref().filter(|(p,_)|clipboard.content().as_ref()==Some(p)).map(|(_,f)|f.clone()));
    if let Some(fragment)=fragment{
        tools_change(ui,move|s|{s.clipboard_text=fragment.clipboard_text();s.clipboard=Some(fragment);s.run("edit.paste",&serde_json::json!({})).map(|_|()).map_err(|e|e.to_string())});
        ui.document_box.grab_focus();caret_ensure_visible(ui);return;
    }
    let identity=document_identity(ui);let selection=ui.current.borrow().as_ref().map(|d|d.session.sel.clone());let weak=Rc::downgrade(ui);
    clipboard.read_text_async(None::<&gio::Cancellable>,move|result|{
        let Some(ui)=weak.upgrade()else{return;};
        if ui.closed.get()||document_identity(&ui)!=identity||ui.current.borrow().as_ref().map(|d|d.session.sel.clone())!=selection{return;}
        match result{Ok(Some(text))=>apply_user_edit(&ui,Edit::Paste(text.to_string())),Ok(None)=>{},Err(e)=>ui.toasts.add_toast(Toast::new(&e.to_string()))}
        ui.document_box.grab_focus();
    });
}
fn context_prepare(ui:&Rc<Ui>,x:f64,y:f64)->Option<DocumentContext>{
    let pos=pos_at_canvas(ui,x,y)?;
    let keep={let value=ui.current.borrow();let d=value.as_ref()?;let(a,b)=d.session.sel.ordered();
        let text=!d.session.sel.is_collapsed()&&a.story==pos.story&&a<=pos&&pos<b;
        let cells=pos.path.cell().zip(d.session.sel.focus.path.cell()).is_some_and(|((tp,r,c),(current,_,_))|pos.story==d.session.sel.focus.story&&tp==current&&d.session.cell_range().is_some()&&wordcraft_engine::cmd::table::selected_cells(&d.session).contains(&(r,c)));
        text||cells};
    let picture=if keep{ui.current.borrow().as_ref().is_some_and(|d|pictures::selected(&d.session).is_some())}else{select_picture_at_canvas(ui,x,y)};
    if !keep&&!picture{let mut value=ui.current.borrow_mut();let d=value.as_mut()?;if let Err(e)=selection::set_caret(&mut d.session,&pos,false){ui.toasts.add_toast(Toast::new(&e));return None;}}
    refresh_selection(ui);
    Some(if picture{DocumentContext::Picture}else if pos.path.cell().is_some(){DocumentContext::Table}else{DocumentContext::Text})
}
fn context_open_format(ui:&Rc<Ui>,name:&str){
    if let Some(menu)=format_button(ui,name){menu.popup();}
}
struct ContextFormatLease {host:glib::WeakRef<gtk::PopoverMenu>,entries:Vec<(gtk::Popover,gtk::Widget,Rc<FormatHeader>)>,restored:Cell<bool>}
impl ContextFormatLease {
    fn restore(&self){
        if self.restored.replace(true){return;}
        for(origin,body,header)in &self.entries{
            if let Some(host)=self.host.upgrade(){host.remove_child(body);}
            if body.parent().is_none(){origin.set_child(Some(body));}
            header.parent_back.borrow_mut().take();header.home();
            EMBEDDED_FORMAT.with(|items|items.borrow_mut().retain(|(p,_,_)|p.upgrade().as_ref()!=Some(origin)));
        }
    }
}
fn native_menu_stack(root:&impl IsA<gtk::Widget>)->Option<gtk::Stack>{
    let root=root.as_ref();if let Some(s)=root.downcast_ref::<gtk::Stack>(){return Some(s.clone());}
    let mut child=root.first_child();while let Some(w)=child{if let Some(s)=native_menu_stack(&w){return Some(s);}child=w.next_sibling();}None
}
fn mount_context_formats(ui:&Rc<Ui>,host:&gtk::PopoverMenu){
    // Official GtkPopoverMenu custom slots hold the existing widget trees.
    // Keep nonmodality constant, including when pinning inside a custom page.
    host.add_css_class("format-popover");host.add_css_class("style-popover");
    host.set_autohide(false);ui.format_group.show(host.upcast_ref());
    ui.format_group.hosts.borrow_mut().retain(|p|p.upgrade().is_some());ui.format_group.hosts.borrow_mut().push(host.clone().upcast::<gtk::Popover>().downgrade());
    let mut entries=Vec::new();
    for(name,label,id)in[("style-menu","Estilo","context-style"),("paragraph-menu","Párrafo","context-paragraph"),("text-menu","Texto","context-text")]{
        let Some(origin)=format_button(ui,name).and_then(|m|m.popover())else{continue;};
        let header=ui.format_group.entries.borrow().iter().find(|(p,_)|p.upgrade().as_ref()==Some(&origin)).map(|(_,h)|h.clone());
        let (Some(body),Some(header))=(origin.child(),header)else{continue;};origin.set_child(None::<&gtk::Widget>);
        if !host.add_child(&body,id){origin.set_child(Some(&body));continue;}
        let weak=host.downgrade();header.parent_back.replace(Some(Rc::new(move||{if let Some(host)=weak.upgrade(){host.set_visible_submenu(Some("main"));}})));
        header.home();
        // GTK generates a native back/title row before every custom submenu.
        // Our shared header already provides back/title/pin in one row. Retain
        // GTK's title for its navigation bookkeeping, but hide this duplicate.
        if let Some(page)=native_menu_stack(host).and_then(|s|s.child_by_name(label)){if let Some(title)=page.first_child(){if title.type_().name()=="GtkModelButton"{title.set_visible(false);}}}
        EMBEDDED_FORMAT.with(|items|items.borrow_mut().push((origin.downgrade(),host.clone().upcast::<gtk::Popover>().downgrade(),header.pin.downgrade())));
        entries.push((origin,body,header));
    }
    if let Some(s)=ui.style.borrow().as_ref(){s.main();}if let Some(s)=ui.paragraph.borrow().as_ref(){s.home();}if let Some(s)=ui.text_tools.borrow().as_ref(){s.home();}
    let lease=Rc::new(ContextFormatLease{host:host.downgrade(),entries,restored:Cell::new(false)});
    host.connect_closed(move |_|lease.restore());
}
fn context_item(menu:&gio::Menu,group:&gio::SimpleActionGroup,pop:&gtk::PopoverMenu,ui:&Rc<Ui>,label:&str,name:&str,enabled:bool,callback:impl Fn(&Rc<Ui>)+'static){
    let action=gio::SimpleAction::new(name,None);action.set_enabled(enabled);
    let weak=Rc::downgrade(ui);let popup=pop.downgrade();
    action.connect_activate(move|_,_|{if let Some(pop)=popup.upgrade(){pop.popdown();}if let Some(ui)=weak.upgrade(){if !ui.closed.get(){callback(&ui);}}});
    group.add_action(&action);let item=gio::MenuItem::new(Some(label),Some(&format!("context.{name}")));
    let accel=match name{"cut"=>Some("<Control>x"),"copy"=>Some("<Control>c"),"paste"=>Some("<Control>v"),"select-all"=>Some("<Control>a"),_=>None};
    if let Some(accel)=accel{item.set_attribute_value("accel",Some(&accel.to_variant()));}menu.append_item(&item);
}
fn show_document_context(ui:&Rc<Ui>,parent:&ApplicationWindow,x:i32,y:i32,kind:DocumentContext)->gtk::PopoverMenu{
    // A single native GMenu supplies separators, submenu navigation and states.
    let root=gio::Menu::new();let pop=gtk::PopoverMenu::from_model(Some(&root));pop.set_widget_name("document-context-menu");
    pop.set_has_arrow(false);pop.set_halign(gtk::Align::Start);
    pop.set_parent(&ui.document_box);pop.set_pointing_to(Some(&gdk::Rectangle::new(x,y,1,1)));
    if let Some(p)=ui.table_popover.borrow().as_ref(){if p.parent().as_ref()==Some(ui.document_box.upcast_ref()){p.set_pointing_to(Some(&gdk::Rectangle::new(x,y,1,1)));}}
    if let Some(s)=ui.picture.borrow().as_ref(){if s.pop.parent().as_ref()==Some(ui.document_box.upcast_ref()){s.pop.set_pointing_to(Some(&gdk::Rectangle::new(x,y,1,1)));}}
    let group=gio::SimpleActionGroup::new();pop.insert_action_group("context",Some(&group));
    let (selected,merge)=ui.current.borrow().as_ref().map(|d|(!d.session.sel.is_collapsed(),wordcraft_gnome::tables::selected_cells(&d.session)>1)).unwrap_or_default();
    let edit=gio::Menu::new();
    context_item(&edit,&group,&pop,ui,"Cortar","cut",selected,|u|{if context_copy(u){apply_user_edit(u,Edit::Delete);}});
    context_item(&edit,&group,&pop,ui,"Copiar","copy",selected,|u|{context_copy(u);});
    let formats=ui.document_box.clipboard().formats();
    let can_paste=formats.contains_type(String::static_type())||formats.contain_mime_type("text/plain")||formats.contain_mime_type("text/plain;charset=utf-8");
    context_item(&edit,&group,&pop,ui,"Pegar","paste",can_paste,context_paste);
    context_item(&edit,&group,&pop,ui,"Seleccionar todo","select-all",true,|u|selected_command(u,"select-all",false));root.append_section(None,&edit);
    let specific=gio::Menu::new();
    match kind {
        DocumentContext::Text=>{
            for(label,id)in[("Estilo","context-style"),("Párrafo","context-paragraph"),("Texto","context-text")]{let page=gio::Menu::new();let item=gio::MenuItem::new(None,None);item.set_attribute_value("custom",Some(&id.to_variant()));page.append_item(&item);specific.append_submenu(Some(label),&page);}
        },
        DocumentContext::Picture=>{
            context_item(&specific,&group,&pop,ui,"Tamaño de la imagen…","picture-size",true,|u|context_picture_editor(u));
            context_item(&specific,&group,&pop,ui,"Restaurar tamaño original","picture-original",true,|u|{tools_change(u,pictures::original);});
            let p=parent.downgrade();context_item(&specific,&group,&pop,ui,"Sustituir imagen…","picture-replace",true,move|u|{if let Some(p)=p.upgrade(){choose_picture(u,&p,true);}});
            context_item(&specific,&group,&pop,ui,"Eliminar imagen","picture-delete",true,|u|{tools_change(u,pictures::delete);});
        },
        DocumentContext::Table=>{
            for(title,prefix,items)in[
                ("Añadir","insert",vec![("Fila encima","table.insertRowAbove"),("Fila debajo","table.insertRowBelow"),("Columna a la izquierda","table.insertColumnLeft"),("Columna a la derecha","table.insertColumnRight")]),
                ("Seleccionar","select",vec![("Celda","table.selectCell"),("Fila","table.selectRow"),("Columna","table.selectColumn"),("Tabla","table.selectTable")]),
                ("Eliminar","delete",vec![("Fila","table.deleteRow"),("Columna","table.deleteColumn"),("Tabla","table.deleteTable")])]{
                let sub=gio::Menu::new();for(i,(label,cmd))in items.into_iter().enumerate(){context_item(&sub,&group,&pop,ui,label,&format!("{prefix}-{i}"),true,move|u|table_command(u,cmd,serde_json::json!({})));}specific.append_submenu(Some(title),&sub);
            }
            context_item(&specific,&group,&pop,ui,"Más opciones de tabla…","table-tools",true,context_table_editor);
            context_item(&specific,&group,&pop,ui,"Combinar celdas","merge",merge,|u|table_command(u,"table.merge",serde_json::json!({})));
            let p=parent.downgrade();context_item(&specific,&group,&pop,ui,"Propiedades y dividir celdas…","table-properties",true,move|u|{if let Some(p)=p.upgrade(){show_table_properties(u,&p);}});
        }
    }
    root.append_section(None,&specific);
    if kind==DocumentContext::Text{mount_context_formats(ui,&pop);}
    // GtkModelButton closes before activating its action helper. Keep the
    // hierarchy and action muxer alive until that signal has fully returned.
    pop.connect_closed(|p|{let p=p.clone();glib::idle_add_local_once(move||{if !p.is_visible()&&p.parent().is_some(){p.unparent();}});});pop.popup();pop
}
fn show_table_context_at(ui:&Rc<Ui>,parent:&ApplicationWindow,x:i32,y:i32){
    let kind=ui.current.borrow().as_ref().map(|d|if pictures::selected(&d.session).is_some(){DocumentContext::Picture}else if wordcraft_gnome::tables::in_table(&d.session){DocumentContext::Table}else{DocumentContext::Text});
    if let Some(kind)=kind{show_document_context(ui,parent,x,y,kind);}
}
fn install_table_context_menu(ui:&Rc<Ui>,parent:&ApplicationWindow){
    let click=gtk::GestureClick::new();click.set_button(gdk::BUTTON_SECONDARY);
    let weak=Rc::downgrade(ui);let parent=parent.downgrade();
    click.connect_pressed(move|gesture,_,x,y|{if let(Some(ui),Some(parent))=(weak.upgrade(),parent.upgrade()){
        if let Some(kind)=context_prepare(&ui,x,y){gesture.set_state(gtk::EventSequenceState::Claimed);ui.document_box.grab_focus();show_document_context(&ui,&parent,x as i32,y as i32,kind);}
    }});ui.document_box.add_controller(click);
}

fn context_model_row(root:&impl IsA<gtk::Widget>,label:&str)->Option<gtk::Widget>{
    let root=root.as_ref();
    // GtkPopoverMenu keeps identically named rows in hidden GtkStack pages.
    // Search only the active page, as a pointer/keyboard user would.
    if let Some(stack)=root.downcast_ref::<gtk::Stack>(){return stack.visible_child().and_then(|page|context_model_row(&page,label));}
    if root.type_().name()=="GtkModelButton" {
        fn has_label(w:&gtk::Widget,label:&str)->bool{if w.downcast_ref::<gtk::Label>().is_some_and(|l|l.text()==label){return true;}let mut c=w.first_child();while let Some(child)=c{if has_label(&child,label){return true;}c=child.next_sibling();}false}
        if has_label(root,label){return Some(root.clone());}
    }
    let mut c=root.first_child();while let Some(child)=c{if let Some(found)=context_model_row(&child,label){return Some(found);}c=child.next_sibling();}None
}
fn context_activate_row(pop:&gtk::PopoverMenu,label:&str){let row=context_model_row(pop,label).unwrap_or_else(||panic!("missing native row {label}"));assert!(row.is_sensitive());assert!(row.activate(),"native menu row must activate");}
fn context_picture_editor(ui:&Rc<Ui>){if let Some(s)=ui.picture.borrow().as_ref(){s.refresh();s.pop.popup();}}
fn context_table_editor(ui:&Rc<Ui>){if let Some(p)=ui.table_popover.borrow().as_ref(){refresh_table_controls(ui,p);p.popup();}}
fn tool_diagnostics()->bool{std::env::args().any(|a|matches!(a.as_str(),"--pin-ui-test"|"--table-menu-demo"|"--table-properties-demo"|"--table-insert-demo"|"--picture-demo"|"--document-properties-demo"|"--document-tools-test"))}
fn context_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application){
    use gdk::prelude::PopupExt;
    opened(ui,Session::new(wordcraft_doc::Document::from_text("Hello world, texto de prueba\nSegundo párrafo")),"Contextos");
    {let mut value=ui.current.borrow_mut();let d=value.as_mut().unwrap();d.session.run("select.range",&serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)})).unwrap();}
    refresh_selection(ui);
    let ui=ui.clone();let parent=parent.clone();let app=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400),move||{
        let (x,y,sel,rev)={let mut value=ui.current.borrow_mut();let d=value.as_mut().unwrap();let c=d.session.layout().caret_on(&Pos::body(0,4),0).unwrap();let r=d.page_frames[c.page].compute_bounds(&parent).unwrap();(r.x()as f64+c.x as f64*d.zoom as f64,r.y()as f64+(c.top+c.height/2.0)as f64*d.zoom as f64,d.session.sel.clone(),d.session.rev())};
        eprintln!("CONTEXT_POINTER_READY {} {}",x as i32,y as i32);
        glib::timeout_add_local_once(std::time::Duration::from_secs(2),move||{
            {let value=ui.current.borrow();let d=value.as_ref().unwrap();assert_eq!(d.session.sel,sel,"right click must preserve selected text");assert_eq!(d.session.rev(),rev);}
            let pop=style_widget(&ui.document_box,"document-context-menu").unwrap().downcast::<gtk::PopoverMenu>().unwrap();assert!(pop.is_mapped());assert_eq!(pop.menu_model().unwrap().n_items(),2);
            let row=context_model_row(&pop,"Copiar").unwrap();let r=row.compute_bounds(&pop).unwrap();let surface=pop.surface().unwrap().downcast::<gdk::Popup>().unwrap();let(tx,ty)=pop.surface_transform();
            eprintln!("CONTEXT_ACTION_READY {} {}",(surface.position_x()as f64-tx+r.x()as f64+r.width()as f64/2.0)as i32,(surface.position_y()as f64-ty+r.y()as f64+r.height()as f64/2.0)as i32);
            glib::timeout_add_local_once(std::time::Duration::from_millis(700),move||{
            assert!(RICH_CLIPBOARD.with(|c|c.borrow().is_some()),"real native row click must copy");

            {let mut value=ui.current.borrow_mut();selection::set_caret(&mut value.as_mut().unwrap().session,&Pos::body(1,0),false).unwrap();}
            let edit=show_document_context(&ui,&parent,100,150,DocumentContext::Text);context_activate_row(&edit,"Pegar");assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());apply_user_edit(&ui,Edit::Undo);
            {let mut value=ui.current.borrow_mut();value.as_mut().unwrap().session.run("select.range",&serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)})).unwrap();}
            let edit=show_document_context(&ui,&parent,100,150,DocumentContext::Text);context_activate_row(&edit,"Cortar");
            assert_eq!(ui.current.borrow_mut().as_mut().unwrap().session.doc.para_at(&Pos::body(0,0)).unwrap().text,", texto de prueba");
            apply_user_edit(&ui,Edit::Undo);
            let edit=show_document_context(&ui,&parent,100,150,DocumentContext::Text);context_activate_row(&edit,"Seleccionar todo");assert!(!ui.current.borrow().as_ref().unwrap().session.sel.is_collapsed());
            for (label,name) in [("Estilo","style-menu"),("Párrafo","paragraph-menu"),("Texto","text-menu")]{let edit=show_document_context(&ui,&parent,100,150,DocumentContext::Text);context_activate_row(&edit,label);let m=format_button(&ui,name).unwrap();assert!(!m.popover().unwrap().is_visible(),"context must not open toolbar popup");assert!(m.popover().unwrap().child().is_none(),"controls must be embedded in context");assert_eq!(edit.visible_submenu().as_deref(),Some(label));edit.popdown();assert!(m.popover().unwrap().child().is_some(),"closing context must return controls");}

            {let mut value=ui.current.borrow_mut();value.as_mut().unwrap().session.run("select.range",&serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)})).unwrap();}
            // Clicking outside the selection moves the caret, without a mutation.
            let (cx,cy)={let mut value=ui.current.borrow_mut();let d=value.as_mut().unwrap();let c=d.session.layout().caret_on(&Pos::body(1,3),0).unwrap();let r=d.page_frames[c.page].compute_bounds(&ui.document_box).unwrap();(r.x()as f64+c.x as f64*d.zoom as f64,r.y()as f64+(c.top+c.height/2.0)as f64*d.zoom as f64)};
            assert_eq!(context_prepare(&ui,cx,cy),Some(DocumentContext::Text));assert!(ui.current.borrow().as_ref().unwrap().session.sel.is_collapsed());
            table_command(&ui,"insert.table",serde_json::json!({"rows":2,"cols":2}));
            let table=show_document_context(&ui,&parent,100,150,DocumentContext::Table);assert!(table.menu_model().unwrap().item_link(1,"section").unwrap().item_link(0,"submenu").is_some());
            context_activate_row(&table,"Seleccionar");context_activate_row(&table,"Columna");assert!(wordcraft_gnome::tables::selected_cells(&ui.current.borrow().as_ref().unwrap().session)>1);
            let table=show_document_context(&ui,&parent,100,150,DocumentContext::Table);context_activate_row(&table,"Más opciones de tabla…");assert!(ui.table_popover.borrow().as_ref().unwrap().is_visible());ui.table_popover.borrow().as_ref().unwrap().popdown();
            document_tools_fixture(&ui);
            let u=ui.clone();let p=parent.clone();let a=app.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(400),move||{
                let (x,y,rev)={let mut value=u.current.borrow_mut();let d=value.as_mut().unwrap();let hit=d.session.layout().find_object(0,|_|true).unwrap();let r=d.page_frames[hit.page].compute_bounds(&u.document_box).unwrap();let rect=hit.bounds();let point=(r.x()as f64+(rect.x+rect.w/2.0)as f64*d.zoom as f64,r.y()as f64+(rect.y+rect.h/2.0)as f64*d.zoom as f64,d.session.rev());d.session.run("caret.docStart",&serde_json::json!({})).unwrap();point};
                assert_eq!(context_prepare(&u,x,y),Some(DocumentContext::Picture));assert_eq!(u.current.borrow().as_ref().unwrap().session.rev(),rev);
                let pop=show_document_context(&u,&p,x as i32,y as i32,DocumentContext::Picture);
                context_activate_row(&pop,"Tamaño de la imagen…");assert!(u.picture.borrow().as_ref().unwrap().pop.is_visible());u.picture.borrow().as_ref().unwrap().pop.popdown();
                let pop=show_document_context(&u,&p,x as i32,y as i32,DocumentContext::Picture);
                context_activate_row(&pop,"Copiar");
                // A separate document must receive the image fragment, not U+FFFC.
                opened(&u,Session::new(wordcraft_doc::Document::from_text("")),"Destino");context_paste(&u);
                assert!(u.current.borrow_mut().as_mut().unwrap().session.layout().find_object(0,|_|true).is_some());
                apply_user_edit(&u,Edit::Undo);assert!(u.current.borrow_mut().as_mut().unwrap().session.layout().find_object(0,|_|true).is_none());
                // An external clipboard owner invalidates our rich fragment.
                u.document_box.clipboard().set_text("External");context_paste(&u);
                glib::timeout_add_local_once(std::time::Duration::from_millis(400),move||{let mut state=u.current.borrow_mut();let d=state.as_mut().unwrap();assert!(d.session.layout().find_object(0,|_|true).is_none());let v=d.session.run("select.all",&serde_json::json!({"whole":true})).unwrap();drop(v);assert_eq!(selection::copy_text(&mut d.session).unwrap(),"External");eprintln!("CONTEXT_UI_OK");a.quit();});
            });
            });
        });
    });
}


