//! Public iterator calls enter ordinary source specialization; only the array leaf is intrinsic.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
    pub(super) fn rewrite_iterator_call(&self,context:&Context,expression:&mut Expr)->Result<bool,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(false);};
        let rewrite=if let ExprKind::Field(base,name)=&callee.kind {
            if name=="from" && matches!(&base.kind,ExprKind::Name(owner) if !context.names.contains_key(owner) && self.generic_records.native_template(owner)==Some(NativeTypeId::OwnedIterator)) {
                Some(("from".to_owned(),None))
            }else if matches!(self.expression_type_hint(context,base),Some(Type::Record(id)) if self.program.records[id].native_type==Some(NativeTypeId::OwnedIterator)) {
                // Only project/link assigns PREFIX to immutable std:iterator declarations;
                // ordinary user names use the disjoint linked-symbol namespace.
                // The bundled next implementation invokes its mutable pull field.
                if name=="_pull" && context.function_name.starts_with(crate::stdlib::iterator::PREFIX){return Ok(false);}
                if !crate::stdlib::iterator::METHODS.contains(&name.as_str()){return Err(Diagnostic::new("iterator-member","Unknown iterator operation.",callee.span));}
                Some((name.clone(),Some((**base).clone())))
            }else{None}
        }else{None};
        if let Some((operation,receiver))=rewrite{
            if let Some(receiver)=receiver{args.insert(0,receiver);}
            callee.kind=ExprKind::Name(format!("{}{operation}",crate::stdlib::iterator::PREFIX));
            return Ok(true);
        }
        Ok(false)
    }
    pub(super) fn iterator_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        if self.rewrite_iterator_call(context,expression)?{
            return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
        }
        let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};
        let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
        let Some((_,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:iterator-runtime") else{return Ok(None);};
        if !context.function_name.starts_with(crate::stdlib::iterator::PREFIX){return Err(Diagnostic::new("iterator-private","Iterator cursor operations are private to the iterator module.",expression.span));}
        Self::arity(args.len(),1,expression.span)?;
        let result=if binding.name=="arrayCursor" {
            // The opaque cursor owns its reserved array field. Its logical generic
            // argument may be a source-span-specific callback contract; the field
            // already records the canonical owning callable storage family.
            let declared=expected.filter(|ty|matches!(ty,Type::Record(id) if self.program.records[*id].native_type==Some(NativeTypeId::IteratorCursor)));
            let storage=declared.and_then(|ty|{let Type::Record(id)=ty else{unreachable!()};self.fields[id].get("_items").copied()});
            let source=self.expression(context,&mut args[0],Mode::Consume,loans,storage)?;
            let Type::Array(element)=source else{return Err(Diagnostic::new("iterator-source","Iterator.from requires an owned array.",args[0].span));};
            let ty=if let Some(declared)=declared{declared}else{self.thread_application(NativeTypeId::IteratorCursor,&[element.ty()],expression.span)?};
            let Type::Record(cursor)=ty else{unreachable!("applied cursor")};
            let storage=*self.fields[cursor].get("_items").expect("reserved owning cursor storage");
            self.require(source,Some(storage),args[0].span)?;
            expression.target=Some(CallTarget::IteratorCursor);ty
        }else{
            let source=self.expression(context,&mut args[0],Mode::Mutate,loans,None)?;
            let Type::Record(cursor)=source else{return Err(Diagnostic::new("iterator-source","Iterator array advancement requires its private owning cursor.",args[0].span));};
            if self.program.records[cursor].native_type!=Some(NativeTypeId::IteratorCursor){return Err(Diagnostic::new("iterator-source","Iterator array advancement requires its private owning cursor.",args[0].span));}
            let parameters=&self.program.records[cursor].application.as_ref().expect("applied iterator cursor").resolved_arguments;
            // The declared pull contract fixes the result even during context-free discovery.
            let ty=self.program.records.iter().enumerate().find_map(|(id,record)|{
                if record.native_type!=Some(NativeTypeId::OwnedIterator) || !record.application.as_ref().is_some_and(|application|application.resolved_arguments==*parameters){return None;}
                self.fields[id].get("_pull").and_then(|&pull|self.checked_generic_callable_shape(pull)).map(|(_,result)|result)
            }).ok_or_else(||Diagnostic::new("iterator-result","Iterator array advancement requires its declared iterator pull contract.",expression.span))?;
            expression.target=Some(CallTarget::IteratorNext);ty
        };
        self.require(result,expected,expression.span)?;
        expression.ty=Some(result);Ok(Some(result))
    }
}
