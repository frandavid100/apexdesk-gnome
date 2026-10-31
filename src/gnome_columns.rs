// Staged section layout; only the public engine command changes the document.
use wordcraft_gnome::document_properties::{ColumnSetup,ColumnScope,ColumnKind};
struct ColumnControls {
    mode:adw::ComboRow,count:gtk::SpinButton,gap:gtk::SpinButton,separator:gtk::Switch,
    custom:adw::PreferencesGroup,widths:Vec<(gtk::SpinButton,gtk::SpinButton)>,rows:Vec<adw::ActionRow>,
    format_note:gtk::Label,scope:adw::ComboRow,scopes:Vec<ColumnScope>,preview:gtk::DrawingArea,text_width:f32,last_count:Cell<u32>,last_mode:Cell<u32>,
}
impl ColumnControls {
    fn setup(&self)->ColumnSetup {
        let kind=match self.mode.selected(){1=>ColumnKind::Left,2=>ColumnKind::Right,3=>ColumnKind::Custom,_=>ColumnKind::Equal};
        let count=if matches!(kind,ColumnKind::Left|ColumnKind::Right){2}else{self.count.value_as_int()as u32};
        ColumnSetup{count,kind,space:self.gap.value()as f32*PT_PER_MM as f32,separator:self.separator.is_active(),widths:self.widths.iter().take(count as usize).map(|(w,g)|(w.value()as f32*PT_PER_MM as f32,g.value()as f32*PT_PER_MM as f32)).collect()}
    }
    fn refresh(&self){
        let preset=matches!(self.mode.selected(),1|2);if preset{self.count.set_value(2.0);}
        let custom=self.mode.selected()==3;let count=self.count.value_as_int()as usize;
        let changed_count=self.last_count.replace(count as u32)!=count as u32;let changed_mode=self.last_mode.replace(self.mode.selected())!=self.mode.selected();
        if custom&&(changed_count||changed_mode){
            let gap=self.gap.value()*PT_PER_MM;let width=((self.text_width as f64-gap*(count.saturating_sub(1))as f64)/count.max(1)as f64).max(18.0);
            for(w,g)in self.widths.iter().take(count){w.set_value(width/PT_PER_MM);g.set_value(gap/PT_PER_MM);}
        }
        self.count.set_sensitive(!preset);self.gap.set_sensitive(count>1&&!custom);self.separator.set_sensitive(count>1);self.custom.set_visible(custom);self.format_note.set_visible(self.mode.selected()!=0);
        for(i,row)in self.rows.iter().enumerate(){row.set_visible(i<count);self.widths[i].1.set_sensitive(i+1<count);}
        self.preview.queue_draw();
    }
}
struct ColumnsDialog {window:adw::Window,controls:Rc<ColumnControls>,apply:gtk::Button}
fn show_columns(ui:&Rc<Ui>,parent:&ApplicationWindow)->ColumnsDialog {
    let (initial,text_width,can_select,can_forward)={let state=ui.current.borrow();let s=&state.as_ref().unwrap().session;
        (ColumnSetup::current(s),wordcraft_engine::cmd::page::sect(s).text_width(),wordcraft_engine::cmd::page::selection_sectionable(s),s.sel.focus.story==wordcraft_doc::StoryRef::Body&&s.sel.focus.path.depth()==0)};
    let identity=document_identity(ui);
    let window=adw::Window::builder().title("Columnas").transient_for(parent).modal(true).default_width(480).default_height(620).build();
    let toolbar=ToolbarView::new();let header=HeaderBar::new();header.set_show_end_title_buttons(false);header.set_show_start_title_buttons(false);
    let cancel=gtk::Button::with_label("Cancelar");header.pack_start(&cancel);let apply=gtk::Button::with_label("Aplicar");apply.add_css_class("suggested-action");apply.set_widget_name("columns-apply");header.pack_end(&apply);toolbar.add_top_bar(&header);
    let page=adw::PreferencesPage::new();let group=adw::PreferencesGroup::new();group.set_title("Distribución");group.set_description(Some("El texto fluye de una columna a la siguiente."));page.add(&group);
    let preview=gtk::DrawingArea::new();preview.set_content_width(320);preview.set_content_height(112);preview.set_halign(gtk::Align::Fill);group.add(&preview);
    let mode=adw::ComboRow::new();mode.set_title("Anchuras");mode.set_model(Some(&gtk::StringList::new(&["Iguales","Izquierda más estrecha","Derecha más estrecha","Personalizadas"])));mode.set_selected(if initial.kind==ColumnKind::Custom{3}else{0});group.add(&mode);
    let count=gtk::SpinButton::with_range(1.0,12.0,1.0);count.set_value(initial.count.clamp(1,12)as f64);count.set_valign(gtk::Align::Center);count.set_width_chars(3);count.set_widget_name("columns-count");let row=adw::ActionRow::new();row.set_title("Número de columnas");row.add_suffix(&count);group.add(&row);
    let gap=properties_number(&group,"Separación (mm)",initial.space);gap.set_range(0.0,300.0/PT_PER_MM);gap.set_widget_name("columns-gap");
    let separator=gtk::Switch::new();separator.set_active(initial.separator);separator.set_valign(gtk::Align::Center);let row=adw::ActionRow::new();row.set_title("Línea entre columnas");row.set_activatable_widget(Some(&separator));row.add_suffix(&separator);group.add(&row);
    let format_note=gtk::Label::new(Some("Las anchuras desiguales se conservan en DOCX. En ODT, el motor las guarda como columnas iguales."));format_note.set_wrap(true);format_note.set_xalign(0.0);format_note.add_css_class("dim-label");format_note.set_margin_start(12);format_note.set_margin_end(12);group.add(&format_note);
    let custom=adw::PreferencesGroup::new();custom.set_title("Anchuras personalizadas");custom.set_description(Some("Anchura y separación posterior, en milímetros. La última columna no tiene separación posterior."));page.add(&custom);
    let mut widths=Vec::new();let mut rows=Vec::new();
    for i in 0..12 {
        let (w,g)=initial.widths.get(i).copied().unwrap_or(((text_width-initial.space*(initial.count.saturating_sub(1))as f32)/initial.count.max(1)as f32,initial.space));
        let row=adw::ActionRow::new();row.set_title(&format!("Columna {}",i+1));let width=mm_input(w as f64/PT_PER_MM,18.0/PT_PER_MM,1584.0/PT_PER_MM);width.set_width_chars(5);width.set_tooltip_text(Some("Anchura (mm)"));width.update_property(&[gtk::accessible::Property::Label(&format!("Anchura de columna {} (mm)",i+1))]);
        let after=mm_input(g as f64/PT_PER_MM,0.0,300.0/PT_PER_MM);after.set_width_chars(5);after.set_tooltip_text(Some("Separación posterior (mm)"));after.update_property(&[gtk::accessible::Property::Label(&format!("Separación tras columna {} (mm)",i+1))]);row.add_suffix(&width);row.add_suffix(&after);custom.add(&row);rows.push(row);widths.push((width,after));
    }
    let scopes_group=adw::PreferencesGroup::new();page.add(&scopes_group);let scope=adw::ComboRow::new();scope.set_title("Aplicar a");let mut labels=vec!["Sección actual","Todo el documento"];let mut scopes=vec![ColumnScope::Section,ColumnScope::Document];if can_select{labels.push("Texto seleccionado");scopes.push(ColumnScope::Selection);}if can_forward{labels.push("Desde aquí en adelante");scopes.push(ColumnScope::Forward);}scope.set_model(Some(&gtk::StringList::new(&labels)));if can_select{scope.set_selected(2);}scopes_group.add(&scope);
    let hint=adw::ActionRow::new();hint.set_title("Texto seleccionado");hint.set_subtitle("Crea secciones continuas alrededor del texto, sin cambiar el resto del documento.");hint.set_visible(can_select);scopes_group.add(&hint);
    let error=gtk::Label::new(None);error.set_wrap(true);error.add_css_class("error");error.set_visible(false);let errors=adw::PreferencesGroup::new();errors.add(&error);page.add(&errors);
    toolbar.set_content(Some(&page));window.set_content(Some(&toolbar));
    let controls=Rc::new(ColumnControls{mode,count,gap,separator,custom,widths,rows,format_note,scope,scopes,preview,text_width,last_count:Cell::new(initial.count.clamp(1,12)),last_mode:Cell::new(if initial.kind==ColumnKind::Custom{3}else{0})});
    {let weak=Rc::downgrade(&controls);controls.mode.connect_selected_notify(move |_|{if let Some(c)=weak.upgrade(){c.refresh();}});}
    {let weak=Rc::downgrade(&controls);controls.count.connect_value_changed(move |_|{if let Some(c)=weak.upgrade(){c.refresh();}});}
    for n in std::iter::once(&controls.gap).chain(controls.widths.iter().flat_map(|(w,g)|[w,g])){let weak=Rc::downgrade(&controls);n.connect_value_changed(move |_|{if let Some(c)=weak.upgrade(){c.preview.queue_draw();}});}
    {let weak=Rc::downgrade(&controls);controls.separator.connect_active_notify(move |_|{if let Some(c)=weak.upgrade(){c.preview.queue_draw();}});}
    {let weak=Rc::downgrade(&controls);controls.preview.set_draw_func(move|area,cr,w,h|{
        let Some(c)=weak.upgrade()else{return;};let Ok(cols)=c.setup().resolved(c.text_width)else{return;};let color=area.color();let margin=14.0;let total=c.text_width as f64;let scale=(w as f64-2.0*margin)/total;let y=12.0;let height=h as f64-24.0;
        cr.set_source_rgba(color.red()as f64,color.green()as f64,color.blue()as f64,0.15);cr.rectangle(margin,y,w as f64-2.0*margin,height);let _=cr.stroke();
        let mut x=margin;for i in 0..cols.count as usize{let (cw,g)=cols.widths.get(i).copied().unwrap_or(((c.text_width-cols.space*(cols.count-1)as f32)/cols.count as f32,cols.space));let width=cw as f64*scale;
            cr.set_source_rgba(color.red()as f64,color.green()as f64,color.blue()as f64,0.6);for line in 0..8{let width=if line==7{width*0.64}else{width};cr.rectangle(x+3.0,y+8.0+line as f64*8.0,(width-6.0).max(1.0),2.0);}let _=cr.fill();
            if cols.separator&&i+1<cols.count as usize{cr.move_to(x+width+g as f64*scale/2.0,y+6.0);cr.line_to(x+width+g as f64*scale/2.0,y+height-6.0);let _=cr.stroke();}x+=width+g as f64*scale;
        }
    });}
    {let weak=window.downgrade();cancel.connect_clicked(move |_|{if let Some(w)=weak.upgrade(){w.close();}});}
    {let c=controls.clone();window.connect_destroy(move |_|{let _=&c;});}
    {let weak=Rc::downgrade(&controls);let ui=Rc::downgrade(ui);let window=window.downgrade();apply.connect_clicked(move |_|{
        let(Some(c),Some(ui))=(weak.upgrade(),ui.upgrade())else{return;};
        if ui.closed.get()||document_identity(&ui)!=identity{error.set_text("El documento ha cambiado. Cierra y vuelve a abrir Columnas.");error.set_visible(true);return;}
        c.count.update();c.gap.update();for(w,g)in &c.widths{w.update();g.update();}
        let setup=c.setup();let scope=c.scopes[c.scope.selected()as usize];let result={let mut d=ui.current.borrow_mut();document_properties::apply_columns(&mut d.as_mut().unwrap().session,&setup,scope)};
        match result{Ok(changed)=>{if changed{after_content_change(&ui);}if let Some(w)=window.upgrade(){w.close();}},Err(e)=>{error.set_text(&e);error.set_visible(true);}}
    });}
    controls.refresh();window.present();ColumnsDialog{window,controls,apply}
}
fn columns_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application){
    let session=Session::new(wordcraft_doc::Document::from_text("Antes\nTexto para distribuir entre columnas\nDespués"));opened(ui,session,"Prueba de columnas");
    {let mut d=ui.current.borrow_mut();d.as_mut().unwrap().session.run("select.range",&serde_json::json!({"anchor":Pos::body(1,0),"focus":Pos::body(2,0)})).unwrap();}refresh_selection(ui);
    let (doc,depth,rev,text)={let d=ui.current.borrow();let s=&d.as_ref().unwrap().session;(s.doc.clone(),s.undo_depth(),s.rev(),s.selected_text())};
    let dialog=show_columns(ui,parent);assert_eq!(dialog.controls.scopes[dialog.controls.scope.selected()as usize],ColumnScope::Selection);dialog.controls.count.set_value(2.0);dialog.controls.gap.set_value(7.0);dialog.controls.separator.set_active(true);assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev);dialog.apply.emit_clicked();assert!(!dialog.window.is_visible());
    {let d=ui.current.borrow();let s=&d.as_ref().unwrap().session;assert_eq!(s.selected_text(),text);assert_eq!(s.undo_depth(),depth+1);let counts:Vec<_>=s.doc.sections().iter().map(|(_,p)|p.columns.count).collect();assert_eq!(counts,vec![1,2,1]);}
    apply_user_edit(ui,Edit::Undo);assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,doc);apply_user_edit(ui,Edit::Redo);
    let rev=ui.current.borrow().as_ref().unwrap().session.rev();let dialog=show_columns(ui,parent);dialog.controls.count.set_value(3.0);dialog.window.close();assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev);
    for menu in ui.format_buttons.borrow().iter(){if let Some(name)=menu.icon_name(){assert!(gtk::IconTheme::for_display(&parent.display()).has_icon(&name));}}
    eprintln!("COLUMNS_UI_OK");app.quit();
}
