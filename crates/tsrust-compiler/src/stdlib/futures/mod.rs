//! Tokio operations preserve lazy task construction and explicit job settlement.
use crate::{native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, Mode};
pub(crate) const RUNTIME: &str = include_str!("runtime.rs");
pub(crate) const CANCELLATION_RUNTIME: &str = include_str!("cancellation_runtime.rs");
const INPUT: NativeParameter=NativeParameter{name:"value",ty:NativeType::ObservedValue,access:Mode::Consume,optional:false,rest:false};
const fn operation(name:&'static str,parameters:&'static [NativeParameter])->NativeBinding {
    NativeBinding{module:"std:futures",name,parameters,result:NativeType::ObservedValue,returned_borrow:None,backend:NativeBackend::Bundled{symbol:"future_intrinsic"},features:&[]}
}
pub(crate) const DELAY:NativeBinding=operation("delay",&[INPUT]);
pub(crate) const YIELD:NativeBinding=operation("yield",&[]);
pub(crate) const ALL:NativeBinding=operation("all",&[INPUT]);
pub(crate) const ALL_POLL:NativeBinding=operation("__all",&[NativeParameter{optional:true,..INPUT}]);
pub(crate) const RACE:NativeBinding=operation("race",&[INPUT]);
pub(crate) const SETTLED:NativeBinding=operation("allSettled",&[INPUT]);
pub(crate) const START:NativeBinding=operation("__start",&[INPUT]);
pub(crate) const CANCEL:NativeBinding=operation("__cancel",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const JOIN:NativeBinding=operation("__join",&[INPUT]);
pub(crate) const TIMEOUT:NativeBinding=operation("__timeout",&[INPUT,INPUT]);
pub(crate) const SLEEP:NativeBinding=operation("__delay",&[INPUT]);
pub(crate) const YIELD_POLL:NativeBinding=operation("__yield",&[]);
pub(crate) const RACE_POLL:NativeBinding=operation("__race",&[NativeParameter{optional:true,..INPUT}]);
pub(crate) const SETTLED_POLL:NativeBinding=operation("__allSettled",&[NativeParameter{optional:true,..INPUT}]);
pub(crate) const CHANNEL:NativeBinding=operation("__channel",&[INPUT]);
pub(crate) const SENDER:NativeBinding=operation("__sender",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const CLOSE:NativeBinding=operation("__close",&[NativeParameter{access:Mode::Mutate,..INPUT}]);
pub(crate) const SEND:NativeBinding=operation("__send",&[NativeParameter{access:Mode::Read,..INPUT},INPUT]);
pub(crate) const MISSED_TICK:crate::native_signatures::Signature=crate::native_signatures::Signature::Union(&[crate::native_signatures::Signature::StringLiteral("burst"),crate::native_signatures::Signature::StringLiteral("delay"),crate::native_signatures::Signature::StringLiteral("skip")]);
pub(crate) const INTERVAL:NativeBinding=operation("__interval",&[INPUT,NativeParameter{ty:NativeType::Structured(&MISSED_TICK),optional:true,..INPUT}]);
pub(crate) const TICK:NativeBinding=operation("__tick",&[NativeParameter{access:Mode::Mutate,..INPUT}]);
pub(crate) const TOKEN:NativeBinding=operation("__token",&[]);
pub(crate) const TOKEN_SHARE:NativeBinding=operation("__tokenShare",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const TOKEN_CHILD:NativeBinding=operation("__tokenChild",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const TOKEN_CANCEL:NativeBinding=operation("__tokenCancel",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const TOKEN_IS_CANCELLED:NativeBinding=operation("__tokenIsCancelled",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const TOKEN_CANCELLED:NativeBinding=operation("__tokenCancelled",&[NativeParameter{access:Mode::Read,..INPUT}]);
pub(crate) const TRY_SEND:NativeBinding=operation("__trySend",&[NativeParameter{access:Mode::Read,..INPUT},INPUT]);
pub(crate) const TRY_RECEIVE:NativeBinding=operation("__tryReceive",&[NativeParameter{access:Mode::Mutate,..INPUT}]);
pub(crate) const RECEIVE:NativeBinding=operation("__receive",&[NativeParameter{access:Mode::Read,..INPUT}]);

pub(crate) fn call(name:&str,args:Vec<crate::syntax::Expr>,span:crate::Span)->crate::syntax::Expr {
    use crate::syntax::{Expr,ExprKind};
    Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:futures#{name}")),span)),args),span)
}
pub(crate) fn unwrap(value:crate::syntax::Expr)->crate::syntax::Expr {
    use crate::syntax::{Expr,ExprKind};let span=value.span;
    Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@native:std:result#unwrap".into()),span)),vec![value]),span)
}
/// Capture evaluated arguments in an ordinary checked task; reuse match's scoped expression block.
pub(crate) fn lazy(expression:&mut crate::syntax::Expr,name:&str,args:Vec<crate::syntax::Expr>) {
    lazy_module(expression,"std:futures",name,args);
}
pub(crate) fn lazy_module(expression:&mut crate::syntax::Expr,module:&str,name:&str,args:Vec<crate::syntax::Expr>){
    lazy_native(expression,module,name,args,false);
}
/// Preserve an explicit Result payload rather than translating it into task failure.
pub(crate) fn lazy_result(expression:&mut crate::syntax::Expr,module:&str,name:&str,args:Vec<crate::syntax::Expr>){
    lazy_native(expression,module,name,args,true);
}
fn lazy_native(expression:&mut crate::syntax::Expr,module:&str,name:&str,mut args:Vec<crate::syntax::Expr>,preserve_result:bool){
    use crate::syntax::{Expr,ExprKind,Stmt,StmtKind,Token,TokenKind,ResultAnnotation,UnaryOp};
    let span=expression.span;let id=usize::MAX/2+span.start;
    let statement=|kind|Stmt{kind,span,labels:Vec::new(),control_target:None};
    let mut body=Vec::new();let mut inputs=Vec::new();
    if matches!(name,"__race"|"__allSettled"|"__all") && matches!(args.as_slice(),[Expr{kind:ExprKind::Array(values),..}] if values.is_empty()){args.clear();}
    for (index,mut value) in args.into_iter().enumerate(){
        if borrows_native_input(module,name,index) && stable_receiver(&value) {inputs.push(value);continue;}
        if index==0 && matches!(name,"__race"|"__allSettled"|"__all") && let ExprKind::Array(items)=&mut value.kind {
            if !items.is_empty() && !items.iter().any(|item|matches!(item.kind,ExprKind::Unary{op:UnaryOp::Spread,..})) {
                value.kind=ExprKind::Record(std::mem::take(items).into_iter().enumerate().map(|(index,value)|(Token{kind:TokenKind::Name,text:format!("future#task{index}"),span:value.span},value)).collect());
            }
        }
        let name=format!("future#{id}#{index}");inputs.push(Expr::new(ExprKind::Name(name.clone()),span));
        body.push(statement(StmtKind::Let{rebindable:false,name:Token{kind:TokenKind::Symbol,text:name,span},annotation:None,value,binding:None}));
    }
    let value=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:{module}#{name}")),span)),inputs),span);let value=if preserve_result || matches!(name,"__allSettled"|"__tick"|"__tokenCancelled"){value}else{unwrap(value)};
    let closure=Expr::new(ExprKind::Closure{id,params:Vec::new(),result:ResultAnnotation::Inferred(span),body:vec![statement(StmtKind::Return(Some(value)))]},span);
    let task=Expr::new(ExprKind::Unary{op:UnaryOp::TaskCreate,value:Box::new(closure),operator:span},span);
    let yielded=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@match:yield".into()),span)),vec![task]),span);
    body.push(statement(StmtKind::Expr(yielded)));expression.kind=ExprKind::Match{id,body};
}
// Before checking, a lowered lazy native call has no concrete Task type yet.
// Expose only its generated return expression to the ordinary hint scheduler.
pub(crate) fn lazy_native_result(expression:&crate::syntax::Expr)->Option<&crate::syntax::Expr>{
 use crate::syntax::{ExprKind,StmtKind,UnaryOp};
 let ExprKind::Match{id,body}=&expression.kind else{return None;};
 if *id<usize::MAX/2{return None;}
 let StmtKind::Expr(yielded)=&body.last()?.kind else{return None;};
 let ExprKind::Call(callee,arguments)=&yielded.kind else{return None;};
 if !matches!(&callee.kind,ExprKind::Name(name)if name=="@match:yield"){return None;}
 let [task]=arguments.as_slice()else{return None;};
 let ExprKind::Unary{op:UnaryOp::TaskCreate,value,..}=&task.kind else{return None;};
 let ExprKind::Closure{id:origin,params,body,..}=&value.kind else{return None;};
 if origin!=id||!params.is_empty(){return None;}
 let [statement]=body.as_slice()else{return None;};
 let StmtKind::Return(Some(result))=&statement.kind else{return None;};
 let ExprKind::Call(callee,_)=&result.kind else{return None;};
 let ExprKind::Name(symbol)=&callee.kind else{return None;};
 crate::native_bindings::by_symbol(symbol)?;
 Some(result)
}
// A lazy call evaluates temporary inputs before entering its task. Retain the
// native call's contextual type at that evaluation point, particularly for empty
// arrays and optional owned option literals. Stable borrowed inputs stay in the
// task and keep their existing loans; typed alternative dispatch stays unchanged.
fn contextualize_lazy_inputs(records:&mut Vec<crate::syntax::Record>,expression:&mut crate::syntax::Expr)->Result<(),crate::Diagnostic>{
    use crate::syntax::{ExprKind,StmtKind,UnaryOp};
    let ExprKind::Match{id,body}=&mut expression.kind else{return Ok(())};
    if *id<usize::MAX/2{return Ok(());}
    let Some(statement)=body.last()else{return Ok(())};
    let StmtKind::Expr(yielded)=&statement.kind else{return Ok(())};
    let ExprKind::Call(_,values)=&yielded.kind else{return Ok(())};
    let Some(task)=values.first()else{return Ok(())};
    let ExprKind::Unary{op:UnaryOp::TaskCreate,value,..}=&task.kind else{return Ok(())};
    let ExprKind::Closure{body:task_body,..}=&value.kind else{return Ok(())};
    let [statement]=task_body.as_slice()else{return Ok(())};
    let StmtKind::Return(Some(call))=&statement.kind else{return Ok(())};
    let ExprKind::Call(callee,arguments)=&call.kind else{return Ok(())};
    let ExprKind::Name(symbol)=&callee.kind else{return Ok(())};
    let Some((_,binding))=crate::native_bindings::by_symbol(symbol)else{return Ok(())};
    if !matches!(binding.module,"std:io"|"std:process"|"std:net"|"std:dns"|"std:compression"){return Ok(());}
    let mut annotations=Vec::new();
    for (position,(argument,parameter)) in arguments.iter().zip(binding.parameters).enumerate(){
        let ExprKind::Name(name)=&argument.kind else{continue};
        if !name.starts_with("future#") || !binding.argument_alternatives(position).is_empty(){continue;}
        let signature=parameter.ty.signature();
        let mut annotation=crate::native_type_resolution::project(records,&signature,argument.span)?;
        if parameter.optional {
            use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind,TypeName};
            let absent=ValueAnnotation{kind:ValueAnnotationKind::Named(TypeName{text:"undefined".into(),span:argument.span}),span:argument.span};
            annotation=ValueAnnotation{kind:ValueAnnotationKind::Union(vec![annotation,absent]),span:argument.span};
        }
        annotations.push((name.clone(),annotation));
    }
    for statement in body {
        if let StmtKind::Let{name,annotation,..}=&mut statement.kind
            && let Some((_,context))=annotations.iter().find(|(input,_)|*input==name.text){
            *annotation=Some(context.clone());
        }
    }
    Ok(())
}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
    use crate::syntax::ExprKind;
    if program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Interval)){crate::native_binding_types::prepare_signature(&mut program.records,&MISSED_TICK,&[],crate::Span::default())?;}
    let mut used=false;
    for function in &mut program.functions {
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|contextualize_lazy_inputs(&mut program.records,expression))?;
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{
            used |= matches!(&expression.kind,ExprKind::Unary{op:crate::syntax::UnaryOp::TaskCreate,..});
            if let ExprKind::Call(callee,args)=&mut expression.kind {
                let operation=match &callee.kind {ExprKind::Name(name)=>name.strip_prefix("@native:std:futures#").filter(|name|matches!(*name,"delay"|"yield"|"race"|"allSettled"|"all")).map(|name|format!("__{name}")),_=>None};
                if let Some(name)=operation {
                    let expected=usize::from(name!="__yield");
                    if args.len()!=expected{return Err(crate::Diagnostic::new("arity",format!("{} expects {expected} arguments.",name.trim_start_matches('_')),expression.span));}
                    let args=std::mem::take(args);lazy(expression,&name,args);
                }
            }Ok(())
        })?;
    }
    if used || program.records.iter().any(|record|matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::AsyncChannel|crate::native_signatures::NativeTypeId::AsyncSender))) {for id in [crate::native_signatures::NativeTypeId::Job,crate::native_signatures::NativeTypeId::AsyncError,crate::native_signatures::NativeTypeId::AsyncChannel,crate::native_signatures::NativeTypeId::AsyncSender,crate::native_signatures::NativeTypeId::SendError,crate::native_signatures::NativeTypeId::ChannelError]{crate::native_type_resolution::intern(&mut program.records,id,crate::Span::default())?;}}
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    include!("runtime.rs");
    pub(crate) use tsrust_native_futures::*;
    include!("runtime_tests.rs");
}

pub(crate) fn borrows_inputs(value:&crate::syntax::Expr)->bool {
    use crate::syntax::{ExprKind,StmtKind};
    let ExprKind::Closure{id,body,..}=&value.kind else{return false;};
    if *id<usize::MAX/2{return false;}
    let [statement]=body.as_slice() else{return false;};
    let StmtKind::Return(Some(value))=&statement.kind else{return false;};
    // Request JSON parsing wraps the same borrowed body read without extending its loan.
    let value=if let ExprKind::Call(_,args)=&value.kind
        && let Some(crate::syntax::Expr{kind:ExprKind::Call(callee,inputs),..})=args.first()
        && matches!(&callee.kind,ExprKind::Name(name) if name=="@native:std:json#parse"){
        let Some(value)=inputs.first()else{return false;};value
    }else{value};
    let ExprKind::Call(callee,arguments)=&value.kind else{return false;};
    let (callee,inputs)=if matches!(&callee.kind,ExprKind::Name(name) if name=="@native:std:result#unwrap"){
        let Some(crate::syntax::Expr{kind:ExprKind::Call(callee,inputs),..})=arguments.first()else{return false;};(callee,inputs)
    }else{(callee,arguments)};
    let ExprKind::Name(symbol)=&callee.kind else{return false;};
    let Some((_,binding))=crate::native_bindings::by_symbol(symbol)else{return false;};
    inputs.iter().enumerate().any(|(index,input)|
        borrows_native_input(binding.module,binding.name,index)
        && stable_receiver(input)
        && !matches!(&input.kind,ExprKind::Name(name) if name.starts_with("future#")))
}

// Existing lazy APIs keep their established receiver loans and owned snapshots.
// I/O, subprocess, networking and DNS operations borrow every stable Read/Mutate input through settlement.
fn borrows_native_input(module:&str,name:&str,index:usize)->bool{
    if matches!(module,"std:io"|"std:process"|"std:net"|"std:dns"|"std:compression"){return crate::native_bindings::lookup(module,name).ok().and_then(|binding|binding.parameters.get(index)).is_some_and(|parameter|matches!(parameter.access,Mode::Read|Mode::Mutate));}
    index==0 && match module{
        "std:signals"=>name=="__receive",
        "std:futures"=>matches!(name,"__send"|"__receive"|"__tick"|"__tokenCancelled"),
        "std:http"=>matches!(name,"__contextRequestForm"|"__contextRequestMultipart"|"__contextRequestText"|"__contextNext"|"__send"|"__bodyNext"),
        "std:sql"=>matches!(name,"__cursor"|"__cursorNext"|"__migrate"|"__begin"|"__query"|"__execute"|"__txquery"|"__txexecute"),
        _=>false,
    }
}

fn stable_receiver(mut value:&crate::syntax::Expr)->bool{
    loop {match &value.kind {crate::syntax::ExprKind::Name(_)=>return true,crate::syntax::ExprKind::Field(base,_)=>value=base,_=>return false}}
}
