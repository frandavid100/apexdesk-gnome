// Diagnostic ownership probes. Never retain the objects being measured.
thread_local! {
    static LIFE_HOSTS: RefCell<Vec<std::rc::Weak<Workspace>>> = const { RefCell::new(Vec::new()) };
    static LIFE_STARTED: Cell<bool> = const { Cell::new(false) };
}
fn lifecycle_mode() -> bool { std::env::args().any(|a|a=="--lifecycle-audit" || a=="--lifecycle-ui-test") }
fn lifecycle_track(host:&Rc<Workspace>) { if lifecycle_mode() { LIFE_HOSTS.with(|v|v.borrow_mut().push(Rc::downgrade(host))); } }
fn lifecycle_rss()->Option<u64> {
    let status=std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find(|s|s.starts_with("VmRSS:"))?.split_whitespace().nth(1)?.parse().ok()
}
#[cfg(target_os="linux")]
fn lifecycle_trim() { unsafe extern "C" { fn malloc_trim(pad:usize)->i32; } unsafe { malloc_trim(0); } }
#[cfg(not(target_os="linux"))]
fn lifecycle_trim() {}
struct ClosedTabProbe {
    ui:std::rc::Weak<Ui>,picture:glib::WeakRef<gtk::Picture>,body:std::sync::Weak<wordcraft_doc::Block>,
}
struct LifecycleAudit {
    host:RefCell<Option<Rc<Workspace>>>,app:Application,file:PathBuf,
    tabs:RefCell<Vec<ClosedTabProbe>>,windows:RefCell<Vec<std::rc::Weak<Workspace>>>,
    wave:Cell<u32>,strict:bool,
}
impl LifecycleAudit {
    fn probe(&self,ui:&Rc<Ui>) {
        let current=ui.current.borrow();let d=current.as_ref().unwrap();
        self.tabs.borrow_mut().push(ClosedTabProbe{ui:Rc::downgrade(ui),picture:d.pages[0].downgrade(),body:std::sync::Arc::downgrade(&d.session.doc.body[0])});
    }
    fn open(self:&Rc<Self>) {
        let host=self.host.borrow().as_ref().unwrap().clone();
        for n in 0..3 {
            let session=open_document(&self.file).unwrap();
            build_tab(&self.app,&host,false,true,false,false,false,false,false,Some((session,format!("Lifecycle {n}"))));
            let ui=selected_ui(&host).unwrap();self.probe(&ui);
            if let Some(s)=ui.search.borrow().as_ref(){s.show(false);s.query.set_text("Lifecycle");s.refresh();s.toggle.set_active(false);}
            if let Some(m)=format_button(&ui,"style-menu"){m.popup();if let Some(p)=m.popover(){p.popdown();}}
            let context=show_document_context(&ui,&host.window,140,160,DocumentContext::Text);context.popdown();
            schedule_thumbnail_refresh(&ui);
        }
        for _ in 0..2 {
            let window=make_window(&self.app,false,true,false,false,false,false,false,Some((open_document(&self.file).unwrap(),"Lifecycle window".into())));
            let child=LIFE_HOSTS.with(|v|v.borrow().last().unwrap().upgrade().unwrap());
            assert_eq!(child.window,window);self.probe(&selected_ui(&child).unwrap());
            self.windows.borrow_mut().push(Rc::downgrade(&child));
        }
        let next=self.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(550),move||next.close());
    }
    fn close(self:&Rc<Self>) {
        let host=self.host.borrow().as_ref().unwrap().clone();
        while host.tabs.n_pages()>1 { let page=host.tabs.nth_page(host.tabs.n_pages()-1);host.tabs.close_page(&page); }
        for weak in self.windows.borrow().iter(){if let Some(w)=weak.upgrade(){w.window.close();}}
        let next=self.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(1250),move||next.sample());
    }
    fn sample(self:&Rc<Self>) {
        let wave=self.wave.get()+1;self.wave.set(wave);
        let tabs=self.tabs.borrow();let ui=tabs.iter().filter(|p|p.ui.upgrade().is_some()).count();
        let documents=tabs.iter().filter(|p|p.body.upgrade().is_some()).count();
        let pictures=tabs.iter().filter(|p|p.picture.upgrade().is_some()).count();
        let windows=self.windows.borrow().iter().filter(|p|p.upgrade().is_some()).count();
        let rss=lifecycle_rss();lifecycle_trim();let trimmed=lifecycle_rss();
        eprintln!("LIFECYCLE_SAMPLE {}",serde_json::json!({"wave":wave,"closed_tabs":tabs.len(),"ui_alive":ui,"documents_alive":documents,"pictures_alive":pictures,"workspaces_alive":windows,"rss_kib":rss,"trimmed_rss_kib":trimmed}));
        if self.strict {assert_eq!((ui,documents,pictures,windows),(0,0,0,0),"closed objects must be released, not just hidden");}
        drop(tabs);
        if wave<3 {self.open();return;}
        let host=self.host.borrow_mut().take().unwrap();let weak=Rc::downgrade(&host);
        let root_ui=Rc::downgrade(&selected_ui(&host).unwrap());let window=host.window.downgrade();
        host.window.close();drop(host);let app=self.app.clone();let hold=app.hold();let strict=self.strict;let file=self.file.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1300),move||{
            let alive=(weak.upgrade().is_some(),root_ui.upgrade().is_some(),window.upgrade().is_some());
            eprintln!("LIFECYCLE_ROOT workspace={} ui={} window={}",alive.0,alive.1,alive.2);
            if strict {assert_eq!(alive,(false,false,false));}
            std::fs::remove_file(file).unwrap();eprintln!("LIFECYCLE_{}",if strict{"UI_OK"}else{"BASELINE_OK"});drop(hold);app.quit();
        });
    }
}
fn lifecycle_start(host:&Rc<Workspace>,app:&Application) {
    if !lifecycle_mode() || LIFE_STARTED.with(|s|s.replace(true)) {return;}
    let text=(0..60).map(|i|format!("Lifecycle paragraph {i}: café, selección y texto de prueba. {}", "Useful document content. ".repeat(5))).collect::<Vec<_>>().join("\n");
    let document=wordcraft_doc::Document::from_text(&text);
    let file=std::env::temp_dir().join(format!("leaf-lifecycle-{}.docx",std::process::id()));
    std::fs::write(&file,wordcraft_engine::io::save_bytes("fixture.docx",&document).unwrap()).unwrap();
    let audit=Rc::new(LifecycleAudit{host:RefCell::new(Some(host.clone())),app:app.clone(),file,tabs:RefCell::new(Vec::new()),windows:RefCell::new(Vec::new()),wave:Cell::new(0),strict:!std::env::args().any(|a|a=="--lifecycle-audit")});
    glib::timeout_add_local_once(std::time::Duration::from_millis(700),move||{eprintln!("LIFECYCLE_START rss_kib={:?}",lifecycle_rss());audit.open();});
}
