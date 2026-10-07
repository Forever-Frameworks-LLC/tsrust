//! Source retention facts reuse the lexical callable body index.
use super::*;
use crate::callable_environment_demand::{Demand,Graph};
use crate::local_capture_plan::BindingId;
use std::collections::{BTreeMap,BTreeSet,VecDeque};



fn direct_captures(value:&Expr,index:&effect_bodies::Index)->Option<BTreeSet<BindingId>> {
    let captures=index.captures(value.span)?.iter().copied().collect();
    let mut probe=vec![Stmt{kind:StmtKind::Expr(value.clone()),span:value.span,labels:Vec::new(),control_target:None}];
    let mut nested=false;
    crate::source_walk::body(&mut probe,&mut |_|Ok(()),&mut |expression|{
        if matches!(expression.kind,ExprKind::Closure{..})&&expression.span!=value.span{nested=true;}
        Ok(())
    }).ok()?;
    // The lexical index reports direct captures. Nested environments require
    // composition with checked capture facts before certifying non-retention.
    (!nested).then_some(captures)
}
fn origins(value:&Expr,index:&effect_bodies::Index)->Option<BTreeSet<BindingId>> {
    let mut result=BTreeSet::new();let mut pending=vec![value];
    while let Some(value)=pending.pop() {
        match &value.kind {
            ExprKind::Name(name)=>{if let Some(binding)=index.binding(name,value.span){result.insert(binding);}},
            ExprKind::Closure{..}=>result.extend(direct_captures(value,index)?),
            ExprKind::Conditional{consequent,alternate,..}=>pending.extend([consequent.as_ref(),alternate.as_ref()]),
            ExprKind::Record(fields)=>pending.extend(fields.iter().map(|(_,value)|value)),
            ExprKind::Array(values)|ExprKind::ArrayOf(values,_)=>pending.extend(values),
            ExprKind::Sequence(values)=>{if let Some(value)=values.last(){pending.push(value);}},
            ExprKind::TypeAssertion{value,..}|ExprKind::OptionalChain(value)|ExprKind::Field(value,_)=>pending.push(value),
            ExprKind::Assign{value,..}|ExprKind::CoalesceAssign{value,..}|ExprKind::LogicalAssign{value,..}=>pending.push(value),
            ExprKind::Unary{op:UnaryOp::NonNull|UnaryOp::Spread|UnaryOp::RestPack,value,..}=>pending.push(value),
            ExprKind::Binary{op:BinaryOp::Coalesce|BinaryOp::PatternDefault,left,right,..}=>pending.extend([left.as_ref(),right.as_ref()]),
            // Calls return their declared result, not their receiver environment.
            // Argument admission is accounted for separately by call destinations.
            ExprKind::Call(..)|ExprKind::String(_)|ExprKind::Integer(_)|ExprKind::Number(_)|ExprKind::Boolean(_)|ExprKind::Null|ExprKind::Undefined|ExprKind::EnumMember{..}=>{},
            ExprKind::Binary{..}|ExprKind::Unary{..}|ExprKind::Update{..}|ExprKind::Template(_)=>{},
            // Checked projections and aggregate lookup need checked provenance.
            _=>return None,
        }
    }
    Some(result)
}
fn unknown(graph:&mut Graph,parameters:&[BindingId]){for &parameter in parameters{graph.unknown(parameter);}}
fn retain(graph:&mut Graph,value:&Expr,index:&effect_bodies::Index,parameters:&[BindingId]) {
    if let Some(origins)=origins(value,index){for origin in origins{graph.retains(origin);}}else{unknown(graph,parameters);}
}
fn transfer(graph:&mut Graph,value:&Expr,destination:Option<BindingId>,index:&effect_bodies::Index,parameters:&[BindingId]) {
    if let Some(origins)=origins(value,index){
        for origin in origins {if let Some(destination)=destination{graph.forwards(origin,destination);}else{graph.unknown(origin);}}
    }else{unknown(graph,parameters);}
}

fn parameter_aliases(program:&Program,index:&effect_bodies::Index)->BTreeSet<BindingId>{
    let mut trusted=BTreeSet::new();let mut aliases=Vec::new();
    for function in &program.functions {
        for (name,annotation,callback) in &function.params {
            if (callback.is_some()||crate::callable_annotations::callback(annotation).is_some())&&let Some(binding)=index.declared(name.span){trusted.insert(binding);}
        }
        let mut pending=function.body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop(){match &statement.kind {
            StmtKind::Let{name,value,..}=>if let ExprKind::Name(source)=&value.kind&&let (Some(source),Some(destination))=(index.binding(source,value.span),index.declared(name.span)){aliases.push((source,destination));},
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body)),
            StmtKind::Loop{body,update,..}=>pending.extend(body.iter().chain(update)),
            StmtKind::Try(region)=>pending.extend(region.statements()),
            _=>{},
        }}
    }
    let mut destinations:BTreeMap<BindingId,Vec<BindingId>>=BTreeMap::new();
    for (source,destination) in aliases{destinations.entry(source).or_default().push(destination);}
    let mut pending=trusted.iter().copied().collect::<VecDeque<_>>();
    while let Some(source)=pending.pop_front(){
        for &destination in destinations.get(&source).into_iter().flatten(){if trusted.insert(destination){pending.push_back(destination);}}
    }
    trusted
}
fn infer(program:&Program)->Result<BTreeMap<BindingId,Demand>,Diagnostic>{
    let index=effect_bodies::Index::new(program)?;let mut graph=Graph::default();
    let mut capture_admission=BTreeSet::new();
    let trusted=parameter_aliases(program,&index);
    let declarations=program.functions.iter().map(|function|(function.name.text.as_str(),function)).collect::<HashMap<_,_>>();
    for function in &program.functions {
        let parameters=function.params.iter().filter_map(|(name,_,_)|index.declared(name.span)).collect::<Vec<_>>();
        for &parameter in &parameters{graph.checked(parameter);}
        let mut pending=function.body.iter().collect::<Vec<_>>();
        while let Some(statement)=pending.pop(){match &statement.kind {
            StmtKind::Let{name,value,annotation,..}=>{
                let destination=index.declared(name.span);
                if let Some(destination)=destination{graph.checked(destination);}else{unknown(&mut graph,&parameters);}
                transfer(&mut graph,value,destination,&index,&parameters);
                if annotation.as_ref().and_then(crate::callable_annotations::ownership).is_some_and(CallbackKind::is_owned){retain(&mut graph,value,&index,&parameters);}
            },
            StmtKind::Return(Some(value))|StmtKind::Throw(value)=>retain(&mut graph,value,&index,&parameters),
            StmtKind::Assign{target,value,..}=>{
                if let ExprKind::Name(name)=&target.kind{transfer(&mut graph,value,index.binding(name,target.span),&index,&parameters);}else{retain(&mut graph,value,&index,&parameters);}
            },
            StmtKind::Block(body)|StmtKind::While{body,..}=>pending.extend(body),
            StmtKind::If{then_body,else_body,..}=>pending.extend(then_body.iter().chain(else_body)),
            StmtKind::Loop{body,update,..}=>pending.extend(body.iter().chain(update)),
            StmtKind::ForOf{body,..}=>{pending.extend(body);unknown(&mut graph,&parameters);},
            StmtKind::Try(region)=>pending.extend(region.statements()),
            StmtKind::Expr(_)|StmtKind::Break|StmtKind::Continue|StmtKind::Return(None)=>{},
            StmtKind::LocalFunction(_)=>unknown(&mut graph,&parameters),
        }}
        // Use the existing iterative walker for expression effects and defaults.
        let mut probe=function.clone();
        crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression|{
            match &mut expression.kind {
                ExprKind::Call(callee,args)=>{
                    let destination=if let ExprKind::Name(name)=&callee.kind{
                        if index.binding(name,callee.span).is_none(){declarations.get(name.as_str()).copied()}else{None}
                    }else{None};
                    if destination.is_none() {
                        if let Some(origins)=origins(callee,&index){for origin in origins{if !trusted.contains(&origin){graph.unknown(origin);}}}else{unknown(&mut graph,&parameters);}
                    }
                    let mut spread=false;
                    for (position,argument) in args.iter().enumerate(){
                        spread|=matches!(argument.kind,ExprKind::Unary{op:UnaryOp::Spread,..});
                        let binding=(!spread).then(||destination.and_then(|function|function.params.get(position).and_then(|(name,_,_)|index.declared(name.span)))).flatten();
                        transfer(&mut graph,argument,binding,&index,&parameters);
                    }
                },
                ExprKind::Assign{target,value,..}|ExprKind::CoalesceAssign{target,value,..}|ExprKind::LogicalAssign{target,value,..}=>{
                    if let ExprKind::Name(name)=&target.kind{transfer(&mut graph,value,index.binding(name,target.span),&index,&parameters);}else{retain(&mut graph,value,&index,&parameters);}
                },
                ExprKind::Record(_)|ExprKind::Array(_)|ExprKind::ArrayOf(_,_)=>retain(&mut graph,expression,&index,&parameters),
                ExprKind::Closure{..}=>{
                    if let Some(captures)=origins(expression,&index){for capture in captures{if trusted.contains(&capture){capture_admission.insert(capture);}}}else{unknown(&mut graph,&parameters);}
                    let ExprKind::Closure{body,params,..}=&mut expression.kind else{unreachable!()};
                    // Each closure has its own indexed body; enclosing analysis
                    // accounts for its capture environment, not its invocation.
                    body.clear();params.clear();
                },
                _=>{},
            }
            Ok(())
        })?;
    }
    let demands=graph.solve();
    // Retained environments already require ownership. A scoped capture still
    // needs checked lifetime admission before the source contract can change.
    for binding in capture_admission{if demands.get(&binding)==Some(&Demand::Scoped){graph.unknown(binding);}}
    Ok(graph.solve())
}


#[cfg(test)]
mod tests {
    use super::*;
    fn demands(source:&str)->Vec<(String,Vec<Demand>)> {
        let formatted=crate::format(source).unwrap();assert_eq!(crate::format(&formatted).unwrap(),formatted);
        let program=crate::syntax::parse(&formatted).unwrap();let index=effect_bodies::Index::new(&program).unwrap();
        let result=infer(&program).unwrap();
        program.functions.iter().filter(|function|!function.params.is_empty()).map(|function|{
            (function.name.text.clone(),function.params.iter().map(|(name,_,_)|result.get(&index.declared(name.span).unwrap()).copied().unwrap_or(Demand::Unknown)).collect())
        }).collect()
    }
    #[test]
    fn immediate_invocation_and_moved_aliases_are_scoped() {
        assert_eq!(demands("function finish(action:Once<Function<int>>):int{const next=action;return next();}function main(){}"),vec![("finish".into(),vec![Demand::Scoped])]);
    }
    #[test]
    fn returning_a_moved_callback_or_its_capturing_wrapper_requires_ownership() {
        for body in ["return next;","return ():int=>next();"] {
            let source=format!("function retain(action:Once<Function<int>>):Once<Function<int>>{{const next=action;{body}}}function main(){{}}");
            assert_eq!(demands(&source),vec![("retain".into(),vec![Demand::Owned])]);
        }
    }
    #[test]
    fn known_forwarding_propagates_scoped_and_retaining_destinations() {
        for (destination,result,expected) in [("return action();","int",Demand::Scoped),("return action;","Once<Function<int>>",Demand::Owned)] {
            let source=format!("function outer(action:Once<Function<int>>):{result}{{return inner(action);}}function inner(action:Once<Function<int>>):{result}{{{destination}}}function main(){{}}");
            assert_eq!(demands(&source),vec![("outer".into(),vec![expected]),("inner".into(),vec![expected])]);
        }
    }
    #[test]
    fn shadowed_bindings_do_not_retain_the_outer_callback() {
        assert_eq!(demands("function finish(action:Once<Function<int>>):int{{const action=():int=>7;const saved:Once<Function<int>>=action;}return action();}function main(){}"),vec![("finish".into(),vec![Demand::Scoped])]);
    }
    #[test]
    fn unknown_callback_destinations_cannot_certify_scoped_arguments() {
        assert_eq!(demands("function forward(action:Once<Function<int>>,destination:Function<void,[action:Once<Function<int>>]>){destination(action);}function main(){}"),vec![("forward".into(),vec![Demand::Unknown,Demand::Scoped])]);
    }
    #[test]
    fn an_unchecked_local_factory_result_cannot_hide_retained_captures() {
        assert_eq!(demands("function outer(action:Once<Function<int>>){const factory=()=>action;return factory();}function main(){}"),vec![("outer".into(),vec![Demand::Unknown])]);
    }
    #[test]
    fn transitive_nested_captures_require_complete_environment_facts() {
        assert_eq!(demands("function outer(action:Once<Function<int>>){return ()=>()=>action();}function main(){}"),vec![("outer".into(),vec![Demand::Unknown])]);
    }
    #[test]
    fn local_aggregate_storage_preserves_existing_owned_environment_requirements() {
        for value in ["{run:action}","[action]"] {
            let source=format!("function store(action:Once<Function<int>>):int{{const saved={value};return 7;}}function main(){{}}");
            assert_eq!(demands(&source),vec![("store".into(),vec![Demand::Owned])]);
        }
    }
    #[test]
    fn uninvoked_local_capture_needs_checked_environment_admission_facts() {
        assert_eq!(demands("function outer(action:Once<Function<int>>):int{const wrapper=()=>action();return 7;}function main(){}"),vec![("outer".into(),vec![Demand::Unknown])]);
    }
}
