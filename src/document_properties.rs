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

/// Column layout is still owned by WordCraft, including section creation around selected text.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ColumnKind {Equal,Left,Right,Custom}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ColumnScope {Section,Document,Selection,Forward}
#[derive(Clone,Debug)]
pub struct ColumnSetup {pub count:u32,pub space:f32,pub separator:bool,pub kind:ColumnKind,pub widths:Vec<(f32,f32)>}
impl ColumnSetup {
    pub fn current(s:&Session)->Self {
        let c=page::sect(s).columns;
        Self{count:c.count,space:c.space,separator:c.separator,kind:if c.widths.is_empty(){ColumnKind::Equal}else{ColumnKind::Custom},widths:c.widths}
    }
    pub fn resolved(&self,text_width:f32)->Result<wordcraft_doc::section::Columns,String>{
        use wordcraft_doc::section::Columns;
        let bad=||"Las columnas no caben en el espacio de texto. Reduce su número, anchura o separación.".to_string();
        if !(1..=12).contains(&self.count)||!self.space.is_finite()||!(0.0..=300.0).contains(&self.space){return Err("Usa entre 1 y 12 columnas y una separación de 0 a 105,83 mm.".into());}
        let count=if matches!(self.kind,ColumnKind::Left|ColumnKind::Right){2}else{self.count};
        let widths=match self.kind {
            ColumnKind::Equal=>Vec::new(),
            ColumnKind::Left=>vec![((text_width-self.space)/3.0,self.space),((text_width-self.space)*2.0/3.0,0.0)],
            ColumnKind::Right=>vec![((text_width-self.space)*2.0/3.0,self.space),((text_width-self.space)/3.0,0.0)],
            ColumnKind::Custom=>{
                if self.widths.len()!=count as usize{return Err("Indica una anchura para cada columna.".into());}
                self.widths.iter().enumerate().map(|(i,(w,g))|(*w,if i+1==count as usize{0.0}else{*g})).collect()
            }
        };
        if widths.is_empty(){if (text_width-self.space*(count-1)as f32)/(count as f32)<18.0{return Err(bad());}}
        else if widths.iter().any(|(w,g)|!w.is_finite()||!g.is_finite()||*w<18.0||!(0.0..=300.0).contains(g))||widths.iter().map(|(w,g)|w+g).sum::<f32>()>text_width+0.02{return Err(bad());}
        Ok(Columns{count,space:widths.first().map(|c|c.1).unwrap_or(self.space),separator:self.separator,widths:if count==1{Vec::new()}else{widths}})
    }
}
pub fn apply_columns(s:&mut Session,setup:&ColumnSetup,scope:ColumnScope)->Result<bool,String>{
    if scope==ColumnScope::Selection&&!page::selection_sectionable(s){return Err("Selecciona texto del cuerpo del documento, fuera de las tablas.".into());}
    if scope==ColumnScope::Forward&&(s.sel.focus.story!=StoryRef::Body||s.sel.focus.path.depth()!=0){return Err("Sitúa el cursor en el cuerpo del documento, fuera de una tabla.".into());}
    let focus=page::sect(s);let mut setup=setup.clone();
    let keep=|old:f32,new:f32|if (old-new).abs()<0.02{old}else{new};
    setup.space=keep(focus.columns.space,setup.space);
    for((w,g),(ow,og))in setup.widths.iter_mut().zip(&focus.columns.widths){*w=keep(*ow,*w);*g=keep(*og,*g);}
    let wanted=setup.resolved(focus.text_width())?;
    let current=page::section_index_of_focus(s);let (a,b)=s.sel.ordered();
    let lo=a.path.0.first().copied().unwrap_or(0)as usize;let raw_hi=b.path.0.first().copied().unwrap_or(0)as usize;
    let hi=if b.off==0&&raw_hi>lo&&s.doc.sections().iter().any(|(end,_)|end+1==raw_hi){raw_hi-1}else{raw_hi};
    let near=|a:f32,b:f32|(a-b).abs()<0.02;
    let same=|a:&wordcraft_doc::section::Columns,b:&wordcraft_doc::section::Columns|a.count==b.count&&near(a.space,b.space)&&a.separator==b.separator&&a.widths.len()==b.widths.len()&&a.widths.iter().zip(&b.widths).all(|((w,g),(x,y))|near(*w,*x)&&near(*g,*y));
    let mut start=0;let mut unchanged=true;
    for(i,(end,p))in s.doc.sections().into_iter().enumerate(){
        let target=match scope{ColumnScope::Document=>true,ColumnScope::Selection=>end>=lo&&start<=hi,_=>i==current};
        if target {
            // Presets are resolved at the focus by the upstream command. Refuse overflow in
            // other target sections instead of silently clipping imported mixed-size documents.
            let check=ColumnSetup{kind:if wanted.widths.is_empty(){ColumnKind::Equal}else{ColumnKind::Custom},count:wanted.count,space:wanted.space,separator:wanted.separator,widths:wanted.widths.clone()};
            check.resolved(p.text_width())?;unchanged&=same(&p.columns,&wanted);
        }start=end+1;
    }
    if unchanged{return Ok(false);}
    let snap=s.edit_snapshot();let pending=s.pending.clone();let sel=s.sel.clone();let mark=s.undo_mark();
    let mut args=json!({"count":wanted.count,"space":wanted.space,"separator":wanted.separator,"apply":match scope{ColumnScope::Section=>"section",ColumnScope::Document=>"document",ColumnScope::Selection=>"selection",ColumnScope::Forward=>"forward"}});
    if !wanted.widths.is_empty(){args["widths"]=json!(wanted.widths);}
    let result=(||{
        s.checkpoint("Columnas");
        if scope==ColumnScope::Section{s.run("caret.set",&json!({"pos":sel.focus}))?;}
        s.join_next_undo();s.run("layout.columns",&args)?;
        if matches!(scope,ColumnScope::Section|ColumnScope::Document){s.run("select.range",&json!({"anchor":sel.anchor,"focus":sel.focus}))?;}
        Ok::<_,wordcraft_engine::CmdError>(())
    })();
    match result {Ok(())=>{s.pending=pending;s.merge_undo_since(mark);Ok(true)},Err(e)=>{s.restore(snap);s.pending=pending;Err(e.to_string())}}
}
