//! Polymorphic argument identity is metadata; ordinary checking owns access effects.
use super::*;
impl Analyzer {
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
