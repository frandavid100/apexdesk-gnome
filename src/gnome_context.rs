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
    if let Some(menu)=style_widget(&ui.format_toolbar,name).and_then(|w|w.downcast::<gtk::MenuButton>().ok()){menu.popup();}
}
fn context_item(menu:&gio::Menu,group:&gio::SimpleActionGroup,pop:&gtk::PopoverMenu,ui:&Rc<Ui>,label:&str,name:&str,enabled:bool,callback:impl Fn(&Rc<Ui>)+'static){
    let action=gio::SimpleAction::new(name,None);action.set_enabled(enabled);
    let weak=Rc::downgrade(ui);let popup=pop.downgrade();
    action.connect_activate(move|_,_|{if let Some(pop)=popup.upgrade(){pop.popdown();}if let Some(ui)=weak.upgrade(){if !ui.closed.get(){callback(&ui);}}});
    group.add_action(&action);menu.append(Some(label),Some(&format!("context.{name}")));
}
fn show_document_context(ui:&Rc<Ui>,parent:&ApplicationWindow,x:i32,y:i32,kind:DocumentContext)->gtk::PopoverMenu{
    // A single native GMenu supplies separators, submenu navigation and states.
    let root=gio::Menu::new();let pop=gtk::PopoverMenu::from_model(Some(&root));pop.set_widget_name("document-context-menu");
    pop.set_parent(&ui.document_box);pop.set_pointing_to(Some(&gdk::Rectangle::new(x,y,1,1)));
    let group=gio::SimpleActionGroup::new();pop.insert_action_group("context",Some(&group));
    let (undo,redo,selected,merge)=ui.current.borrow().as_ref().map(|d|(d.session.can_undo(),d.session.can_redo(),!d.session.sel.is_collapsed(),wordcraft_gnome::tables::selected_cells(&d.session)>1)).unwrap_or_default();
    let history=gio::Menu::new();
    context_item(&history,&group,&pop,ui,"Deshacer","undo",undo,|u|{apply_user_edit(u,Edit::Undo);});
    context_item(&history,&group,&pop,ui,"Rehacer","redo",redo,|u|{apply_user_edit(u,Edit::Redo);});root.append_section(None,&history);
    let edit=gio::Menu::new();
    context_item(&edit,&group,&pop,ui,"Cortar","cut",selected,|u|{if context_copy(u){apply_user_edit(u,Edit::Delete);}});
    context_item(&edit,&group,&pop,ui,"Copiar","copy",selected,|u|{context_copy(u);});
    let can_paste=ui.document_box.clipboard().formats().contain_gtype(String::static_type());
    context_item(&edit,&group,&pop,ui,"Pegar","paste",can_paste,context_paste);
    context_item(&edit,&group,&pop,ui,"Seleccionar todo","select-all",true,|u|selected_command(u,"select-all",false));root.append_section(None,&edit);
    let specific=gio::Menu::new();
    match kind {
        DocumentContext::Text=>{
            for (label,name,widget) in [("Estilo…","style","style-menu"),("Párrafo…","paragraph","paragraph-menu"),("Texto…","text","text-menu")]{context_item(&specific,&group,&pop,ui,label,name,true,move|u|context_open_format(u,widget));}
        },
        DocumentContext::Picture=>{
            context_item(&specific,&group,&pop,ui,"Tamaño de la imagen…","picture-size",true,|u|context_open_format(u,"picture-menu"));
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
            context_item(&specific,&group,&pop,ui,"Combinar celdas","merge",merge,|u|table_command(u,"table.merge",serde_json::json!({})));
            let p=parent.downgrade();context_item(&specific,&group,&pop,ui,"Propiedades y dividir celdas…","table-properties",true,move|u|{if let Some(p)=p.upgrade(){show_table_properties(u,&p);}});
        }
    }
    root.append_section(None,&specific);
    pop.connect_closed(|p|{p.unparent();});pop.popup();pop
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

fn context_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application){
    opened(ui,Session::new(wordcraft_doc::Document::from_text("Hello world, texto de prueba\nSegundo párrafo")),"Contextos");
    {let mut value=ui.current.borrow_mut();let d=value.as_mut().unwrap();d.session.run("select.range",&serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)})).unwrap();}
    refresh_selection(ui);
    let ui=ui.clone();let parent=parent.clone();let app=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400),move||{
        let (x,y,sel,rev)={let mut value=ui.current.borrow_mut();let d=value.as_mut().unwrap();let c=d.session.layout().caret_on(&Pos::body(0,4),0).unwrap();let r=d.page_frames[c.page].compute_bounds(&parent).unwrap();(r.x()as f64+c.x as f64*d.zoom as f64,r.y()as f64+(c.top+c.height/2.0)as f64*d.zoom as f64,d.session.sel.clone(),d.session.rev())};
        eprintln!("CONTEXT_POINTER_READY {} {}",x as i32,y as i32);
        glib::timeout_add_local_once(std::time::Duration::from_secs(2),move||{
            {let value=ui.current.borrow();let d=value.as_ref().unwrap();assert_eq!(d.session.sel,sel,"right click must preserve selected text");assert_eq!(d.session.rev(),rev);}
            let pop=style_widget(&ui.document_box,"document-context-menu").unwrap().downcast::<gtk::PopoverMenu>().unwrap();assert!(pop.is_mapped());assert_eq!(pop.menu_model().unwrap().n_items(),3);
            pop.activate_action("context.copy",None).unwrap();
            {let mut value=ui.current.borrow_mut();selection::set_caret(&mut value.as_mut().unwrap().session,&Pos::body(1,0),false).unwrap();}
            context_paste(&ui);assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());apply_user_edit(&ui,Edit::Undo);
            // Clicking outside the selection moves the caret, without a mutation.
            let (cx,cy)={let mut value=ui.current.borrow_mut();let d=value.as_mut().unwrap();let c=d.session.layout().caret_on(&Pos::body(1,3),0).unwrap();let r=d.page_frames[c.page].compute_bounds(&ui.document_box).unwrap();(r.x()as f64+c.x as f64*d.zoom as f64,r.y()as f64+(c.top+c.height/2.0)as f64*d.zoom as f64)};
            assert_eq!(context_prepare(&ui,cx,cy),Some(DocumentContext::Text));assert!(ui.current.borrow().as_ref().unwrap().session.sel.is_collapsed());
            table_command(&ui,"insert.table",serde_json::json!({"rows":2,"cols":2}));
            let table=show_document_context(&ui,&parent,100,150,DocumentContext::Table);assert!(table.menu_model().unwrap().item_link(2,"section").unwrap().item_link(0,"submenu").is_some());
            table.activate_action("context.select-2",None).unwrap();assert!(wordcraft_gnome::tables::selected_cells(&ui.current.borrow().as_ref().unwrap().session)>1);
            document_tools_fixture(&ui);
            let u=ui.clone();let p=parent.clone();let a=app.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(400),move||{
                let (x,y,rev)={let mut value=u.current.borrow_mut();let d=value.as_mut().unwrap();let hit=d.session.layout().find_object(0,|_|true).unwrap();let r=d.page_frames[hit.page].compute_bounds(&u.document_box).unwrap();let rect=hit.bounds();let point=(r.x()as f64+(rect.x+rect.w/2.0)as f64*d.zoom as f64,r.y()as f64+(rect.y+rect.h/2.0)as f64*d.zoom as f64,d.session.rev());d.session.run("caret.docStart",&serde_json::json!({})).unwrap();point};
                assert_eq!(context_prepare(&u,x,y),Some(DocumentContext::Picture));assert_eq!(u.current.borrow().as_ref().unwrap().session.rev(),rev);
                let pop=show_document_context(&u,&p,x as i32,y as i32,DocumentContext::Picture);
                pop.activate_action("context.copy",None).unwrap();
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
}
