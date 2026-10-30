//! Sidebar search uses WordCraft's literal matcher, including table text.
use wordcraft_doc::Pos;
use wordcraft_engine::Session;
use crate::selection::{self, Rect};

#[derive(Clone, Debug)]
pub struct Hit {
    pub start: Pos,
    pub end: Pos,
    pub before: String,
    pub matched: String,
    pub after: String,
    pub page: Option<usize>,
    pub rects: Vec<Rect>,
    pub revision: u64,
    pub document: u64,
}

pub fn query(s: &mut Session, text: &str, match_case: bool) -> Result<Vec<Hit>, String> {
    if text.is_empty() { s.find.results.clear(); return Ok(Vec::new()); }
    // Advanced Find searches the body without moving the live selection.
    s.run("edit.advancedFind", &serde_json::json!({"text":text,"in":"main",
        "matchCase":match_case,"special":false,"regex":false,"wildcards":false,
        "wholeWord":false,"matchPrefix":false,"matchSuffix":false,
        "ignorePunct":false,"ignoreSpace":false,"highlight":false})).map_err(|e|e.to_string())?;
    let positions = s.find.results.clone();
    let layout = s.layout();
    let revision = s.rev();let document=s.document_id();
    Ok(positions.into_iter().map(|(start,end)| {
        let p = s.doc.para_at(&start).map(|p| p.text.as_str()).unwrap_or("");
        let before = p.get(..start.off).unwrap_or("").chars().rev().take(28).collect::<Vec<_>>().into_iter().rev().collect();
        let same = start.story == end.story && start.path == end.path;
        let matched = if same { p.get(start.off..end.off).unwrap_or(text).to_string() } else { text.to_string() };
        let tail = s.doc.para_at(&end).and_then(|p|p.text.get(end.off..)).unwrap_or("");
        let after = tail.chars().take(55).collect();
        let rects: Vec<Rect> = layout.selection_rects(&s.doc,&start,&end,0).into_iter()
            .map(|(page,r)|Rect {page,x:r.x as f64,y:r.y as f64,width:r.w as f64,height:r.h as f64}).collect();
        let page = rects.first().map(|r|r.page);
        Hit {start,end,before,matched,after,page,rects,revision,document}
    }).collect())
}

pub fn select(s: &mut Session, hit: &Hit) -> Result<(), String> {
    if s.document_id()!=hit.document || s.rev()!=hit.revision {return Err("El documento ha cambiado; la búsqueda se ha actualizado".into());}
    selection::select_range(s,&hit.start,&hit.end)
}

pub fn replace(s: &mut Session, hit: &Hit, with: &str) -> Result<(), String> {
    select(s,hit)?;
    // Literal input: replacement strings such as ^p or $1 must not be expanded.
    crate::editing::apply(s, if with.is_empty() {crate::editing::Edit::Delete} else {crate::editing::Edit::Insert(with.to_owned())}).map(|_|())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::{Document,StoryRef};
    #[test]
    fn results_include_tables_and_pages_beyond_the_first() {
        let mut s=crate::sample_session();
        let table=query(&mut s,"Mara O.",true).unwrap();
        assert_eq!(table.len(),1);assert!(table[0].start.path.cell().is_some());
        assert!(table[0].page.is_some());
        replace(&mut s,&table[0],"Otra persona").unwrap();
        assert!(query(&mut s,"Mara O.",true).unwrap().is_empty());
        let text=(0..160).map(|i|format!("Coincidencia número {i}\n")).collect::<String>();
        let mut s=Session::new(Document::from_text(&text));
        let hits=query(&mut s,"Coincidencia",true).unwrap();
        assert_eq!(hits.len(),160);assert!(hits.last().unwrap().page.unwrap()>0);
    }
    #[test]
    fn results_preserve_selection_case_and_undo_and_reject_stale_rows() {
        let mut s=Session::new(Document::from_text("Árbol árbol árbol"));
        let selection=s.sel.clone();let rev=s.rev();
        let hits=query(&mut s,"árbol",false).unwrap();
        assert_eq!(hits.len(),3);assert_eq!(s.sel,selection);assert_eq!(s.rev(),rev);
        assert_eq!(query(&mut s,"árbol",true).unwrap().len(),2);
        assert!(!hits[1].rects.is_empty());assert!(hits[1].before.contains("Árbol"));
        replace(&mut s,&hits[1],"$1 ^p").unwrap();
        assert_eq!(s.doc.plain_text(StoryRef::Body),"Árbol $1 ^p árbol");
        assert!(replace(&mut s,&hits[2],"stale").is_err());
        s.run("edit.undo",&serde_json::json!({})).unwrap();
        assert_eq!(s.doc.plain_text(StoryRef::Body),"Árbol árbol árbol");
        assert_eq!(query(&mut s,"árbol",false).unwrap().len(),3);
        let mut other=Session::new(Document::from_text("árbol diferente"));
        assert!(replace(&mut other,&hits[0],"wrong document").is_err());
    }
}
