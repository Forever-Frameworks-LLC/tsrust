//! Assignment state follows every pending completion through catch and finally.
use super::*;

pub(super) fn join_normal(left:Option<bool>,right:Option<bool>)->Option<bool> {
    match (left,right) {
        (Some(a),Some(b))=>Some(a&&b),
        (Some(a),None)|(None,Some(a))=>Some(a),
        (None,None)=>None,
    }
}
pub(super) fn merge(result:&mut Flow, next:Flow) {
    result.normal=join_normal(result.normal,next.normal);
    for exit in next.exits {
        if let Some(previous)=result.exits.iter_mut().find(|previous|previous.kind==exit.kind) {
            previous.assigned &= exit.assigned;
        }else{result.exits.push(exit);}
    }
}
