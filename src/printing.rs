//! A private, disposable PDF snapshot. Printing never saves the live session.
use std::{path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};
use wordcraft_doc::Document;
use wordcraft_engine::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct PrintFile { directory: PathBuf }
impl PrintFile {
    pub fn prepare(document: Document) -> Result<Self, String> {
        let tick = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?.as_nanos();
        let directory = std::env::temp_dir().join(format!("leaf-print-{}-{tick}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
        builder.create(&directory).map_err(|e| format!("No se pudo preparar la impresión: {e}"))?;
        let file = Self { directory };
        crate::pdf_export::export(&mut Session::new(document), &file.path())?;
        Ok(file)
    }
    pub fn path(&self) -> PathBuf { self.directory.join("document.pdf") }
    pub fn directory(&self) -> &Path { &self.directory }
}
impl Drop for PrintFile {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.directory); }
}
