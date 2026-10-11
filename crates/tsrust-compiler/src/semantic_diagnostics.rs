//! Bounded diagnostic replay. Successful replay trees are always discarded.
use crate::{Diagnostic,Span,syntax::{Program,Stmt,StmtKind,ExprKind}};
use std::collections::{HashMap,HashSet};

struct Statement {
    key: Span,
    extent: Span,
    scope: Span,
    declaration: Option<String>,
    terminal: bool,
}
fn contains(outer:Span,inner:Span)->bool {outer.start<=inner.start && inner.end<=outer.end}
fn terminal(statement:&Stmt)->bool {
    match &statement.kind {
        StmtKind::Return(_)|StmtKind::Throw(_)|StmtKind::Break|StmtKind::Continue=>true,
        StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>body.iter().any(terminal),
        StmtKind::If{then_body,else_body,..}=>then_body.iter().chain(else_body).any(terminal),
        StmtKind::Loop{body,update,..}=>body.iter().chain(update).any(terminal),
        StmtKind::Try(region)=>region.statements().any(terminal),
        _=>false,
    }
}
fn extent(statement:&Stmt)->Span {
    let mut extent=statement.span;
    let mut copy=statement.clone();
    let _=crate::source_walk::body(std::slice::from_mut(&mut copy),&mut |annotation|{
        extent.end=extent.end.max(annotation.span.end);Ok(())
    },&mut |_|Ok(()));
    let _=crate::source_walk::body(std::slice::from_mut(&mut copy),&mut |_|Ok(()),&mut |expression|{
        extent.end=extent.end.max(expression.span.end);Ok(())
    });
    extent
}
fn body_extent(body:&[Stmt])->Span {
    body.iter().map(extent).fold(Span::new(usize::MAX,0),|range,next|Span::new(range.start.min(next.start),range.end.max(next.end)))
}
fn index(program:&Program)->(Vec<Statement>,HashMap<Span,String>) {
    let mut statements=Vec::new();let mut names=HashMap::new();let mut seen=HashSet::new();
    let mut pending=program.functions.iter().map(|function|(function.body.clone(),function.scope_span)).collect::<Vec<_>>();
    while let Some((body,scope))=pending.pop() {
        for mut statement in body {
            if !seen.insert(statement.span) {continue;}
            let range=extent(&statement);
            let declaration=match &statement.kind {StmtKind::Let{name,..}=>Some(name.text.clone()),_=>None};
            statements.push(Statement{key:statement.span,extent:range,scope,declaration,terminal:terminal(&statement)});
            let _=crate::source_walk::body(std::slice::from_mut(&mut statement),&mut |_|Ok(()),&mut |expression|{
                if let ExprKind::Name(name)=&expression.kind {names.insert(expression.span,name.clone());}
                if let ExprKind::Closure{body,..}=&expression.kind {pending.push((body.clone(),expression.span));}
                Ok(())
            });
            let mut nested=|body:&[Stmt]|pending.push((body.to_vec(),body_extent(body)));
            match &statement.kind {
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>nested(body),
                StmtKind::If{then_body,else_body,..}=>{nested(then_body);nested(else_body);}
                StmtKind::Loop{body,update,..}=>{nested(body);nested(update);}
                StmtKind::Try(region)=>{
                    nested(&region.body);
                    if let Some(handler)=&region.catch {nested(&handler.body);}
                    if let Some(body)=&region.finally {nested(body);}
                }
                _=>{}
            }
        }
    }
    (statements,names)
}

fn recoverable(code:&str)->bool {
    matches!(code,"type"|"unknown"|"name"|"transferred"|"partial-transfer"|"conflicting-access"|"borrowed-view"|"borrowed-place"|"borrowed-element")
}

pub(crate) fn collect(mut program:Program, first:Diagnostic)->Vec<Diagnostic> {
    if !recoverable(first.code) {return vec![first];}
    let (statements,names)=index(&program);
    let mut errors=Vec::new();let mut next=first;
    for _ in 0..128 {
        let cascade=matches!(next.code,"unknown"|"name") && names.get(&next.span).is_some_and(|name|{
            statements.iter().filter(|statement|statement.declaration.as_ref()==Some(name) && contains(statement.scope,next.span))
                .min_by_key(|statement|statement.scope.end.saturating_sub(statement.scope.start))
                .is_some_and(|declaration|program.diagnostic_skips.contains(&declaration.key))
        });
        if !cascade {errors.push(next.clone());}
        if errors.len()>=64 || !recoverable(next.code) {break;}
        let Some(statement)=statements.iter().filter(|statement|contains(statement.extent,next.span))
            .min_by_key(|statement|statement.extent.end.saturating_sub(statement.extent.start)) else {break;};
        // Do not invent a fallthrough path by dropping a return, throw, or jump.
        if statement.terminal || !program.diagnostic_skips.insert(statement.key) {break;}
        // Replay from pristine state: valid earlier moves and loans survive, while
        // partial mutations made by the failing statement cannot leak forward.
        let result=match crate::analysis::analyze(program.clone()) {
            Err(error) if error.code=="recursion-inference"=>crate::analysis::analyze_recursive(program.clone()),
            result=>result,
        };
        match result {Ok(_)=>break,Err(error)=>next=error}
    }
    errors.sort_by_key(|error|(error.span.start,error.span.end));
    errors.dedup_by(|a,b|a.span==b.span && a.code==b.code && a.message==b.message);
    errors
}
