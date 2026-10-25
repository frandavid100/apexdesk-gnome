use apexdesk_gnome_phase1::{edit_copy, open_document, run_smoke, summary};
use std::path::Path;

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    match args.next().unwrap_or_default().as_str() {
        "smoke" => {
            let dir = args.next().unwrap_or_else(|| "target/phase1-smoke".into());
            for entry in run_smoke(Path::new(&dir))? {
                println!("{entry}");
            }
            Ok(())
        }
        "inspect" => {
            let path = args.next().ok_or("Falta el archivo")?;
            let (doc, fmt) = open_document(Path::new(&path))?;
            let s = summary(&doc, fmt);
            println!(
                "Formato: {}; párrafos: {}; negritas: {}; título: {:?}",
                s.format, s.paragraphs, s.bold_spans, s.title
            );
            Ok(())
        }
        "edit-copy" => {
            let src = args.next().ok_or("Falta el origen")?;
            let dst = args.next().ok_or("Falta el destino")?;
            let from = args.next().ok_or("Falta el texto buscado")?;
            let to = args.next().ok_or("Falta el texto sustituto")?;
            let n = edit_copy(Path::new(&src), Path::new(&dst), &from, &to)?;
            println!("{n} sustituciones; copia verificada: {dst}");
            Ok(())
        }
        _ => Err("Uso: smoke [DIRECTORIO] | inspect ARCHIVO | edit-copy ORIGEN DESTINO BUSCAR REEMPLAZO".into()),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}
