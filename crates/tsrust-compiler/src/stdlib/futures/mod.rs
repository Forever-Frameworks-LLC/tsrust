//! Tokio operations preserve lazy task construction and explicit job settlement.
use crate::{native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, Mode};
pub(crate) const RUNTIME: &str = include_str!("runtime.rs");
const INPUT: NativeParameter=NativeParameter{name:"value",ty:NativeType::ObservedValue,access:Mode::Consume,optional:false,rest:false};
const fn operation(name:&'static str,parameters:&'static [NativeParameter])->NativeBinding {
    NativeBinding{module:"std:futures",name,parameters,result:NativeType::ObservedValue,returned_borrow:None,backend:NativeBackend::Bundled{symbol:"future_intrinsic"},features:&[]}
}
pub(crate) const DELAY:NativeBinding=operation("delay",&[INPUT]);
pub(crate) const YIELD:NativeBinding=operation("yield",&[]);
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
pub(crate) fn lazy_module(expression:&mut crate::syntax::Expr,module:&str,name:&str,mut args:Vec<crate::syntax::Expr>){
    use crate::syntax::{Expr,ExprKind,Stmt,StmtKind,Token,TokenKind,ResultAnnotation,UnaryOp};
    let span=expression.span;let id=usize::MAX/2+span.start;
    let statement=|kind|Stmt{kind,span,labels:Vec::new(),control_target:None};
    let mut body=Vec::new();let mut inputs=Vec::new();
    if matches!(name,"__race"|"__allSettled") && matches!(args.as_slice(),[Expr{kind:ExprKind::Array(values),..}] if values.is_empty()){args.clear();}
    for (index,mut value) in args.into_iter().enumerate(){
        if index==0 && ((module=="std:signals" && name=="__receive") || (module=="std:futures" && matches!(name,"__send"|"__receive")) || (module=="std:http" && matches!(name,"__contextRequestText"|"__contextNext"|"__send"|"__bodyNext")) || (module=="std:sql" && matches!(name,"__migrate"|"__begin"|"__query"|"__execute"|"__txquery"|"__txexecute"))) && stable_receiver(&value) {inputs.push(value);continue;}
        if index==0 && matches!(name,"__race"|"__allSettled") && let ExprKind::Array(items)=&mut value.kind {
            if !items.is_empty() && !items.iter().any(|item|matches!(item.kind,ExprKind::Unary{op:UnaryOp::Spread,..})) {
                value.kind=ExprKind::Record(std::mem::take(items).into_iter().enumerate().map(|(index,value)|(Token{kind:TokenKind::Name,text:format!("future#task{index}"),span:value.span},value)).collect());
            }
        }
        let name=format!("future#{id}#{index}");inputs.push(Expr::new(ExprKind::Name(name.clone()),span));
        body.push(statement(StmtKind::Let{rebindable:false,name:Token{kind:TokenKind::Symbol,text:name,span},annotation:None,value,binding:None}));
    }
    let value=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:{module}#{name}")),span)),inputs),span);let value=if name=="__allSettled"{value}else{unwrap(value)};
    let closure=Expr::new(ExprKind::Closure{id,params:Vec::new(),result:ResultAnnotation::Inferred(span),body:vec![statement(StmtKind::Return(Some(value)))]},span);
    let task=Expr::new(ExprKind::Unary{op:UnaryOp::TaskCreate,value:Box::new(closure),operator:span},span);
    let yielded=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@match:yield".into()),span)),vec![task]),span);
    body.push(statement(StmtKind::Expr(yielded)));expression.kind=ExprKind::Match{id,body};
}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
    use crate::syntax::ExprKind;
    let mut used=false;
    for function in &mut program.functions {
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{
            used |= matches!(&expression.kind,ExprKind::Unary{op:crate::syntax::UnaryOp::TaskCreate,..});
            if let ExprKind::Call(callee,args)=&mut expression.kind {
                let operation=match &callee.kind {ExprKind::Name(name)=>name.strip_prefix("@native:std:futures#").filter(|name|matches!(*name,"delay"|"yield"|"race"|"allSettled")).map(|name|format!("__{name}")),_=>None};
                if let Some(name)=operation {
                    let expected=usize::from(name!="__yield");
                    if args.len()!=expected{return Err(crate::Diagnostic::new("arity",format!("{} expects {expected} arguments.",name.trim_start_matches('_')),expression.span));}
                    let args=std::mem::take(args);lazy(expression,&name,args);
                }
            }Ok(())
        })?;
    }
    if used {for id in [crate::native_signatures::NativeTypeId::Job,crate::native_signatures::NativeTypeId::AsyncError,crate::native_signatures::NativeTypeId::AsyncChannel,crate::native_signatures::NativeTypeId::AsyncSender]{crate::native_type_resolution::intern(&mut program.records,id,crate::Span::default())?;}}
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    include!("runtime.rs");
    pub(crate) use tsrust_native_futures::*;
    include!("runtime_tests.rs");
}

pub(crate) fn borrows_receiver(value:&crate::syntax::Expr)->bool {
    use crate::syntax::{ExprKind,StmtKind};
    let ExprKind::Closure{id,body,..}=&value.kind else{return false;};
    if *id<usize::MAX/2{return false;}
    let [statement]=body.as_slice() else{return false;};
    let StmtKind::Return(Some(value))=&statement.kind else{return false;};
    let ExprKind::Call(_,arguments)=&value.kind else{return false;};
    let Some(crate::syntax::Expr{kind:ExprKind::Call(callee,inputs),..})=arguments.first()else{return false;};
    !matches!(inputs.first().map(|value|&value.kind),Some(ExprKind::Name(name)) if name.starts_with("future#"))
        && matches!(&callee.kind,ExprKind::Name(name) if matches!(name.as_str(),"@native:std:signals#__receive"|"@native:std:futures#__send"|"@native:std:futures#__receive"|"@native:std:http#__send"|"@native:std:http#__contextRequestText"|"@native:std:http#__contextNext"|"@native:std:http#__bodyNext"|"@native:std:sql#__migrate"|"@native:std:sql#__begin"|"@native:std:sql#__query"|"@native:std:sql#__execute"|"@native:std:sql#__txquery"|"@native:std:sql#__txexecute"))
}

fn stable_receiver(mut value:&crate::syntax::Expr)->bool{
    loop {match &value.kind {crate::syntax::ExprKind::Name(_)=>return true,crate::syntax::ExprKind::Field(base,_)=>value=base,_=>return false}}
}
