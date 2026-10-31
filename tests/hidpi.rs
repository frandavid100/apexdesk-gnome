use wordcraft_gnome::{sample_session,render_page_premultiplied};
#[test]
fn raster_density_is_not_capped_at_document_zoom(){
    let mut s=sample_session();let rev=s.rev();let sel=s.sel.clone();let depth=s.undo_depth();
    let (w,h)={let l=s.layout();(l.pages[0].w,l.pages[0].h)};
    for scale in [0.28,0.56,1.0,2.0,4.0]{
        let image=render_page_premultiplied(&mut s,0,scale).unwrap();
        assert_eq!((image.width,image.height),((w*scale).ceil()as u32,(h*scale).ceil()as u32));
        assert_eq!(image.rgba.len(),image.width as usize*image.height as usize*4);
    }
    assert_eq!(s.rev(),rev);assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth);
    assert!(render_page_premultiplied(&mut s,0,10000.0).is_err(),"retain allocation guard");
}
