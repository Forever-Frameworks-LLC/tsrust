//! Direct local replacements receive independent native lifetime inference.
use super::*;

impl Analyzer {
    /// A local callback array has one final family. A callback appended while the
    /// array still had a narrower family is moved into the final one.
    fn native_view_array_carriers(&self,context:&Context,body:&mut [Stmt])->Result<(),Diagnostic> {
        let finals=context.bindings.iter().enumerate().filter_map(|(id,binding)|match binding.ty {
            Type::Array(ArrayElement::StoredCallable(family,0)) if self.callables.storage.scoped_native_family(family).is_some()=>Some((id,family)),
            _=>None,
        }).collect::<HashMap<_,_>>();
        if finals.is_empty() {return Ok(());}
        let convert=|value:&mut Expr,family:usize| {
            if let Some(Type::StoredCallable(source))=value.ty && source!=family && self.callables.storage.scoped_native_family(source).is_some() {
                value.wrap_record_carrier(Type::StoredCallable(family));
            }
        };
        for statement in body.iter_mut() {
            if let StmtKind::Let{binding:Some(binding),value,..}=&mut statement.kind
                && let Some(&family)=finals.get(binding)
                && let ExprKind::Array(values)=&mut value.kind {
                for value in values {convert(value,family);}
                value.ty=Some(Type::Array(ArrayElement::StoredCallable(family,0)));
            }
        }
        crate::source_walk::body(body,&mut |_|Ok(()),&mut |expression|{
            if let ExprKind::Call(callee,arguments)=&mut expression.kind
                && let ExprKind::Field(base,name)=&mut callee.kind
                && name=="push"
                && let Some(binding)=base.binding
                && matches!(base.kind,ExprKind::Name(_))
                && let Some(&family)=finals.get(&binding)
                && let Some(value)=arguments.first_mut() {
                convert(value,family);
                base.ty=Some(Type::Array(ArrayElement::StoredCallable(family,0)));
            }
            Ok(())
        })
    }
    pub(super) fn native_view_storage_epochs(&self,context:&mut Context,body:&mut [Stmt])->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        self.native_view_array_carriers(context,body)?;
        // A shadow at this lexical sequence replaces the live storage for all
        // subsequent uses. Branch/loop joins require separate carrier planning.
        // Only the declaring block can shadow the local for the rest of its scope.
        let declared=body.iter().filter_map(|statement|match &statement.kind {StmtKind::Let{binding:Some(binding),..}=>Some(*binding),_=>None}).collect::<Vec<_>>();
        for statement in body {
            let StmtKind::Assign{target,value,operator:None}=&mut statement.kind else{continue;};
            if !matches!(target.kind,ExprKind::Name(_)) {continue;}
            let Some(binding)=target.binding else{continue;};
            if !declared.contains(&binding) {continue;}
            let slot=&context.bindings[binding];
            if slot.parameter || slot.capture_reference || slot.borrowed_place.is_some() || slot.mutable_place {continue;}
            let Some(ty)=value.ty else{continue;};
            if !self.native_shape_contains_view(ty) {continue;}
            // A self-dependent RHS may transfer the old value. Its ordinary
            // assignment already retains the same lifetime and disposal rules.
            let mut reads_self=false;
            crate::source_walk::expression(value,&mut |expression|{reads_self|=expression.binding==Some(binding);Ok(())})?;
            if !reads_self {context.storage_exits.record_array_result_write(statement.span,ty);}
        }
        Ok(())
    }
}
