//! Replacement obligations cross calls without changing point-sensitive loans.
use super::*;

#[derive(Clone,PartialEq,Eq)]
struct Replacement { parameter:usize, path:Vec<String>, ty:Type }
struct Call { span:Span,callee:usize, arguments:Vec<Option<(usize,Vec<String>)>> }
fn calls(function:&CheckedFunction)->Vec<Call>{
    let mut result=Vec::new();
    crate::fixed_storage::visit_function_expressions(function,&mut |expr|{
        if let(Some(CallTarget::Function(callee)),ExprKind::Call(_,args))=(expr.target,&expr.kind){
            result.push(Call{span:expr.span,callee,arguments:args.iter().map(|arg|Analyzer::root_binding(arg).zip(Analyzer::direct_fields(arg)).map(|(id,(_,path))|(id,path))).collect()});
        }
    });
    result
}
impl Analyzer {
    pub(super) fn record_parameter_carriers(&mut self)->Result<(),Diagnostic>{
        let mut functions=std::mem::take(&mut self.checked);
        // The record outlives the call. A stored callback cannot keep a borrow
        // of the callee's parameters or locals past it.
        for function in functions.iter().flatten().filter(|function|!function.template){
            for write in record_storage_writes::collect(&function.body){
                let binding=&function.bindings[write.binding];
                if write.binding>=function.parameter_count || !binding.parameter || binding.mode!=Mode::Mutate || write.path.is_empty(){continue;}
                if let Some(ty) = self.nullable_parameter_store_type(write.value) {
                    if self.borrows_local_storage(ty) {
                        return Err(Diagnostic::new("callback-escape","A callback stored through a record parameter cannot borrow a variable of this function. Capture numbers and booleans, or build the record where the variable is declared.",write.value.span));
                    }
                    continue;
                }
                let origins=match write.value.ty{
                    Some(Type::Closure(id))=>vec![id],
                    Some(Type::BorrowedCallable(family))=>self.callables.storage.origins(family).map(<[usize]>::to_vec).unwrap_or_default(),
                    // A callback parameter is lent for the call only.
                    Some(Type::Callback(_))=>return Err(Diagnostic::new("callback-escape","A callback parameter cannot be stored through a record parameter; the record outlives the call. Build the record where the callback is created.",write.value.span)),
                    _=>continue,
                };
                if origins.iter().any(|&id|self.closures[id].lifetime_count>0){
                    return Err(Diagnostic::new("callback-escape","A callback stored through a record parameter cannot borrow a variable of this function. Capture numbers and booleans, or build the record where the variable is declared.",write.value.span));
                }
            }
        }
        let mut replacements=functions.iter().map(|function|{
            let Some(function)=function.as_ref().filter(|function|!function.template)else{return Vec::new();};
            record_storage_writes::collect(&function.body).into_iter().filter_map(|write|{
                let binding=&function.bindings[write.binding];
                (write.binding<function.parameter_count && binding.parameter && binding.mode==Mode::Mutate && !write.path.is_empty() && (matches!(write.value.ty,Some(Type::Closure(_) | Type::BorrowedCallable(_))) || self.nullable_parameter_store_type(write.value).is_some()))
                    .then(||Replacement{parameter:write.binding,path:write.path,ty:write.value.ty.unwrap()})
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        let calls=functions.iter().map(|function|function.as_ref().filter(|function|!function.template).map(calls).unwrap_or_default()).collect::<Vec<_>>();
        // Relay functions need the same associated-field conversion obligations.
        loop {
            let mut added=Vec::new();
            for(index,function)in functions.iter().enumerate(){
                let Some(function)=function.as_ref()else{continue;};
                for call in &calls[index]{for replacement in &replacements[call.callee]{
                    let Some(Some((root,prefix)))=call.arguments.get(replacement.parameter)else{continue;};
                    let binding=&function.bindings[*root];
                    if *root>=function.parameter_count || !binding.parameter || binding.mode!=Mode::Mutate{continue;}
                    let mut path=prefix.clone();path.extend(replacement.path.iter().cloned());
                    let replacement=Replacement{parameter:*root,path,ty:replacement.ty};
                    if !replacements[index].contains(&replacement) && !added.contains(&(index,replacement.clone())){added.push((index,replacement));}
                }}
            }
            if added.is_empty(){break;}
            for(index,replacement)in added{replacements[index].push(replacement);}
        }
        for(index,function)in functions.iter_mut().enumerate(){
            let Some(function)=function.as_mut().filter(|function|!function.template)else{continue;};
            let mut context=Context::new(function.name.clone());
            context.storage_exits=function.storage_exits.clone();
            context.bindings=std::mem::take(&mut function.bindings);
            self.mark_control_stored(&mut context,&function.body);
            for call in &calls[index]{for replacement in &replacements[call.callee]{
                let Some(Some((root,prefix)))=call.arguments.get(replacement.parameter)else{continue;};
                let binding=&context.bindings[*root];
                if binding.parameter || binding.capture_reference{continue;}
                let Type::Record(record)=binding.ty else{continue;};
                let mut path=prefix.clone();path.extend(replacement.path.iter().cloned());
                let Some(previous)=self.record_field_type(Type::Record(record),&path)else{continue;};
                if self.nullable_scoped_record_declaration(previous).is_some() && replacement.ty.union().is_some() {
                    context.control_stored.insert(*root);
                }
                if previous==replacement.ty{continue;}
                if let Some(joined) = self.join_nullable_record_unions(previous,replacement.ty,call.span)? {
                    if let Some(record) = self.record_type_with_field(record,&path,joined) {
                        context.bindings[*root].ty=Type::Record(record);
                    }
                    continue;
                }
                if let Some(family)=self.common_borrowed_conditional_family(previous,replacement.ty,call.span)?
                    && let Some(record)=self.record_type_with_field(record,&path,Type::BorrowedCallable(family)){
                    context.bindings[*root].ty=Type::Record(record);
                }
            }}
            self.record_sequence_carriers(&mut context,&mut function.body)?;
            function.bindings=context.bindings;
            function.storage_exits=context.storage_exits;
        }
        self.record_parameter_replacements=replacements.into_iter().enumerate().filter(|(_,values)|!values.is_empty()).map(|(id,values)|(id,values.into_iter().map(|value|(value.parameter,value.path,value.ty)).collect())).collect();
        self.checked=functions;
        Ok(())
    }
}
