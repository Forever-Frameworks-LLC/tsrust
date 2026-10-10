//! Input views carry lifetime origins without borrowing their own parameter slot.
use super::*;
use super::native_view_loans::{LifetimeSource, Loan, Segment, VariantGuard};

impl Analyzer {
    fn native_shape_children(&self, ty: Type) -> Vec<(Type, Option<Segment>, Option<usize>)> {
        match ty {
            Type::Array(element) => vec![(element.ty(), Some(Segment::Element), None)],
            Type::StoredCallable(family)=>self.callables.storage.scoped_native_family(family).into_iter().flat_map(|layout|
                layout.alternatives.iter().map(|alternative|(Type::Closure(alternative.origin),Some(Segment::CallableAlternative(alternative.origin)),None))
            ).collect(),
            Type::Closure(id) => self.closures[id].captures.iter().map(|capture|
                (self.closures[id].function.bindings[capture.inner].ty, Some(Segment::Field(format!("capture{}",capture.inner))), None)).collect(),
            Type::Record(id) => self.program.records[id].fields.iter().map(|field|
                (field.ty.expect("checked field"), Some(Segment::Field(field.name.text.clone())), None)).collect(),
            Type::Union(id) | Type::BorrowedUnion(id) => self.unions.members(id).into_iter().map(|member|
                (member, None, if let Type::Record(id)=member {Some(id)} else {None})).collect(),
            _ => vec![],
        }
    }

    pub(super) fn native_shape_contains_view(&self, ty: Type) -> bool {
        let mut pending=vec![ty];
        let mut seen=HashSet::new();
        while let Some(ty)=pending.pop() {
            if !seen.insert(ty) {continue;}
            if let Type::Record(id)=ty
                && self.program.records[id].native_type.is_some_and(|id|crate::native_types::declaration(id).borrow_access().is_some()) {
                return true;
            }
            pending.extend(self.native_shape_children(ty).into_iter().map(|(ty,_,_)|ty));
        }
        false
    }

    /// A storage type that reaches itself and holds a byte view. Its native layout
    /// would need one lifetime shared by every level, which is not planned yet.
    pub(super) fn reject_recursive_native_view_storage(&self, ty: Type, span: Span) -> Result<(), Diagnostic> {
        if !self.native_borrow_types || !self.native_shape_contains_view(ty) {return Ok(());}
        let mut pending=vec![(ty,Vec::<Type>::new())];
        while let Some((ty,mut ancestors))=pending.pop() {
            if !self.native_shape_contains_view(ty) {continue;}
            if ancestors.contains(&ty) {
                return Err(Diagnostic::new("native-view-shape","A type that contains itself cannot hold a byte view yet. Keep the views in a separate array or record.",span));
            }
            ancestors.push(ty);
            for (child,_,_) in self.native_shape_children(ty) {pending.push((child,ancestors.clone()));}
        }
        Ok(())
    }

    pub(super) fn seed_native_parameters(&self, context: &mut Context) -> Result<(), Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        for root in 0..context.bindings.len() {
            let binding=&context.bindings[root];
            if !binding.parameter || !self.native_shape_contains_view(binding.ty) {continue;}
            let creation=binding.declaration_span.unwrap_or_default();
            let mut pending=vec![(binding.ty,Vec::<Segment>::new(),Vec::<VariantGuard>::new(),Vec::<Type>::new())];
            let mut loans=Vec::new();
            while let Some((ty,path,guards,mut ancestors))=pending.pop() {
                if let Type::Record(id)=ty
                    && let Some(access)=self.program.records[id].native_type.and_then(|id|crate::native_types::declaration(id).borrow_access()) {
                    let owner_fields=path.iter().take_while(|segment|matches!(segment,Segment::Field(_)))
                        .map(|segment|match segment {Segment::Field(name)=>name.clone(),_=>unreachable!()}).collect();
                    loans.push(Loan{root,owner_fields,source:LifetimeSource::carried(path.clone(),guards.clone()),access,creation,retained:false,path,guards});
                    continue;
                }
                if !self.native_shape_contains_view(ty) {continue;}
                if ancestors.contains(&ty) {
                    return Err(Diagnostic::new("native-view-shape", "Recursive input storage containing native views requires recursive lifetime planning.",creation));
                }
                ancestors.push(ty);
                for (child,segment,variant) in self.native_shape_children(ty) {
                    let mut child_path=path.clone();
                    let mut child_guards=guards.clone();
                    if let Some(success)=variant {child_guards.push(VariantGuard{path:path.clone(),success});}
                    if let Some(segment)=segment {child_path.push(segment);}
                    pending.push((child,child_path,child_guards,ancestors.clone()));
                }
            }
            native_view_projections::normalize(&mut loans);
            Self::set_native_view_loans(context,root,loans);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn checked(source:&str)->Result<CheckedProgram,Diagnostic> {
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=
            Some(crate::native_signatures::NativeTypeId::ByteView);
        analyze(parsed)
    }
    #[test]
    fn source_function_return_preserves_consumed_view_lifetime() {
        let checked=checked("type Window={};function identity(input:Window):Window{return input;}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="identity").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),1);
        assert_eq!(origins[0].source,LifetimeSource::Carried(vec![]));
    }
    #[test]
    fn record_fields_and_array_ranks_have_distinct_carried_slots() {
        let checked=checked("type Window={};type Pair={first:Window;rows:Window[][];};function identity(input:Pair):Pair{return input;}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="identity").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),2);
        assert!(origins.iter().any(|loan|loan.source==LifetimeSource::Carried(vec![Segment::Field("first".into())])));
        assert!(origins.iter().any(|loan|loan.source==LifetimeSource::Carried(vec![Segment::Field("rows".into()),Segment::Element,Segment::Element])));
    }
    #[test]
    fn recursive_view_carrier_gets_source_diagnostic() {
        let error=checked("type Window={};type Chain={next:Chain[];view:Window;};function identity(input:Chain):Chain{return input;}function main(){}").err().expect("recursive carrier must be diagnosed");
        assert_eq!(error.code,"native-view-shape");
    }
    #[test]
    fn unrelated_recursive_types_do_not_trigger_view_diagnostic() {
        checked("type Window={};type Chain={next:Chain[];value:int;};function identity(input:Chain):Chain{return input;}function main(){}").unwrap();
    }
    #[test]
    fn result_input_origin_retains_success_variant_guard() {
        let checked=checked("type Window={};type Outcome=Result<Window,int>;function identity(input:Outcome):Outcome{return input;}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="identity").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),1);
        assert_eq!(origins[0].guards.len(),1);
        assert_eq!(origins[0].path.len(),1);
    }
    #[test]
    fn alternative_inputs_retain_independent_origins() {
        let checked=checked("type Window={};function choose(left:Window,right:Window,flag:boolean):Window{if(flag){return left;}return right;}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="choose").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),2);
        assert_ne!(origins[0].root,origins[1].root);
    }

    #[test]
    fn forwarding_call_keeps_returned_input_origin() {
        let checked=checked("type Window={};function identity(input:Window):Window{return input;}function forward(input:Window):Window{return identity(input);}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="forward").unwrap();
        let origins=function.native_returns.as_ref().expect("forwarded view needs its source lifetime");
        assert_eq!(origins.len(),1);
        assert_eq!(origins[0].root,0);
        assert_eq!(origins[0].source,LifetimeSource::Carried(vec![]));
    }

    #[test]
    fn nested_forwarding_selects_only_the_returned_field() {
        let checked=checked("type Window={};type Pair={one:Window;two:Window;};function first(input:Pair):Window{return input.one;}function identity(input:Window):Window{return input;}function forward(input:Pair):Window{return identity(first(input));}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="forward").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),1);
        assert_eq!(origins[0].source,LifetimeSource::Carried(vec![Segment::Field("one".into())]));
    }
    #[test]
    fn forwarding_alternatives_keep_both_actual_sources() {
        let checked=checked("type Window={};function choose(left:Window,right:Window,flag:boolean):Window{if(flag){return left;}return right;}function forward(first:Window,second:Window,flag:boolean):Window{return choose(first,second,flag);}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="forward").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),2);
        assert_ne!(origins[0].root,origins[1].root);
    }
    #[test]
    fn saved_function_call_keeps_returned_origin() {
        let checked=checked("type Window={};function identity(input:Window):Window{return input;}function forward(input:Window):Window{const saved=identity;return saved(input);}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="forward").unwrap();
        assert_eq!(function.native_returns.as_ref().unwrap().len(),1);
    }

    #[test]
    fn sibling_calls_keep_distinct_output_paths() {
        let checked=checked("type Window={};type Pair={one:Window;two:Window;};function identity(input:Window):Window{return input;}function assemble(first:Window,second:Window):Pair{return{one:identity(first),two:identity(second)};}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="assemble").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),2);
        assert!(origins.iter().any(|loan|loan.root==0&&loan.path==vec![Segment::Field("one".into())]));
        assert!(origins.iter().any(|loan|loan.root==1&&loan.path==vec![Segment::Field("two".into())]));
    }

    #[test]
    fn arrow_call_keeps_returned_input_origin() {
        let checked=checked("type Window={};function forward(input:Window):Window{const identity=(value:Window):Window=>value;return identity(input);}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="forward").unwrap();
        let origins=function.native_returns.as_ref().expect("arrow result needs an input lifetime");
        assert_eq!(origins.len(),1);
        assert_eq!(origins[0].source,LifetimeSource::Carried(vec![]));
    }

    #[test]
    fn arrow_alternatives_with_scalar_capture_keep_both_inputs() {
        let checked=checked("type Window={};function choose(left:Window,right:Window,flag:boolean):Window{const callback=(first:Window,second:Window):Window=>{if(flag){return first;}return second;};return callback(left,right);}function main(){}").unwrap();
        let function=checked.functions.iter().find(|function|function.name=="choose").unwrap();
        let origins=function.native_returns.as_ref().unwrap();
        assert_eq!(origins.len(),2);
        assert_ne!(origins[0].root,origins[1].root);
    }

}

#[cfg(test)]
#[test]
fn native_view_scoped_family_parameters_seed_each_environment_slot() {
    for mutable in [false,true] {
        let source="type Window={};type Pair={one:Window;two:Window;};function first(pair:Pair){return ()=>pair;}function second(pair:Pair){return ()=>pair;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(if mutable {crate::native_signatures::NativeTypeId::ByteViewMut}else{crate::native_signatures::NativeTypeId::ByteView});
        let mut analyzer=Analyzer::new(parsed).unwrap();
        let factories=analyzer.program.functions.iter().enumerate().filter_map(|(id,function)|matches!(function.name.text.as_str(),"first"|"second").then_some(id)).collect::<Vec<_>>();
        for factory in factories {analyzer.function(factory).unwrap();}
        let result=analyzer.closures[0].function.result;
        let structural=analyzer.callables.shapes.intern([],result,Some(CallbackKind::OwnedOnce));
        let callback=CallbackContract{structural,parameter_names:vec![],params:vec![],result,kind:CallbackKind::OwnedOnce};
        let scoped=analyzer.callables.storage.intern_scoped_native(&analyzer.callables.shapes,&callback,0,[0,1],&analyzer.closures,Span::default()).unwrap().unwrap();
        let mut context=Context::new("seed".into());
        let token=Token{text:"action".into(),kind:TokenKind::Name,span:Span::default()};
        let root=Analyzer::bind(&mut context,&token,scoped,false).unwrap();
        context.bindings[root].parameter=true;
        analyzer.seed_native_parameters(&mut context).unwrap();
        let loans=context.bindings[root].native_view_loans.as_ref().expect("scoped family parameters carry their payload lifetimes");
        assert_eq!(loans.len(),4);
        for origin in [0,1] {
            let capture=analyzer.closures[origin].captures[0].inner;
            for field in ["one","two"] {
                let path=vec![Segment::CallableAlternative(origin),Segment::Field(format!("capture{capture}")),Segment::Field(field.into())];
                assert!(loans.iter().any(|loan|loan.root==root && loan.path==path && loan.source==LifetimeSource::Carried(path.clone()) && loan.access==if mutable {crate::native_return_borrows::BorrowAccess::Exclusive}else{crate::native_return_borrows::BorrowAccess::ReadOnly}));
            }
        }
        let Type::StoredCallable(family)=scoped else {unreachable!()};
        analyzer.remember_native_family_returns(&mut context,family,Span::default()).unwrap();
        let mut callee=Expr::new(ExprKind::Name("action".into()),Span::default());callee.binding=Some(root);callee.ty=Some(scoped);
        let mut call=Expr::new(ExprKind::Call(Box::new(callee),vec![]),Span::default());
        call.target=Some(CallTarget::StoredCallable(root));call.callable_origin=Some(CallableOrigin::Storage(family));call.ty=Some(result);
        let returned=Analyzer::native_view_value_loans(&context,&call).unwrap();
        assert_eq!(returned.len(),4);
        assert!(returned.iter().all(|loan|loan.root==root && loan.path.len()==1 && matches!(loan.source,LifetimeSource::Carried(ref path) if matches!(path.first(),Some(Segment::CallableAlternative(_))))));
        let mut construction=Context::new("construction".into());
        let token=Token{text:"pair".into(),kind:TokenKind::Name,span:Span::default()};
        let pair=Analyzer::bind(&mut construction,&token,result,false).unwrap();construction.bindings[pair].parameter=true;
        analyzer.seed_native_parameters(&mut construction).unwrap();
        let capture=analyzer.closures[0].captures[0].inner;
        construction.native_view_environments.get_or_insert_with(Default::default).insert(0,vec![(capture,pair)]);
        let mut closure=Expr::new(ExprKind::Closure{id:0,params:vec![],result:ResultAnnotation::Inferred(Span::default()),body:vec![]},Span::default());closure.ty=Some(Type::Closure(0));closure.wrap_storage(family);
        let stored=Analyzer::native_view_value_loans(&construction,&closure).unwrap();
        assert_eq!(stored.len(),2);
        assert!(stored.iter().all(|loan|loan.root==pair && loan.path.first()==Some(&Segment::CallableAlternative(0))));
        let token=Token{text:"action".into(),kind:TokenKind::Name,span:Span::default()};
        let action=Analyzer::bind(&mut construction,&token,scoped,false).unwrap();Analyzer::set_native_view_loans(&mut construction,action,stored);
        analyzer.remember_native_family_returns(&mut construction,family,Span::default()).unwrap();
        let mut callee=Expr::new(ExprKind::Name("action".into()),Span::default());callee.binding=Some(action);callee.ty=Some(scoped);
        let mut call=Expr::new(ExprKind::Call(Box::new(callee),vec![]),Span::default());call.target=Some(CallTarget::StoredCallable(action));call.callable_origin=Some(CallableOrigin::Storage(family));call.ty=Some(result);
        let returned=Analyzer::native_view_value_loans(&construction,&call).unwrap();
        assert_eq!(returned.len(),2,"inactive alternatives must not invent owners or reject construction");
        assert!(returned.iter().all(|loan|loan.root==pair && loan.source==LifetimeSource::Carried(loan.path.clone())));

    }
}
