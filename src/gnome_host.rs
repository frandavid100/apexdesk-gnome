// Explicit window owners; signal handlers only borrow their Workspace weakly.
thread_local! {static WORKSPACES:RefCell<Vec<Rc<Workspace>>>=const{RefCell::new(Vec::new())};}
fn shutdown_workspaces(){
    let owners=WORKSPACES.with(|v|std::mem::take(&mut *v.borrow_mut()));
    for host in owners {for(_,ui)in host.docs.borrow().iter(){release_document(ui);}host.allow_close.set(true);host.window.destroy();}
}
fn forget_workspace(host:&Rc<Workspace>){WORKSPACES.with(|v|v.borrow_mut().retain(|h|!Rc::ptr_eq(h,host)));}
fn release_document(ui:&Rc<Ui>) {
    if ui.closed.replace(true){return;}
    if let Some(source)=ui.autosave_source.borrow_mut().take(){source.remove();}
    ui.printing.cancel();
    // The bar lives inside a tab: detach its strong TabView reference before
    // closing a whole window, where individual pages are not detached first.
    ui.tab_bar.set_view(None::<&adw::TabView>);
    if let Some(controller)=ui.outside_controller.borrow_mut().take(){if let Some(widget)=controller.widget(){widget.remove_controller(&controller);}}
    let current=ui.current.borrow_mut().take();
    if let Some(mut doc)=current {doc.session.finish_recovery();}
    // A popup closed before mapping may never emit closed. Restore borrowed
    // format bodies explicitly on tab teardown, then detach transient surfaces.
    let contexts=std::mem::take(&mut *ui.context_popovers.borrow_mut());
    for(weak,lease)in contexts{if let Some(lease)=lease{lease.restore();}if let Some(pop)=weak.upgrade(){pop.popdown();if pop.parent().is_some(){pop.unparent();}}}
    for menu in ui.format_buttons.borrow().iter(){if let Some(pop)=menu.popover(){pop.popdown();}menu.set_popover(None::<&gtk::Popover>);}
    if let Some(pop)=ui.table_popover.borrow_mut().take(){pop.popdown();if pop.parent().is_some(){pop.unparent();}}
    if let Some(panel)=ui.picture.borrow_mut().take(){panel.pop.popdown();if panel.pop.parent().is_some(){panel.pop.unparent();}}
    ui.style.borrow_mut().take();ui.paragraph.borrow_mut().take();ui.text_tools.borrow_mut().take();
    clear_canvas(&ui.document_box);clear_box(&ui.thumbnails_box);
    ui.tab_page.borrow_mut().take();
}

fn cancel_tab_close(ui:&Rc<Ui>) {
    if let Some((view,page))=ui.pending_tab_close.borrow_mut().take(){view.close_page_finish(&page,false);}
}

// GNOME host lifecycle. WordCraft remains an unmodified, pinned dependency.
struct Workspace {
    window: ApplicationWindow,
    tabs: adw::TabView,
    overview: adw::TabOverview,
    docs: RefCell<Vec<(adw::TabPage, Rc<Ui>)>>,
    allow_close: Cell<bool>,
}

fn close_workspace(host: &Rc<Workspace>) {
    let pending = host.docs.borrow().iter().find(|(_, ui)| ui.dirty.get() && !ui.closing_confirmed.get()).map(|(_,ui)| ui.clone());
    if let Some(ui) = pending {
        let host2 = host.clone(); let next = ui.clone();
        ask_discard(&ui, &host.window, move || { next.closing_confirmed.set(true); close_workspace(&host2); });
    } else {
        for (_, ui) in host.docs.borrow().iter() {
            release_document(ui);
        }
        host.allow_close.set(true);
        let window=host.window.clone();glib::idle_add_local_once(move ||window.close());
    }
}

fn make_window(app: &Application, sample: bool, sidebar: bool, narrow: bool, dark: bool,
    select_demo: bool, edit_demo: bool, format_demo: bool,
    initial: Option<(Session,String)>) -> ApplicationWindow {
    let window = ApplicationWindow::builder().application(app).title("Leaf")
        .default_width(if narrow {640} else {1140}).default_height(if narrow {600} else {840}).build();
    let tabs = adw::TabView::new();
    // Preserve the editor's Ctrl+Home/End and Ctrl+PageUp/Down behavior.
    tabs.remove_shortcuts(adw::TabViewShortcuts::CONTROL_HOME | adw::TabViewShortcuts::CONTROL_END);
    let overview=adw::TabOverview::new();overview.set_view(Some(&tabs));overview.set_child(Some(&tabs));overview.set_enable_new_tab(false);
    window.set_content(Some(&overview));
    let host = Rc::new(Workspace {window:window.clone(),tabs,overview,docs:RefCell::new(Vec::new()),allow_close:Cell::new(false)});
    WORKSPACES.with(|v|v.borrow_mut().push(host.clone()));
    lifecycle_track(&host);
    {let weak=Rc::downgrade(&host);window.connect_destroy(move |_|{if let Some(host)=weak.upgrade(){for(_,ui)in host.docs.borrow().iter(){release_document(ui);}forget_workspace(&host);}});}
    setup_main_actions(&host,app);
    {
        let host2 = host.clone();
        host.tabs.connect_close_page(glib::clone!(#[weak] host2, #[upgrade_or] glib::Propagation::Proceed, move |view, page| {
            let ui = host2.docs.borrow().iter().find(|(p,_)| p == page).map(|(_,ui)|ui.clone());
            if let Some(ui) = ui {
                let view = view.clone(); let page = page.clone(); let next = ui.clone();
                ui.pending_tab_close.replace(Some((view.clone(),page.clone())));
                ask_discard(&ui, &host2.window, move || {
                    release_document(&next);
                    next.pending_tab_close.borrow_mut().take();
                    view.close_page_finish(&page,true);
                });
                glib::Propagation::Stop
            } else { glib::Propagation::Proceed }
        }));
    }
    {
        let host2 = host.clone();
        host.tabs.connect_page_detached(glib::clone!(#[weak] host2, move |view,page,_| {
            host2.docs.borrow_mut().retain(|(p,_)|p != page);
            if view.n_pages() == 0 {host2.allow_close.set(true);host2.window.close();}
        }));
    }
    {
        let host2=host.clone();
        window.connect_close_request(glib::clone!(#[weak] host2, #[upgrade_or] glib::Propagation::Proceed, move |_| {
            if host2.allow_close.get() {forget_workspace(&host2);glib::Propagation::Proceed}
            else {
                for (_,ui) in host2.docs.borrow().iter(){ui.closing_confirmed.set(false);}
                close_workspace(&host2);glib::Propagation::Stop
            }
        }));
    }
    build_tab(app,&host,sample,sidebar,narrow,dark,select_demo,edit_demo,format_demo,initial);
    if std::env::args().any(|a|a=="--tabs-test") {
        blank_tab(app,&host);
        let host2=host.clone();let app=app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1000),move || {
            if host2.tabs.n_pages()!=2 {std::process::exit(1);}
            let page=host2.tabs.nth_page(1);host2.tabs.close_page(&page);
            if host2.tabs.n_pages()!=1 {std::process::exit(1);}
            eprintln!("TABS_OK");app.quit();
        });
    }
    lifecycle_start(&host,app);
    window
}

fn blank_tab(app:&Application, host:&Rc<Workspace>) {
    build_tab(app,host,false,true,false,false,false,false,false,
        Some((Session::new(wordcraft_doc::Document::new()),"Documento nuevo".into())));
}

fn show_preferences(parent:&ApplicationWindow) {
    let dialog=adw::PreferencesWindow::new();
    dialog.set_transient_for(Some(parent));dialog.set_modal(true);dialog.set_title(Some("Preferencias"));
    let page=adw::PreferencesPage::new();let group=adw::PreferencesGroup::new();
    group.set_title("Autoguardado");
    group.set_description(Some("Recuperación: copia separada cada 10 segundos. Archivo: guarda cada 30 segundos; sin nombre usa recuperación hasta el primer guardado."));
    let row=adw::ComboRow::new();row.set_title("Guardar automáticamente");
    row.set_model(Some(&gtk::StringList::new(&["Desactivado","Archivo de recuperación","En el propio archivo"])));
    row.set_selected(wordcraft_gnome::preferences::autosave_mode());
    let dialog2=dialog.clone();
    row.connect_selected_notify(glib::clone!(#[weak] dialog2, move |row| {
        if let Err(e)=wordcraft_gnome::preferences::set_autosave_mode(row.selected()) {
            dialog2.add_toast(Toast::new(&format!("No se guardaron las preferencias: {e}")));
        }
    }));
    group.add(&row);page.add(&group);dialog.add(&page);dialog.present();
}

fn install_autosave(ui:&Rc<Ui>) {
    let weak=Rc::downgrade(ui);let ticks=Rc::new(Cell::new(0u32));
    let source=glib::timeout_add_local(std::time::Duration::from_secs(1),move || {
        let Some(ui)=weak.upgrade() else{return glib::ControlFlow::Break};
        if ui.closed.get() {return glib::ControlFlow::Break;}
        let mode=wordcraft_gnome::preferences::autosave_mode();
        let mut reloaded=false;let mut saved=false;let mut error=None;
        {
            let mut state=ui.current.borrow_mut();
            if let Some(doc)=state.as_mut() {
                let path=ui.save_path.borrow().clone().or_else(||doc.session.path.clone());
                if mode==1 || (mode==2 && path.is_none()) || ui.restoring.get() || !doc.session.recovery_ready() {
                    // Never tick a dirty document in disabled mode: recovery_tick schedules writes.
                    if mode!=0 || ui.restoring.get() || !ui.dirty.get() {
                        let id=doc.session.document_id();doc.session.recovery_tick();
                        reloaded=id!=doc.session.document_id();
                        if reloaded {ui.restoring.set(false);}
                    }
                }
                ticks.set(ticks.get()+1);
                if mode==2 && ui.dirty.get() && ticks.get()%30==0 {
                    if let Some(path)=path {
                        match wordcraft_gnome::saving::save(&mut doc.session,&path) {
                            Ok(())=>{ui.save_path.replace(Some(path));doc.session.finish_recovery();saved=true;},
                            Err(e)=>error=Some(e),
                        }
                    }
                }
                if let Some(e)=doc.session.recovery_error() {error=Some(e.to_string());}
            }
        }
        if reloaded {after_content_change(&ui);ui.document_name.replace("Documento recuperado".into());sync_tab(&ui);ui.source_path.replace(None);ui.save_path.replace(None);}
        if saved {ui.source_path.replace(None);ui.dirty.set(false);sync_tab(&ui);}
        if let Some(e)=error {
            if ui.autosave_error.borrow().as_deref()!=Some(&e) {
                ui.toasts.add_toast(Toast::new(&format!("Autoguardado: {e}")));ui.autosave_error.replace(Some(e));
            }
        } else {ui.autosave_error.replace(None);}
        glib::ControlFlow::Continue
    });
    ui.autosave_source.replace(Some(source));
}

fn sync_tab(ui:&Rc<Ui>) {
    let text=format!("{}{}",if ui.dirty.get(){"● "}else{""},ui.document_name.borrow().as_str());
    ui.title.set_title(&text);ui.title.set_subtitle("");
    if let Some(page)=ui.tab_page.borrow().as_ref(){page.set_title(&text);}
}

fn show_recovery(ui:&Rc<Ui>, parent:&ApplicationWindow, app:&Application, host:&Rc<Workspace>) {
    let records=ui.current.borrow().as_ref().map(|d|d.session.recovery_documents()).unwrap_or_default();
    let dialog=adw::Window::builder().transient_for(parent).modal(true).title("Recuperar documentos")
        .default_width(420).default_height(240).build();
    let toolbar=ToolbarView::new();toolbar.add_top_bar(&HeaderBar::new());
    let list=gtk::Box::new(gtk::Orientation::Vertical,8);list.set_margin_top(16);list.set_margin_bottom(16);list.set_margin_start(16);list.set_margin_end(16);
    if records.is_empty(){list.append(&gtk::Label::new(Some("No hay documentos pendientes de recuperar.")));}
    for record in records {
        let button=gtk::Button::with_label(&record.name);let app=app.clone();let host=host.clone();let dialog=dialog.clone();
        button.connect_clicked(glib::clone!(#[weak] app, #[weak] host, #[weak] dialog, move |_| {
            blank_tab(&app,&host);
            if let Some((_,ui))=host.docs.borrow().last() {
                let mut state=ui.current.borrow_mut();if let Some(doc)=state.as_mut() {
                    doc.session.recovery_tick();
                    let id=record.id.clone();let ui2=Rc::downgrade(ui);
                    
                    let attempts=Cell::new(0u32);
                    glib::timeout_add_local(std::time::Duration::from_millis(100),move || {
                        let Some(ui2)=ui2.upgrade()else{return glib::ControlFlow::Break;};
                        if ui2.closed.get(){return glib::ControlFlow::Break;}
                        let mut state=ui2.current.borrow_mut();
                        let Some(doc)=state.as_mut() else{return glib::ControlFlow::Break;};
                        doc.session.recovery_tick();
                        if !doc.session.recovery_ready() {
                            attempts.set(attempts.get()+1);
                            if attempts.get()<200{return glib::ControlFlow::Continue;}
                            ui2.toasts.add_toast(Toast::new("No se pudo cargar la lista de recuperación"));
                            return glib::ControlFlow::Break;
                        }
                        match doc.session.run("file.recover",&serde_json::json!({"restore":id})) {
                            Ok(_)=>ui2.restoring.set(true),Err(e)=>ui2.toasts.add_toast(Toast::new(&e.to_string())),
                        }
                        glib::ControlFlow::Break
                    });
                }
            }
            dialog.close();
        }));list.append(&button);
    }
    toolbar.set_content(Some(&list));dialog.set_content(Some(&toolbar));dialog.present();
}
