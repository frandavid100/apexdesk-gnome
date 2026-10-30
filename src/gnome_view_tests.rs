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
        {let value=ui.current.borrow();let d=value.as_ref().unwrap();assert_eq!(d.arrangement.columns,2);
            let a=d.page_frames[0].compute_bounds(&ui.document_box).unwrap();let b=d.page_frames[1].compute_bounds(&ui.document_box).unwrap();
            assert!((a.y()-b.y()).abs()<1.0);assert!(b.x()>a.x()+a.width());
            let button=d.thumb_buttons[0].compute_bounds(&ui.thumbnails_box).unwrap();let image=d.thumbs[0].compute_bounds(&ui.thumbnails_box).unwrap();
            let left=image.x()-button.x();let right=button.x()+button.width()-image.x()-image.width();
            let top=image.y()-button.y();let bottom=button.y()+button.height()-image.y()-image.height();
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
                glib::timeout_add_local_once(std::time::Duration::from_secs(3),move||{
                    assert!((ui.scroll.vadjustment().value()-scroll).abs()<2.0,"popover return moved the viewport");
                    let value=ui.current.borrow();let d=value.as_ref().unwrap();
                    assert_eq!(d.session.sel.focus.path,expected.path,"click must stay on the clicked paragraph");
                    assert!(d.session.sel.focus.off.abs_diff(expected.off)<=1);
                    eprintln!("VIEW_POINTER_OK");app.quit();
                });
            });
        });
    });
}
