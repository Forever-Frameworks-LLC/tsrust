//! State owners stay in the component tree; expressions and closures get weak bindings.
use crate::{Diagnostic,Span,syntax::{Expr,ExprKind,Program,Stmt,StmtKind,Token,TokenKind}};
use std::collections::BTreeMap;

type Scope=BTreeMap<String,String>;
fn name(value:&str,span:Span)->Expr {Expr::new(ExprKind::Name(value.into()),span)}
fn call(target:&str,args:Vec<Expr>,span:Span)->Expr {Expr::new(ExprKind::Call(Box::new(name(target,span)),args),span)}
fn statement(kind:StmtKind,span:Span)->Stmt {Stmt{kind,span,labels:vec![],control_target:None}}
struct Lower<'a> {signal:&'a str,bind:&'a str,retain:&'a str,next:usize,constructors:Vec<Span>}
impl Lower<'_> {
    fn expression(&mut self,value:&mut Expr,scope:&Scope)->Result<(),Diagnostic> {
        crate::source_walk::expression_pruned(value,&mut |value| {
            let span=value.span;
            if let ExprKind::Name(source)=&value.kind {
                if let Some(target)=scope.get(source) {*value=call(self.bind,vec![name(target,span)],span);return Ok(false);}
            }
            if let ExprKind::Match{body,..}=&mut value.kind {
                let owners=scope.iter().filter(|(source,target)|source==target).map(|(source,_)|Token{text:source.clone(),span,kind:TokenKind::Name}).collect::<Vec<_>>();
                self.body(body,scope,&owners,false)?;return Ok(false);
            }
            if let ExprKind::Closure{params,body,..}=&mut value.kind {
                let mut captures=scope.clone();
                for parameter in params {captures.remove(&parameter.name.text);}
                let mut bindings=vec![];
                for (source,target) in &mut captures {
                    let outer=target.clone();let hidden=format!("@ui:capture:{}",self.next);self.next+=1;
                    *target=hidden.clone();bindings.push((source.clone(),hidden,outer));
                }
                self.body(body,&captures,&[],false)?;
                // Only materialize references actually used by this closure or a nested closure.
                let mut used=std::collections::BTreeSet::new();
                crate::source_walk::body(body,&mut |_|Ok(()),&mut |expr| {if let ExprKind::Name(n)=&expr.kind {used.insert(n.clone());}Ok(())})?;
                let mut prefix=vec![];
                for (_,hidden,outer) in bindings {
                    if !used.contains(&hidden) {continue;}
                    prefix.push(statement(StmtKind::Let{rebindable:false,name:Token{text:hidden,span,kind:TokenKind::Name},annotation:None,value:call(self.bind,vec![name(&outer,span)],span),binding:None},span));
                }
                if !prefix.is_empty() {
                    let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                    prefix.push(statement(StmtKind::Expr(call("@match:yield",vec![original],span)),span));
                    let id=usize::MAX-self.next;self.next+=1;
                    *value=Expr::new(ExprKind::Match{id,body:prefix},span);
                }
                return Ok(false);
            }
            Ok(true)
        })
    }
    fn body(&mut self,statements:&mut [Stmt],inherited:&Scope,retained:&[Token],allow_state:bool)->Result<(),Diagnostic> {
        let mut scope=inherited.clone();let mut owners=retained.to_vec();
        for statement in statements {
            match &mut statement.kind {
                StmtKind::Let{rebindable,name,value,..}=>{
                    let state=matches!(&value.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(n) if n==self.signal));
                    self.expression(value,&scope)?;
                    if state {
                        if !allow_state || *rebindable {return Err(Diagnostic::new("ui-state-owner","Create component state with a direct const declaration outside anonymous callbacks.",name.span));}
                        if scope.contains_key(&name.text) {return Err(Diagnostic::new("ui-state-owner","Component state owner names cannot be shadowed in this scope.",name.span));}
                        if let ExprKind::Call(callee,_) = &value.kind {self.constructors.push(callee.span);}
                        owners.push(name.clone());scope.insert(name.text.clone(),name.text.clone());
                    } else {
                        if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Component state owner names cannot be shadowed in this scope.",name.span));}
                        scope.remove(&name.text);
                    }
                }
                StmtKind::Return(Some(value))=>{
                    self.expression(value,&scope)?;
                    for owner in &owners {
                        let span=value.span;let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                        *value=call(self.retain,vec![original,name(&owner.text,owner.span)],span);
                    }
                }
                StmtKind::Block(nested)=>self.body(nested,&scope,&owners,allow_state)?,
                StmtKind::If{condition,then_body,else_body}=>{self.expression(condition,&scope)?;self.body(then_body,&scope,&owners,allow_state)?;self.body(else_body,&scope,&owners,allow_state)?;},
                StmtKind::While{condition,body}=>{self.expression(condition,&scope)?;self.body(body,&scope,&owners,allow_state)?;},
                StmtKind::Loop{condition,body,update,..}=>{if let Some(value)=condition {self.expression(value,&scope)?;}self.body(body,&scope,&owners,allow_state)?;self.body(update,&scope,&owners,allow_state)?;},
                StmtKind::ForOf{name,iterable,body,..}=>{self.expression(iterable,&scope)?;let mut inner=scope.clone();if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Loop variables cannot shadow component state owners.",name.span));}inner.remove(&name.text);self.body(body,&inner,&owners,allow_state)?;},
                StmtKind::Try(region)=>{
                    self.body(&mut region.body,&scope,&owners,allow_state)?;
                    if let Some(catch)=&mut region.catch {
                        let mut inner=scope.clone();if let Some(name)=&catch.name {if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Catch variables cannot shadow component state owners.",name.span));}inner.remove(&name.text);}
                        self.body(&mut catch.body,&inner,&owners,allow_state)?;
                    }
                    if let Some(finally)=&mut region.finally {self.body(finally,&scope,&owners,allow_state)?;}
                }
                StmtKind::LocalFunction(function)=>{
                    // Named local functions retain their ordinary checked capture semantics.
                    self.body(&mut function.body,&Scope::new(),&[],allow_state)?;
                }
                StmtKind::Expr(value)|StmtKind::Throw(value)=>self.expression(value,&scope)?,
                StmtKind::Assign{target,value,..}=>{self.expression(target,&scope)?;self.expression(value,&scope)?;},
                _=>{}
            }
        }
        Ok(())
    }
}
pub(crate) fn retain_state(program:&mut Program,names:&BTreeMap<String,String>)->Result<(),Diagnostic> {
    let (Some(signal),Some(bind),Some(retain))=(names.get("signal"),names.get("bindState"),names.get("retainComponentState")) else{return Ok(());};
    let mut lower=Lower{signal,bind,retain,next:0,constructors:vec![]};
    for function in &mut program.functions {
        if names.values().any(|name|name==&function.name.text) {continue;}
        lower.body(&mut function.body,&Scope::new(),&[],true)?;
        crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression| {
            if matches!(&expression.kind,ExprKind::Name(name) if name==signal) && !lower.constructors.contains(&expression.span) {
                return Err(Diagnostic::new("ui-state-owner","Create component state with a direct const declaration; signal constructor aliases are not supported.",expression.span));
            }
            Ok(())
        })?;
    }
    Ok(())
}
