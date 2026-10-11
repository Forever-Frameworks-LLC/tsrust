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
    pub(super) fn map_direct_return_route(&self,carrier:&Carrier<'_>,pattern:&str,cleanup:String,id:usize)->exception_contract_emit::InlineExit {
        let binding=format!("direct_map_return{id}");
        exception_contract_emit::InlineExit{binding:binding.clone(),ty:carrier.native.clone(),code:format!("let (pending_return,{pattern})={{let completed={binding};match completed{{{}::Return(owner)=>owner,_=>unreachable!(\"selected Map return\")}}}};{cleanup};unreachable!(\"Map return cleanup exits\");",carrier.name)}
    }
    pub(super) fn attach_map_return_storage(&self,fun:&CheckedFunction,body:&[Stmt],places:&[String],carrier:&Carrier<'_>) {
        let value=|place:&str,start:usize,point:Option<(usize,usize)>| {
            let (binding,fields)=self.map_storage_binding(fun,place).expect("checked return Map storage");
            let target=carrier.layout.get(&crate::capture_loan_owners::Place{binding,fields:fields.clone()}).copied().unwrap_or_else(||self.map_storage_leaf_type(fun.bindings[binding].ty,&fields).expect("return Map path"));
            let transferred=point.map_or_else(||fun.storage_exits.transfer_before(start,binding,&fields),|point|fun.storage_exits.transfer_at_expression(point,binding,&fields));
            let input=fun.storage_exits.map_native_input(start,binding).or(Some(fun.bindings[binding].ty)).and_then(|root|self.map_storage_leaf_type(root,&fields));
            self.map_typed_storage_leaf_value(place,transferred,input,target)
        };
        // A phase carrier describes only this body. Later phases can hold a
        // disjoint callback origin and must not be projected into an earlier return.
        let start=body.first().map_or(0,|statement|statement.span.start);
        let end=body.last().map_or(start,|statement|statement.span.end);
        let points=fun.storage_exits.starts().filter(|point|*point>=start && *point<end).map(|start|(start,places.iter().map(|place|value(place,start,None)).collect())).collect();
        let expressions=fun.storage_exits.expression_points().filter(|point|point.0>=start && point.1<=end).map(|point|(point,places.iter().map(|place|value(place,point.0,Some(point))).collect())).collect();
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

#[cfg(test)]
mod tests {
    #[test]
    fn direct_map_return_keeps_payload_allocation_and_exact_environment_drop_order() {
        let source="type Row={name:string;other:string;};type Holder={run:ReadOnly<Function<int>>;};function observe(value:int):void{}function pick(a:Row,early:boolean):string{const old=[3];const current=[7];const handlers=new Map<string,Holder>();handlers.set(\"old\",{run:():int=>old[0]});try{if(early){return a.name;}handlers.clear();handlers.set(\"new\",{run:():int=>current[0]});}finally{const item=handlers.get(early?\"old\":\"new\");if(item!==undefined){observe(item.run());}else{throw 99;}}return a.name;}function main(){}";
        let source=crate::formatting::format(source).unwrap();assert_eq!(crate::formatting::format(&source).unwrap(),source);
        let checked=crate::analysis::analyze(crate::syntax::parse(&source).unwrap()).unwrap();let mut native=crate::emit::emit(&checked);
        let function=native.find("fn function0(").unwrap();let opening=function+native[function..].find('{').unwrap()+1;
        native.insert_str(opening,"use std::sync::atomic::Ordering::SeqCst;assert_eq!(value0,EXPECTED.load(SeqCst));assert_eq!(OLD.load(SeqCst),usize::from(value0==7));assert_eq!(NEW.load(SeqCst),0);");
        let main=native.find("fn main()").unwrap();let opening=main+native[main..].find('{').unwrap()+1;native.insert_str(opening,"map_return_probe();");
        native.push_str(r#"
static EXPECTED:std::sync::atomic::AtomicI64=std::sync::atomic::AtomicI64::new(0);
static OLD:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static NEW:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl<'a> Drop for Closure0<'a>{fn drop(&mut self){OLD.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
impl<'a> Drop for Closure1<'a>{fn drop(&mut self){NEW.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
fn map_return_probe(){use std::sync::atomic::Ordering::SeqCst;for early in [true,false]{
 OLD.store(0,SeqCst);NEW.store(0,SeqCst);EXPECTED.store(if early{3}else{7},SeqCst);
 let row=Record0{field0:String::Owned("payload".into()),field1:String::Owned("sibling".into())};let pointer=row.field0.as_ptr();
 let returned=match tsr_result_function1(row,early){Ok(value)=>value,Err(_)=>panic!("unexpected source error")};assert_eq!(returned.as_ptr(),pointer,"Map return cloned the owned field");
 assert_eq!(OLD.load(SeqCst),1);assert_eq!(NEW.load(SeqCst),usize::from(!early));drop(returned);
}}
"#);
        let directory=std::env::temp_dir().join(format!("tsrust-map-return-probe-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());let input=directory.join("probe.rs");std::fs::write(&input,native).unwrap();
        for level in ["0","3"]{let binary=directory.join(format!("probe-{level}"));let compiled=std::process::Command::new("rustc").args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&input).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());}
    }
}
