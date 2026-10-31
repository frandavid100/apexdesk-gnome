// GTK owns the native print/portal UI; WordCraft supplies the vector PDF.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrintOutcome { Submitted, Dismissed, Failed }
#[derive(Default)]
struct PrintState {
    pending: RefCell<Option<gio::Cancellable>>,
    outcome: Cell<Option<PrintOutcome>>,
}
impl PrintState {
    fn cancel(&self) { if let Some(c) = self.pending.borrow().as_ref() { c.cancel(); } }
}
fn finish_print(ui: &Ui, outcome: PrintOutcome) {
    ui.printing.pending.borrow_mut().take(); ui.printing.outcome.set(Some(outcome));
    if let Some(host) = ui.host.upgrade() { refresh_main_actions(&host); }
}
fn print_document(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    if ui.closed.get() || ui.printing.pending.borrow().is_some() { return; }
    let Some(document) = ui.current.borrow().as_ref().map(|d| d.session.doc.clone()) else { return; };
    let cancel = gio::Cancellable::new();
    ui.printing.pending.replace(Some(cancel.clone())); ui.printing.outcome.set(None);
    if let Some(host) = ui.host.upgrade() { refresh_main_actions(&host); }
    ui.toasts.add_toast(Toast::new("Preparando impresión…"));
    let title = format!("Imprimir — {}", ui.document_name.borrow());
    let weak = Rc::downgrade(ui); let parent = parent.downgrade();
    // Keep the main loop alive for cleanup if the last tab closes during export.
    let hold = parent.upgrade().and_then(|p| p.application()).map(|app| app.hold());
    glib::MainContext::default().spawn_local(async move {
        let prepared = gio::spawn_blocking(move || wordcraft_gnome::printing::PrintFile::prepare(document)).await;
        let Some(ui) = weak.upgrade() else { return; };
        let Some(parent) = parent.upgrade().filter(|_| !ui.closed.get() && !cancel.is_cancelled()) else { finish_print(&ui, PrintOutcome::Dismissed); return; };
        let file = match prepared {
            Ok(Ok(file)) => file,
            result => {
                let message = match result { Ok(Err(e)) => e, _ => "No se pudo preparar el PDF de impresión".into() };
                ui.toasts.add_toast(Toast::new(&message)); finish_print(&ui, PrintOutcome::Failed); return;
            }
        };
        let dialog = gtk::PrintDialog::new(); dialog.set_title(&title); dialog.set_modal(true);
        let source = gio::File::for_path(file.path()); let weak = Rc::downgrade(&ui);
        dialog.print_file(Some(&parent), None, &source, Some(&cancel), move |result| {
            // Retain the source until GTK/portal has consumed its descriptor.
            drop(file); drop(hold);
            if let Some(ui) = weak.upgrade() {
                let outcome = match result {
                    Ok(()) => { if !ui.closed.get() { ui.toasts.add_toast(Toast::new("Documento enviado a impresión")); } PrintOutcome::Submitted },
                    Err(e) if e.matches(gtk::DialogError::Cancelled) || e.matches(gtk::DialogError::Dismissed) || e.matches(gio::IOErrorEnum::Cancelled) => PrintOutcome::Dismissed,
                    Err(e) => { if !ui.closed.get() { ui.toasts.add_toast(Toast::new(&format!("No se pudo imprimir: {e}"))); } PrintOutcome::Failed }
                };
                finish_print(&ui, outcome);
            }
        });
    });
}

fn print_ui_test(ui: &Rc<Ui>, parent: &ApplicationWindow, app: &Application) {
    let document = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let ui = ui.clone(); let parent = parent.clone(); let app = app.clone();
    gtk::prelude::WidgetExt::activate_action(parent,"win.print", None).unwrap();
    // Duplicate activation must not create a second dialog/job.
    print_document(&ui, &parent);
    let attempts = Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        attempts.set(attempts.get()+1); assert!(attempts.get()<300, "print callback timed out");
        if let Some(outcome) = ui.printing.outcome.get() {
            assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, document);
            let expected = match std::env::var("LEAF_PRINT_TEST").as_deref() { Ok("cancel") => PrintOutcome::Dismissed, Ok("error") => PrintOutcome::Failed, _ => PrintOutcome::Submitted };
            assert_eq!(outcome, expected); assert!(ui.printing.pending.borrow().is_none());
            eprintln!("PRINT_UI_OK {outcome:?}"); app.quit(); return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });
}
