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
fn show_text_options(ui:&Rc<Ui>,parent:&ApplicationWindow){
    let w=panel_window(parent,"Texto");let state=selection_character(ui);
    let p=panel_page(&w,"Fuente");let g=panel_group(&p,"Formato de caracteres");
    for(label,key,cmd)in[("Negrita","bold","format.bold"),("Cursiva","italic","format.italic"),("Tachado","strike","format.strikethrough"),("Tachado doble","doubleStrike","format.doubleStrikethrough"),("Versalitas","smallCaps","format.smallCaps"),("Mayúsculas tipográficas","caps","format.allCaps"),("Texto oculto","hidden","format.hidden")]{panel_toggle(&g,label,state[key].as_bool().unwrap_or(false),ui,&w,cmd);}
    panel_action(&g,"Subíndice",ui,&w,"format.subscript",serde_json::json!({}));
    panel_action(&g,"Superíndice",ui,&w,"format.superscript",serde_json::json!({}));
    let g=panel_group(&p,"Tamaño y mayúsculas");
    panel_action(&g,"Aumentar tamaño",ui,&w,"format.growFont",serde_json::json!({}));panel_action(&g,"Reducir tamaño",ui,&w,"format.shrinkFont",serde_json::json!({}));
    panel_choice(&g,"Cambiar mayúsculas/minúsculas",&[("Tipo oración","sentence"),("minúsculas","lower"),("MAYÚSCULAS","upper"),("Cada Palabra","title"),("Invertir mayúsculas","toggle")],ui,&w,"format.changeCase","mode");
    panel_action(&g,"Borrar formato de texto",ui,&w,"format.clear",serde_json::json!({}));
    let g=panel_group(&p,"Espaciado de caracteres");
    let spacing=panel_number(&g,"Espaciado (pt)",state["spacing"].as_f64().unwrap_or(0.0),-100.0,100.0);
    let scale=panel_number(&g,"Escala (%)",state["scale"].as_f64().unwrap_or(100.0),1.0,600.0);
    let position=panel_number(&g,"Desplazamiento vertical (pt)",state["position"].as_f64().unwrap_or(0.0),-100.0,100.0);
    let apply=gtk::Button::with_label("Aplicar espaciado");g.add(&apply);
    {let ui=ui.clone();let w=w.clone();apply.connect_clicked(move|_|{
        spacing.update();scale.update();position.update();
        let args=serde_json::json!({"props":{"spacing":spacing.value(),"scale":scale.value(),"position":position.value()}});
        if let Err(e)=format_execute(&ui,"format.set",args){w.add_toast(Toast::new(&e));}
    });}
    let p=panel_page(&w,"Color y efectos");let g=panel_group(&p,"Color");
    panel_color(&g,"Color del texto",ui,&w,"format.color");panel_action(&g,"Color automático",ui,&w,"format.color",serde_json::json!({"color":"auto"}));
    panel_choice(&g,"Resaltado",&[("Sin resaltado","none"),("Amarillo","yellow"),("Verde","brightGreen"),("Turquesa","turquoise"),("Rosa","pink"),("Azul","blue"),("Rojo","red"),("Azul oscuro","darkBlue"),("Verde azulado","teal"),("Verde oscuro","green"),("Violeta","violet"),("Rojo oscuro","darkRed"),("Amarillo oscuro","darkYellow"),("Gris 50 %","gray50"),("Gris 25 %","gray25"),("Negro","black")],ui,&w,"format.highlight","color");
    let g=panel_group(&p,"Subrayado");
    panel_action(&g,"Quitar subrayado",ui,&w,"format.underline",serde_json::json!({"value":false}));
    panel_choice(&g,"Estilo de subrayado",&[("Sencillo","single"),("Doble","double"),("Grueso","thick"),("Punteado","dotted"),("Discontinuo","dash"),("Punto y raya","dotDash"),("Dos puntos y raya","dotDotDash"),("Ondulado","wave"),("Solo palabras","words")],ui,&w,"format.underline","style");
    panel_color(&g,"Color del subrayado",ui,&w,"format.underlineColor");panel_action(&g,"Subrayado de color automático",ui,&w,"format.underlineColor",serde_json::json!({"color":"auto"}));
    let g=panel_group(&p,"Efectos de texto");
    for(label,key,cmd)in[("Contorno","outline","format.outline"),("Sombra","shadow","format.shadow"),("Relieve","emboss","format.emboss"),("Grabado","engrave","format.engrave"),("Borde del texto","border","format.border")]{panel_toggle(&g,label,state[key].as_bool().unwrap_or(false),ui,&w,cmd);}
    let p=panel_page(&w,"Anotaciones");let g=panel_group(&p,"Caracteres encerrados");
    panel_choice(&g,"Forma",&[("Círculo","circle"),("Cuadrado","square"),("Triángulo","triangle"),("Rombo","diamond")],ui,&w,"format.enclose","shape");
    panel_action(&g,"Quitar forma",ui,&w,"format.enclose",serde_json::json!({"shape":null}));
    let g=panel_group(&p,"Guía fonética sobre el texto seleccionado");
    let ruby=panel_entry(&g,"Anotación","");let size=panel_number(&g,"Tamaño (pt)",8.0,1.0,96.0);let offset=panel_number(&g,"Separación (pt)",0.0,-20.0,100.0);
    let b=gtk::Button::with_label("Aplicar guía fonética");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{size.update();offset.update();if let Err(e)=format_execute(&ui,"format.phonetic",serde_json::json!({"ruby":ruby.text().as_str(),"size":size.value(),"offset":offset.value(),"align":"center"})){w.add_toast(Toast::new(&e));}});}
    panel_action(&g,"Quitar guía fonética",ui,&w,"format.phonetic",serde_json::json!({"ruby":null}));
    w.present();
}
fn show_paragraph_options(ui:&Rc<Ui>,parent:&ApplicationWindow){
    let w=panel_window(parent,"Párrafo");let state=selection_paragraph(ui);
    let p=panel_page(&w,"Párrafo");let g=panel_group(&p,"Alineación y dirección");
    panel_choice(&g,"Alineación",&[("Izquierda","left"),("Centro","center"),("Derecha","right"),("Justificada","justify"),("Distribuida","distribute")],ui,&w,"para.align","value");
    panel_action(&g,"De izquierda a derecha",ui,&w,"para.ltr",serde_json::json!({"value":true}));panel_action(&g,"De derecha a izquierda",ui,&w,"para.rtl",serde_json::json!({"value":true}));
    let marks=ui.current.borrow().as_ref().is_some_and(|d|d.session.view.marks);panel_toggle(&g,"Mostrar marcas de formato (¶)",marks,ui,&w,"view.marks");
    let g=panel_group(&p,"Sangrías y separación (puntos)");
    let left=panel_number(&g,"Sangría izquierda",state["indentLeft"].as_f64().unwrap_or(0.0),-1584.0,1584.0);
    let right=panel_number(&g,"Sangría derecha",state["indentRight"].as_f64().unwrap_or(0.0),-1584.0,1584.0);
    let first=panel_number(&g,"Primera línea (negativa: francesa)",state["indentFirst"].as_f64().unwrap_or(0.0),-1584.0,1584.0);
    let b=gtk::Button::with_label("Aplicar sangrías");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{left.update();right.update();first.update();if let Err(e)=format_execute(&ui,"para.indents",serde_json::json!({"left":left.value(),"right":right.value(),"firstLine":first.value()})){w.add_toast(Toast::new(&e));}});}
    let before=panel_number(&g,"Espacio anterior",state["spaceBefore"].as_f64().unwrap_or(0.0),0.0,1584.0);let after=panel_number(&g,"Espacio posterior",state["spaceAfter"].as_f64().unwrap_or(0.0),0.0,1584.0);
    let b=gtk::Button::with_label("Aplicar separación");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{before.update();after.update();if let Err(e)=format_execute(&ui,"para.spacing",serde_json::json!({"before":before.value(),"after":after.value()})){w.add_toast(Toast::new(&e));}});}
    let g=panel_group(&p,"Interlineado");
    for(label,cmd)in[("Sencillo","para.single"),("1,5 líneas","para.oneAndHalf"),("Doble","para.double")]{panel_action(&g,label,ui,&w,cmd,serde_json::json!({}));}
    let line=panel_number(&g,"Interlineado exacto (pt)",14.0,1.0,500.0);let b=gtk::Button::with_label("Aplicar interlineado exacto");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{line.update();if let Err(e)=format_execute(&ui,"para.lineSpacing",serde_json::json!({"exactly":line.value()})){w.add_toast(Toast::new(&e));}});}
    let g=panel_group(&p,"Saltos y conservación");
    for(label,key,cmd)in[("Conservar con el siguiente","keepNext","para.keepNext"),("Conservar líneas juntas","keepLines","para.keepLines"),("Salto de página anterior","pageBreakBefore","para.pageBreakBefore"),("Control de viudas y huérfanas","widowControl","para.widowControl")]{panel_toggle(&g,label,state[key].as_bool().unwrap_or(false),ui,&w,cmd);}
    let p=panel_page(&w,"Listas y orden");let g=panel_group(&p,"Listas");
    panel_choice(&g,"Viñetas",&[("Punto","•"),("Círculo","○"),("Cuadrado","▪"),("Guion","–")],ui,&w,"para.bullets","kind");
    panel_action(&g,"Quitar viñetas",ui,&w,"para.bullets",serde_json::json!({"off":true}));
    panel_choice(&g,"Numeración",&[("1. 2. 3.","numbered"),("1) 2) 3)","numberedParen"),("A. B. C.","upperLetter"),("a. b. c.","lowerLetter"),("i. ii. iii.","lowerRoman")],ui,&w,"para.numbering","kind");
    panel_action(&g,"Quitar numeración",ui,&w,"para.numbering",serde_json::json!({"off":true}));
    panel_choice(&g,"Lista multinivel",&[("1 / 1.1 / 1.1.1","legal"),("Esquema","outline")],ui,&w,"para.multilevel","kind");
    let level=panel_number(&g,"Nivel de lista (1–9)",1.0,1.0,9.0);level.set_digits(0);level.set_increments(1.0,1.0);let b=gtk::Button::with_label("Aplicar nivel");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{level.update();if let Err(e)=format_execute(&ui,"para.listLevel",serde_json::json!({"level":level.value_as_int()-1})){w.add_toast(Toast::new(&e));}});}
    panel_action(&g,"Reiniciar numeración en 1",ui,&w,"para.restartNumbering",serde_json::json!({}));
    let g=panel_group(&p,"Ordenar párrafos seleccionados");panel_action(&g,"Ascendente (A–Z)",ui,&w,"para.sort",serde_json::json!({"descending":false}));panel_action(&g,"Descendente (Z–A)",ui,&w,"para.sort",serde_json::json!({"descending":true}));
    let p=panel_page(&w,"Bordes y tabulaciones");let g=panel_group(&p,"Sombreado");panel_color(&g,"Color de fondo del párrafo",ui,&w,"para.shading");panel_action(&g,"Sin sombreado",ui,&w,"para.shading",serde_json::json!({"color":null}));
    let g=panel_group(&p,"Bordes");
    panel_choice(&g,"Bordes del párrafo",&[("Sin bordes","none"),("Todos","all"),("Exterior","outside"),("Interior","inside"),("Superior","top"),("Inferior","bottom"),("Izquierdo","left"),("Derecho","right"),("Línea horizontal","horizontalLine")],ui,&w,"para.borders","kind");
    let width=panel_number(&g,"Grosor del borde (pt)",0.5,0.25,12.0);let color=panel_entry(&g,"Color del borde (RGB hexadecimal)","000000");let b=gtk::Button::with_label("Aplicar borde exterior personalizado");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{width.update();let hex=color.text();if hex.len()!=6||!hex.chars().all(|c|c.is_ascii_hexdigit()){w.add_toast(Toast::new("Escribe seis cifras hexadecimales, por ejemplo 2266AA"));return;}if let Err(e)=format_execute(&ui,"para.borders",serde_json::json!({"kind":"outside","width":width.value(),"color":hex.as_str()})){w.add_toast(Toast::new(&e));}});}
    let g=panel_group(&p,"Tabulaciones");let pos=panel_number(&g,"Posición (pt)",36.0,0.0,1584.0);let b=gtk::Button::with_label("Añadir tabulación izquierda");g.add(&b);
    {let ui=ui.clone();let w=w.clone();b.connect_clicked(move|_|{pos.update();if let Err(e)=format_execute(&ui,"para.tabs",serde_json::json!({"set":[{"pos":pos.value(),"align":"left"}]})){w.add_toast(Toast::new(&e));}});}
    panel_action(&g,"Borrar tabulaciones",ui,&w,"para.tabs",serde_json::json!({"clearAll":true}));
    panel_action(&g,"Restablecer formato de párrafo",ui,&w,"para.reset",serde_json::json!({}));w.present();
}
