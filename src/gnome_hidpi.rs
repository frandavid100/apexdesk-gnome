// Keep raster pixels separate from GTK's logical layout units.
mod page_paintable {
    use super::*;
    use glib::subclass::prelude::*;
    use gdk::subclass::prelude::*;
    mod imp {
        use super::*;
        #[derive(Default)]
        pub struct PagePaintable {pub data:std::cell::OnceCell<(gdk::MemoryTexture,i32,i32)>}
        #[glib::object_subclass]
        impl ObjectSubclass for PagePaintable {
            const NAME:&'static str="LeafPagePaintable";
            type Type=super::PagePaintable;
            type Interfaces=(gdk::Paintable,);
        }
        impl ObjectImpl for PagePaintable {}
        impl PaintableImpl for PagePaintable {
            fn snapshot(&self,snapshot:&gdk::Snapshot,width:f64,height:f64){
                if let Some((texture,_,_))=self.data.get(){texture.snapshot(snapshot,width,height);}
            }
            fn intrinsic_width(&self)->i32 {self.data.get().map_or(0,|d|d.1)}
            fn intrinsic_height(&self)->i32 {self.data.get().map_or(0,|d|d.2)}
            fn intrinsic_aspect_ratio(&self)->f64 {self.data.get().map_or(0.0,|d|d.1 as f64/d.2.max(1) as f64)}
            fn flags(&self)->gdk::PaintableFlags {gdk::PaintableFlags::SIZE | gdk::PaintableFlags::CONTENTS}
        }
    }
    glib::wrapper! {pub struct PagePaintable(ObjectSubclass<imp::PagePaintable>) @implements gdk::Paintable;}
    pub fn new(image:wordcraft_gnome::PageImage,width:i32,height:i32)->PagePaintable {
        let paintable:PagePaintable=glib::Object::new();
        paintable.imp().data.set((texture(image),width,height)).unwrap();
        paintable
    }
    impl PagePaintable {
        pub fn pixels(&self)->(i32,i32){let d=self.imp().data.get().unwrap();(d.0.width(),d.0.height())}
    }
}
fn sync_raster_scale(ui:&Rc<Ui>){
    let density=ui.document_box.scale_factor().max(1);
    let Ok(mut current)=ui.current.try_borrow_mut()else{return;};
    let Some(doc)=current.as_mut()else{return;};
    if doc.raster_scale==density{return;}
    for i in doc.painted_pages.drain(){doc.pages[i].set_paintable(None::<&gdk::Paintable>);}
    for i in doc.painted_thumbs.drain(){doc.thumbs[i].set_paintable(None::<&gdk::Paintable>);}
    doc.raster_scale=density;
}
fn hidpi_assert(ui:&Rc<Ui>,zoom:f32){
    let current=ui.current.borrow();let d=current.as_ref().unwrap();
    assert_eq!(d.raster_scale,ui.document_box.scale_factor());
    assert!((d.zoom-zoom).abs()<0.001);
    assert!(!d.painted_pages.is_empty());assert!(!d.painted_thumbs.is_empty());
    for &i in &d.painted_pages {
        let p=d.pages[i].paintable().unwrap().downcast::<page_paintable::PagePaintable>().unwrap();
        let (w,h)=d.sizes[i];let density=d.raster_scale as f32;
        assert_eq!(p.pixels(),((w*zoom*density).ceil()as i32,(h*zoom*density).ceil()as i32));
        assert_eq!(p.intrinsic_width(),(w*zoom).ceil()as i32);
        assert_eq!(p.intrinsic_height(),(h*zoom).ceil()as i32);
        assert!((d.page_frames[i].width()as f32-w*zoom).abs()<3.0,"raster density changed page allocation");
    }
    for &i in &d.painted_thumbs {
        let p=d.thumbs[i].paintable().unwrap().downcast::<page_paintable::PagePaintable>().unwrap();
        let (w,h)=d.sizes[i];let density=d.raster_scale as f32;
        assert_eq!(p.pixels(),((w*THUMB_SCALE*density).ceil()as i32,(h*THUMB_SCALE*density).ceil()as i32));
        assert_eq!(p.intrinsic_width(),(w*THUMB_SCALE).ceil()as i32);
    }
}
fn hidpi_ui_test(ui:&Rc<Ui>,app:&Application){
    opened(ui,sample_session(),"Prueba HiDPI");
    ui.sidebar.set_show_sidebar(true);
    ui.current.borrow_mut().as_mut().unwrap().fit_to_width=false;
    zoom_to(ui,1.0);
    let u=ui.clone();let a=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(350),move||{
        refresh_pages(&u);refresh_thumbnails(&u);hidpi_assert(&u,1.0);
        let identity=document_identity(&u);
        u.current.borrow_mut().as_mut().unwrap().raster_scale=0;
        refresh_pages(&u);refresh_thumbnails(&u);
        assert_eq!(document_identity(&u),identity);
        zoom_to(&u,2.0);
        glib::timeout_add_local_once(std::time::Duration::from_millis(350),move||{
            refresh_pages(&u);refresh_thumbnails(&u);hidpi_assert(&u,2.0);
            assert_eq!(document_identity(&u),identity);
            eprintln!("HIDPI_UI_OK scale={}",u.document_box.scale_factor());a.quit();
        });
    });
}
