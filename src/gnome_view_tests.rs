// Real GTK regression: focus restoration after a bottom popover must not pan
// the tall canvas before the subsequent pointer event is hit-tested.
fn view_pointer_test(ui:&Rc<Ui>,menu:&gtk::MenuButton,window:&ApplicationWindow,app:&Application){
    let text=(0..140).map(|i|format!("Párrafo {i}: texto de prueba para comprobar el cursor al cerrar un menú y volver al documento.\n")).collect::<String>();
    opened(ui,Session::new(wordcraft_doc::Document::from_text(&text)),"Prueba del visor");
    apply_user_edit(ui,Edit::Insert("x".into()));
    assert!(ui.title.subtitle().is_empty());assert!(ui.title.title().starts_with("● "));
    ui.current.borrow_mut().as_mut().unwrap().fit_to_width=false;
    zoom_to(ui,0.5);
    let ui=ui.clone();let menu=menu.clone();let window=window.clone();let app=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(300),move||{
        {assert_eq!(ui.sidebar_selector.height(),ui.format_toolbar.height(),"bottom bars must align");let value=ui.current.borrow();let d=value.as_ref().unwrap();assert_eq!(d.arrangement.columns,2);
            let a=d.page_frames[0].compute_bounds(&ui.document_box).unwrap();let b=d.page_frames[1].compute_bounds(&ui.document_box).unwrap();
            assert!((a.y()-b.y()).abs()<1.0);assert!(b.x()>a.x()+a.width());
            let button=d.thumb_buttons[0].compute_bounds(&ui.thumbnails_box).unwrap();let inside=d.thumbs[0].parent().unwrap();let image=inside.compute_bounds(&ui.thumbnails_box).unwrap();
            assert!(inside.last_child().unwrap().downcast::<gtk::Label>().unwrap().text()=="1");
            let left=image.x()-button.x();let right=button.x()+button.width()-image.x()-image.width();
            let top=image.y()-button.y();let bottom=button.y()+button.height()-image.y()-image.height();
            assert!(left>=11.0&&top>=11.0);
            assert!((left-right).abs()<1.1&& (top-bottom).abs()<1.1,"thumbnail inset must be symmetric");
        }
        zoom_to(&ui,0.85);
        let next=ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(300),move||{
            let (page,top)={let mut value=next.current.borrow_mut();let d=value.as_mut().unwrap();assert_eq!(d.arrangement.columns,1);
                let c=d.session.layout().caret_on(&wordcraft_doc::Pos::body(80,5),0).unwrap();
                (c.page,d.arrangement.pages[c.page].y+c.top as f64*d.zoom as f64)
            };
            assert!(page>0);
            let adj=next.scroll.vadjustment();adj.set_value(top-adj.page_size()*0.35);
            let ui=next.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(250),move||{
                let (x,y)={let value=ui.current.borrow();let d=value.as_ref().unwrap();let r=&d.arrangement.pages[page];(r.x+r.width-65.0,top+4.0)};
                let expected=pos_at_canvas(&ui,x,y).unwrap();assert!(expected.path.last()>50);
                let canvas=ui.document_box.compute_bounds(&window).unwrap();let m=menu.compute_bounds(&window).unwrap();
                let scroll=ui.scroll.vadjustment().value();
                eprintln!("VIEW_POINTER_READY {} {} {} {}",(m.x()+m.width()/2.0)as i32,(m.y()+m.height()/2.0)as i32,(canvas.x()as f64+x)as i32,(canvas.y()as f64+y)as i32);
                glib::timeout_add_local_once(std::time::Duration::from_secs(1),move||{
                    assert!((ui.scroll.vadjustment().value()-scroll).abs()<2.0,"popover dismissal moved the viewport");
                    // GtkPopover consumes the dismissing click. Verify it first,
                    // then let xdotool send a fresh canvas click at the same point.
                    eprintln!("VIEW_POINTER_DISMISSED");
                    glib::timeout_add_local_once(std::time::Duration::from_secs(2),move||{
                        assert!((ui.scroll.vadjustment().value()-scroll).abs()<2.0,"canvas focus moved the viewport");
                        let value=ui.current.borrow();let d=value.as_ref().unwrap();
                        assert_eq!(d.session.sel.focus.path,expected.path,"click must stay on the clicked paragraph");
                        assert!(d.session.sel.focus.off.abs_diff(expected.off)<=1);
                        eprintln!("VIEW_POINTER_OK");app.quit();
                    });
                });
            });
        });
    });
}


// Start without a fixture: the shipped application must open a clean document.
fn chrome_ui_test(ui:&Rc<Ui>,menu:&gtk::MenuButton,app:&Application){
    assert_eq!(&*ui.document_name.borrow(),"Documento nuevo");
    assert!(ui.current.borrow().as_ref().unwrap().session.is_untouched_blank());
    assert_eq!(ui.sidebar_selector.height(),ui.format_toolbar.height());
    assert!(menu.popover().unwrap().is::<gtk::PopoverMenu>());
    let model=menu.menu_model().unwrap();assert_eq!(model.n_items(),2);
    for(section,actions)in[(0,["win.undo","win.redo"]),(1,["win.save","win.save-as"]) ]{
        let part=model.item_link(section,"section").unwrap();assert_eq!(part.n_items(),2);
        for(i,action)in actions.iter().enumerate(){assert_eq!(part.item_attribute_value(i as i32,"action",None).unwrap().get::<String>().unwrap(),*action);}
    }
    let h=ui.host.upgrade().unwrap();assert!(h.window.lookup_action("example").is_none());
    assert!(!h.window.lookup_action("undo").unwrap().is_enabled());
    apply_user_edit(ui,Edit::Insert("Prueba".into()));refresh_main_actions(&h);
    assert!(h.window.lookup_action("undo").unwrap().is_enabled());
    gio::prelude::ActionGroupExt::activate_action(&h.window,"undo",None);
    assert!(ui.current.borrow().as_ref().unwrap().session.can_redo());
    gio::prelude::ActionGroupExt::activate_action(&h.window,"redo",None);
    assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());
    eprintln!("DOCUMENT_CHROME_OK");app.quit();
}
