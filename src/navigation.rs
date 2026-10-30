//! Read the resolved heading levels; never infer headings from font size or names.
use wordcraft_doc::{Document,Pos,StoryRef};
#[derive(Clone,Debug)]
pub struct Heading {pub pos:Pos,pub level:u8,pub text:String}
pub fn headings(doc:&Document)->Vec<Heading>{
    let mut counters=wordcraft_doc::numbering::Counters::default();
    doc.para_paths(StoryRef::Body).into_iter().filter_map(|path|{
        let pos=Pos{story:StoryRef::Body,path,off:0};let p=doc.para_at(&pos)?;
        let resolved=doc.styles.resolve_para(&p.props);
        let number=p.props.numbering.or(resolved.numbering).filter(|n|n.num!=0)
            .and_then(|n|counters.next_label(&doc.numbering,n.num,n.level))
            .filter(|(label,def)|!label.is_empty()&&!matches!(def.format,wordcraft_doc::section::NumFormat::Bullet|wordcraft_doc::section::NumFormat::None));
        let level=resolved.outline_level.filter(|l|*l<9)?;
        let text=p.plain_text().trim().to_owned();if text.is_empty()||resolved.style.starts_with("TOC"){return None;}
        let text=match number{Some((label,def))if def.suffix==wordcraft_doc::numbering::LevelSuffix::Nothing=>format!("{label}{text}"),Some((label,_))=>format!("{label} {text}"),None=>text};
        Some(Heading{pos,level,text})
    }).collect()
}
#[cfg(test)]
mod tests{
    use super::*;use wordcraft_engine::Session;use serde_json::json;
    #[test]
    fn inherited_levels_explicit_body_and_undo_are_respected(){
        let mut s=Session::new(Document::from_text("Sección\nSubsección\nTexto"));
        s.run("para.heading1",&json!({})).unwrap();
        s.run("caret.set",&json!({"pos":Pos::body(1,0)})).unwrap();s.run("para.heading2",&json!({})).unwrap();
        let h=headings(&s.doc);assert_eq!(h.iter().map(|h|h.level).collect::<Vec<_>>(),vec![0,1]);assert_eq!(h[1].text,"Subsección");
        s.run("outline.level",&json!({"level":"body"})).unwrap();assert_eq!(headings(&s.doc).len(),1);
        s.run("edit.undo",&json!({})).unwrap();assert_eq!(headings(&s.doc).len(),2);
    }
}
