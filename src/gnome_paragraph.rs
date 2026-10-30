// Paragraph controls share the styles popover's internal-page navigation.
struct ParagraphPopover {
    ui: std::rc::Weak<Ui>,
    popover: glib::WeakRef<gtk::Popover>,
    stack: gtk::Stack,
    main: gtk::Box,
    detail: gtk::Box,
    message: gtk::Label,
    spacing: Cell<f64>,
    shade: RefCell<String>,
    border: RefCell<serde_json::Value>,
}
fn paragraph_icon(name:&str)->gtk::Widget {
    let area=gtk::DrawingArea::new();area.set_content_width(18);area.set_content_height(18);
    let kind=name.to_string();
    area.set_draw_func(move|w,cr,_,_|{
        let c=w.color();cr.set_source_rgba(c.red() as f64,c.green() as f64,c.blue() as f64,c.alpha() as f64);cr.set_line_width(1.4);
        match kind.as_str(){
            "format-justify-left-symbolic"|"format-justify-center-symbolic"|"format-justify-right-symbolic"|"format-justify-fill-symbolic"=>{
                for (i,width) in [14.0,10.0,14.0,8.0].into_iter().enumerate(){
                    let width=if kind.contains("fill"){14.0}else{width};
                    let x=if kind.contains("center"){(18.0-width)/2.0}else if kind.contains("right"){16.0-width}else{2.0};
                    cr.move_to(x,3.0+i as f64*4.0);cr.line_to(x+width,3.0+i as f64*4.0);
                }let _=cr.stroke();
            },
            "view-list-bullet-symbolic"|"view-list-ordered-symbolic"=>{
                for i in 0..3{let y=4.0+i as f64*5.0;cr.move_to(7.0,y);cr.line_to(17.0,y);let _=cr.stroke();
                    if kind.contains("bullet"){cr.arc(2.0,y,1.2,0.0,std::f64::consts::TAU);let _=cr.fill();}
                    else{cr.set_font_size(6.5);cr.move_to(0.0,y+2.0);let _=cr.show_text(&(i+1).to_string());}
                }
            },
            "format-indent-less-symbolic"|"format-indent-more-symbolic"=>{
                for y in [2.0,16.0]{cr.move_to(1.0,y);cr.line_to(17.0,y);}for y in [6.0,10.0]{cr.move_to(8.0,y);cr.line_to(17.0,y);}
                if kind.contains("less"){cr.move_to(5.0,5.0);cr.line_to(1.0,9.0);cr.line_to(5.0,13.0);}else{cr.move_to(1.0,5.0);cr.line_to(5.0,9.0);cr.line_to(1.0,13.0);}let _=cr.stroke();
            },
            "replace"=>{cr.move_to(1.0,5.0);cr.line_to(16.0,5.0);cr.line_to(12.0,1.0);cr.move_to(16.0,5.0);cr.line_to(12.0,9.0);cr.move_to(17.0,13.0);cr.line_to(2.0,13.0);cr.line_to(6.0,9.0);cr.move_to(2.0,13.0);cr.line_to(6.0,17.0);let _=cr.stroke();},
            "spacing"=>{for y in [4.0,9.0,14.0]{cr.move_to(8.0,y);cr.line_to(17.0,y);}cr.move_to(3.0,2.0);cr.line_to(3.0,16.0);cr.move_to(0.5,5.0);cr.line_to(3.0,2.0);cr.line_to(5.5,5.0);cr.move_to(0.5,13.0);cr.line_to(3.0,16.0);cr.line_to(5.5,13.0);let _=cr.stroke();},
            "shading"=>{cr.rectangle(2.0,3.0,14.0,12.0);let _=cr.stroke();cr.rectangle(4.0,8.0,10.0,5.0);let _=cr.fill();},
            _=>{cr.set_dash(&[2.0,2.0],0.0);cr.rectangle(2.0,2.0,14.0,14.0);let _=cr.stroke();cr.set_dash(&[],0.0);cr.move_to(2.0,16.0);cr.line_to(16.0,16.0);let _=cr.stroke();}
        }
    });area.upcast()
}
impl ParagraphPopover {
    fn execute(self:&Rc<Self>,command:&str,args:serde_json::Value){
        if let Some(ui)=self.ui.upgrade(){match format_execute(&ui,command,args){Ok(())=>{self.message.set_visible(false);if self.stack.visible_child_name().as_deref()==Some("main"){self.home();}},Err(e)=>{self.message.set_text(&e);self.message.set_visible(true);}}}
    }
    fn action(self:&Rc<Self>,label:&str,name:&str,command:&'static str,args:serde_json::Value)->gtk::Button{
        let b=style_menu_button(label);b.set_widget_name(name);let weak=Rc::downgrade(self);
        b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.execute(command,args.clone());}});b
    }
    fn icon_action(self:&Rc<Self>,icon:&str,label:&str,name:&str,active:bool,command:&'static str,args:serde_json::Value)->gtk::ToggleButton{
        let b=gtk::ToggleButton::new();b.set_child(Some(&paragraph_icon(icon)));b.set_tooltip_text(Some(label));b.set_widget_name(name);b.set_active(active);b.set_hexpand(true);b.add_css_class("flat");
        let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.execute(command,args.clone());}});b
    }
    fn split(self:&Rc<Self>,row:&gtk::Box,button:&impl IsA<gtk::Widget>,page:&'static str,label:&str){
        let group=gtk::Box::new(gtk::Orientation::Horizontal,0);group.add_css_class("linked");group.set_hexpand(true);group.append(button);
        let more=gtk::Button::from_icon_name("go-next-symbolic");more.set_tooltip_text(Some(&format!("Opciones de {label}")));more.set_widget_name(&format!("paragraph-more-{page}"));more.add_css_class("flat");group.append(&more);row.append(&group);
        let weak=Rc::downgrade(self);more.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.page(page);}});
    }
    fn home(self:&Rc<Self>){
        clear_box(&self.main);let Some(ui)=self.ui.upgrade()else{return;};let state=selection_paragraph(&ui);
        let row=gtk::Box::new(gtk::Orientation::Horizontal,4);
        for(icon,label,value)in[("format-justify-left-symbolic","Alinear a la izquierda","left"),("format-justify-center-symbolic","Centrar","center"),("format-justify-right-symbolic","Alinear a la derecha","right"),("format-justify-fill-symbolic","Justificar","justify")]{
            row.append(&self.icon_action(icon,label,&format!("paragraph-align-{value}"),state["align"].as_str().unwrap_or("left")==value,"para.align",serde_json::json!({"value":value})));
        }self.main.append(&row);self.main.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let row=gtk::Box::new(gtk::Orientation::Horizontal,4);
        let list_kind={let value=ui.current.borrow();value.as_ref().and_then(|d|{
            let p=d.session.doc.para_at(&d.session.sel.focus)?;let props=d.session.doc.styles.resolve_para(&p.props);let n=props.numbering?;
            d.session.doc.numbering.level(n.num,n.level).map(|l|format!("{:?}",l.format))
        })};
        let bullet=list_kind.as_ref().is_some_and(|s|s.to_lowercase().contains("bullet"));let numbered=list_kind.is_some()&&!bullet;
        let b=self.icon_action("view-list-bullet-symbolic","Lista con viñetas","paragraph-bullets",bullet,"para.bullets",serde_json::json!({}));self.split(&row,&b,"bullets","viñetas");
        let b=self.icon_action("view-list-ordered-symbolic","Lista numerada","paragraph-numbering",numbered,"para.numbering",serde_json::json!({}));self.split(&row,&b,"numbering","numeración");
        row.append(&self.icon_action("format-indent-less-symbolic","Reducir sangría","paragraph-outdent",false,"para.outdent",serde_json::json!({})));
        let b=self.icon_action("format-indent-more-symbolic","Aumentar sangría","paragraph-indent",false,"para.indent",serde_json::json!({}));self.split(&row,&b,"indents","sangrías");self.main.append(&row);
        let row=gtk::Box::new(gtk::Orientation::Horizontal,4);
        let b=self.icon_action("spacing",&format!("Aplicar interlineado {}",self.spacing.get()),"paragraph-spacing",state["lineSpacing"]["rule"].as_str()==Some("multiple") && state["lineSpacing"]["value"].as_f64()==Some(self.spacing.get()),"para.lineSpacing",serde_json::json!({"value":self.spacing.get()}));self.split(&row,&b,"spacing","interlineado");
        let b=self.icon_action("shading","Aplicar el último sombreado elegido","paragraph-shading",state["shading"].is_array(),"para.shading",serde_json::json!({"color":self.shade.borrow().as_str()}));self.split(&row,&b,"shading","sombreado");
        let b=self.icon_action("borders","Aplicar el último borde elegido","paragraph-borders",state["borders"].is_object(),"para.borders",self.border.borrow().clone());self.split(&row,&b,"borders","bordes");self.main.append(&row);
        self.main.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let more=style_menu_button("Más opciones de párrafo…");more.set_widget_name("paragraph-more-options");self.main.append(&more);let weak=Rc::downgrade(self);more.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.page("options");}});
        self.stack.set_visible_child_name("main");
    }
    fn title(&self,label:&str){let l=gtk::Label::new(Some(label));l.set_xalign(0.0);l.add_css_class("heading");l.set_margin_top(8);self.detail.append(&l);}
    fn number(&self,label:&str,value:f64,min:f64,max:f64)->gtk::SpinButton{
        let row=gtk::Box::new(gtk::Orientation::Horizontal,8);let l=gtk::Label::new(Some(label));l.set_xalign(0.0);l.set_hexpand(true);l.set_wrap(true);row.append(&l);
        let n=gtk::SpinButton::with_range(min,max,0.5);n.set_digits(1);n.set_width_chars(6);n.set_value(value);row.append(&n);self.detail.append(&row);n
    }
    fn button(self:&Rc<Self>,label:&str,name:&str,command:&'static str,args:serde_json::Value){self.detail.append(&self.action(label,name,command,args));}
    fn page(self:&Rc<Self>,page:&str){
        clear_box(&self.detail);self.message.set_visible(false);
        let Some(ui)=self.ui.upgrade()else{return;};let state=selection_paragraph(&ui);
        match page{
            "bullets"|"numbering"=>{
                self.title(if page=="bullets"{"Viñetas"}else{"Numeración"});
                let (command,choices):(&'static str,Vec<(&str,&str)>)=if page=="bullets"{("para.bullets",vec![("• Punto","•"),("○ Círculo","○"),("▪ Cuadrado","▪"),("– Guion","–")])}else{("para.numbering",vec![("1. 2. 3.","numbered"),("1) 2) 3)","numberedParen"),("A. B. C.","upperLetter"),("a. b. c.","lowerLetter"),("i. ii. iii.","lowerRoman")])};
                for(label,kind)in choices{self.button(label,&format!("paragraph-list-{kind}"),command,serde_json::json!({"kind":kind}));}
                self.button("Quitar lista","paragraph-list-none",command,serde_json::json!({"off":true}));
                self.title("Lista multinivel");self.button("1 / 1.1 / 1.1.1","paragraph-list-legal","para.multilevel",serde_json::json!({"kind":"legal"}));self.button("Esquema","paragraph-list-outline","para.multilevel",serde_json::json!({"kind":"outline"}));
                let level=self.number("Nivel (1–9)",state["numbering"]["level"].as_f64().unwrap_or(0.0)+1.0,1.0,9.0);level.set_digits(0);level.set_increments(1.0,1.0);
                let b=style_menu_button("Aplicar nivel");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){level.update();s.execute("para.listLevel",serde_json::json!({"level":level.value_as_int()-1}));}});
                self.button("Reiniciar numeración en 1","paragraph-list-restart","para.restartNumbering",serde_json::json!({}));
            },
            "indents"=>{
                self.title("Sangrías (pt)");let left=self.number("Izquierda",state["indentLeft"].as_f64().unwrap_or(0.0),-1584.0,1584.0);let right=self.number("Derecha",state["indentRight"].as_f64().unwrap_or(0.0),-1584.0,1584.0);let first=self.number("Primera línea (−: francesa)",state["indentFirst"].as_f64().unwrap_or(0.0),-1584.0,1584.0);
                let b=style_menu_button("Aplicar sangrías");b.set_widget_name("paragraph-apply-indents");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){left.update();right.update();first.update();s.execute("para.indents",serde_json::json!({"left":left.value(),"right":right.value(),"firstLine":first.value()}));}});
            },
            "spacing"=>{
                self.title("Interlineado");for(value,label)in[(1.0,"Sencillo"),(1.15,"1,15 líneas"),(1.5,"1,5 líneas"),(2.0,"Doble")]{let b=style_menu_button(label);b.set_widget_name(&format!("paragraph-line-{value}"));self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.spacing.set(value);s.execute("para.lineSpacing",serde_json::json!({"value":value}));}});}
                let exact=self.number("Exacto (pt)",14.0,1.0,500.0);let b=style_menu_button("Aplicar interlineado exacto");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){exact.update();s.execute("para.lineSpacing",serde_json::json!({"exactly":exact.value()}));}});
                self.title("Separación entre párrafos (pt)");let before=self.number("Anterior",state["spaceBefore"].as_f64().unwrap_or(0.0),0.0,1584.0);let after=self.number("Posterior",state["spaceAfter"].as_f64().unwrap_or(0.0),0.0,1584.0);let b=style_menu_button("Aplicar separación");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){before.update();after.update();s.execute("para.spacing",serde_json::json!({"before":before.value(),"after":after.value()}));}});
            },
            "shading"=>{
                self.title("Sombreado");let grid=gtk::Grid::new();grid.set_column_spacing(6);grid.set_row_spacing(6);
                for(i,(label,color))in[("Amarillo suave","FFF2CC"),("Azul suave","DDEBF7"),("Verde suave","E2F0D9"),("Rosa suave","FCE4D6"),("Gris claro","E7E6E6"),("Gris oscuro","808080"),("Azul","4472C4"),("Negro","000000")].into_iter().enumerate(){let b=gtk::Button::new();let l=gtk::Label::new(None);l.set_markup(&format!("<span background=\"#{color}\">     </span>"));b.set_child(Some(&l));b.set_tooltip_text(Some(label));b.set_widget_name(&format!("paragraph-shade-{color}"));grid.attach(&b,(i%4)as i32,(i/4)as i32,1,1);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.shade.replace(color.into());s.execute("para.shading",serde_json::json!({"color":color}));}});}
                self.detail.append(&grid);self.color_entry("Aplicar color RGB",false);self.button("Sin sombreado","paragraph-shade-none","para.shading",serde_json::json!({"color":null}));
            },
            "borders"=>{
                self.title("Bordes");let width=self.number("Grosor (pt)",0.5,0.25,12.0);
                let color=gtk::Entry::new();color.set_placeholder_text(Some("Color RGB, por ejemplo 2266AA"));color.set_text("000000");self.detail.append(&color);
                for(label,kind)in[("Sin bordes","none"),("Todos","all"),("Exterior","outside"),("Interior","inside"),("Superior","top"),("Inferior","bottom"),("Izquierdo","left"),("Derecho","right"),("Línea horizontal","horizontalLine")]{let b=style_menu_button(label);b.set_widget_name(&format!("paragraph-border-{kind}"));self.detail.append(&b);let weak=Rc::downgrade(self);let width=width.clone();let color=color.clone();b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){width.update();if let Some(hex)=s.valid_color(&color){let args=serde_json::json!({"kind":kind,"width":width.value(),"color":hex});s.border.replace(args.clone());s.execute("para.borders",args);}}});}
            },
            _=>{
                self.title("Dirección y formato");self.button("Alineación distribuida","paragraph-distribute","para.align",serde_json::json!({"value":"distribute"}));
                self.button("De izquierda a derecha","paragraph-ltr","para.ltr",serde_json::json!({"value":true}));self.button("De derecha a izquierda","paragraph-rtl","para.rtl",serde_json::json!({"value":true}));
                let marks=ui.current.borrow().as_ref().is_some_and(|d|d.session.view.marks);self.toggle("Mostrar marcas de formato (¶)",marks,"view.marks");
                self.title("Saltos y conservación");for(label,key,cmd)in[("Conservar con el siguiente","keepNext","para.keepNext"),("Conservar líneas juntas","keepLines","para.keepLines"),("Salto de página anterior","pageBreakBefore","para.pageBreakBefore"),("Control de viudas y huérfanas","widowControl","para.widowControl")]{self.toggle(label,state[key].as_bool().unwrap_or(false),cmd);}
                self.title("Ordenar párrafos seleccionados");self.button("Ascendente (A–Z)","paragraph-sort-asc","para.sort",serde_json::json!({"descending":false}));self.button("Descendente (Z–A)","paragraph-sort-desc","para.sort",serde_json::json!({"descending":true}));
                self.title("Tabulaciones");let pos=self.number("Posición (pt)",36.0,0.0,1584.0);let b=style_menu_button("Añadir tabulación izquierda");self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){pos.update();s.execute("para.tabs",serde_json::json!({"set":[{"pos":pos.value(),"align":"left"}]}));}});
                self.button("Borrar tabulaciones","paragraph-tabs-clear","para.tabs",serde_json::json!({"clearAll":true}));self.button("Restablecer formato de párrafo","paragraph-reset","para.reset",serde_json::json!({}));
            }
        }
        self.stack.set_visible_child_name("detail");
    }
    fn toggle(self:&Rc<Self>,label:&str,active:bool,command:&'static str){
        let b=gtk::CheckButton::with_label(label);b.set_active(active);self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_toggled(move|b|{if let Some(s)=weak.upgrade(){s.execute(command,serde_json::json!({"value":b.is_active()}));}});
    }
    fn valid_color(&self,e:&gtk::Entry)->Option<String>{let c=e.text().trim().trim_start_matches('#').to_string();if c.len()==6&&c.chars().all(|c|c.is_ascii_hexdigit()){Some(c)}else{self.message.set_text("Escribe seis cifras RGB, por ejemplo 2266AA");self.message.set_visible(true);None}}
    fn color_entry(self:&Rc<Self>,label:&str,_border:bool){
        let e=gtk::Entry::new();e.set_placeholder_text(Some("RGB hexadecimal: FFF2CC"));self.detail.append(&e);let b=style_menu_button(label);self.detail.append(&b);let weak=Rc::downgrade(self);b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){if let Some(c)=s.valid_color(&e){s.shade.replace(c.clone());s.execute("para.shading",serde_json::json!({"color":c}));}}});
    }
}
fn install_paragraph_popover(ui:&Rc<Ui>,menu:&gtk::MenuButton)->Rc<ParagraphPopover>{
    let pop=gtk::Popover::new();pop.add_css_class("format-popover");let root=gtk::Box::new(gtk::Orientation::Vertical,6);root.set_width_request(300);root.set_margin_top(8);root.set_margin_bottom(8);root.set_margin_start(8);root.set_margin_end(8);
    let stack=gtk::Stack::new();stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);stack.set_vhomogeneous(false);
    let main=gtk::Box::new(gtk::Orientation::Vertical,6);let detail=gtk::Box::new(gtk::Orientation::Vertical,8);
    let scroll=gtk::ScrolledWindow::new();scroll.set_policy(gtk::PolicyType::Never,gtk::PolicyType::Automatic);scroll.set_max_content_height(420);scroll.set_propagate_natural_height(true);scroll.set_child(Some(&detail));
    let detail_page=gtk::Box::new(gtk::Orientation::Vertical,6);let back=style_menu_button("‹ Párrafo");back.set_widget_name("paragraph-back");detail_page.append(&back);detail_page.append(&gtk::Separator::new(gtk::Orientation::Horizontal));detail_page.append(&scroll);
    stack.add_named(&main,Some("main"));stack.add_named(&detail_page,Some("detail"));root.append(&stack);let message=gtk::Label::new(None);message.set_wrap(true);message.set_max_width_chars(35);message.set_visible(false);root.append(&message);pop.set_child(Some(&root));
    let s=Rc::new(ParagraphPopover{ui:Rc::downgrade(ui),popover:pop.downgrade(),stack,main,detail,message,spacing:Cell::new(1.5),shade:RefCell::new("FFF2CC".into()),border:RefCell::new(serde_json::json!({"kind":"outside","width":0.5,"color":"000000"}))});
    let weak=Rc::downgrade(&s);back.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){s.home();}});
    let state=s.clone();pop.connect_show(move |_|state.home());menu.set_popover(Some(&pop));s
}
fn show_paragraph_options(ui:&Rc<Ui>,_parent:&ApplicationWindow){if let Some(s)=ui.paragraph.borrow().as_ref(){if let Some(p)=s.popover.upgrade(){p.popup();}s.page("options");}}

fn paragraph_ui_test(s:&Rc<ParagraphPopover>,menu:&gtk::MenuButton,app:&Application){
    menu.popup();let ui=s.ui.upgrade().unwrap();
    let click=|name:&str|style_widget(&s.stack,name).unwrap_or_else(||panic!("missing {name}")).downcast::<gtk::Button>().unwrap().emit_clicked();
    click("paragraph-align-center");assert_eq!(selection_paragraph(&ui)["align"],"center");
    click("paragraph-indent");assert!(selection_paragraph(&ui)["indentLeft"].as_f64().unwrap()>0.0);
    click("paragraph-more-bullets");assert_eq!(s.stack.visible_child_name().as_deref(),Some("detail"));
    click("paragraph-list-•");assert!(selection_paragraph(&ui)["numbering"].is_object());click("paragraph-back");
    click("paragraph-more-spacing");click("paragraph-line-1.5");assert_eq!(selection_paragraph(&ui)["lineSpacing"]["value"],1.5);click("paragraph-back");
    click("paragraph-more-shading");click("paragraph-shade-FFF2CC");assert!(selection_paragraph(&ui)["shading"].is_array());click("paragraph-back");
    click("paragraph-more-borders");click("paragraph-border-outside");assert!(selection_paragraph(&ui)["borders"].is_object());click("paragraph-back");
    click("paragraph-more-options");assert!(style_widget(&s.detail,"paragraph-sort-asc").is_some());click("paragraph-back");
    assert_eq!(s.stack.visible_child_name().as_deref(),Some("main"));assert_eq!(app.windows().len(),1);
    assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());eprintln!("PARAGRAPH_POPOVER_OK");app.quit();
}
