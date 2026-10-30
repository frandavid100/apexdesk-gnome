// Native formatting panels. All document mutations use upstream public commands.
fn format_execute(ui:&Rc<Ui>, command:&str, args:serde_json::Value)->Result<(),String>{
    let changed={let mut value=ui.current.borrow_mut();let doc=value.as_mut().ok_or("No hay documento")?;
        let rev=doc.session.rev();doc.session.run(command,&args).map_err(|e|e.to_string())?;doc.session.rev()!=rev};
    if changed {after_content_change(ui);} else if command=="view.marks" {
        let dirty=ui.dirty.get();let subtitle=ui.title.subtitle().to_string();
        after_content_change(ui);ui.dirty.set(dirty);ui.title.set_subtitle(&subtitle);sync_tab(ui);
    } else {refresh_selection(ui);}
    Ok(())
}
fn panel_window(parent:&ApplicationWindow,title:&str)->adw::PreferencesWindow{
    let w=adw::PreferencesWindow::new();w.set_transient_for(Some(parent));w.set_modal(true);
    w.set_title(Some(title));w.set_default_size(620,620);w
}
fn panel_page(w:&adw::PreferencesWindow,title:&str)->adw::PreferencesPage{
    let p=adw::PreferencesPage::new();p.set_title(title);w.add(&p);p
}
fn panel_group(p:&adw::PreferencesPage,title:&str)->adw::PreferencesGroup{
    let g=adw::PreferencesGroup::new();g.set_title(title);p.add(&g);g
}
fn panel_action(g:&adw::PreferencesGroup,label:&str,ui:&Rc<Ui>,w:&adw::PreferencesWindow,command:&'static str,args:serde_json::Value){
    let row=adw::ActionRow::new();row.set_title(label);let b=gtk::Button::with_label("Aplicar");b.set_valign(gtk::Align::Center);row.add_suffix(&b);g.add(&row);
    let ui=ui.clone();let w=w.clone();b.connect_clicked(move |_|{if let Err(e)=format_execute(&ui,command,args.clone()){w.add_toast(Toast::new(&e));}});
}
fn panel_toggle(g:&adw::PreferencesGroup,label:&str,active:bool,ui:&Rc<Ui>,w:&adw::PreferencesWindow,command:&'static str){
    let row=adw::SwitchRow::new();row.set_title(label);row.set_active(active);g.add(&row);
    let ui=ui.clone();let w=w.clone();row.connect_active_notify(move|r|{
        if let Err(e)=format_execute(&ui,command,serde_json::json!({"value":r.is_active()})){w.add_toast(Toast::new(&e));}
    });
}
fn panel_number(g:&adw::PreferencesGroup,label:&str,value:f64,min:f64,max:f64)->gtk::SpinButton{
    let row=adw::ActionRow::new();row.set_title(label);let n=gtk::SpinButton::with_range(min,max,0.5);n.set_digits(1);n.set_value(value);n.set_valign(gtk::Align::Center);row.add_suffix(&n);g.add(&row);n
}
fn panel_entry(g:&adw::PreferencesGroup,label:&str,text:&str)->gtk::Entry{
    let row=adw::ActionRow::new();row.set_title(label);let e=gtk::Entry::new();e.set_text(text);e.set_valign(gtk::Align::Center);row.add_suffix(&e);g.add(&row);e
}
fn panel_choice(g:&adw::PreferencesGroup,label:&str,options:&[(&str,&str)],ui:&Rc<Ui>,w:&adw::PreferencesWindow,command:&'static str,key:&'static str){
    let row=adw::ComboRow::new();row.set_title(label);
    let labels:Vec<&str>=std::iter::once("Elegir…").chain(options.iter().map(|o|o.0)).collect();row.set_model(Some(&gtk::StringList::new(&labels)));row.set_selected(0);g.add(&row);
    let values:Vec<String>=options.iter().map(|x|x.1.to_string()).collect();let ui=ui.clone();let w=w.clone();
    row.connect_selected_notify(move|r|{if let Some(v)=r.selected().checked_sub(1).and_then(|i|values.get(i as usize)){
        let args=serde_json::json!({key:v});if let Err(e)=format_execute(&ui,command,args){w.add_toast(Toast::new(&e));}
    }});
}
fn panel_color(g:&adw::PreferencesGroup,label:&str,ui:&Rc<Ui>,w:&adw::PreferencesWindow,command:&'static str){
    let row=adw::ActionRow::new();row.set_title(label);let b=gtk::Button::with_label("Elegir…");b.set_valign(gtk::Align::Center);row.add_suffix(&b);g.add(&row);
    let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{
        let dialog=gtk::ColorDialog::new();dialog.set_title("Elegir color");dialog.set_with_alpha(false);
        let ui=ui.clone();let w2=w.clone();
        dialog.choose_rgba(Some(&w),None::<&gdk::RGBA>,None::<&gio::Cancellable>,move|result|{
            match result{
                Ok(c)=>{let hex=format!("{:02X}{:02X}{:02X}",(c.red()*255.0).round() as u8,(c.green()*255.0).round() as u8,(c.blue()*255.0).round() as u8);
                    if let Err(e)=format_execute(&ui,command,serde_json::json!({"color":hex})){w2.add_toast(Toast::new(&e));}},
                Err(e) if e.matches(gtk::DialogError::Dismissed)||e.matches(gtk::DialogError::Cancelled)=>{},
                Err(e)=>w2.add_toast(Toast::new(&e.to_string())),
            }
        });
    });
}
fn selection_character(ui:&Rc<Ui>)->serde_json::Value{
    let value=ui.current.borrow();let Some(doc)=value.as_ref() else{return serde_json::json!({});};
    let style=doc.session.doc.para_at(&doc.session.sel.focus).and_then(|p|p.props.style.as_deref());
    serde_json::to_value(doc.session.doc.styles.resolve_char(style,&doc.session.caret_props())).unwrap_or_default()
}
fn selection_paragraph(ui:&Rc<Ui>)->serde_json::Value{
    let value=ui.current.borrow();let Some(doc)=value.as_ref() else{return serde_json::json!({});};
    doc.session.doc.para_at(&doc.session.sel.focus).and_then(|p|serde_json::to_value(doc.session.doc.styles.resolve_para(&p.props)).ok()).unwrap_or_default()
}
fn show_text_options(ui:&Rc<Ui>,_parent:&ApplicationWindow){if let Some(s)=ui.text_tools.borrow().as_ref(){if let Some(p)=s.popover.upgrade(){p.popup();}s.page("options");}}
