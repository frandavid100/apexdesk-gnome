//! Atomic page setup, preserving every section's other properties.
use serde_json::json;
use wordcraft_doc::{Pos,StoryRef,section::SectionProps};
use wordcraft_engine::{Session,cmd::page};

#[derive(Clone,Copy,Debug)]
pub struct PageSetup {pub width:f32,pub height:f32,pub top:f32,pub bottom:f32,pub left:f32,pub right:f32}
impl PageSetup {
    pub fn current(s:&Session)->Self {let p=page::sect(s);Self{width:p.page_w,height:p.page_h,top:p.margin_top,bottom:p.margin_bottom,left:p.margin_left,right:p.margin_right}}
    fn section(self,mut p:SectionProps)->Result<SectionProps,String>{
        let nums=[self.width,self.height,self.top,self.bottom,self.left,self.right];
        if nums.iter().any(|n|!n.is_finite())||!(7.2..=1584.0).contains(&self.width)||!(7.2..=1584.0).contains(&self.height)||nums[2..].iter().any(|n|*n<0.0){return Err("Revisa el tamaño de página y los márgenes".into());}
        if !page::margins_fit((self.width,self.height),(self.top,self.bottom,self.left,self.right),p.gutter){return Err("Los márgenes no dejan espacio suficiente para el texto".into());}
        // Displaying millimetres to two decimals must not dirty an unchanged page.
        let keep=|old:f32,new:f32|if (old-new).abs()<0.02{old}else{new};
        p.page_w=keep(p.page_w,self.width);p.page_h=keep(p.page_h,self.height);p.landscape=p.page_w>p.page_h;
        p.margin_top=keep(p.margin_top,self.top);p.margin_bottom=keep(p.margin_bottom,self.bottom);p.margin_left=keep(p.margin_left,self.left);p.margin_right=keep(p.margin_right,self.right);Ok(p)
    }
}
pub fn apply(s:&mut Session,setup:PageSetup,whole:bool)->Result<bool,String>{
    let current=page::section_index_of_focus(s);
    let paths=s.doc.para_paths(StoryRef::Body);
    let mut start=0;let mut changes=Vec::new();
    for (i,(end,old)) in s.doc.sections().into_iter().enumerate(){
        if whole||i==current {
            let new=setup.section(old.clone())?;
            if new!=*old {
                let path=paths.iter().find(|p|p.0.first().is_some_and(|n|(*n as usize)>=start&&(*n as usize)<=end)).ok_or("La sección no contiene un párrafo editable")?;
                changes.push((Pos{story:StoryRef::Body,path:path.clone(),off:0},new));
            }
        }start=end+1;
    }
    if changes.is_empty(){return Ok(false);}
    let snap=s.edit_snapshot();let mark=s.undo_mark();let sel=s.sel.clone();let pending=s.pending.clone();
    let result=(||{
        // Anchor the grouped undo before moving to any other section.
        s.checkpoint("Propiedades del documento");
        for (pos,props) in changes {
            s.run("caret.set",&json!({"pos":pos}))?;
            s.join_next_undo();
            s.run("layout.pageSetup",&json!({"section":props}))?;
        }
        s.run("select.range",&json!({"anchor":sel.anchor,"focus":sel.focus}))?;
        Ok::<_,wordcraft_engine::CmdError>(())
    })();
    match result {Ok(())=>{s.pending=pending;s.merge_undo_since(mark);Ok(true)},Err(e)=>{s.restore(snap);s.pending=pending;Err(e.to_string())}}
}
