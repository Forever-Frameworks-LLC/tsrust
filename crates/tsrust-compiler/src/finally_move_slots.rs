//! Owned roots moved through cleanup need an explicit initialized state in Rust.
//! Source availability checking remains authoritative; these slots never clone owners.
use crate::analysis::{CheckedFunction,CheckedProgram};
use crate::syntax::{ExprKind,StmtKind};
use std::collections::{HashMap,HashSet};

#[derive(Default)]
pub(super) struct Index(HashMap<usize,HashSet<usize>>);
impl Index {
    pub(super) fn new(program:&CheckedProgram)->Self {
        let mut index=Self::default();
        for function in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            let mut slots=HashSet::new();
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
                    StmtKind::Return(Some(value)) if cleanup.is_some()=>{
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
    pub(super) fn parameters(&self,function:&CheckedFunction)->Vec<usize> {
        let mut slots=self.0.get(&(function as *const CheckedFunction as usize)).into_iter().flatten().copied().filter(|&slot|function.bindings[slot].parameter).collect::<Vec<_>>();slots.sort_unstable();slots
    }
}

#[cfg(test)]
mod tests {
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
