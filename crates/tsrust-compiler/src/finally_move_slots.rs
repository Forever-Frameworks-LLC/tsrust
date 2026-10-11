//! Owned roots moved through cleanup need an explicit initialized state in Rust.
//! Source availability checking remains authoritative; these slots never clone owners.
use crate::analysis::{CheckedFunction,CheckedProgram};
use crate::syntax::{ExprKind,StmtKind};
use std::collections::{HashMap,HashSet};

#[derive(Default)]
pub(super) struct Index(HashMap<usize,HashSet<usize>>,HashMap<usize,HashSet<usize>>);
impl Index {
    pub(super) fn new(program:&CheckedProgram)->Self {
        let mut index=Self::default();
        for function in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            let mut slots=HashSet::new();
            let mut partial=HashSet::new();
            let mut boundaries=HashMap::<usize,usize>::new();
            let mut pending=function.body.iter().map(|statement|(statement,None)).collect::<Vec<_>>();
            while let Some((statement,cleanup))=pending.pop() {
                match &statement.kind {
                    StmtKind::Try(region)=>{
                        let guarded=region.finally.as_ref().map(|_|statement.span.end).or(cleanup);
                        pending.extend(region.body.iter().map(|statement|(statement,guarded)));
                        if let Some(handler)=&region.catch {pending.extend(handler.body.iter().map(|statement|(statement,guarded)));}
                        if let Some(body)=&region.finally {pending.extend(body.iter().map(|statement|(statement,cleanup)));}
                    }
                    StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body.iter().map(|statement|(statement,cleanup))),
                    StmtKind::Loop{body,update,..}=>pending.extend(body.iter().chain(update).map(|statement|(statement,cleanup))),
                    StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body).map(|statement|(statement,cleanup))),
                    StmtKind::Return(Some(value))=>{
                        partial.extend(function.storage_exits.partial_places_within((statement.span.start,statement.span.end),value.span).into_iter().map(|place|place.binding));
                        if cleanup.is_none(){continue;}
                        // The checked return snapshot records actual transfers, including
                        // calls and closure captures. Borrowed arguments are not moves.
                        let roots=function.bindings.iter().enumerate().filter_map(|(id,binding)|{
                            let moved=function.storage_exits.transfer_at_expression((statement.span.start,statement.span.end),id,&[])?;
                            (!binding.ty.is_copy() && !binding.capture_reference && binding.borrowed_place.is_none() && !binding.mutable_place
                                && moved.start>=value.span.start && moved.end<=value.span.end).then_some(id)
                        });
                        for root in roots {boundaries.entry(root).and_modify(|end|*end=(*end).min(cleanup.unwrap())).or_insert(cleanup.unwrap());slots.insert(root);}
                    },
                    _=>{}
                }
            }
            if !partial.is_empty(){index.1.insert(function as *const CheckedFunction as usize,partial);}
            if slots.is_empty(){continue;}
            let expressions=crate::error_effects::checked_expressions(function);
            slots.retain(|root|expressions.iter().any(|value|value.span.start>=boundaries[root]
                && (value.binding==Some(*root) || matches!(value.kind,ExprKind::Closure{..})
                    && value.closure_origin().or_else(||match value.ty {Some(crate::syntax::Type::Closure(id))=>Some(id),_=>None}).is_some_and(|id|program.closures[id].captures.iter().any(|capture|capture.outer==*root)))));
            if !slots.is_empty(){index.0.insert(function as *const CheckedFunction as usize,slots);}
        }
        index
    }
    pub(super) fn contains(&self,function:&CheckedFunction,slot:usize)->bool {
        self.0.get(&(function as *const CheckedFunction as usize)).is_some_and(|slots|slots.contains(&slot))
    }
    pub(super) fn partial_root(&self,function:&CheckedFunction,slot:usize)->bool {
        self.1.get(&(function as *const CheckedFunction as usize)).is_some_and(|slots|slots.contains(&slot))
    }
    pub(super) fn parameters(&self,function:&CheckedFunction)->Vec<usize> {
        let mut slots=self.0.get(&(function as *const CheckedFunction as usize)).into_iter().flatten().copied().filter(|&slot|function.bindings[slot].parameter).collect::<Vec<_>>();slots.sort_unstable();slots
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn try_locals_drop_before_finally_on_both_return_and_normal_paths() {
        assert_try_local_drops_before_cleanup(LocalCleanup::WholeReturn);
    }
    #[test]
    fn try_locals_drop_before_partial_field_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::PartialReturn);
    }
    #[test]
    fn conditionally_partial_try_locals_drop_before_whole_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::PartialLocal);
    }
    #[test]
    fn map_try_locals_drop_before_whole_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::MapReturn);
    }
    #[test]
    fn nested_try_locals_drop_before_partial_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::NestedPartialReturn);
    }
    #[test]
    fn catch_locals_drop_before_partial_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::CatchPartialReturn);
    }
    #[test]
    fn returned_local_field_keeps_its_sibling_lexical_drop() {
        assert_try_local_drops_before_cleanup(LocalCleanup::LocalPartialReturn);
    }
    #[test]
    fn map_try_locals_drop_before_partial_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::MapPartialReturn);
    }
    #[test]
    fn unconditional_partial_return_preserves_lexical_disposal() {
        assert_try_local_drops_before_cleanup(LocalCleanup::AlwaysPartialReturn);
    }
    #[test]
    fn map_catch_locals_drop_before_partial_return_cleanup() {
        assert_try_local_drops_before_cleanup(LocalCleanup::MapCatchPartialReturn);
    }
    #[test]
    fn map_terminal_partial_returns_preserve_lexical_disposal() {
        assert_try_local_drops_before_cleanup(LocalCleanup::MapTerminalPartialReturn);
    }
    #[test]
    fn map_replacement_cleanup_keeps_body_local_disposal_order() {
        assert_try_local_drops_before_cleanup(LocalCleanup::MapReplacementPartialReturn);
    }
    #[test]
    #[ignore = "Escaping loop controls still run cleanup before try-local disposal"]
    fn loop_control_partial_returns_keep_lexical_disposal() {
        assert_try_local_drops_before_cleanup(LocalCleanup::ControlPartialReturn);
    }
    #[derive(Debug)]
    enum LocalCleanup {WholeReturn,PartialReturn,PartialLocal,MapReturn,NestedPartialReturn,CatchPartialReturn,LocalPartialReturn,MapPartialReturn,AlwaysPartialReturn,MapCatchPartialReturn,MapTerminalPartialReturn,MapReplacementPartialReturn,ControlPartialReturn}
    fn assert_try_local_drops_before_cleanup(case:LocalCleanup) {
        let partial=matches!(case,LocalCleanup::PartialReturn|LocalCleanup::NestedPartialReturn|LocalCleanup::CatchPartialReturn|LocalCleanup::LocalPartialReturn|LocalCleanup::MapPartialReturn|LocalCleanup::AlwaysPartialReturn|LocalCleanup::MapCatchPartialReturn|LocalCleanup::MapTerminalPartialReturn|LocalCleanup::MapReplacementPartialReturn|LocalCleanup::ControlPartialReturn);
        let local_partial=matches!(case,LocalCleanup::PartialLocal);
        let map=matches!(case,LocalCleanup::MapReturn|LocalCleanup::MapPartialReturn|LocalCleanup::MapCatchPartialReturn|LocalCleanup::MapTerminalPartialReturn|LocalCleanup::MapReplacementPartialReturn);
        let source=if matches!(case,LocalCleanup::ControlPartialReturn) {
            "type Marker={value:int;};type Row={name:string;};function cleanup():void{}function pick(a:Row,early:boolean):string{for(let i=0;i<1;i+=1){try{const marker:Marker={value:1};if(early){return a.name;}break;}finally{cleanup();}}return a.name;}function main(){}"
        }else if matches!(case,LocalCleanup::AlwaysPartialReturn) {
            "type Marker={value:int;};type Row={name:string;};function cleanup():void{}function pick(a:Row,early:boolean):string{try{const marker:Marker={value:1};return a.name;}finally{cleanup();}}function main(){}"
        }else if matches!(case,LocalCleanup::LocalPartialReturn) {
            "type Marker={value:int;};type Row={name:string;};type Local={name:string;marker:Marker;};function cleanup():void{}function pick(a:Row,early:boolean):string{try{const local:Local={name:a.name,marker:{value:1}};if(early){return local.name;}}finally{cleanup();}return \"owner\";}function main(){}"
        }else if matches!(case,LocalCleanup::CatchPartialReturn) {
            "type Marker={value:int;};type Row={name:string;};function cleanup():void{}function pick(a:Row,early:boolean):string{try{throw 7;}catch(error){const marker:Marker={value:1};if(early){return a.name;}}finally{cleanup();}return a.name;}function main(){}"
        }else if matches!(case,LocalCleanup::NestedPartialReturn) {
            "type Marker={value:int;};type Row={name:string;};function cleanup():void{}function pick(a:Row,early:boolean):string{try{const marker:Marker={value:1};try{if(early){return a.name;}}finally{}}finally{cleanup();}return a.name;}function main(){}"
        }else if map {
            "type Marker={value:int;};type Holder={run:ReadOnly<Function<int>>;};function cleanup():void{}function pick(a:string,early:boolean):string{const old=[3];const current=[7];const handlers=new Map<string,Holder>();handlers.set(\"old\",{run:():int=>old[0]});try{const marker:Marker={value:1};if(early){return a;}handlers.clear();handlers.set(\"new\",{run:():int=>current[0]});}finally{cleanup();const item=handlers.get(early?\"old\":\"new\");if(item!==undefined){if(item.run()!==(early?3:7)){throw 99;}}}return a;}function main(){}"
        }else if local_partial {
            "type Marker={value:int;};type Pair={left:Marker;right:Marker;};function cleanup():void{}function pick(a:string,early:boolean):string{try{const pair:Pair={left:{value:1},right:{value:2}};if(early){consume(pair.left);return a;}}finally{cleanup();}return a;}function consume(value:Marker):void{}function main(){}"
        }else if partial {
            "type Marker={value:int;};type Row={name:string;};function cleanup():void{}function pick(a:Row,early:boolean):string{try{const marker:Marker={value:1};if(early){return a.name;}}finally{cleanup();}return a.name;}function main(){}"
        }else{"type Marker={value:int;};function cleanup():void{}function pick(a:string,early:boolean):string{try{const marker:Marker={value:1};if(early){return a;}}finally{cleanup();}return a;}function main(){}"};
        let source=if matches!(case,LocalCleanup::MapPartialReturn|LocalCleanup::MapCatchPartialReturn|LocalCleanup::MapTerminalPartialReturn|LocalCleanup::MapReplacementPartialReturn){source.replace("type Holder", "type Row={name:string;};type Holder").replace("a:string", "a:Row").replace("return a;", "return a.name;")}else{source.to_owned()};
        let source=if matches!(case,LocalCleanup::MapCatchPartialReturn){source.replace("try{const marker", "try{throw 7;}catch(error){const marker")}else{source};
        let source=if matches!(case,LocalCleanup::MapTerminalPartialReturn){source.replace(";}finally{cleanup();", ";return a.name;}finally{cleanup();").replace("}return a.name;}function main", "}}function main")}else{source};
        let source=if matches!(case,LocalCleanup::MapReplacementPartialReturn){source.replace("}return a.name;}", "return \"owner\";}}") }else{source};
        let formatted=crate::formatting::format(&source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let checked=crate::analysis::analyze(crate::syntax::parse(&formatted).unwrap()).unwrap();
        let mut native=crate::emit::emit(&checked);
        let cleanup=native.find("fn function0(").unwrap();let opening=cleanup+native[cleanup..].find('{').unwrap()+1;
        let expected=if local_partial{2}else{1};
        native.insert_str(opening,&format!("assert_eq!(LOCAL_DROPS.load(std::sync::atomic::Ordering::SeqCst),{expected},\"try local survived into finally\");"));
        let main=native.find("fn main()").unwrap();let opening=main+native[main..].find('{').unwrap()+1;
        let owner=if partial{"Record1{field0:String::Owned(\"owner\".into())}"}else{"String::Owned(\"owner\".into())"};
        let call=if map || matches!(case,LocalCleanup::ControlPartialReturn){"tsr_result_function1(owned_arg,early).unwrap_or_else(|_|panic!(\"unexpected source error\"))"}else{"function1(owned_arg,early)"};
        let pointer=if partial{"owned_arg.field0.as_ptr()"}else{"owned_arg.as_ptr()"};
        let identity=if matches!(case,LocalCleanup::LocalPartialReturn){"early"}else if matches!(case,LocalCleanup::MapReplacementPartialReturn){"false"}else{"true"};
        native.insert_str(opening,&format!("for early in [false,true]{{LOCAL_DROPS.store(0,std::sync::atomic::Ordering::SeqCst);let owned_arg={owner};let pointer={pointer};let returned={call};assert_eq!(returned,\"owner\");if {identity}{{assert_eq!(returned.as_ptr(),pointer,\"return cloned allocation\");}}assert_eq!(LOCAL_DROPS.load(std::sync::atomic::Ordering::SeqCst),{expected});}}"));
        native.push_str("static LOCAL_DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);impl Drop for Record0{fn drop(&mut self){LOCAL_DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}");
        let directory=std::env::temp_dir().join(format!("tsrust-finally-local-drops-{}-{case:?}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());let input=directory.join("probe.rs");std::fs::write(&input,native).unwrap();
        for level in ["0","3"]{let binary=directory.join(format!("probe-{level}"));let compiled=std::process::Command::new("rustc").args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&input).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());}
    }

    #[test]
    fn finally_return_slots_keep_allocation_and_exact_drop_order() {
        let source="type Row={name:string;};function cleanup():void{}function pick(a:Row,early:boolean):Row{try{if(early){return a;}}finally{cleanup();}return a;}function replace(a:Row,early:boolean):Row{try{if(early){return a;}}finally{cleanup();if(early){return {name:\"replacement\"};}}return a;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let checked=crate::analysis::analyze(crate::syntax::parse(&formatted).unwrap()).unwrap();
        let mut native=crate::emit::emit(&checked);
        let cleanup=native.find("fn function0(").unwrap();
        let opening=cleanup+native[cleanup..].find('{').unwrap()+1;
        native.insert_str(opening,"assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),0,\"owner disposed before cleanup\");");
        let main=native.find("fn main()").unwrap();let opening=main+native[main..].find('{').unwrap()+1;
        native.insert_str(opening,"slot_probe();");
        native.push_str(r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
fn slot_probe(){
    use std::sync::atomic::Ordering::SeqCst;
    for early in [false,true] {
        DROPS.store(0,SeqCst);
        let owner=Record0{field0:String::Owned("owner".into())};let pointer=owner.field0.as_ptr();
        let returned=function1(owner,early);
        assert_eq!(returned.field0.as_ptr(),pointer,"return cloned allocation");
        assert_eq!(DROPS.load(SeqCst),0);drop(returned);assert_eq!(DROPS.load(SeqCst),1);
        DROPS.store(0,SeqCst);
        let owner=Record0{field0:String::Owned("owner".into())};let returned=function2(owner,early);
        assert_eq!(returned.field0.as_ref(),if early{"replacement"}else{"owner"});
        assert_eq!(DROPS.load(SeqCst),usize::from(early));drop(returned);
        assert_eq!(DROPS.load(SeqCst),if early{2}else{1});
    }
}
"#);
        let directory=std::env::temp_dir().join(format!("tsrust-finally-return-slots-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());let input=directory.join("probe.rs");std::fs::write(&input,native).unwrap();
        for level in ["0","3"] {
            let binary=directory.join(format!("probe-{level}"));
            let compiled=std::process::Command::new("rustc").args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&input).arg("-o").arg(&binary).output().unwrap();
            assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
            assert!(std::process::Command::new(binary).status().unwrap().success());
        }
    }
}

#[cfg(test)]
mod partial_return_tests {
    #[test]
    fn partial_finally_returns_keep_sibling_alive_and_dispose_each_owner_once() {
        let source="type Token={text:string;};type Pair={left:Token;right:Token;};function cleanup(value:ReadOnly<Token>):void{}function pick(a:Pair,early:boolean):Token{try{if(early){return a.left;}}finally{cleanup(a.right);}return a.left;}function replace(a:Pair,early:boolean):Token{try{if(early){return a.left;}}finally{cleanup(a.right);if(early){return a.right;}}return a.left;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let checked=crate::analysis::analyze(crate::syntax::parse(&formatted).unwrap()).unwrap();
        let mut native=crate::emit::emit(&checked);
        let cleanup=native.find("fn function0(").unwrap();let opening=cleanup+native[cleanup..].find('{').unwrap()+1;
        native.insert_str(opening,"assert_eq!(LEFT.load(std::sync::atomic::Ordering::SeqCst),0);assert_eq!(RIGHT.load(std::sync::atomic::Ordering::SeqCst),0);assert_eq!(value0.field0.as_ref(),\"right\");");
        let main=native.find("fn main()").unwrap();let opening=main+native[main..].find('{').unwrap()+1;native.insert_str(opening,"partial_probe();");
        native.push_str(r#"
static LEFT:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static RIGHT:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0{fn drop(&mut self){let counter=if self.field0.as_ref()=="left"{&LEFT}else{&RIGHT};counter.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
fn partial_probe(){use std::sync::atomic::Ordering::SeqCst;
 for early in [false,true]{for replace in [false,true]{
  LEFT.store(0,SeqCst);RIGHT.store(0,SeqCst);
  let left=Record0{field0:String::Owned("left".into())};let right=Record0{field0:String::Owned("right".into())};
  let pointer=if replace && early{right.field0.as_ptr()}else{left.field0.as_ptr()};
  let pair=Record1{field0:left,field1:right};let returned=if replace{function2(pair,early)}else{function1(pair,early)};
  assert_eq!(returned.field0.as_ptr(),pointer,"partial return cloned payload");
  assert_eq!(LEFT.load(SeqCst),usize::from(replace&&early));assert_eq!(RIGHT.load(SeqCst),usize::from(!(replace&&early)));
  drop(returned);assert_eq!(LEFT.load(SeqCst),1);assert_eq!(RIGHT.load(SeqCst),1);
 }}
}
"#);
        let directory=std::env::temp_dir().join(format!("tsrust-finally-partial-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());let input=directory.join("probe.rs");std::fs::write(&input,native).unwrap();
        for level in ["0","3"]{let binary=directory.join(format!("probe-{level}"));let compiled=std::process::Command::new("rustc").args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&input).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());}
    }
}

#[cfg(test)]
mod cleanup_size_tests {
    #[test]
    fn nested_finally_direct_exit_code_does_not_grow_exponentially() {
        let emit=|depth|{
            let mut body="if(early){return a.name;}".to_owned();
            for _ in 0..depth {body=format!("try{{{body}}}finally{{console.log(a.other);}}");}
            let source=format!("type Row={{name:string;other:string;}};function pick(a:Row,early:boolean):string{{{body}return a.name;}}function main(){{}}");
            let formatted=crate::formatting::format(&source).unwrap();assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
            let program=crate::analysis::analyze(crate::syntax::parse(&formatted).unwrap()).unwrap();crate::emit::emit(&program).len()
        };
        let shallow=emit(6);let deep=emit(10);
        assert!(deep<shallow*3,"nested cleanup duplicated abrupt alternatives: {shallow} -> {deep}");
    }
}
