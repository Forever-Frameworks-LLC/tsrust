//! Array removals transfer analysis identities without cloning native captures.
use super::*;

#[inline(never)]
pub(super) fn transfer(engine:&mut Engine<'_>,expression:&Expr,value:&mut expression_flow::Value,state:&mut FlowState)->bool {
        let Some(alternatives)=value.alternatives.as_mut() else {return true;};
        let Some(current)=engine.current else {return false;};
        let request=&engine.entries[current].request;
        for alternative in alternatives {
            let environment_store::Alternative::Body {environment:Some(source),..}=alternative else {continue;};
            let bindings=match state.captures.get(source) {
                Some(environment_state::Capture::Live {bindings,..})=>bindings.clone(),
                Some(_)=>return false,
                None=>match engine.environments.get(*source) {Some(record)=>record.bindings.clone(),None=>return false},
            };
            let Some(destination)=engine.environments.allocate_transfer(expression.span,request.producer,&request.context,*source,&bindings) else {return false;};
            if state.captures.contains_key(&destination) {
                let Some(environment_state::Capture::Live {instance,..})=state.captures.get_mut(&destination) else {return false;};
                *instance=environment_state::Instance::Merged;
                if state.update(destination,&bindings,engine.declarations).is_none() {return false;}
            } else if state.install(destination,&bindings,environment_state::Instance::OwnedSingle).is_none() {return false;}
            *source=destination;
        }
        true
    }
