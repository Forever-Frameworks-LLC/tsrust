//! Return values and precise Map owners share one consuming control payload.
use super::*;
type Layout=std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>;
pub(super) struct Carrier<'a> {
    pub(super) name:String,
    pub(super) native:String,
    pub(super) controls:Vec<(crate::completion::CompletionKind,&'a Layout)>,
    pub(super) layout:&'a Layout,
}
impl Emitter<'_> {
    pub(super) fn map_return_carrier<'a>(&self,fun:&CheckedFunction,name:String,places:&[String],storage:&str,layout:&'a Layout)->Carrier<'a> {
        let result=self.native_type(fun.result);
        let returned=self.map_error_storage_type(fun,places,layout);
        Carrier{controls:vec![],native:format!("{name}<{result},{returned},{storage}>"),name,layout}
    }
    pub(super) fn attach_map_return_storage(&self,fun:&CheckedFunction,body:&[Stmt],places:&[String],carrier:&Carrier<'_>) {
        let value=|place:&str,start:usize,point:Option<(usize,usize)>| {
            let (binding,fields)=self.map_storage_binding(fun,place).expect("checked return Map storage");
            let target=carrier.layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("return Map path"));
            let transferred=point.map_or_else(||fun.storage_exits.transfer_before(start,binding,&fields),|point|fun.storage_exits.transfer_at_expression(point,binding,&fields));
            let input=fun.storage_exits.map_native_input(start,binding).or(Some(fun.bindings[binding].ty)).and_then(|root|self.map_storage_leaf_type(root,&fields));
            self.map_typed_storage_leaf_value(place,transferred,input,target)
        };
        let points=fun.storage_exits.starts().map(|start|(start,places.iter().map(|place|value(place,start,None)).collect())).collect();
        let expressions=fun.storage_exits.expression_points().map(|point|(point,places.iter().map(|place|value(place,point.0,Some(point))).collect())).collect();
        let boundary=body.first().map_or(0,|statement|statement.span.start);
        let initial=places.iter().map(|place|value(place,boundary,None)).collect();
        self.exception_regions.attach_return_storage(carrier.name.clone(),initial,points,expressions);
    }
    #[inline(never)]
    pub(super) fn map_precise_return_cleanup(&self,fun:&CheckedFunction,region:&ExceptionRegion,declarations:&[Stmt],first:usize,_carrier:&Carrier<'_>,cleanup_name:&str,cleanup_contract:&crate::error_effects::FailureSet,runtime:Option<Type>,places:&[String],storage:&str,pattern:&str,span:crate::Span)->String {
        let cleanup=region.finally.as_ref().expect("return cleanup");
        let boundary=region.body.first().map_or(span.start,|statement|statement.span.start);
        let plan=fun.storage_exits.map_cleanup_plan(boundary,Some(crate::completion::CompletionKind::Return)).map(|plan|plan.function(fun));
        let planned=plan.as_ref().unwrap_or(fun);let body=plan.as_ref().map_or(cleanup.as_slice(),|plan|plan.body.as_slice());
        let returned=fun.storage_exits.map_completion_layout(boundary,crate::analysis::MapCompletionPhase::Cleaned,Some(crate::completion::CompletionKind::Return));
        let owner=self.map_outgoing_return_storage(fun,span,returned);
        let resumed=if !crate::completion::definitely_abrupt(cleanup,0){self.exception_regions.dispatch_return("pending_return",span,owner.as_deref())}else{"unreachable!(\"cleanup cannot resume return\")".into()};
        let normal=format!("let {pattern}=storage;{resumed}");
        self.map_finish_cleanup(fun,planned,region,body,declarations,first,cleanup_name,cleanup_contract,runtime,places,storage,pattern,span,returned,&normal,"drop(pending_return);")
    }
}
