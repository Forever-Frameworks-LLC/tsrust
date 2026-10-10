//! Direct local replacements receive independent native lifetime inference.
use super::*;

impl Analyzer {
    pub(super) fn native_view_storage_epochs(&self,context:&mut Context,body:&mut [Stmt])->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        // A shadow at this lexical sequence replaces the live storage for all
        // subsequent uses. Branch/loop joins require separate carrier planning.
        for statement in body {
            let StmtKind::Assign{target,value,operator:None}=&mut statement.kind else{continue;};
            if !matches!(target.kind,ExprKind::Name(_)) {continue;}
            let Some(binding)=target.binding else{continue;};
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
