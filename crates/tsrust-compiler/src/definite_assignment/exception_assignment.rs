//! Assignment state follows every pending completion through catch and finally.
use super::*;
use crate::syntax::ExceptionRegion;

fn join_normal(left:Option<bool>,right:Option<bool>)->Option<bool> {
    match (left,right) {
        (Some(a),Some(b))=>Some(a&&b),
        (Some(a),None)|(None,Some(a))=>Some(a),
        (None,None)=>None,
    }
}
fn merge(result:&mut Flow, next:Flow) {
    result.normal=join_normal(result.normal,next.normal);
    for exit in next.exits {
        if let Some(previous)=result.exits.iter_mut().find(|previous|previous.kind==exit.kind) {
            previous.assigned &= exit.assigned;
        }else{result.exits.push(exit);}
    }
}

#[inline(never)]
pub(super) fn region_flow(region:&mut ExceptionRegion, variable:Variable, assigned:bool)->Result<Flow,Diagnostic> {
    let mut result=flow(&mut region.body,variable,assigned)?;
    if let Some(handler)=&mut region.catch {
        let mut caught=None;
        result.exits.retain(|exit| {
            if exit.kind==CompletionKind::Throw {caught=join_normal(caught,Some(exit.assigned));false}else{true}
        });
        if let Some(assigned)=caught {merge(&mut result,flow(&mut handler.body,variable,assigned || handler.name.as_ref().is_some_and(|name| variable.initialized_by(name)))?);}
    }
    let Some(finally)=&mut region.finally else{return Ok(result);};
    let mut completed=Flow{normal:None,exits:Vec::new()};
    if let Some(assigned)=result.normal {
        merge(&mut completed,flow(finally,variable,assigned)?);
    }
    for exit in result.exits {
        let mut cleanup=flow(finally,variable,exit.assigned)?;
        if let Some(assigned)=cleanup.normal.take() {
            cleanup.exits.push(Exit{kind:exit.kind,assigned});
        }
        merge(&mut completed,cleanup);
    }
    Ok(completed)
}
