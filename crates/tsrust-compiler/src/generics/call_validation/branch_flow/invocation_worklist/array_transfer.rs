//! Array removals transfer analysis identities without cloning native captures.
use super::*;

fn captured(engine:&Engine<'_>,state:&FlowState,source:environment_store::Handle)->Option<Vec<Binding>> {
    match state.captures.get(&source) {
        Some(environment_state::Capture::Live {bindings,..})=>Some(bindings.clone()),
        Some(_)=>None,
        None=>engine.environments.get(source).map(|record|record.bindings.clone()),
    }
}

#[inline(never)]
pub(super) fn transfer(engine:&mut Engine<'_>,expression:&Expr,value:&mut expression_flow::Value,state:&mut FlowState)->bool {
    let Some(alternatives)=value.alternatives.as_ref() else {return true;};
    let Some(current)=engine.current else {return false;};
    let request=&engine.entries[current].request;
    let producer=request.producer;let context=request.context.clone();
    let mut mapping=HashMap::new();let mut pending=Vec::new();
    let mut queue=alternatives.iter().filter_map(|alternative|match alternative {
        environment_store::Alternative::Body {environment:Some(handle),..}=>Some(*handle),_=>None,
    }).collect::<Vec<_>>();
    while let Some(source)=queue.pop() {
        if mapping.contains_key(&source) {continue;}
        let Some(bindings)=captured(engine,state,source) else {return false;};
        // Captures whose lexical owner is still present are borrowed shared
        // state. Removing their callback must preserve that state identity.
        if bindings.iter().any(|captured|state.iter().any(|binding|binding.identity==captured.identity)) {
            mapping.insert(source,source);continue;
        }
        let Some(destination)=engine.environments.allocate_transfer(expression.span,producer,&context,source,&bindings) else {return false;};
        mapping.insert(source,destination);
        for binding in &bindings {
            let alternatives=binding.alternatives.iter().flatten().chain(binding.elements.iter().flat_map(|shape|shape.iter()).flat_map(|element|element.alternatives.iter().flatten()));
            for alternative in alternatives {
                if let environment_store::Alternative::Body {environment:Some(child),..}=alternative {queue.push(*child);}
            }
        }
        pending.push((destination,bindings));
    }
    fn remap(alternatives:&mut Option<Vec<environment_store::Alternative>>,mapping:&HashMap<environment_store::Handle,environment_store::Handle>) {
        for alternative in alternatives.iter_mut().flatten() {
            if let environment_store::Alternative::Body {environment:Some(handle),..}=alternative {
                if let Some(destination)=mapping.get(handle) {*handle=*destination;}
            }
        }
    }
    // Commit only after the complete graph validates. The iterative mapping
    // preserves shared edges and cycles without recursive graph duplication.
    let mut outgoing=state.clone();
    for (destination,mut bindings) in pending {
        for binding in &mut bindings {
            remap(&mut binding.alternatives,&mapping);
            if let Some(shape)=&mut binding.elements {
                for element in shape.prefix.iter_mut().chain(shape.tail.iter_mut()) {remap(&mut element.alternatives,&mapping);}
            }
        }
        if outgoing.captures.contains_key(&destination) {
            let Some(environment_state::Capture::Live {instance,..})=outgoing.captures.get_mut(&destination) else {return false;};
            *instance=environment_state::Instance::Merged;
            if outgoing.update(destination,&bindings,engine.declarations).is_none() {return false;}
        } else if outgoing.install(destination,&bindings,environment_state::Instance::OwnedSingle).is_none() {return false;}
    }
    remap(&mut value.alternatives,&mapping);*state=outgoing;true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binding(slot:usize)->Binding {
        Binding {alternatives:None,identity:Some(crate::local_capture_plan::BindingId {owner:1,declaration_start:slot}),name:format!("capture{slot}"),scope:Span::new(0,100),targets:vec![Target::Unknown],origins:vec![body_origins::Origin::Unknown],selection:Selection::default(),elements:None,mutable:true,contract:None}
    }
    fn edge(handle:environment_store::Handle)->environment_store::Alternative {
        environment_store::Alternative::Body {declaration:Span::new(80,81),environment:Some(handle)}
    }
    fn make_value(alternatives:Option<Vec<environment_store::Alternative>>)->expression_flow::Value {
        let mut result=expression_flow::Value::default();result.alternatives=alternatives;result
    }
    fn handle(value:&expression_flow::Value)->environment_store::Handle {
        let environment_store::Alternative::Body {environment:Some(handle),..}=value.alternatives.as_ref().unwrap()[0] else {panic!("owned handle")};handle
    }
    #[test]
    fn owned_capture_cycles_remap_once_and_invalid_children_leave_state_unchanged() {
        let program=crate::syntax::parse("function main() {}").unwrap();let effects=effect_bodies::Index::new(&program).unwrap();let declarations=HashMap::new();
        let mut engine=Engine::new(&declarations,&effects);engine.solve(program.functions[0].name.span,&[],&[]).unwrap();engine.current=Some(0);
        let a=binding(1);let b=binding(2);
        let root=engine.environments.allocate(Span::new(10,11),Span::new(30,31),&[a.clone()]).unwrap();
        let child=engine.environments.allocate(Span::new(12,13),Span::new(30,31),&[b.clone()]).unwrap();
        let mut a=a;a.alternatives=Some(vec![edge(child)]);let mut b=b;b.alternatives=Some(vec![edge(root)]);
        let mut state=FlowState::default();state.install(root,&[a],environment_state::Instance::OwnedSingle).unwrap();state.install(child,&[b],environment_state::Instance::OwnedSingle).unwrap();
        let mut value=make_value(Some(vec![edge(root)]));let before=state.clone();
        let expression=Expr::new(ExprKind::Undefined,Span::new(50,51));
        assert!(transfer(&mut engine,&expression,&mut value,&mut state));let transferred=handle(&value);assert_ne!(transferred,root);
        assert!(state.captures[&root]==before.captures[&root]);assert!(state.captures[&child]==before.captures[&child]);
        let environment_state::Capture::Live {bindings,..}=&state.captures[&transferred] else {panic!("root live")};
        let child_value=make_value(bindings[0].alternatives.clone());let moved_child=handle(&child_value);assert_ne!(moved_child,child);
        let environment_state::Capture::Live {bindings,..}=&state.captures[&moved_child] else {panic!("child live")};assert_eq!(bindings[0].alternatives,Some(vec![edge(transferred)]));
        for _ in 0..128 {assert!(transfer(&mut engine,&expression,&mut value,&mut state));assert_eq!(handle(&value),transferred);}
        assert_eq!(state.captures.len(),4);
        let mut borrowed=before.clone();
        let environment_state::Capture::Live {bindings,..}=&borrowed.captures[&root] else {panic!("borrowed root")};
        borrowed.bindings=bindings.clone();let unchanged=borrowed.clone();let mut borrowed_value=make_value(Some(vec![edge(root)]));
        assert!(transfer(&mut engine,&expression,&mut borrowed_value,&mut borrowed));assert_eq!(handle(&borrowed_value),root);assert!(borrowed==unchanged);
        let mut invalid=before;invalid.captures.insert(child,environment_state::Capture::Unknown);let saved=invalid.clone();let mut value=make_value(Some(vec![edge(root)]));let original=value.clone();
        assert!(!transfer(&mut engine,&expression,&mut value,&mut invalid));assert!(invalid==saved);assert!(value==original);
    }
}
