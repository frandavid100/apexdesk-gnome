use serde_json::{Value,json};
use wordcraft_doc::{Document,StoryRef,Table};
use wordcraft_engine::Session;
use wordcraft_gnome::{selection,tables};
fn run(s:&mut Session,cmd:&str,args:Value){s.run(cmd,&args).unwrap_or_else(|e|panic!("{cmd}: {e}"));}
fn fresh(rows:usize,cols:usize)->Session{let mut s=Session::new(Document::new());run(&mut s,"insert.table",json!({"rows":rows,"cols":cols}));s}
fn table(s:&Session)->&Table{s.doc.body.iter().find_map(|b|b.as_table()).unwrap()}
#[test]
fn structure_and_history_preserve_text(){
 let mut s=fresh(2,2);run(&mut s,"text.insert",json!({"text":"Primera celda: á ñ €"}));
 run(&mut s,"table.insertRowBelow",json!({}));assert_eq!(table(&s).rows.len(),3);
 run(&mut s,"table.insertColumnRight",json!({}));assert_eq!(table(&s).cols(),3);
 run(&mut s,"table.deleteColumn",json!({}));assert_eq!(table(&s).cols(),2);
 run(&mut s,"edit.undo",json!({}));assert_eq!(table(&s).cols(),3);
 run(&mut s,"edit.redo",json!({}));assert_eq!(table(&s).cols(),2);
 run(&mut s,"table.deleteRow",json!({}));assert_eq!(table(&s).rows.len(),2);
 assert!(s.doc.plain_text(StoryRef::Body).contains("á ñ €"));
 run(&mut s,"table.deleteTable",json!({}));assert!(!tables::in_table(&s));
 run(&mut s,"edit.undo",json!({}));assert_eq!(table(&s).rows.len(),2);
}
#[test]
fn tab_navigation_adds_a_row_only_at_the_end(){
 let mut s=fresh(1,2);let rev=s.rev();
 assert!(!tables::navigate(&mut s,false,false).unwrap());assert_eq!(s.rev(),rev);
 assert_eq!(s.sel.focus.path.cell().unwrap().2,1);
 assert!(!tables::navigate(&mut s,true,false).unwrap());assert_eq!(s.sel.focus.path.cell().unwrap().2,0);
 tables::navigate(&mut s,false,false).unwrap();assert!(tables::navigate(&mut s,false,false).unwrap());
 assert_eq!(table(&s).rows.len(),2);assert_eq!(s.sel.focus.path.cell().unwrap().1,1);
 run(&mut s,"edit.undo",json!({}));assert_eq!(table(&s).rows.len(),1);
 assert!(tables::navigate(&mut s,false,true).unwrap());
 assert!(s.doc.para_at(&s.sel.focus).unwrap().plain_text().contains('\t'));
}
#[test]
fn dragged_empty_cells_highlight_merge_split_and_undo(){
 let mut s=fresh(2,3);let a=s.sel.focus.clone();let mut b=a.clone();b.path.0[2]=1;
 selection::select_range(&mut s,&a,&b).unwrap();assert_eq!(tables::selected_cells(&s),2);
 assert!(s.cell_range().is_some());assert_eq!(selection::geometry(&mut s).highlights.len(),2);
 run(&mut s,"table.merge",json!({}));assert_eq!(table(&s).rows[0].cells[0].span(),2);
 run(&mut s,"table.split",json!({"columns":2,"rows":1}));assert_eq!(table(&s).rows[0].cells.len(),3);
 run(&mut s,"edit.undo",json!({}));assert_eq!(table(&s).rows[0].cells[0].span(),2);
 run(&mut s,"edit.undo",json!({}));assert_eq!(table(&s).rows[0].cells.len(),3);
}
#[test]
fn selecting_a_column_does_not_shade_unselected_cells(){
 let mut s=fresh(3,3);run(&mut s,"table.selectColumn",json!({}));assert_eq!(tables::selected_cells(&s),3);
 assert_eq!(selection::geometry(&mut s).highlights.len(),3);
 run(&mut s,"table.shading",json!({"color":"DDEEFF"}));
 assert!(table(&s).rows.iter().all(|r|r.cells[0].props.shading.is_some()&&r.cells[1].props.shading.is_none()));
}
#[test]
fn table_size_content_and_appearance_round_trip_in_docx_and_odt(){
 for ext in ["docx","odt"] {
  let mut s=fresh(2,2);run(&mut s,"text.insert",json!({"text":"Cabecera á ñ"}));
  run(&mut s,"table.properties",json!({"headerRow":true,"align":"center","allowBreak":false}));
  run(&mut s,"table.columnWidth",json!({"width":110}));run(&mut s,"table.rowHeight",json!({"height":30,"rule":"atLeast"}));
  run(&mut s,"table.cellAlign",json!({"value":"center"}));
  run(&mut s,"table.shading",json!({"color":"DDEEFF"}));
  run(&mut s,"table.borders",json!({"kind":"all","width":1,"color":"225588"}));
  tables::navigate(&mut s,false,false).unwrap();run(&mut s,"text.insert",json!({"text":"Segunda"}));
  let path=std::env::temp_dir().join(format!("wc-table-{}.{ext}",std::process::id()));
  wordcraft_engine::io::save_path(&path,&s.doc).unwrap();let reopened=wordcraft_gnome::open_document(&path).unwrap();
  let t=table(&reopened);assert_eq!(t.rows.len(),2);assert_eq!(t.cols(),2);
  let text=reopened.doc.plain_text(StoryRef::Body);assert!(text.contains("Cabecera á ñ")&&text.contains("Segunda"));
  assert!(t.rows[0].cells[0].props.shading.is_some(),"{ext}: shading");
  assert!((t.grid[0]-110.0).abs()<1.0,"{ext}: grid {:?}",t.grid);
  std::fs::remove_file(path).unwrap();
 }
}
#[test]
fn long_table_paginates_and_renders_with_repeated_headers(){
 let mut s=fresh(90,3);run(&mut s,"text.insert",json!({"text":"Cabecera repetida"}));
 run(&mut s,"table.repeatHeader",json!({"value":true}));run(&mut s,"table.selectTable",json!({}));
 run(&mut s,"table.rowHeight",json!({"height":24}));let n=s.layout().pages.len();assert!(n>=2);
 let image=wordcraft_gnome::render_page_premultiplied(&mut s,n-1,0.28).unwrap();assert!(!image.rgba.is_empty());
}

#[test]
fn moving_between_cells_keeps_redo_and_does_not_add_undo_steps(){
 let mut s=fresh(2,2);run(&mut s,"text.insert",json!({"text":"Antes"}));
 run(&mut s,"edit.undo",json!({}));assert!(s.can_redo());let rev=s.rev();
 tables::navigate(&mut s,false,false).unwrap();assert_eq!(s.rev(),rev);assert!(s.can_redo());
 run(&mut s,"edit.redo",json!({}));assert!(s.doc.plain_text(StoryRef::Body).contains("Antes"));
 run(&mut s,"table.selectColumn",json!({}));tables::navigate(&mut s,false,false).unwrap();
 assert!(s.cell_range().is_none());
}
