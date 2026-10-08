use super::exception_emit::fallible_call;
#[cfg(test)]
use super::exception_emit::typed_error_enum;

#[derive(Default)]
pub(crate) struct ControlStack(std::cell::RefCell<Vec<crate::syntax::ControlTarget>>);
pub(crate) struct ControlScope<'a> {stack:&'a ControlStack,previous_depth:usize}
pub(crate) struct ControlBoundary<'a> {stack:&'a ControlStack,previous:Vec<crate::syntax::ControlTarget>}
impl ControlStack {
    pub(crate) fn callable_boundary(&self)->ControlBoundary<'_> {
        ControlBoundary{stack:self,previous:std::mem::take(&mut *self.0.borrow_mut())}
    }
    pub(crate) fn enter(&self,target:crate::syntax::ControlTarget)->ControlScope<'_> {
        let mut targets=self.0.borrow_mut();let previous_depth=targets.len();targets.push(target);
        ControlScope{stack:self,previous_depth}
    }
    pub(crate) fn snapshot(&self)->Vec<crate::syntax::ControlTarget> {self.0.borrow().clone()}
}
impl Drop for ControlBoundary<'_> {
    fn drop(&mut self) {*self.stack.0.borrow_mut()=std::mem::take(&mut self.previous);}
}
impl Drop for ControlScope<'_> {
    fn drop(&mut self) {self.stack.0.borrow_mut().truncate(self.previous_depth);}
}

/// Consume a checked throw value into the enclosing transport error layout.
pub(crate) fn checked_throw_value(value:&str,ty:crate::syntax::Type,error_name:&str,destination:&crate::error_effects::FailureSet,unions:&crate::unions::UnionArena,runtime:Option<crate::syntax::Type>,span:crate::Span)->Result<String,crate::Diagnostic> {
    let error_name=super::exception_emit::constructor_path(error_name);
    let source=crate::error_effects::FailureSet{explicit:vec![ty],..Default::default()};
    let mapping=source.native_injection(destination,unions,runtime,span)?;
    if let crate::syntax::Type::Union(id)=ty {
        let arms=mapping.iter().enumerate().map(|(index,target)|format!("Union{}::V{index}(payload)=>{error_name}::V{target}(payload)",id.index())).collect::<Vec<_>>().join(",");
        Ok(format!("match {value} {{ {arms} }}"))
    }else{
        let target=mapping.first().expect("checked throw payload");
        Ok(format!("{error_name}::V{target}({value})"))
    }
}

/// Move transport alternatives into the language-level catch binding type.
pub(crate) fn checked_catch_value(value:&str,error_name:&str,contract:&crate::error_effects::FailureSet,unions:&crate::unions::UnionArena,runtime:Option<crate::syntax::Type>,span:crate::Span)->Result<String,crate::Diagnostic> {
    let error_name=super::exception_emit::constructor_path(error_name);
    let catch=contract.catch_type(unions,runtime,span)?;
    let payloads=contract.native_payloads(unions,runtime,span)?;
    let arms=payloads.iter().enumerate().map(|(index,_)| {
        let destination=match catch {
            Some(crate::syntax::Type::Union(id))=>format!("Union{}::V{index}(payload)",id.index()),
            _=>"payload".into(),
        };
        format!("{error_name}::V{index}(payload)=>{destination}")
    }).collect::<Vec<_>>().join(",");
    Ok(if arms.is_empty() {format!("match {value} {{}}") }else{format!("match {value} {{ {arms} }}")})
}

struct Storage { carry_error:bool, default:String, points:std::collections::BTreeMap<usize,String>, expressions:std::collections::BTreeMap<(usize,usize),String> }
impl Storage {
    fn at(&self, span:crate::Span)->&str { self.expressions.get(&(span.start,span.end)).map(String::as_str).unwrap_or_else(||self.points.range(..=span.start).next_back().map_or(self.default.as_str(),|(_,value)|value.as_str())) }
}

struct NativeRegion {
    id:usize,
    locals:Vec<crate::syntax::ControlTarget>,
    storage:Option<Box<Storage>>,
    transport:Option<Box<(String,crate::error_effects::FailureSet,Option<crate::syntax::Type>)>>,
}

#[derive(Default)]
pub(crate) struct RegionStack(std::cell::RefCell<Vec<NativeRegion>>);

pub(crate) struct RegionScope<'a> { stack:&'a RegionStack, previous_depth:usize }
pub(crate) struct CallableScope<'a> { stack:&'a RegionStack, previous:Vec<NativeRegion> }

impl RegionStack {
    pub(crate) fn callable_boundary(&self)->CallableScope<'_> {
        CallableScope{stack:self,previous:std::mem::take(&mut *self.0.borrow_mut())}
    }
    pub(crate) fn enter(&self,id:usize,locals:Vec<crate::syntax::ControlTarget>)->RegionScope<'_> {
        let mut regions=self.0.borrow_mut();
        let previous_depth=regions.len();
        regions.push(NativeRegion{id,locals,storage:None,transport:None});
        RegionScope{stack:self,previous_depth}
    }
    pub(crate) fn enter_typed(&self,id:usize,locals:Vec<crate::syntax::ControlTarget>,name:String,contract:crate::error_effects::FailureSet,runtime:Option<crate::syntax::Type>)->RegionScope<'_> {
        let scope=self.enter(id,locals);
        self.0.borrow_mut().last_mut().expect("entered region").transport=Some(Box::new((name,contract,runtime)));
        scope
    }
    pub(crate) fn attach_storage(&self, storage:String, points:std::collections::BTreeMap<usize,String>, expressions:std::collections::BTreeMap<(usize,usize),String>, carry_error:bool) {
        self.0.borrow_mut().last_mut().expect("entered storage region").storage=Some(Box::new(Storage{carry_error,default:storage,points,expressions}));
    }
    pub(crate) fn has_native_storage(&self)->bool { self.0.borrow().last().is_some_and(|region|region.storage.is_some()) }
    pub(crate) fn throw_value(&self,value:&str,ty:crate::syntax::Type,unions:&crate::unions::UnionArena,span:crate::Span)->Result<String,crate::Diagnostic> {
        let regions=self.0.borrow();
        let region=regions.last().ok_or_else(||crate::Diagnostic::new("error-effect","Throw emission requires a typed completion region.",span))?;
        let (name,contract,runtime)=region.transport.as_deref().ok_or_else(||crate::Diagnostic::new("error-effect","Throw emission requires a destination error layout.",span))?;
        let value=checked_throw_value(value,ty,name,contract,unions,*runtime,span)?;
        let value=region.storage.as_ref().filter(|storage|storage.carry_error).map_or(value.clone(),|storage|format!("({value},{})",storage.at(span)));
        Ok(format!("break 'exception_region{} TsrCompletion::Throw({value});",region.id))
    }
    /// Resolve a declared callee's saved effect contract before emitting its Result bridge.
    #[inline(never)]
    pub(crate) fn function_result_call(&self,call:&str,index:usize,source_name:&str,effects:&crate::error_effects::ErrorEffects,unions:&crate::unions::UnionArena,span:crate::Span)->Result<String,crate::Diagnostic> {
        let source=effects.function_errors.get(index).ok_or_else(||crate::Diagnostic::new("error-effect","Result call requires a saved callee error contract.",span))?;
        let summary=effects.functions.get(index).ok_or_else(||crate::Diagnostic::new("error-effect","Result call requires a saved callee effect summary.",span))?;
        if *summary != (source.runtime || source.opaque || !source.explicit.is_empty()) {
            return Err(crate::Diagnostic::new("error-effect","Callee effect summary disagrees with its saved error contract.",span));
        }
        self.result_call(call,source_name,source,unions,span)
    }
    /// Resolve a closure's saved invocation contract before emitting its Result bridge.
    #[inline(never)]
    pub(crate) fn closure_result_call(&self,call:&str,index:usize,source_name:&str,effects:&crate::error_effects::ErrorEffects,unions:&crate::unions::UnionArena,span:crate::Span)->Result<String,crate::Diagnostic> {
        let source=effects.closure_errors.get(index).ok_or_else(||crate::Diagnostic::new("error-effect","Result call requires a saved closure error contract.",span))?;
        let summary=effects.closures.get(index).ok_or_else(||crate::Diagnostic::new("error-effect","Result call requires a saved closure effect summary.",span))?;
        if *summary != (source.runtime || source.opaque || !source.explicit.is_empty()) {
            return Err(crate::Diagnostic::new("error-effect","Closure effect summary disagrees with its saved error contract.",span));
        }
        self.result_call(call,source_name,source,unions,span)
    }
    /// Propagate a checked Result call through the active region without copying its payload.
    #[inline(never)]
    pub(crate) fn result_call(&self,call:&str,source_name:&str,source:&crate::error_effects::FailureSet,unions:&crate::unions::UnionArena,span:crate::Span)->Result<String,crate::Diagnostic> {
        let regions=self.0.borrow();
        let region=regions.last().ok_or_else(||crate::Diagnostic::new("error-effect","Result call emission requires a typed completion region.",span))?;
        let (name,contract,runtime)=region.transport.as_deref().ok_or_else(||crate::Diagnostic::new("error-effect","Result call emission requires a destination error layout.",span))?;
        let pending=checked_fallible_call(call,source_name,name,source,contract,unions,*runtime,span)?;
        if let Some(storage)=region.storage.as_ref().filter(|storage|storage.carry_error) {
            let storage=storage.at(span);
            return Ok(format!("match {pending} {{ TsrCompletion::Normal(value)=>value, TsrCompletion::Throw(error)=>break 'exception_region{} TsrCompletion::Throw((error,{storage})), TsrCompletion::Return(value)=>break 'exception_region{} TsrCompletion::Return(value), TsrCompletion::Break(target)=>break 'exception_region{} TsrCompletion::Break(target), TsrCompletion::Continue(target)=>break 'exception_region{} TsrCompletion::Continue(target) }}",region.id,region.id,region.id,region.id));
        }
        Ok(super::exception_emit::propagate_completion(region.id,&pending))
    }
    pub(crate) fn current_transport(&self)->Option<(String,crate::error_effects::FailureSet,Option<crate::syntax::Type>)> {
        self.0.borrow().last().and_then(|region|region.transport.as_deref().cloned())
    }
    pub(crate) fn current_id(&self)->Option<usize> {self.0.borrow().last().map(|region|region.id)}
    pub(crate) fn control(&self,target:crate::syntax::ControlTarget,continuing:bool)->String {
        let regions=self.0.borrow();
        control_transfer(target,continuing,regions.last().map(|region|(region.id,region.locals.as_slice())))
    }
    pub(crate) fn dispatch(&self,pending:&str,active:&[crate::syntax::ControlTarget],span:crate::Span)->Result<String,crate::Diagnostic> {
        use crate::syntax::ControlTarget;
        let regions=self.0.borrow();
        let region=regions.last().ok_or_else(||crate::Diagnostic::new("error-effect","Completion dispatch requires an enclosing region.",span))?;
        let mut breaks=vec![];let mut continues=vec![];
        for target in active.iter().filter(|target|region.locals.contains(target)) {
            let pattern=match target {ControlTarget::Loop(id)|ControlTarget::Tail(id)=>format!("TsrControlTarget::Loop({id})"),ControlTarget::Block(id)=>format!("TsrControlTarget::Block({id})")};
            breaks.push(format!("{pattern}=>{{ {} }}",control_transfer(*target,false,None)));
            if !matches!(target,ControlTarget::Block(_)) {continues.push(format!("{pattern}=>{{ {} }}",control_transfer(*target,true,None)));}
        }
        breaks.push(format!("target=>break 'exception_region{} TsrCompletion::Break(target)",region.id));
        continues.push(format!("target=>break 'exception_region{} TsrCompletion::Continue(target)",region.id));
        Ok(format!("match {pending} {{ TsrCompletion::Normal(())=>{{}}, TsrCompletion::Return(value)=>break 'exception_region{id} TsrCompletion::Return(value), TsrCompletion::Throw(error)=>break 'exception_region{id} TsrCompletion::Throw(error), TsrCompletion::Break(target)=>match target {{ {breaks} }}, TsrCompletion::Continue(target)=>match target {{ {continues} }} }}",id=region.id,breaks=breaks.join(","),continues=continues.join(",")))
    }
}

impl Drop for RegionScope<'_> {
    fn drop(&mut self) {self.stack.0.borrow_mut().truncate(self.previous_depth);}
}
impl Drop for CallableScope<'_> {
    fn drop(&mut self) {*self.stack.0.borrow_mut()=std::mem::take(&mut self.previous);}
}

/// Collect destinations introduced inside the emitted region, without recursion.
pub(crate) fn local_control_targets(body:&[crate::syntax::Stmt])->Vec<crate::syntax::ControlTarget> {
    use crate::syntax::{StmtKind,ControlTarget};
    let mut pending=body.iter().collect::<Vec<_>>();
    let mut targets=vec![];
    while let Some(stmt)=pending.pop() {
        match &stmt.kind {
            StmtKind::Block(body)=> {
                if !stmt.labels.is_empty() {targets.push(ControlTarget::Block(stmt.span.start));}
                pending.extend(body);
            }
            StmtKind::Loop{body,update,..}=> {
                targets.push(ControlTarget::Tail(stmt.span.start));
                pending.extend(body);pending.extend(update);
            }
            StmtKind::While{body,..}|StmtKind::ForOf{body,..}=> {
                targets.push(ControlTarget::Loop(stmt.span.start));pending.extend(body);
            }
            StmtKind::If{then_body,else_body,..}=> {pending.extend(then_body);pending.extend(else_body);}
            StmtKind::Try(region)=>pending.extend(region.statements()),
            _=>{}
        }
    }
    targets
}

/// Route checked control targets through cleanup only when they leave this region.
pub(crate) fn control_transfer(target:crate::syntax::ControlTarget,continuing:bool,region:Option<(usize,&[crate::syntax::ControlTarget])>)->String {
    use crate::syntax::ControlTarget;
    if let Some((id,locals))=region && !locals.contains(&target) {
        let destination=match target {
            ControlTarget::Loop(label)|ControlTarget::Tail(label)=>format!("TsrControlTarget::Loop({label})"),
            ControlTarget::Block(label)=>format!("TsrControlTarget::Block({label})"),
        };
        let kind=if continuing {"Continue"}else{"Break"};
        return format!("break 'exception_region{id} TsrCompletion::{kind}({destination});");
    }
    match (target,continuing) {
        (ControlTarget::Loop(label)|ControlTarget::Tail(label),false)=>format!("break 'loop_{label};"),
        (ControlTarget::Block(label),false)=>format!("break 'block_{label};"),
        (ControlTarget::Loop(label),true)=>format!("continue 'loop_{label};"),
        (ControlTarget::Tail(label),true)=>format!("break 'body_{label};"),
        (ControlTarget::Block(_),true)=>unreachable!("checked continue target"),
    }
}

/// Convert outgoing region errors using a checked total injection map.
pub(crate) fn checked_completion_conversion(pending:&str,source_name:&str,destination_name:&str,source:&crate::error_effects::FailureSet,destination:&crate::error_effects::FailureSet,unions:&crate::unions::UnionArena,runtime:Option<crate::syntax::Type>,span:crate::Span)->Result<String,crate::Diagnostic> {
    let mapping=source.native_injection(destination,unions,runtime,span)?;
    Ok(super::exception_emit::completion_conversion(pending,source_name,destination_name,&mapping))
}

/// Resolve transport tags from checked contracts before generating a call bridge.
pub(crate) fn checked_fallible_call(
    call:&str,source_name:&str,destination_name:&str,
    source:&crate::error_effects::FailureSet,destination:&crate::error_effects::FailureSet,
    unions:&crate::unions::UnionArena,runtime:Option<crate::syntax::Type>,span:crate::Span,
)->Result<String,crate::Diagnostic> {
    let mapping=source.native_injection(destination,unions,runtime,span)?;
    Ok(fallible_call(call,source_name,destination_name,&mapping))
}

/// Resolve each canonical payload with the emitter's native type environment.
#[cfg(test)]
pub(crate) fn checked_error_enum(
    name:&str,contract:&crate::error_effects::FailureSet,
    unions:&crate::unions::UnionArena,runtime:Option<crate::syntax::Type>,span:crate::Span,
    mut native_type:impl FnMut(crate::syntax::Type)->String,
)->Result<String,crate::Diagnostic> {
    let payloads=contract.native_payloads(unions,runtime,span)?;
    if payloads.is_empty() {return Ok(String::new());}
    let types=payloads.into_iter().map(&mut native_type).collect::<Vec<_>>();
    let types=types.iter().map(String::as_str).collect::<Vec<_>>();
    Ok(typed_error_enum(name,&types))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closure_result_calls_require_saved_closure_contracts() {
        use crate::syntax::Type;
        use crate::error_effects::{FailureSet,ErrorEffects};
        let stack=RegionStack::default();let unions=crate::unions::UnionArena::default();let span=crate::Span{start:2,end:9};
        let source=FailureSet{explicit:vec![Type::String],..Default::default()};
        let effects=ErrorEffects{closures:vec![true],closure_errors:vec![source.clone()],..Default::default()};
        let _scope=stack.enter_typed(7,vec![],"Caller".into(),source.clone(),None);
        assert_eq!(stack.closure_result_call("callee()",0,"Callee",&effects,&unions,span).unwrap(),stack.result_call("callee()","Callee",&source,&unions,span).unwrap());
        let error=stack.closure_result_call("callee()",1,"Missing",&effects,&unions,span).unwrap_err();assert_eq!(error.code,"error-effect");assert_eq!(error.span,span);
        let stale=ErrorEffects{closures:vec![false],closure_errors:vec![source],..Default::default()};
        assert_eq!(stack.closure_result_call("callee()",0,"Callee",&stale,&unions,span).unwrap_err().code,"error-effect");
    }
    #[test]
    fn function_result_calls_require_saved_callee_contracts() {
        use crate::syntax::Type;
        use crate::error_effects::{FailureSet,ErrorEffects};
        let stack=RegionStack::default();let unions=crate::unions::UnionArena::default();let span=crate::Span{start:2,end:9};
        let source=FailureSet{explicit:vec![Type::String],..Default::default()};
        let effects=ErrorEffects{functions:vec![true],function_errors:vec![source.clone()],..Default::default()};
        let _scope=stack.enter_typed(7,vec![],"Caller".into(),source.clone(),None);
        let actual=stack.function_result_call("function0()",0,"Callee",&effects,&unions,span).unwrap();
        assert_eq!(actual,stack.result_call("function0()","Callee",&source,&unions,span).unwrap());
        let error=stack.function_result_call("function1()",1,"Missing",&effects,&unions,span).unwrap_err();
        assert_eq!(error.code,"error-effect");assert_eq!(error.span,span);
        let stale=ErrorEffects{functions:vec![false],function_errors:vec![source],..Default::default()};
        assert_eq!(stack.function_result_call("function0()",0,"Callee",&stale,&unions,span).unwrap_err().code,"error-effect");
    }
    #[test]
    fn result_calls_use_active_contract_and_move_owned_payloads_once() {
        use crate::syntax::Type;
        use crate::error_effects::FailureSet;
        let stack=RegionStack::default();
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:8,end:12};
        let source=FailureSet{explicit:vec![Type::String],..Default::default()};
        assert_eq!(stack.result_call("call()","Source",&source,&unions,span).unwrap_err().code,"error-effect");
        let destination=FailureSet{explicit:vec![Type::Int,Type::String],..Default::default()};
        let scope=stack.enter_typed(99,vec![],"Destination<i64,Box<Owner<'_>>>".into(),destination,None);
        let effects=crate::error_effects::ErrorEffects{closures:vec![true],closure_errors:vec![source.clone()],..Default::default()};
        let bridge=stack.closure_result_call("callee()",0,"Source<Box<Owner<'_>>>",&effects,&unions,span).unwrap();
        {
            let _inner=stack.enter_typed(100,vec![],"Empty".into(),FailureSet::default(),None);
            let error=stack.result_call("call()","Source",&source,&unions,span).unwrap_err();
            assert_eq!(error.code,"error-effect");assert_eq!(error.span,span);
        }
        assert_eq!(stack.current_id(),Some(99));
        let rust=format!(r#"{}
struct Owner<'a>(&'a std::cell::Cell<usize>);
impl Drop for Owner<'_> {{ fn drop(&mut self) {{ self.0.set(self.0.get()+1); }} }}
enum Source<P> {{ V0(P) }}
enum Destination<I,P> {{ V0(I),V1(P) }}
fn fallible<'a>(fail:bool,calls:&std::cell::Cell<usize>,drops:&'a std::cell::Cell<usize>)->Result<Box<Owner<'a>>,Source<Box<Owner<'a>>>> {{
 calls.set(calls.get()+1);let owner=Box::new(Owner(drops));if fail {{Err(Source::V0(owner))}} else {{Ok(owner)}}
}}
fn main() {{
 let calls=std::cell::Cell::new(0);let drops=std::cell::Cell::new(0);
 for fail in [false,true] {{
  let before=drops.get();
  let owner=Box::new(Owner(&drops));let calls_ref=&calls;
  let callee=move || {{calls_ref.set(calls_ref.get()+1);if fail {{Err(Source::V0(owner))}} else {{Ok(owner)}}}};
  let result:TsrCompletion<(),Box<Owner<'_>>,Destination<i64,Box<Owner<'_>>>>='exception_region99: {{
   let value={bridge};break 'exception_region99 TsrCompletion::Return(value);
  }};
  assert_eq!(drops.get(),before);
  if fail {{assert!(matches!(result,TsrCompletion::Throw(Destination::V1(_))));}}
  else {{assert!(matches!(result,TsrCompletion::Return(_)));}}
  drop(result);assert_eq!(drops.get(),before+1);
 }}
 assert_eq!(calls.get(),2);assert_eq!(drops.get(),2);
}}
"#,include_str!("runtime_completion.rs"));
        let directory=std::env::temp_dir().join(format!("tsrust-contextual-result-call-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();let path=directory.join("call.rs");let binary=directory.join("call");std::fs::write(&path,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(path).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));
        std::fs::remove_dir_all(directory).unwrap();drop(scope);assert_eq!(stack.current_id(),None);
    }
    #[test]
    fn completion_dispatch_uses_only_active_local_destinations() {
        use crate::syntax::ControlTarget;
        let stack=RegionStack::default();
        let span=crate::Span{start:0,end:1};
        let _scope=stack.enter(0,vec![ControlTarget::Tail(7),ControlTarget::Block(9)]);
        let code=stack.dispatch("pending",&[ControlTarget::Tail(7),ControlTarget::Loop(11)],span).unwrap();
        assert!(code.contains("break 'body_7;"),"{code}");assert!(!code.contains("'block_9"),"{code}");assert!(!code.contains("'loop_11"),"{code}");
        let escaping=stack.dispatch("pending",&[],span).unwrap();
        let rust=format!(r#"{}
fn main() {{
 let mut updates=0;
 let result:TsrCompletion<(),(),String>='exception_region0: {{
  'loop_7:loop {{
   'body_7: {{
    let pending:TsrCompletion<(),(),String>=TsrCompletion::Continue(TsrControlTarget::Loop(7));
    {code}
    panic!("continue failed");
   }}
   updates+=1;if updates==3 {{break 'loop_7;}}
  }}
  TsrCompletion::Normal(())
 }};
 assert!(matches!(result,TsrCompletion::Normal(())));assert_eq!(updates,3);
 let escaped:TsrCompletion<(),(),String>='exception_region0: {{
  let pending:TsrCompletion<(),(),String>=TsrCompletion::Continue(TsrControlTarget::Loop(11));
  {escaping}
  TsrCompletion::Normal(())
 }};
 assert!(matches!(escaped,TsrCompletion::Continue(TsrControlTarget::Loop(11))));
}}
"#,include_str!("runtime_completion.rs"));
        let directory=std::env::temp_dir().join(format!("tsrust-control-dispatch-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();let source=directory.join("dispatch.rs");let binary=directory.join("dispatch");std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn active_control_targets_follow_lexical_scope() {
        use crate::syntax::ControlTarget;
        let controls=ControlStack::default();
        assert!(controls.snapshot().is_empty());
        {
            let _outer=controls.enter(ControlTarget::Tail(7));
            assert_eq!(controls.snapshot(),vec![ControlTarget::Tail(7)]);
            {
                let _inner=controls.enter(ControlTarget::Block(9));
                assert_eq!(controls.snapshot(),vec![ControlTarget::Tail(7),ControlTarget::Block(9)]);
            }
            assert_eq!(controls.snapshot(),vec![ControlTarget::Tail(7)]);
            {let _callable=controls.callable_boundary();assert!(controls.snapshot().is_empty());}
            assert_eq!(controls.snapshot(),vec![ControlTarget::Tail(7)]);
        }
        assert!(controls.snapshot().is_empty());
    }
    #[test]
    fn typed_region_throw_uses_current_contract_and_restores_outer_layout() {
        use crate::syntax::Type;
        use crate::error_effects::FailureSet;
        let stack=RegionStack::default();
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:2,end:3};
        assert_eq!(stack.throw_value("value",Type::String,&unions,span).unwrap_err().code,"error-effect");
        let contract=FailureSet{explicit:vec![Type::String],..FailureSet::default()};
        let _outer=stack.enter_typed(1,vec![],"Outer".into(),contract.clone(),None);
        assert_eq!(stack.throw_value("value",Type::String,&unions,span).unwrap(),"break 'exception_region1 TsrCompletion::Throw(Outer::V0(value));");
        {
            let inner=FailureSet{explicit:vec![Type::String,Type::Int],..FailureSet::default()};
            let _inner=stack.enter_typed(2,vec![],"Inner".into(),inner,None);
            assert_eq!(stack.throw_value("value",Type::String,&unions,span).unwrap(),"break 'exception_region2 TsrCompletion::Throw(Inner::V1(value));");
        }
        assert_eq!(stack.throw_value("value",Type::String,&unions,span).unwrap(),"break 'exception_region1 TsrCompletion::Throw(Outer::V0(value));");
    }
    #[test]
    fn throw_injection_widens_union_and_rejects_missing_destination() {
        use crate::syntax::Type;
        use crate::error_effects::FailureSet;
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:8,end:12};
        let ty=unions.intern(vec![Type::String,Type::Int],span).unwrap();
        let destination=FailureSet{explicit:vec![Type::String,Type::Boolean,Type::Int],..FailureSet::default()};
        let Type::Union(id)=ty else {panic!("union");};
        let code=checked_throw_value("value",ty,"Error0",&destination,&unions,None,span).unwrap();
        assert!(code.contains(&format!("Union{}::V0(payload)=>Error0::V1(payload)",id.index())),"{code}");
        assert!(code.contains(&format!("Union{}::V1(payload)=>Error0::V2(payload)",id.index())),"{code}");
        assert_eq!(checked_throw_value("value",Type::String,"Error0",&destination,&unions,None,span).unwrap(),"Error0::V2(value)");
        let error=checked_throw_value("value",ty,"Empty",&FailureSet::default(),&unions,None,span).unwrap_err();
        assert_eq!(error.code,"error-effect");assert_eq!(error.span,span);
    }
    #[test]
    fn catch_conversion_executes_with_move_only_native_payload() {
        use crate::syntax::Type;
        use crate::error_effects::FailureSet;
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:0,end:1};
        let contract=FailureSet{explicit:vec![Type::String,Type::Int],..FailureSet::default()};
        let declaration=checked_error_enum("Error0",&contract,&unions,None,span,|ty|match ty {Type::Int=>"i64".into(),Type::String=>"Box<Owner>".into(),_=>panic!("payload")}).unwrap();
        let conversion=checked_catch_value("error","Error0",&contract,&unions,None,span).unwrap();
        let Type::Union(id)=contract.catch_type(&unions,None,span).unwrap().unwrap() else {panic!("union");};
        let rust=format!(r#"
struct Owner(std::rc::Rc<std::cell::Cell<usize>>);
impl Drop for Owner {{fn drop(&mut self){{self.0.set(self.0.get()+1);}}}}
enum Union{index} {{ V0(i64),V1(Box<Owner>) }}
{declaration}
fn main() {{
 let drops=std::rc::Rc::new(std::cell::Cell::new(0));
 for error in [Error0::V0(7),Error0::V1(Box::new(Owner(drops.clone())))] {{
  let caught={conversion};
  assert_eq!(drops.get(),0);
  match caught {{Union{index}::V0(value)=>assert_eq!(value,7),Union{index}::V1(owner)=>{{assert_eq!(owner.0.get(),0);drop(owner);}}}}
 }}
 assert_eq!(drops.get(),1);
}}
"#,index=id.index());
        let directory=std::env::temp_dir().join(format!("tsrust-catch-native-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("catch.rs");
        let binary=directory.join("catch");
        std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(&source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(&binary).output().unwrap();
        assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn catch_value_conversion_matches_canonical_catch_type() {
        use crate::syntax::Type;
        use crate::error_effects::FailureSet;
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:4,end:9};
        let single=FailureSet{explicit:vec![Type::String],..FailureSet::default()};
        assert_eq!(checked_catch_value("error","Error0",&single,&unions,None,span).unwrap(),"match error { Error0::V0(payload)=>payload }");
        let mixed=FailureSet{explicit:vec![Type::String,Type::Int],..FailureSet::default()};
        let expression=checked_catch_value("error","Error1",&mixed,&unions,None,span).unwrap();
        let Type::Union(id)=mixed.catch_type(&unions,None,span).unwrap().unwrap() else {panic!("union");};
        assert!(expression.contains(&format!("Error1::V0(payload)=>Union{}::V0(payload)",id.index())));
        assert!(expression.contains(&format!("Error1::V1(payload)=>Union{}::V1(payload)",id.index())));
        assert_eq!(checked_catch_value("error","Empty",&FailureSet::default(),&unions,None,span).unwrap(),"match error {}");
    }
    #[test]
    fn callable_boundary_clears_and_restores_outer_completion_context() {
        use crate::syntax::ControlTarget;
        let stack=RegionStack::default();
        let _outer=stack.enter(1,vec![]);
        {
            let _callable=stack.callable_boundary();
            assert_eq!(stack.current_id(),None);
            assert_eq!(stack.control(ControlTarget::Loop(7),true),"continue 'loop_7;");
            {
                let _local=stack.enter(3,vec![]);
                assert_eq!(stack.current_id(),Some(3));
            }
            assert_eq!(stack.current_id(),None);
        }
        assert_eq!(stack.current_id(),Some(1));
        assert_eq!(stack.control(ControlTarget::Loop(7),true),"break 'exception_region1 TsrCompletion::Continue(TsrControlTarget::Loop(7));");
    }
    #[test]
    fn emitter_context_restores_outer_region_after_nested_scope() {
        use crate::syntax::ControlTarget;
        let stack=RegionStack::default();
        let target=ControlTarget::Loop(7);
        assert_eq!(stack.control(target,true),"continue 'loop_7;");
        {
            let _outer=stack.enter(1,vec![target]);
            assert_eq!(stack.control(target,true),"continue 'loop_7;");
            {
                let _inner=stack.enter(2,vec![]);
                assert_eq!(stack.control(target,true),"break 'exception_region2 TsrCompletion::Continue(TsrControlTarget::Loop(7));");
            }
            assert_eq!(stack.control(target,true),"continue 'loop_7;");
            assert_eq!(stack.current_id(),Some(1));
        }
        assert_eq!(stack.current_id(),None);
        assert_eq!(stack.control(target,true),"continue 'loop_7;");
    }
    #[test]
    fn region_target_plan_excludes_enclosing_loop_and_keeps_local_tail() {
        use crate::syntax::{StmtKind,ControlTarget};
        let parsed=crate::syntax::parse("function main(){outer:while(true){try{local:for(let i=0;i<2;i++){continue local;}done:{break done;}}finally{}}}").unwrap();
        let outer=&parsed.functions[0].body[0];
        let StmtKind::While{body,..}=&outer.kind else {panic!("outer");};
        let StmtKind::Try(region)=&body[0].kind else {panic!("region");};
        let targets=local_control_targets(&region.body);
        assert!(!targets.contains(&ControlTarget::Loop(outer.span.start)));
        assert!(targets.iter().any(|target|matches!(target,ControlTarget::Tail(_))));
        assert!(targets.iter().any(|target|matches!(target,ControlTarget::Block(_))));
    }
    #[test]
    fn control_routing_distinguishes_local_and_escaping_targets() {
        use crate::syntax::ControlTarget::*;
        assert_eq!(control_transfer(Tail(7),true,None),"break 'body_7;");
        assert_eq!(control_transfer(Loop(7),true,Some((2,&[Loop(7)]))),"continue 'loop_7;");
        assert_eq!(control_transfer(Block(7),false,Some((2,&[Block(7)]))),"break 'block_7;");
        assert_eq!(control_transfer(Tail(7),true,Some((2,&[]))),"break 'exception_region2 TsrCompletion::Continue(TsrControlTarget::Loop(7));");
        assert_eq!(control_transfer(Block(7),false,Some((2,&[]))),"break 'exception_region2 TsrCompletion::Break(TsrControlTarget::Block(7));");
    }
    #[test]
    fn native_error_declaration_resolves_canonical_contract_types() {
        use crate::error_effects::FailureSet;
        use crate::syntax::Type;
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:0,end:1};
        let contract=FailureSet{explicit:vec![Type::String,Type::Int,Type::String],..FailureSet::default()};
        let mut resolved=vec![];
        let declaration=checked_error_enum("FunctionError0",&contract,&unions,None,span,|ty| {
            resolved.push(ty);
            match ty {Type::Int=>"i64".into(),Type::String=>"String".into(),_=>panic!("unexpected payload")}
        }).unwrap();
        assert_eq!(resolved,vec![Type::Int,Type::String]);
        assert!(declaration.contains("V0(i64),V1(String)"),"{declaration}");
        assert_eq!(checked_error_enum("Unused",&FailureSet::default(),&unions,None,span,|_|panic!("infallible payload")).unwrap(),"");
        let opaque=FailureSet{opaque:true,..FailureSet::default()};
        assert_eq!(checked_error_enum("Unresolved",&opaque,&unions,None,span,|_|panic!("unresolved payload")).unwrap_err().code,"error-effect");
    }
    #[test]
    fn call_mapping_uses_checked_contract_and_rejects_missing_payload() {
        use crate::error_effects::FailureSet;
        use crate::syntax::Type;
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span{start:12,end:18};
        let source=FailureSet{explicit:vec![Type::String],..FailureSet::default()};
        let destination=FailureSet{explicit:vec![Type::String,Type::Int],..FailureSet::default()};
        let expression=checked_fallible_call("callee()","Error0","Error1",&source,&destination,&unions,None,span).unwrap();
        assert!(expression.contains("Error0::V0(payload)=>Error1::V1(payload)"),"{expression}");
        let completion=checked_completion_conversion("pending","Error0","Error1",&source,&destination,&unions,None,span).unwrap();
        assert!(completion.contains("Error0::V0(payload)=>Error1::V1(payload)"),"{completion}");
        assert_eq!(checked_completion_conversion("pending","Error0","Empty",&source,&FailureSet::default(),&unions,None,span).unwrap_err().code,"error-effect");
        let error=checked_fallible_call("callee()","Error0","Error1",&source,&FailureSet::default(),&unions,None,span).unwrap_err();
        assert_eq!(error.code,"error-effect");
        assert_eq!(error.span,span);
    }
}
