//! URL members lower to native calls so views and mutations retain real loans.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
    pub(super) fn url_property_receiver<'a>(&self,context:&Context,target:&'a Expr)->Option<&'a Expr> {
        let ExprKind::Field(receiver,name)=&target.kind else{return None};
        (matches!(self.expression_type_hint(context,receiver),Some(Type::Record(id)) if self.program.records[id].native_type==Some(NativeTypeId::Url))
            && crate::stdlib::url::property(name).is_some()).then_some(receiver)
    }
    pub(super) fn url_store_selection_loans(&self,context:&Context,target:&Expr,loans:&Loans)->Option<Loans> {
        let receiver=self.url_property_receiver(context,target)?;
        let mut active=loans.clone();
        // A computed setter commits to the selected owner before checking its RHS.
        // Read reservations permit self reads and prevent replacement or invalidation.
        for root in Self::loan_roots(receiver,Mode::Read) {
            active.entry(root).or_default().push((Mode::Read,receiver.span));
        }
        Some(active)
    }
    pub(super) fn finish_url_store(&self,context:&mut Context,target:&Expr,loans:&Loans)->Result<bool,Diagnostic> {
        let Some(receiver)=self.url_property_receiver(context,target) else{return Ok(false)};
        Self::storage_access(context,receiver,Mode::Mutate,target.span,loans)?;
        // The setter runs after RHS evaluation; its checked failure belongs to that state.
        exception_state::record_point(context,target.span);
        Ok(true)
    }
    pub(super) fn url_getter(&mut self,context:&mut Context,expression:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
        let ExprKind::Field(receiver,name)=&expression.kind else{return Ok(None)};
        if !matches!(self.expression_type_hint(context,receiver),Some(Type::Record(id)) if self.program.records[id].native_type==Some(NativeTypeId::Url)) {return Ok(None)}
        let Some(property)=crate::stdlib::url::property(name) else{return Err(Diagnostic::new("url-property",format!("Url has no property {name:?}."),expression.span))};
        if mode==Mode::Mutate {
            let Some(setter)=property.setter else{return Err(Diagnostic::new("readonly","URL origin is read-only.",expression.span))};
            let mut receiver=(**receiver).clone();
            self.expression(context,&mut receiver,Mode::Mutate,loans,None)?;
            let error=Type::Record(self.schema_native_dependency(NativeTypeId::UrlError,expression.span)?);
            expression.kind=ExprKind::Field(Box::new(receiver),name.clone());
            // Keep ordinary assignment children; the marker gives the setter its checked error effect.
            expression.target=Some(CallTarget::ResultUnwrap(error));
            return Ok(Some(native_type(setter.parameters[1].ty,&self.program.records)));
        }
        *expression=crate::stdlib::url::call(&property.getter,vec![(**receiver).clone()],expression.span);
        self.expression(context,expression,mode,loans,expected).map(Some)
    }
    pub(super) fn url_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
        let ExprKind::Call(callee,args)=&expression.kind else{return Ok(None)};
        let ExprKind::Field(receiver,name)=&callee.kind else{return Ok(None)};
        let identity=self.expression_type_hint(context,receiver).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
        let Some(binding)=crate::stdlib::url::method(identity,name) else{return Ok(None)};
        let mut arguments=vec![(**receiver).clone()];arguments.extend(args.iter().cloned());
        *expression=crate::stdlib::url::call(binding,arguments,expression.span);
        self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
    }
}
