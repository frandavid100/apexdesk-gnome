// Shared controls move between two real toolbar positions; no duplicate actions.
struct AdaptiveTools {ui:std::rc::Weak<Ui>,history:gtk::Box,formats:gtk::Box,top_history:gtk::Box,top_formats:gtk::Box,bottom:gtk::Box,separator:gtk::Separator,narrow:Cell<bool>,generation:Cell<u64>}
impl AdaptiveTools {
    fn sync_width(self:&Rc<Self>,width:i32){
        if width<=0{return;}let narrow=tool_diagnostics()||if self.narrow.get(){width<680}else{width<640};
        let Some(ui)=self.ui.upgrade()else{return;};
        if self.narrow.replace(narrow)==narrow{ui.format_toolbar.set_visible(narrow);return;}
        let generation=self.generation.get()+1;self.generation.set(generation);ui.format_reflowing.set(true);
        let open=ui.format_buttons.borrow().iter().filter_map(|m|m.popover().filter(|p|p.is_visible()).map(|p|{p.popdown();m.clone()})).collect::<Vec<_>>();
        for group in [&self.history,&self.formats]{if let Some(parent)=group.parent().and_then(|p|p.downcast::<gtk::Box>().ok()){parent.remove(group);}}
        if narrow{self.bottom.prepend(&self.history);self.bottom.insert_child_after(&self.formats,Some(&self.separator));}
        else{self.top_history.append(&self.history);self.top_formats.append(&self.formats);}
        ui.format_toolbar.set_visible(narrow);self.separator.set_visible(narrow);
        for m in ui.format_buttons.borrow().iter(){m.set_direction(if narrow{gtk::ArrowType::Up}else{gtk::ArrowType::Down});}
        let weak=Rc::downgrade(self);glib::timeout_add_local_once(std::time::Duration::from_millis(40),move||{if let Some(a)=weak.upgrade(){if a.generation.get()!=generation{return;}if let Some(ui)=a.ui.upgrade(){for menu in open{menu.popup();}ui.format_reflowing.set(false);}}});
    }
}
fn history_icon(redo:bool)->gtk::DrawingArea{let a=gtk::DrawingArea::new();a.set_content_width(18);a.set_content_height(18);a.set_draw_func(move|w,c,_,_|{let f=w.color();c.set_source_rgba(f.red()as f64,f.green()as f64,f.blue()as f64,f.alpha()as f64);if redo{c.translate(18.0,0.0);c.scale(-1.0,1.0);}c.set_line_width(1.6);c.set_line_cap(gtk::cairo::LineCap::Round);c.set_line_join(gtk::cairo::LineJoin::Round);c.move_to(6.0,3.0);c.line_to(2.0,7.0);c.line_to(6.0,11.0);c.move_to(2.0,7.0);c.line_to(11.0,7.0);c.curve_to(17.0,7.0,17.0,15.0,11.0,15.0);let _=c.stroke();});a}
fn format_icon_button(menu:&gtk::MenuButton,label:&str,kind:&str){
    let icon=gtk::DrawingArea::new();icon.set_content_width(18);icon.set_content_height(18);let kind=kind.to_string();
    icon.set_draw_func(move|w,c,_,_|{let f=w.color();c.set_source_rgba(f.red()as f64,f.green()as f64,f.blue()as f64,f.alpha()as f64);c.set_font_size(17.0);c.move_to(2.0,15.0);let _=c.show_text(match kind.as_str(){"style"=>"T","paragraph"=>"¶",_=>"A"});c.set_line_width(1.3);if kind=="style"{for y in[4.0,8.0,12.0]{c.move_to(12.0,y);c.line_to(17.0,y);}}else if kind=="text"{c.move_to(2.0,17.0);c.line_to(15.0,17.0);}let _=c.stroke();});
    menu.set_child(Some(&icon));menu.add_css_class("flat");menu.add_css_class("image-button");menu.set_tooltip_text(Some(label));menu.update_property(&[gtk::accessible::Property::Label(label)]);
}
fn format_button(ui:&Ui,name:&str)->Option<gtk::MenuButton>{ui.format_buttons.borrow().iter().find(|m|m.widget_name()==name).cloned()}

struct TextTools {ui:std::rc::Weak<Ui>,popover:glib::WeakRef<gtk::Popover>,stack:gtk::Stack,detail:gtk::Box,header:Rc<FormatHeader>}
impl TextTools {
    fn home(self:&Rc<Self>){self.header.home();self.stack.set_visible_child_name("main");}
    fn execute(&self,cmd:&str,args:serde_json::Value){if let Some(ui)=self.ui.upgrade(){match format_execute(&ui,cmd,args){Ok(())=>{if let Some(p)=self.popover.upgrade(){close_format_popover(&p);}ui.document_box.grab_focus();caret_ensure_visible(&ui);},Err(e)=>ui.toasts.add_toast(Toast::new(&e))}}}
    fn action(self:&Rc<Self>,label:&str,cmd:&'static str,args:serde_json::Value){let b=style_menu_button(label);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.execute(cmd,args.clone());}});self.detail.append(&b);}
    fn heading(&self,label:&str){let l=gtk::Label::new(Some(label));l.set_xalign(0.0);l.add_css_class("heading");l.set_margin_top(8);self.detail.append(&l);}
    fn choice(self:&Rc<Self>,title:&str,options:&[(&str,&str)],cmd:&'static str,key:&str){self.heading(title);for(label,value)in options{self.action(label,cmd,serde_json::json!({key:*value}));}}
    fn color(self:&Rc<Self>,title:&str,cmd:&'static str){self.heading(title);
        let choose=style_menu_button("Elegir color…");self.detail.append(&choose);let weak=Rc::downgrade(self);choose.connect_clicked(move |_|{let Some(s)=weak.upgrade()else{return;};let Some(ui)=s.ui.upgrade()else{return;};let Some(host)=ui.host.upgrade()else{return;};let identity=document_identity(&ui);if let Some(p)=s.popover.upgrade(){close_format_popover(&p);}let dialog=gtk::ColorDialog::new();dialog.set_title("Elegir color");dialog.set_with_alpha(false);let weak=Rc::downgrade(&s);dialog.choose_rgba(Some(&host.window),None::<&gdk::RGBA>,None::<&gio::Cancellable>,move|result|{let Some(s)=weak.upgrade()else{return;};let Some(ui)=s.ui.upgrade()else{return;};if ui.closed.get()||document_identity(&ui)!=identity{return;}match result{Ok(c)=>{let color=format!("{:02X}{:02X}{:02X}",(c.red()*255.0).round()as u8,(c.green()*255.0).round()as u8,(c.blue()*255.0).round()as u8);s.execute(cmd,serde_json::json!({"color":color}));},Err(e)if e.matches(gtk::DialogError::Cancelled)||e.matches(gtk::DialogError::Dismissed)=>{},Err(e)=>ui.toasts.add_toast(Toast::new(&e.to_string()))}});});let e=gtk::Entry::new();e.set_placeholder_text(Some("RGB: 2266AA"));self.detail.append(&e);let b=style_menu_button("Aplicar color");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){let c=e.text().trim().trim_start_matches('#').to_string();if c.len()==6&&c.bytes().all(|b|b.is_ascii_hexdigit()){s.execute(cmd,serde_json::json!({"color":c}));}else if let Some(ui)=s.ui.upgrade(){ui.toasts.add_toast(Toast::new("Introduce seis cifras hexadecimales"));}}});}
    fn toggle(self:&Rc<Self>,label:&str,on:bool,cmd:&'static str){let b=gtk::CheckButton::with_label(label);b.set_active(on);self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_toggled(move|b|{if let Some(s)=weak.upgrade(){s.execute(cmd,serde_json::json!({"value":b.is_active()}));}});}
    fn number(&self,label:&str,value:f64,min:f64,max:f64)->gtk::SpinButton{let row=gtk::Box::new(gtk::Orientation::Horizontal,6);let l=gtk::Label::new(Some(label));l.set_xalign(0.0);l.set_hexpand(true);l.set_wrap(true);row.append(&l);let n=gtk::SpinButton::with_range(min,max,0.5);n.set_digits(1);n.set_value(value);row.append(&n);self.detail.append(&row);n}
    fn page(self:&Rc<Self>,page:&str){
        clear_box(&self.detail);let Some(ui)=self.ui.upgrade()else{return;};let state=selection_character(&ui);let weak=Rc::downgrade(self);let home=page=="options";
        self.header.back(if home{"Texto"}else{"Opciones de texto"},move||{if let Some(s)=weak.upgrade(){if home{s.home();}else{s.page("options");}}});
        match page{
            "options"=>{for(label,page)in[("Fuente y caracteres","character"),("Color y efectos","color"),("Anotaciones","annotations")]{let b=style_menu_button(label);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.page(page);}});self.detail.append(&b);}},
            "character"=>{
                self.heading("Formato de caracteres");for(label,key,cmd)in[("Negrita","bold","format.bold"),("Cursiva","italic","format.italic"),("Tachado","strike","format.strikethrough"),("Tachado doble","doubleStrike","format.doubleStrikethrough"),("Versalitas","smallCaps","format.smallCaps"),("Mayúsculas tipográficas","caps","format.allCaps"),("Texto oculto","hidden","format.hidden")]{self.toggle(label,state[key].as_bool().unwrap_or(false),cmd);}
                for(label,cmd)in[("Subíndice","format.subscript"),("Superíndice","format.superscript"),("Aumentar tamaño","format.growFont"),("Reducir tamaño","format.shrinkFont"),("Borrar formato de texto","format.clear")]{self.action(label,cmd,serde_json::json!({}));}
                self.choice("Mayúsculas/minúsculas",&[("Tipo oración","sentence"),("minúsculas","lower"),("MAYÚSCULAS","upper"),("Cada Palabra","title"),("Invertir","toggle")],"format.changeCase","mode");
                self.heading("Espaciado de caracteres");let spacing=self.number("Espaciado (pt)",state["spacing"].as_f64().unwrap_or(0.0),-100.0,100.0);let scale=self.number("Escala (%)",state["scale"].as_f64().unwrap_or(100.0),1.0,600.0);let position=self.number("Desplazamiento (pt)",state["position"].as_f64().unwrap_or(0.0),-100.0,100.0);
                let b=style_menu_button("Aplicar espaciado");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){spacing.update();scale.update();position.update();s.execute("format.set",serde_json::json!({"props":{"spacing":spacing.value(),"scale":scale.value(),"position":position.value()}}));}});
            },
            "color"=>{
                self.color("Color del texto","format.color");self.action("Color automático","format.color",serde_json::json!({"color":"auto"}));
                self.choice("Resaltado",&[("Sin resaltado","none"),("Amarillo","yellow"),("Verde","brightGreen"),("Turquesa","turquoise"),("Rosa","pink"),("Azul","blue"),("Rojo","red"),("Azul oscuro","darkBlue"),("Verde azulado","teal"),("Verde oscuro","green"),("Violeta","violet"),("Rojo oscuro","darkRed"),("Amarillo oscuro","darkYellow"),("Gris 50 %","gray50"),("Gris 25 %","gray25"),("Negro","black")],"format.highlight","color");
                self.action("Quitar subrayado","format.underline",serde_json::json!({"value":false}));self.choice("Subrayado",&[("Sencillo","single"),("Doble","double"),("Grueso","thick"),("Punteado","dotted"),("Discontinuo","dash"),("Punto y raya","dotDash"),("Dos puntos y raya","dotDotDash"),("Ondulado","wave"),("Solo palabras","words")],"format.underline","style");self.color("Color del subrayado","format.underlineColor");self.action("Color automático del subrayado","format.underlineColor",serde_json::json!({"color":"auto"}));
                self.heading("Efectos");for(label,key,cmd)in[("Contorno","outline","format.outline"),("Sombra","shadow","format.shadow"),("Relieve","emboss","format.emboss"),("Grabado","engrave","format.engrave"),("Borde del texto","border","format.border")]{self.toggle(label,state[key].as_bool().unwrap_or(false),cmd);}
            },
            _=>{
                self.choice("Caracteres encerrados",&[("Círculo","circle"),("Cuadrado","square"),("Triángulo","triangle"),("Rombo","diamond")],"format.enclose","shape");self.action("Quitar forma","format.enclose",serde_json::json!({"shape":null}));
                self.heading("Guía fonética");let ruby=gtk::Entry::new();ruby.set_placeholder_text(Some("Anotación"));self.detail.append(&ruby);let size=self.number("Tamaño (pt)",8.0,1.0,96.0);let offset=self.number("Separación (pt)",0.0,-20.0,100.0);let b=style_menu_button("Aplicar guía fonética");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){size.update();offset.update();s.execute("format.phonetic",serde_json::json!({"ruby":ruby.text().as_str(),"size":size.value(),"offset":offset.value(),"align":"center"}));}});self.action("Quitar guía fonética","format.phonetic",serde_json::json!({"ruby":null}));
            }
        }self.stack.set_visible_child_name("detail");
    }
}
fn install_text_pages(ui:&Rc<Ui>,pop:&gtk::Popover,root:&gtk::Box,_inline:&gtk::Box,header:&Rc<FormatHeader>,more:&gtk::Button)->Rc<TextTools>{
    let stack=gtk::Stack::new();stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);stack.set_interpolate_size(true);stack.set_vhomogeneous(false);stack.set_hhomogeneous(false);
    let main=gtk::Box::new(gtk::Orientation::Vertical,8);let mut child=root.first_child().and_then(|w|w.next_sibling()).and_then(|w|w.next_sibling());while let Some(w)=child{child=w.next_sibling();root.remove(&w);main.append(&w);}
    let detail=gtk::Box::new(gtk::Orientation::Vertical,6);let scroll=gtk::ScrolledWindow::new();scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_max_content_height(420);scroll.set_propagate_natural_height(true);scroll.set_child(Some(&detail));stack.add_named(&main,Some("main"));stack.add_named(&scroll,Some("detail"));root.append(&stack);root.set_width_request(330);
    let s=Rc::new(TextTools{ui:Rc::downgrade(ui),popover:pop.downgrade(),stack,detail,header:header.clone()});let weak=Rc::downgrade(&s);more.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.page("options");}});let state=s.clone();pop.connect_show(move |_|{if !state.ui.upgrade().is_some_and(|u|u.format_reflowing.get()){state.home();}});s.home();s
}

fn adaptive_ui_test(ui:&Rc<Ui>,window:&ApplicationWindow,app:&Application){
    let a=ui.adaptive.borrow().as_ref().unwrap().clone();assert!(!a.narrow.get());assert!(!ui.format_toolbar.is_visible());
    let buttons=ui.format_buttons.borrow().clone();assert_eq!(a.history.parent().as_ref(),Some(a.top_history.upcast_ref()));assert_eq!(a.formats.parent().as_ref(),Some(a.top_formats.upcast_ref()));
    let style=ui.style.borrow().as_ref().unwrap().clone();buttons[0].popup();style.show_detail("Heading1");popover_pin(&buttons[0].popover().unwrap()).set_active(true);
    let rev=ui.current.borrow().as_ref().unwrap().session.rev();let selection=ui.current.borrow().as_ref().unwrap().session.sel.clone();window.set_default_size(520,800);
    let ui=ui.clone();let window=window.clone();let app=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(600),move||{
        assert!(a.narrow.get());assert!(ui.format_toolbar.is_visible());assert_eq!(a.formats.parent().as_ref(),Some(a.bottom.upcast_ref()));assert_eq!(*ui.format_buttons.borrow(),buttons);assert!(buttons[0].popover().unwrap().is_mapped());assert_eq!(style.stack.visible_child_name().as_deref(),Some("detail"));assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev);assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel,selection);
        eprintln!("ADAPTIVE_NARROW_OK");window.set_default_size(1200,850);
        glib::timeout_add_local_once(std::time::Duration::from_millis(600),move||{
            assert!(!a.narrow.get());assert!(!ui.format_toolbar.is_visible());assert!(buttons[0].popover().unwrap().is_mapped());assert_eq!(style.stack.visible_child_name().as_deref(),Some("detail"));buttons[0].popdown();
            eprintln!("ADAPTIVE_WIDE_OK");let origin=buttons[0].popover().unwrap();let body=origin.child().unwrap();let context=show_document_context(&ui,&window,100,150,DocumentContext::Text);context_activate_row(&context,"Estilo");assert_eq!(context.visible_submenu().as_deref(),Some("Estilo"));assert!(origin.child().is_none());assert!(!origin.is_visible());assert!(style.stack.interpolates_size());assert!(native_menu_stack(&context).unwrap().interpolates_size());
            style_click(&style,"style-apply-Normal");assert!(context.is_visible(),"pinned context must stay open");style.main();style.header.button.emit_clicked();assert_eq!(context.visible_submenu().as_deref(),Some("main"));buttons[1].popup();assert!(!context.is_visible());assert_eq!(origin.child().as_ref(),Some(&body));assert!(buttons[1].popover().unwrap().is_visible());buttons[1].popdown();popover_pin(&origin).set_active(false);
            let context=show_document_context(&ui,&window,100,150,DocumentContext::Text);context_activate_row(&context,"Texto");let text=ui.text_tools.borrow().as_ref().unwrap().clone();text.page("options");text.page("character");text.header.button.emit_clicked();assert_eq!(text.stack.visible_child_name().as_deref(),Some("detail"));text.header.button.emit_clicked();assert_eq!(text.stack.visible_child_name().as_deref(),Some("main"));text.header.button.emit_clicked();assert_eq!(context.visible_submenu().as_deref(),Some("main"));assert_eq!(app.windows().len(),1);context.popdown();
            let settings=ui.document_box.settings();let old=settings.property::<bool>("gtk-enable-animations");settings.set_property("gtk-enable-animations",false);buttons[2].popup();text.page("options");assert!(!text.stack.is_transition_running());assert!(text.stack.interpolates_size());buttons[2].popdown();settings.set_property("gtk-enable-animations",old);
            eprintln!("ADAPTIVE_UI_OK");app.quit();
        });
    });
}
