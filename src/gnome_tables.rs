// Native table controls. Document changes remain upstream commands.
fn table_command(ui:&Rc<Ui>,command:&str,args:serde_json::Value) {
    if let Err(e)=format_execute(ui,command,args) {ui.toasts.add_toast(Toast::new(&e));}
    caret_ensure_visible(ui);
    ui.document_box.grab_focus();
    if let Some(menu)=style_widget(&ui.format_toolbar,"table-menu").and_then(|w|w.downcast::<gtk::MenuButton>().ok()){if let Some(pop)=menu.popover(){refresh_table_controls(ui,&pop);}}
}

fn table_navigate(ui:&Rc<Ui>,backwards:bool,literal:bool) {
    let result={let mut state=ui.current.borrow_mut();state.as_mut().map(|d|wordcraft_gnome::tables::navigate(&mut d.session,backwards,literal))};
    match result {Some(Ok(true))=>after_content_change(ui),Some(Err(e))=>ui.toasts.add_toast(Toast::new(&e)),_=>refresh_selection(ui)}
    caret_ensure_visible(ui);
}

fn show_insert_table(ui:&Rc<Ui>,parent:&ApplicationWindow) {
    let w=panel_window(parent,"Insertar tabla");w.set_default_size(440,640);w.set_search_enabled(false);
    let p=panel_page(&w,"Tabla");let g=panel_group(&p,"Tamaño de la tabla");
    let grid=gtk::Grid::new();grid.set_row_spacing(3);grid.set_column_spacing(3);grid.set_halign(gtk::Align::Center);
    let label=gtk::Label::new(Some("Elige columnas × filas"));g.add(&label);
    for row in 1..=5 {for col in 1..=6 {
        let b=gtk::Button::with_label("□");b.set_tooltip_text(Some(&format!("{col} columnas × {row} filas")));accessible_name(&b,&format!("Insertar tabla de {col} columnas y {row} filas"));
        b.set_size_request(30,28);grid.attach(&b,col-1,row-1,1,1);
        let motion=gtk::EventControllerMotion::new();let l=label.clone();motion.connect_enter(move|_,_,_|l.set_text(&format!("{col} columnas × {row} filas")));b.add_controller(motion);
        let ui=ui.clone();let win=w.clone();b.connect_clicked(move|_|{
            if let Err(e)=format_execute(&ui,"insert.table",serde_json::json!({"rows":row,"cols":col})) {win.add_toast(Toast::new(&e));}
            else {win.close();ui.document_box.grab_focus();caret_ensure_visible(&ui);}
        });
    }}g.add(&grid);
    let g=panel_group(&p,"Tamaño personalizado");
    let rows=panel_number(&g,"Filas",3.0,1.0,1000.0);rows.set_digits(0);rows.set_increments(1.0,5.0);
    let cols=panel_number(&g,"Columnas",3.0,1.0,63.0);cols.set_digits(0);cols.set_increments(1.0,5.0);
    let b=gtk::Button::with_label("Insertar tabla");b.add_css_class("suggested-action");g.add(&b);
    let ui=ui.clone();let win=w.clone();b.connect_clicked(move|_|{
        rows.update();cols.update();
        if let Err(e)=format_execute(&ui,"insert.table",serde_json::json!({"rows":rows.value_as_int(),"cols":cols.value_as_int()})){win.add_toast(Toast::new(&e));}
        else {win.close();ui.document_box.grab_focus();caret_ensure_visible(&ui);}
    });w.present();
}

fn table_property_toggle(g:&adw::PreferencesGroup,label:&str,on:bool,key:&'static str,ui:&Rc<Ui>,w:&adw::PreferencesWindow) {
    let row=adw::SwitchRow::new();row.set_title(label);row.set_active(on);g.add(&row);
    let ui=ui.clone();let w=w.clone();row.connect_active_notify(move|r|{
        if let Err(e)=format_execute(&ui,"table.properties",serde_json::json!({key:r.is_active()})){w.add_toast(Toast::new(&e));}
    });
}

fn table_measurement(g:&adw::PreferencesGroup,label:&str,value:f64,min:f64,ui:&Rc<Ui>,w:&adw::PreferencesWindow,command:&'static str,key:&'static str,extra:serde_json::Value) {
    let row=adw::ActionRow::new();row.set_title(label);
    let n=gtk::SpinButton::with_range(min,1584.0,0.5);n.set_digits(1);n.set_value(value);n.set_valign(gtk::Align::Center);
    let b=gtk::Button::with_label("Aplicar");b.set_valign(gtk::Align::Center);accessible_name(&n,label);accessible_name(&b,&format!("Aplicar {label}"));row.set_activatable_widget(Some(&n));row.add_suffix(&n);row.add_suffix(&b);g.add(&row);
    let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{n.update();let mut args=extra.clone();args[key]=serde_json::json!(n.value());
        if let Err(e)=format_execute(&ui,command,args){w.add_toast(Toast::new(&e));}
    });
}

fn show_table_properties(ui:&Rc<Ui>,parent:&ApplicationWindow) {
    let state={let mut state=ui.current.borrow_mut();let Some(doc)=state.as_mut() else{return};
        match doc.session.run("table.properties",&serde_json::json!({})){Ok(v)=>v,Err(e)=>{ui.toasts.add_toast(Toast::new(&e.to_string()));return;}}};
    let w=panel_window(parent,"Propiedades de la tabla");w.set_search_enabled(false);
    let p=panel_page(&w,"Tamaño");let g=panel_group(&p,"Tabla");
    panel_choice(&g,"Alineación de tabla",&[("Izquierda","left"),("Centrada","center"),("Derecha","right")],ui,&w,"table.properties","align");
    panel_choice(&g,"Ajustar ancho",&[("Al contenido","contents"),("Al ancho de página","window"),("Ancho fijo","fixed")],ui,&w,"table.autofit","mode");
    panel_action(&g,"Distribuir columnas uniformemente",ui,&w,"table.distributeColumns",serde_json::json!({}));
    panel_action(&g,"Distribuir filas uniformemente",ui,&w,"table.distributeRows",serde_json::json!({}));
    let g=panel_group(&p,"Columna y fila seleccionadas (puntos)");
    table_measurement(&g,"Ancho de columna",state["columnWidth"].as_f64().unwrap_or(72.0),6.0,ui,&w,"table.columnWidth","width",serde_json::json!({}));
    table_measurement(&g,"Altura mínima de fila",state["row"]["height"].as_f64().unwrap_or(18.0),1.0,ui,&w,"table.rowHeight","height",serde_json::json!({"rule":"atLeast"}));
    panel_action(&g,"Restaurar altura automática de la fila actual",ui,&w,"table.properties",serde_json::json!({"rowHeight":null}));
    let g=panel_group(&p,"Paginación de la fila actual");
    table_property_toggle(&g,"Repetir como cabecera",state["row"]["header"].as_bool().unwrap_or(false),"headerRow",ui,&w);
    table_property_toggle(&g,"Permitir dividir entre páginas",!state["row"]["cantSplit"].as_bool().unwrap_or(false),"allowBreak",ui,&w);
    let p=panel_page(&w,"Celdas");let g=panel_group(&p,"Alineación del contenido");
    panel_choice(&g,"Alineación horizontal y vertical",&[("Arriba · izquierda","topLeft"),("Arriba · centro","topCenter"),("Arriba · derecha","topRight"),("Centro · izquierda","centerLeft"),("Centro","center"),("Centro · derecha","centerRight"),("Abajo · izquierda","bottomLeft"),("Abajo · centro","bottomCenter"),("Abajo · derecha","bottomRight")],ui,&w,"table.cellAlign","value");
    panel_choice(&g,"Dirección del texto",&[("Horizontal","horizontal"),("Vertical hacia abajo","down"),("Vertical hacia arriba","up")],ui,&w,"table.textDirection","value");
    let g=panel_group(&p,"Dividir celdas");let cols=panel_number(&g,"Columnas",2.0,1.0,63.0);cols.set_digits(0);cols.set_increments(1.0,1.0);
    let rows=panel_number(&g,"Filas",1.0,1.0,63.0);rows.set_digits(0);rows.set_increments(1.0,1.0);
    let hint=gtk::Label::new(Some("Varias celdas seleccionadas se combinan antes de dividirse."));hint.set_wrap(true);hint.add_css_class("dim-label");g.add(&hint);
    let b=gtk::Button::with_label("Dividir celdas seleccionadas");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{cols.update();rows.update();if let Err(e)=format_execute(&ui,"table.split",serde_json::json!({"columns":cols.value_as_int(),"rows":rows.value_as_int()})){w.add_toast(Toast::new(&e));}});}
    let p=panel_page(&w,"Aspecto");let g=panel_group(&p,"Bordes y sombreado de la selección");
    panel_color(&g,"Color de fondo",ui,&w,"table.shading");panel_action(&g,"Sin color de fondo",ui,&w,"table.shading",serde_json::json!({"color":null}));
    panel_choice(&g,"Bordes",&[("Todos","all"),("Exteriores","outside"),("Interiores","inside"),("Ninguno","none"),("Superior","top"),("Inferior","bottom"),("Izquierdo","left"),("Derecho","right")],ui,&w,"table.borders","kind");
    let width=panel_number(&g,"Grosor de bordes (pt)",0.5,0.25,12.0);let color=panel_entry(&g,"Color de bordes (RRGGBB)","808080");
    let b=gtk::Button::with_label("Aplicar a todos los bordes");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{width.update();let c=color.text().trim().to_string();if c.len()!=6||!c.bytes().all(|b|b.is_ascii_hexdigit()){w.add_toast(Toast::new("Introduce seis caracteres hexadecimales, por ejemplo 808080"));return;}
        if let Err(e)=format_execute(&ui,"table.borders",serde_json::json!({"kind":"all","width":width.value(),"color":c})){w.add_toast(Toast::new(&e));}});}
    w.present();
}

fn install_table_controls(ui:&Rc<Ui>,parent:&ApplicationWindow,bar:&gtk::Box)->gtk::MenuButton {
    let menu=gtk::MenuButton::new();menu.set_widget_name("table-menu");format_menu_label(&menu,"Tabla");menu.set_direction(gtk::ArrowType::Up);menu.set_tooltip_text(Some("Insertar y editar tablas"));
    let pop=gtk::Popover::new();pop.add_css_class("format-popover");let content=gtk::Box::new(gtk::Orientation::Vertical,6);
    content.set_margin_top(8);content.set_margin_bottom(8);content.set_margin_start(8);content.set_margin_end(8);
    format_popover_header(&pop,&content,"Tabla",&ui.format_group);
    let insert=gtk::Button::with_label("Insertar tabla…");insert.add_css_class("flat");insert.set_visible(tool_diagnostics());content.append(&insert);
    {let ui=ui.clone();let p=parent.clone();let pop=pop.clone();insert.connect_clicked(move|_|{close_format_popover(&pop);show_insert_table(&ui,&p);});}
    let context=gtk::Box::new(gtk::Orientation::Vertical,6);context.set_widget_name("table-context");content.append(&context);
    let heading=gtk::Label::new(Some("Tabla actual"));heading.add_css_class("heading");context.append(&heading);
    for items in [
        vec![("Seleccionar fila","table.selectRow"),("Seleccionar columna","table.selectColumn")],
        vec![("Seleccionar celda","table.selectCell"),("Seleccionar tabla","table.selectTable")],
        vec![("Fila encima","table.insertRowAbove"),("Fila debajo","table.insertRowBelow")],
        vec![("Columna izquierda","table.insertColumnLeft"),("Columna derecha","table.insertColumnRight")],
        vec![("Combinar celdas","table.merge"),("Dividir tabla","table.splitTable")],
        vec![("Eliminar fila","table.deleteRow"),("Eliminar columna","table.deleteColumn")],
    ] {
        let row=gtk::Box::new(gtk::Orientation::Horizontal,4);
        for(label,command)in items {let b=gtk::Button::with_label(label);b.add_css_class("flat");b.set_hexpand(true);row.append(&b);b.set_widget_name(&format!("table-action-{command}"));if command=="table.merge"{b.set_widget_name("table-merge");}
            let ui=ui.clone();let pop=pop.clone();b.connect_clicked(move|_|{close_format_popover(&pop);table_command(&ui,command,serde_json::json!({}));});}
        context.append(&row);
    }
    let properties=gtk::Button::with_label("Propiedades y dividir celdas…");properties.add_css_class("flat");context.append(&properties);
    {let ui=ui.clone();let parent=parent.clone();let pop=pop.clone();properties.connect_clicked(move|_|{close_format_popover(&pop);show_table_properties(&ui,&parent);});}
    let remove=gtk::Button::with_label("Eliminar tabla");remove.set_widget_name("table-delete");remove.add_css_class("destructive-action");context.append(&remove);
    {let ui=ui.clone();let pop=pop.clone();remove.connect_clicked(move|_|{close_format_popover(&pop);table_command(&ui,"table.deleteTable",serde_json::json!({}));});}
    {let ui=ui.clone();pop.connect_show(move|p|refresh_table_controls(&ui,p));}
    pop.set_child(Some(&content));menu.set_popover(Some(&pop));bar.append(&menu);menu
}

fn refresh_table_controls(ui:&Rc<Ui>,pop:&gtk::Popover){
    let Some(root)=pop.child()else{return;};let value=ui.current.borrow();let doc=value.as_ref();
    if let Some(w)=style_widget(&root,"table-context"){w.set_visible(doc.is_some_and(|d|wordcraft_gnome::tables::in_table(&d.session)));}
    if let Some(w)=style_widget(&root,"table-merge"){w.set_sensitive(doc.is_some_and(|d|wordcraft_gnome::tables::selected_cells(&d.session)>1));}
}

