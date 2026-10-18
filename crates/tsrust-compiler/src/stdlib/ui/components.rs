//! Make component state ownership visible to the ordinary move/error checker.
use crate::{Diagnostic,syntax::{Expr,ExprKind,Program,Stmt,StmtKind,Token}};
use std::collections::BTreeMap;

pub(crate) fn retain_state(program:&mut Program,names:&BTreeMap<String,String>)->Result<(),Diagnostic> {
    let (Some(signal),Some(retain))=(names.get("signal"),names.get("retainComponentState")) else{return Ok(());};
    fn body(statements:&mut [Stmt],inherited:&[Token],signal:&str,retain:&str,constructors:&mut Vec<crate::Span>)->Result<(),Diagnostic> {
        let mut owners=inherited.to_vec();
        for statement in statements {
            if let StmtKind::Let{name,..}=&statement.kind {
                if owners.iter().any(|owner|owner.text==name.text) {return Err(Diagnostic::new("ui-state-owner","Component state owner names cannot be shadowed in this scope.",name.span));}
            }
            match &mut statement.kind {
                StmtKind::Let{rebindable,name,value,..} if matches!(&value.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(target) if target==signal))=>{
                    if *rebindable {return Err(Diagnostic::new("ui-state-owner","Declare component signal owners with const; update their value through handles.",name.span));}
                    if let ExprKind::Call(callee,_) = &value.kind { constructors.push(callee.span); }
                    owners.push(name.clone());
                }
                StmtKind::Return(Some(value))=>{
                    for owner in &owners {
                        let span=value.span;
                        let original=std::mem::replace(value,Expr::new(ExprKind::Undefined,span));
                        *value=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(retain.into()),span)),vec![original,Expr::new(ExprKind::Name(owner.text.clone()),owner.span)]),span);
                    }
                }
                StmtKind::Block(nested)=>body(nested,&owners,signal,retain,constructors)?,
                StmtKind::If{then_body,else_body,..}=>{body(then_body,&owners,signal,retain,constructors)?;body(else_body,&owners,signal,retain,constructors)?;},
                StmtKind::While{body:nested,..}|StmtKind::ForOf{body:nested,..}|StmtKind::Loop{body:nested,..}=>body(nested,&owners,signal,retain,constructors)?,
                StmtKind::Try(region)=>{
                    body(&mut region.body,&owners,signal,retain,constructors)?;
                    if let Some(catch)=&mut region.catch {body(&mut catch.body,&owners,signal,retain,constructors)?;}
                    if let Some(finally)=&mut region.finally {body(finally,&owners,signal,retain,constructors)?;}
                }
                StmtKind::LocalFunction(function)=>body(&mut function.body,&[],signal,retain,constructors)?,
                _=>{}
            }
        }
        Ok(())
    }
    for function in &mut program.functions {
        if !names.values().any(|name|name==&function.name.text) {
            let mut constructors=Vec::new();
            body(&mut function.body,&[],signal,retain,&mut constructors)?;
            crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression| {
                if matches!(&expression.kind,ExprKind::Name(name) if name==signal) && !constructors.contains(&expression.span) {
                    return Err(Diagnostic::new("ui-state-owner","Create component state with a direct const declaration; signal constructor aliases and anonymous callback state are not supported.",expression.span));
                }
                Ok(())
            })?;
        }
    }
    Ok(())
}
