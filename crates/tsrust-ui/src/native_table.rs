//! Portable shared-column tables. Ordinary semantic View tags keep their explicit layout.
use super::*;

fn rows<'a>(item:&'a Item,output:&mut Vec<&'a Item>){
    for child in &item.children{
        if child.display_none{continue;}
        match child.table_display{
            Some(4)=>output.push(child),
            Some(1..=3)=>rows(child,output),
            _=>{},
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn view(tag:&str,classes:&str,children:Vec<Node>)->Node{
        Node::view(children.into_iter().map(crate::Child::Node).collect(),classes.into()).unwrap().with_semantics(&format!(r#"{{"tag":"{tag}"}}"#)).unwrap()
    }
    fn cell(text:&str,tag:&str,classes:&str)->Node{
        view(tag,&format!("table-cell whitespace-nowrap p-2 align-middle {classes}"),vec![Node::new(Kind::Text,String::new(),text.into(),vec![]).unwrap()])
    }
    #[test]
    fn table_columns_spans_caption_and_overflow_share_native_geometry(){
        let caption=view("caption","table-caption mt-4",vec![Node::new(Kind::Text,String::new(),"Recent invoices".into(),vec![]).unwrap()]);
        let head=view("thead","table-header-group [&_tr]:border-b",vec![view("tr","table-row",vec![cell("Invoice","th","w-[100px]"),cell("A long payment method","th","")])]);
        let body=view("tbody","table-row-group [&_tr:last-child]:border-0",vec![view("tr","table-row border-b",vec![cell("INV001","td",""),cell("Paid","td","")])]);
        let total=cell("Total","td","").with_semantics(r#"{"tag":"td","colSpan":2}"#).unwrap();
        let footer=view("tfoot","table-footer-group",vec![view("tr","table-row",vec![total])]);
        let table=view("table","table caption-bottom w-full",vec![caption,head,body,footer]);
        let root=view("div","overflow-auto w-full min-w-0",vec![table]);
        let mut painter=Painter::new();let (_,tree)=painter.scene(&root,200.,1.,None).unwrap();
        let find=|tag|tree.nodes.iter().filter(|(_,node)|node.role()==tag).map(|(_,node)|node.bounds().unwrap()).collect::<Vec<_>>();
        let columns=find(accesskit::Role::ColumnHeader);let cells=find(accesskit::Role::Cell);
        assert_eq!(columns.len(),2);assert_eq!(cells.len(),3);
        for index in 0..2{assert!((columns[index].x0-cells[index].x0).abs()<0.1);assert!((columns[index].width()-cells[index].width()).abs()<0.1);}
        assert!((cells[2].width()-columns.iter().map(|cell|cell.width()).sum::<f64>()).abs()<0.1);
        let captions=find(accesskit::Role::Caption);assert!(captions[0].y0>cells[2].y0);
        assert!(painter.scrolls.values().any(|region|region.viewport.content.0>region.viewport.size.unwrap().0));
    }
    #[test]
    fn native_table_keeps_pixel_preference_and_excludes_hidden_rows(){
        let header=view("tr","table-row",vec![cell("ID","th","w-[100px]"),cell("Description","th","")]);
        let hidden=view("tr","table-row hidden",vec![cell("A hidden row must not widen this first column","td","")]);
        let table=view("table","table w-full",vec![view("thead","table-header-group",vec![header]),view("tbody","table-row-group",vec![hidden])]);
        let mut painter=Painter::new();let (_,tree)=painter.scene(&table,600.,1.,None).unwrap();
        let headers=tree.nodes.iter().filter(|(_,node)|node.role()==accesskit::Role::ColumnHeader).map(|(_,node)|node.bounds().unwrap()).collect::<Vec<_>>();
        assert_eq!(headers.len(),2);assert!((headers[0].width()-100.).abs()<0.1);assert!((headers[1].width()-500.).abs()<0.1);
    }
}
impl Painter{
    pub(super) fn prepare_tables(&mut self,item:&Item)->Result<bool,String>{
        let mut changed=false;
        for child in &item.children{changed|=self.prepare_tables(child)?;}
        if item.display_none||item.table_display!=Some(0){return Ok(changed);}
        let mut available=self.tree.layout(item.id).map_err(|error|error.to_string())?.size.width;
        for caption in item.children.iter().filter(|child|!child.display_none&&child.table_display==Some(6)){
            self.measure_tree(caption.id,Size{width:AvailableSpace::MinContent,height:AvailableSpace::MaxContent})?;
            available=available.max(self.tree.layout(caption.id).map_err(|error|error.to_string())?.size.width);
        }
        let mut table_rows=Vec::new();rows(item,&mut table_rows);
        let mut measurements=Vec::new();
        for row in &table_rows{
            let mut cells=Vec::new();
            for cell in row.children.iter().filter(|cell|!cell.display_none){
                if cell.table_display!=Some(5){return Err("A table-row requires table-cell children".into());}
                let original=self.tree.style(cell.id).map_err(|error|error.to_string())?.clone();
                let raw=original.size.width.into_raw();
                let hint=if raw.tag()==taffy::style::CompactLength::LENGTH_TAG{Some(raw.value())}else{None};
                let mut intrinsic=original.clone();intrinsic.size.width=auto();
                self.tree.set_style(cell.id,intrinsic).map_err(|error|error.to_string())?;
                let mut measured=|width|->Result<f32,String>{
                    self.measure_tree(cell.id,Size{width,height:AvailableSpace::MaxContent})?;
                    Ok(self.tree.layout(cell.id).map_err(|error|error.to_string())?.size.width)
                };
                let min=measured(AvailableSpace::MinContent)?.max(hint.unwrap_or(0.));
                let max=measured(AvailableSpace::MaxContent)?.max(min);
                self.tree.set_style(cell.id,original).map_err(|error|error.to_string())?;
                cells.push(crate::table::Cell{span:cell.semantics.col_span.unwrap_or(1) as usize,min,max,constrained:hint.is_some()});
            }
            measurements.push(cells);
        }
        let columns=crate::table::columns(&measurements)?;
        let widths=crate::table::fit(&columns,available)?;
        let width=widths.iter().sum::<f32>();
        for row in table_rows{
            let mut style=self.tree.style(row.id).map_err(|error|error.to_string())?.clone();
            style.grid_template_columns=widths.iter().map(|width|length(*width)).collect();
            style.size.width=length(width);style.flex_shrink=0.;
            self.tree.set_style(row.id,style).map_err(|error|error.to_string())?;
            let mut column=1usize;
            for cell in row.children.iter().filter(|cell|!cell.display_none){
                let span=cell.semantics.col_span.unwrap_or(1) as usize;
                // Taffy grid lines use i16. Reject overflow instead of wrapping placement.
                let end=column.checked_add(span).filter(|end|*end<=i16::MAX as usize).ok_or("Native table column count exceeds 32766")?;
                let mut style=self.tree.style(cell.id).map_err(|error|error.to_string())?.clone();
                style.grid_column=Line{start:line(column as i16),end:line(end as i16)};
                style.size.width=auto();style.min_size.width=length(0.);style.flex_shrink=0.;
                self.tree.set_style(cell.id,style).map_err(|error|error.to_string())?;
                column=end;
            }
        }
        let mut style=self.tree.style(item.id).map_err(|error|error.to_string())?.clone();
        style.size.width=length(width.max(available));style.flex_shrink=0.;
        self.tree.set_style(item.id,style).map_err(|error|error.to_string())?;
        // Change visual order only; Item, ownership, focus and accessibility stay in source order.
        let mut children=item.children.iter().collect::<Vec<_>>();
        children.sort_by_key(|child|match child.table_display{Some(6)=>if item.caption_bottom{4}else{0},Some(1)=>1,Some(3)=>3,_=>2});
        self.tree.set_children(item.id,&children.iter().map(|child|child.id).collect::<Vec<_>>()).map_err(|error|error.to_string())?;
        Ok(true)
    }
}
