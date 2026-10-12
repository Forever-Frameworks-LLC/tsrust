//! Error transport owns the checked throw payloads independently of controls.
use super::*;
type Layout=std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>;
impl Emitter<'_> {
    pub(super) fn map_outgoing_error_storage(&self,fun:&CheckedFunction,span:crate::Span,layout:Option<&Layout>)->Option<String> {
        let layout=layout?;
        let destinations=self.exception_regions.error_storage_values(span)?;
        Some(self.map_project_outgoing_storage(fun,layout,destinations))
    }
    pub(super) fn map_outgoing_return_storage(&self,fun:&CheckedFunction,span:crate::Span,layout:Option<&Layout>)->Option<String> {
        Some(self.map_project_outgoing_storage(fun,layout?,self.exception_regions.return_storage_values(span)?))
    }
    pub(super) fn map_project_outgoing_storage(&self,fun:&CheckedFunction,layout:&Layout,destinations:Vec<native_storage_availability::Value>)->String {
        let values=destinations.into_iter().map(|value| {
            let Some((binding,fields))=self.map_storage_binding(fun,&value.place) else {
                self.field_native_place(fun,&value.place).expect("checked outgoing field storage");
                return value.place;
            };
            let source=layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("checked outgoing Map path"));
            let target=value.target.expect("typed Map destination");
            // The selected completion has already restored an initialized native
            // owner. Source-name availability is not revived by forwarding it.
            let projected=self.map_typed_storage_leaf_value(&value.place,None,Some(source),target);
            projected.live.unwrap_or(projected.place)
        }).collect::<Vec<_>>();
        format!("({},)",values.join(","))
    }

    pub(super) fn map_error_storage_type(&self,fun:&CheckedFunction,places:&[String],layout:&Layout)->String {
        format!("({},)",places.iter().map(|place| {
            let (binding,fields)=self.map_storage_binding(fun,place).expect("checked Map storage");
            let ty=layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("checked Map path"));
            self.native_type(ty)
        }).collect::<Vec<_>>().join(","))
    }
    pub(super) fn attach_map_error_storage(&self,fun:&CheckedFunction,body:&[Stmt],places:&[String],layout:&Layout,fields:&[(String,Type)]) {
        let extra=fields.iter().map(|(place,ty)|native_storage_availability::Value{place:place.clone(),empty:String::new(),live:None,transferred:None,target:Some(*ty)}).collect::<Vec<_>>();
        let value=|place:&str,start:usize,point:Option<(usize,usize)>| {
            let (binding,fields)=self.map_storage_binding(fun,place).expect("checked Map storage");
            let target=layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("checked Map path"));
            let transferred=point.map_or_else(||fun.storage_exits.transfer_before(start,binding,&fields),|point|fun.storage_exits.transfer_at_expression(point,binding,&fields));
            let input=Some(fun.storage_exits.map_native_input(start,binding).unwrap_or(fun.bindings[binding].ty)).and_then(|root|self.map_storage_leaf_type(root,&fields));
            self.map_typed_storage_leaf_value(place,transferred,input,target)
        };
        let points=fun.storage_exits.starts().map(|start|(start,places.iter().map(|place|value(place,start,None)).chain(extra.iter().cloned()).collect())).collect();
        let expressions=fun.storage_exits.expression_points().map(|point|(point,places.iter().map(|place|value(place,point.0,Some(point))).chain(extra.iter().cloned()).collect())).collect();
        let boundary=body.first().map_or(0,|statement|statement.span.start);
        let initial=places.iter().map(|place|value(place,boundary,None)).chain(extra.iter().cloned()).collect();
        self.exception_regions.attach_error_storage(initial,points,expressions);
    }
}

pub(super) fn extend_tuple(base:&str,fields:&[String])->String {
    if fields.is_empty(){base.into()}else{format!("{}{},)",base.strip_suffix(')').expect("native storage tuple"),fields.join(","))}
}
