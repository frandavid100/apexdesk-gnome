// Native GMenu/PopoverMenu rows: normal labels, right-hand accelerators,
// separators and disabled history actions, as in Files.
fn selected_ui(host:&Workspace)->Option<Rc<Ui>>{
    let page=host.tabs.selected_page()?;host.docs.borrow().iter().find(|(p,_)|*p==page).map(|(_,u)|u.clone())
}
fn refresh_main_actions(host:&Workspace){
    let ui=selected_ui(host);let (undo,redo)=ui.as_ref().and_then(|u|u.current.borrow().as_ref().map(|d|(d.session.can_undo(),d.session.can_redo()))).unwrap_or((false,false));
    for(name,on)in[("undo",undo),("redo",redo),("tab-overview",host.tabs.n_pages()>0)]{
        if let Some(a)=host.window.lookup_action(name).and_then(|a|a.downcast::<gio::SimpleAction>().ok()){a.set_enabled(on);}
    }
}
fn window_action(host:&Rc<Workspace>,name:&str,callback:impl Fn(&Rc<Workspace>)+'static){
    let action=gio::SimpleAction::new(name,None);let weak=Rc::downgrade(host);
    action.connect_activate(move|_,_|{if let Some(h)=weak.upgrade(){callback(&h);}});host.window.add_action(&action);
}
fn host_app(host:&Workspace)->Option<Application>{host.window.application()?.downcast::<Application>().ok()}
fn setup_main_actions(host:&Rc<Workspace>,app:&Application){
    window_action(host,"new-tab",|h|{if let Some(app)=host_app(h){blank_tab(&app,h);}});
    window_action(host,"new-window",|h|{if let Some(app)=host_app(h){make_window(&app,false,false,false,false,false,false,false,None);}});
    window_action(host,"tab-overview",|h|h.overview.set_open(true));
    window_action(host,"open",|h|{if let Some(ui)=selected_ui(h){ui.open_button.emit_clicked();}});
    window_action(host,"recover",|h|{if let(Some(ui),Some(app))=(selected_ui(h),host_app(h)){show_recovery(&ui,&h.window,&app,h);}});
    window_action(host,"undo",|h|{if let Some(ui)=selected_ui(h){apply_user_edit(&ui,Edit::Undo);ui.document_box.grab_focus();}});
    window_action(host,"redo",|h|{if let Some(ui)=selected_ui(h){apply_user_edit(&ui,Edit::Redo);ui.document_box.grab_focus();}});
    window_action(host,"save",|h|{if let Some(ui)=selected_ui(h){save_document(&ui,&h.window,false,Box::new(||{}));}});
    window_action(host,"save-as",|h|{if let Some(ui)=selected_ui(h){save_document(&ui,&h.window,true,Box::new(||{}));}});
    window_action(host,"preferences",|h|show_preferences(&h.window));
    window_action(host,"shortcuts",|h|show_keyboard_shortcuts(&h.window));
    window_action(host,"help",|h|show_editor_help(&h.window));
    window_action(host,"about",|h|{
        let w=adw::AboutWindow::new();w.set_transient_for(Some(&h.window));w.set_application_name("WordCraft GNOME");w.set_version("0.1");w.set_developer_name("Proyecto WordCraft GNOME");w.set_license_type(gtk::License::Gpl30);
        w.set_website("https://github.com/frandavid100/apexdesk-gnome");w.set_issue_url("https://github.com/frandavid100/apexdesk-gnome/issues");w.set_comments("Editor de documentos para GNOME basado en el motor WordCraft.");w.present();
    });
    for(name,keys)in[("new-tab",vec!["<Control>n","<Control>t"]),("new-window",vec!["<Control><Shift>n"]),("tab-overview",vec!["<Control><Shift>o"]),("open",vec!["<Control>o"]),("preferences",vec!["<Control>comma"]),("shortcuts",vec!["<Control>question"]),("help",vec!["F1"]) ]{app.set_accels_for_action(&format!("win.{name}"),&keys);}
    let weak=Rc::downgrade(host);host.tabs.connect_selected_page_notify(move |_|{if let Some(h)=weak.upgrade(){refresh_main_actions(&h);}});
}
fn menu_item(menu:&gio::Menu,label:&str,action:&str,accel:Option<&str>){
    let item=gio::MenuItem::new(Some(label),Some(&format!("win.{action}")));
    // Display history shortcuts without stealing Ctrl+Z from native text fields.
    if let Some(accel)=accel{item.set_attribute_value("accel",Some(&accel.to_variant()));}menu.append_item(&item);
}
fn install_main_menu(button:&gtk::MenuButton){
    let root=gio::Menu::new();let files=gio::Menu::new();
    menu_item(&files,"Ventana nueva","new-window",Some("<Control><Shift>n"));menu_item(&files,"Pestaña nueva","new-tab",Some("<Control>n"));menu_item(&files,"Mostrar todas las pestañas","tab-overview",Some("<Control><Shift>o"));menu_item(&files,"Abrir…","open",Some("<Control>o"));
    menu_item(&files,"Recuperar documentos…","recover",None);root.append_section(None,&files);
    let help=gio::Menu::new();menu_item(&help,"Preferencias","preferences",Some("<Control>comma"));menu_item(&help,"Atajos del teclado","shortcuts",Some("<Control>question"));menu_item(&help,"Ayuda","help",Some("F1"));menu_item(&help,"Acerca de WordCraft GNOME","about",None);root.append_section(None,&help);button.set_menu_model(Some(&root));
}
fn install_document_menu(button:&gtk::MenuButton){
    let root=gio::Menu::new();
    let history=gio::Menu::new();menu_item(&history,"Deshacer","undo",Some("<Control>z"));menu_item(&history,"Rehacer","redo",Some("<Control><Shift>z"));root.append_section(None,&history);
    let save=gio::Menu::new();menu_item(&save,"Guardar","save",Some("<Control>s"));menu_item(&save,"Guardar como…","save-as",Some("<Control><Shift>s"));root.append_section(None,&save);button.set_menu_model(Some(&root));
}
fn show_editor_help(parent:&ApplicationWindow){
    let w=adw::Window::builder().transient_for(parent).title("Ayuda de WordCraft GNOME").default_width(520).default_height(520).build();let toolbar=ToolbarView::new();toolbar.add_top_bar(&HeaderBar::new());
    let text="DOCUMENTOS\nPestaña nueva en el menú principal crea un documento vacío. El menú principal permite abrir documentos DOCX/ODT, ventanas nuevas y mostrar todas las pestañas. Guardar: Ctrl+S; Guardar como: Ctrl+Mayús+S.\n\nNAVEGACIÓN\nLos botones inferiores del panel izquierdo alternan Páginas y Títulos. Pulsa una miniatura o un título para ir a esa parte del documento. La lupa abre la búsqueda; Escape vuelve a la vista anterior.\n\nBUSCAR Y REEMPLAZAR\nCtrl+F abre la búsqueda. Sus tres puntos permiten distinguir mayúsculas y activar Reemplazar. El botón junto al segundo campo reemplaza todas las coincidencias en un solo paso de deshacer; cada resultado conserva su reemplazo individual.\n\nEDICIÓN\nLa barra inferior contiene Estilo, Párrafo, Texto y Tabla. Las flechas abren subopciones. Ctrl+Z deshace y Ctrl+Mayús+Z rehace.\n\nAUTOGUARDADO\nPreferencias permite desactivar el autoguardado, usar un archivo de recuperación o guardar en el propio archivo. Recuperar documentos está en el menú principal.";
    let l=gtk::Label::new(Some(text));l.set_wrap(true);l.set_selectable(true);l.set_xalign(0.0);l.set_margin_start(24);l.set_margin_end(24);l.set_margin_top(18);l.set_margin_bottom(24);
    let scroll=gtk::ScrolledWindow::new();scroll.set_child(Some(&l));toolbar.set_content(Some(&scroll));w.set_content(Some(&toolbar));w.present();
}
fn show_keyboard_shortcuts(parent:&ApplicationWindow){
    let groups=[("Documentos",vec![("Pestaña nueva","<Control>n"),("Ventana nueva","<Control><Shift>n"),("Mostrar todas las pestañas","<Control><Shift>o"),("Abrir","<Control>o"),("Guardar","<Control>s"),("Guardar como","<Control><Shift>s"),("Cerrar pestaña","<Control>w")]),("Edición y búsqueda",vec![("Deshacer","<Control>z"),("Rehacer","<Control><Shift>z"),("Buscar","<Control>f"),("Reemplazar","<Control>h"),("Negrita","<Control>b"),("Cursiva","<Control>i"),("Subrayado","<Control>u")]),("Interfaz",vec![("Mostrar u ocultar el panel lateral","F9"),("Preferencias","<Control>comma"),("Ayuda","F1")])];
    let mut xml=String::from("<interface><object class=\"GtkShortcutsWindow\" id=\"shortcuts\"><child><object class=\"GtkShortcutsSection\"><property name=\"section-name\">editor</property><property name=\"title\">WordCraft GNOME</property>");
    for(title,shortcuts)in groups{xml.push_str(&format!("<child><object class=\"GtkShortcutsGroup\"><property name=\"title\">{}</property>",glib::markup_escape_text(title)));for(title,key)in shortcuts{xml.push_str(&format!("<child><object class=\"GtkShortcutsShortcut\"><property name=\"title\">{}</property><property name=\"accelerator\">{}</property></object></child>",glib::markup_escape_text(title),glib::markup_escape_text(key)));}xml.push_str("</object></child>");}
    xml.push_str("</object></child></object></interface>");let builder=gtk::Builder::from_string(&xml);let w=builder.object::<gtk::ShortcutsWindow>("shortcuts").unwrap();w.set_transient_for(Some(parent));w.present();
}

