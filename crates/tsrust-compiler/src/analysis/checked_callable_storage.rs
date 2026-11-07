//! Register already checked environments without repeating source access effects.
use super::*;
impl Analyzer {
    #[cfg(test)]
    pub(super) fn trace_stored_result_error(&self,family:usize,origin:usize,function:&str,error:&Diagnostic){
        if error.message!="Callable environment signature does not match its storage contract."{return;}
        let Some(declaration)=self.callables.storage.declaration(family)else{return;};
        let actual=self.closures[origin].function.result;let expected=self.callables.callbacks[declaration].result;
        let closure=&self.closures[origin];let parameters=closure.parameters.iter().map(|&slot|(closure.function.bindings[slot].ty,closure.function.bindings[slot].mode)).collect::<Vec<_>>();
        eprintln!("STORAGE_RESULT_REJECT function={function} family={family} origin={origin} origin_name={} task={} actual={actual:?} expected={expected:?} parameters={parameters:?} expected_parameters={:?}",closure.function.name,self.task_origins.contains(&origin),self.callables.callbacks[declaration].params);
        let mut pending=vec![actual,expected];let mut seen=HashSet::new();while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty{
            Type::Union(id)|Type::BorrowedUnion(id)=>{let members=self.unions.members(id);eprintln!("STORAGE_RESULT_UNION id={id:?} members={members:?}");pending.extend(members);}
            Type::Record(id)=>{let fields=self.program.records[id].fields.iter().map(|field|(field.name.text.as_str(),field.ty.unwrap())).collect::<Vec<_>>();eprintln!("STORAGE_RESULT_RECORD id={id} declaration={} fields={fields:?}",self.scoped_records.declaration(id));pending.extend(fields.into_iter().map(|(_,ty)|ty));}
            Type::Callback(id)=>eprintln!("STORAGE_RESULT_CALLBACK id={id} kind={:?} protocol={:?}",self.callables.callbacks[id].kind,self.callables.callbacks[id].structural),
            Type::Closure(id)=>eprintln!("STORAGE_RESULT_CLOSURE id={id} task={} result={:?}",self.task_origins.contains(&id),self.closures[id].function.result),
            Type::StoredCallable(id)=>eprintln!("STORAGE_RESULT_FAMILY id={id} declaration={:?} origins={:?}",self.callables.storage.declaration(id),self.callables.storage.origins(id)),
            _=>{}
        }}
    }

    // A source template checks the owned contract now; only its caller-specific
    // specialization can register a concrete environment. Templates are not emitted.
    pub(super) fn abstract_stored_callable(&self, context: &Context, family: usize, actual: Type, span: Span) -> Result<Option<Type>, Diagnostic> {
        if !context.template || !matches!(actual, Type::Callback(_)) || self.callables.storage.is_borrowed(family) { return Ok(None); }
        let declaration = self.callables.storage.declaration(family).ok_or_else(|| Diagnostic::new("callable-layout", "Unknown callable storage signature.", span))?;
        self.require(actual, Some(Type::Callback(declaration)), span)?;
        Ok(Some(Type::StoredCallable(family)))
    }
    // Conversion consumes only checked metadata; the source expression was
    // already evaluated and its ownership effects must not be repeated.
    pub(super) fn checked_stored_callable_value(&mut self,context:&mut Context,value:&mut Expr,family:usize,actual:Type)->Result<Type,Diagnostic> {
        let declaration=self.callables.storage.declaration(family).ok_or_else(||Diagnostic::new("callable-layout","Unknown callable storage signature.",value.span))?;
        let expected=Type::Callback(declaration);
        if matches!(actual, Type::StoredCallable(_)) {
            self.require(actual, Some(Type::StoredCallable(family)), value.span)?;
            return Ok(actual);
        }
        if let Type::Closure(origin)=actual {
            let result=self.closures[origin].function.result;
            let wanted=self.callables.callbacks[declaration].result;
            if let (Type::Closure(source),Type::Closure(destination))=(result,wanted)
                && source!=destination
                && let (Some(left),Some(right))=(self.deferred_generic_values.get(&source),self.deferred_generic_values.get(&destination))
                && left.declaration==right.declaration
                && [source,destination].into_iter().all(|id|!self.closures[id].borrowed && self.closures[id].lifetime_count==0)
            {
                self.polymorphic_storage.observe(source,destination);
                self.callables.storage.note_nested_result(source,destination);
            }
        }
        self.require(actual, Some(expected), value.span)?;
        if let Some(ty) = self.abstract_stored_callable(context, family, actual, value.span)? {
            value.wrap_storage(family);
            return Ok(ty);
        }
        let Type::Closure(mut origin) = actual else {
            return Err(Diagnostic::new(
                "callable-layout",
                "Stored callable construction requires a concrete owned environment.",
                value.span,
            ));
        };
        if self.callables.callbacks[declaration].kind.is_once()
            && self.closures[origin].invocation != Mode::Consume
        {
            origin = self.once_return(origin, context, value.span);
            value.wrap_once(origin);
        }
        if context.function_name.starts_with("@stdlib:iterator#create")
            && let Some(ty)=self.callables.storage.intern_finite_owned(
                &self.callables.shapes,&self.callables.callbacks[declaration],declaration,
                [origin],&self.closures,value.span,
            )? {
            let Type::StoredCallable(scoped)=ty else{unreachable!()};
            value.wrap_storage(scoped);
            return Ok(ty);
        }
        if let Some(ty) = self.callables.storage.intern_scoped_native(
            &self.callables.shapes, &self.callables.callbacks[declaration], declaration,
            [origin], &self.closures, value.span,
        )? {
            let Type::StoredCallable(scoped) = ty else { unreachable!() };
            value.wrap_storage(scoped);
            return Ok(ty);
        }
        if !context.template && !self.discovering_results && !self.effects_only {
            let registered=self.callables.storage.register_origin(
                family,
                origin,
                &self.callables.callbacks,
                &self.callables.shapes,
                &self.closures,
                value.span,
            );
            #[cfg(test)]
            if let Err(error)=&registered{self.trace_stored_result_error(family,origin,&context.function_name,error);}
            registered?;
        } else if self.speculative_access_run && !context.template && !self.discovering_results {
            self.callables.storage.note_speculative_origin(family, origin);
        }
        value.wrap_storage(family);
        Ok(Type::StoredCallable(family))
    }

}
