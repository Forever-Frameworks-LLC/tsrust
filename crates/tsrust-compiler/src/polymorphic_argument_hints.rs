//! Polymorphic argument identity is metadata; ordinary checking owns access effects.
use super::*;
impl Analyzer {
    pub(super) fn polymorphic_argument_annotations(&self, context: &Context, arguments: &[Expr]) -> Result<HashMap<Type,crate::type_syntax::ValueAnnotation>,Diagnostic> {
        let mut annotations=HashMap::new();
        for argument in arguments {
            let hint=argument.ty.or_else(||self.expression_type_hint(context,argument));
            if let Some(Type::Closure(origin))=hint && self.deferred_generic_values.contains_key(&origin) {
                annotations.insert(Type::Closure(origin),self.checked_value_annotation(Type::Closure(origin),argument.span)?);
            }
        }
        Ok(annotations)
    }

    pub(super) fn prepare_polymorphic_argument_hint(&mut self, context: &Context, argument: &mut Expr) -> Result<Option<Type>,Diagnostic> {
        let ExprKind::Name(name)=&argument.kind else {return Ok(None)};
        if context.names.contains_key(name) || context.pending_names.contains(name) || self.generic_functions.template(name).is_none() {return Ok(None);}
        if let Some(Type::Closure(origin))=argument.ty && self.deferred_generic_values.get(&origin).is_some_and(|source|source.declaration==*name && source.span==argument.span) {return Ok(argument.ty);}
        // Capture layout checking may access compiler binding state. Use a
        // preparation context so inference never consumes the actual argument.
        let mut preparation=context.clone();
        let ty=self.deferred_captured_generic_value(&mut preparation,name,usize::MAX-argument.span.start,argument.span,&Loans::new())?;
        argument.ty=Some(ty);
        Ok(Some(ty))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_declaration_hint_retains_source_and_reuses_identity_without_rewriting() {
        let source=crate::format("function identity<T>(value:T):T{return value;}function main(){}").unwrap();
        let mut analyzer=Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let context=Context::new("main".into());
        let span=Span::new(100,108);
        let mut argument=Expr::new(ExprKind::Name("identity".into()),span);
        let ty=analyzer.prepare_polymorphic_argument_hint(&context,&mut argument).unwrap().expect("polymorphic hint");
        let Type::Closure(origin)=ty else {panic!("source environment identity")};
        assert_eq!(analyzer.deferred_generic_values[&origin].declaration,"identity");
        assert!(analyzer.closures[origin].polymorphic);
        assert!(analyzer.closures[origin].parameters.is_empty(),"do not invent a monomorphic contract");
        assert!(matches!(&argument.kind,ExprKind::Name(name) if name=="identity"));
        assert_eq!(argument.ty,Some(ty));
        let annotation=analyzer.checked_value_annotation(ty,span).expect("polymorphic internal annotation");
        let mut parameter=crate::type_syntax::parse("A").unwrap();
        crate::type_substitution::apply(&mut parameter,&HashMap::from([("A".into(),annotation)])).unwrap();
        assert_eq!(analyzer.callable_resolver().resolve_value_annotation(&parameter,false).unwrap(),ty);
        assert!(matches!(parameter.kind,crate::type_syntax::ValueAnnotationKind::Polymorphic {..}));
        let mut remapped=parameter.clone();
        remap_annotation(&mut remapped,&[Some(7)]).unwrap();
        assert!(matches!(remapped.kind,crate::type_syntax::ValueAnnotationKind::Polymorphic {origin:7,..}));
        assert!(remap_annotation(&mut parameter.clone(),&[None]).is_err());
        assert!(remap_annotation(&mut parameter.clone(),&[]).is_err());


        let count=analyzer.closures.len();
        assert_eq!(analyzer.prepare_polymorphic_argument_hint(&context,&mut argument).unwrap(),Some(ty));
        assert_eq!(analyzer.closures.len(),count);
        assert!(context.bindings.is_empty());
        let mut shadowed=context.clone();
        shadowed.names.insert("identity".into(),0);
        assert_eq!(analyzer.prepare_polymorphic_argument_hint(&shadowed,&mut argument).unwrap(),None,"lexical binding shadows generic declaration");
        let mut pending=context.clone();
        pending.pending_names.insert("identity".into());
        assert_eq!(analyzer.prepare_polymorphic_argument_hint(&pending,&mut argument).unwrap(),None);
        assert_eq!(analyzer.closures.len(),count);
        let mut ordinary=Expr::new(ExprKind::Integer("7".into()),span);
        assert_eq!(analyzer.prepare_polymorphic_argument_hint(&context,&mut ordinary).unwrap(),None);
    }
}

pub(super) fn remap_annotation(value: &mut crate::type_syntax::ValueAnnotation, ids: &[Option<usize>]) -> Result<(),Diagnostic> {
    use crate::type_syntax::ValueAnnotationKind as Kind;
    let mut pending=vec![value];
    while let Some(value)=pending.pop() {
        match &mut value.kind {
            Kind::Polymorphic {origin,..} => {
                *origin=ids.get(*origin).copied().flatten().ok_or_else(||Diagnostic::new("callable-layout","Polymorphic annotation retains an unresolved environment identity.",value.span))?;
            }
            Kind::Named(_)=>{},
            Kind::Group(inner)|Kind::Array(inner)=>pending.push(inner),
            Kind::Applied{arguments,..}|Kind::Union(arguments)|Kind::Tuple(arguments)=>pending.extend(arguments),
            Kind::Callable{parameters,result,..}=>{
                pending.extend(parameters.iter_mut().map(|parameter|&mut parameter.annotation));
                pending.push(result);
            }
        }
    }
    Ok(())
}
