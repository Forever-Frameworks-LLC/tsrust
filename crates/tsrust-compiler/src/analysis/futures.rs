//! Lazy operations reuse checked task closures; jobs retain explicit settlement ownership.
use super::*;
use crate::native_signatures::NativeTypeId;
use crate::stdlib::futures as surface;
impl Analyzer {
    fn future_input(&mut self,context:&mut Context,value:&mut Expr,loans:&Loans)->Result<Type,Diagnostic>{
        let hint=self.expression_type_hint(context,value);
        let returning=context.returning_closure;context.returning_closure=true;
        let result=self.expression(context,value,Mode::Consume,loans,hint);
        context.returning_closure=returning;result
    }

    pub(super) fn job_parameters(&self,ty:Type)->Option<(Type,Type)> {
        let Type::Record(id)=ty else{return None;};let record=&self.program.records[id];
        if record.native_type!=Some(NativeTypeId::Job){return None;}
        let values=&record.application.as_ref()?.resolved_arguments;Some((values[0],values[1]))
    }
    pub(super) fn future_shape(&mut self,ty:Type,span:Span)->Result<(Type,Type),Diagnostic>{
        if let Some(shape)=self.job_parameters(ty){return Ok(shape);}
        if let Type::Record(id)=ty && self.program.records[id].native_type.is_none() {
            let fields=self.program.records[id].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
            let mut output=None;let mut errors=Vec::new();
            for ty in fields {let(result,error)=self.future_shape(ty,span)?;if output.is_some_and(|old|old!=result){return Err(Diagnostic::new("future-result","Task groups require the same result type for every input.",span));}output=Some(result);errors.push(error);}
            return Ok((output.unwrap_or(Type::Void),self.unions.intern(errors,span)?));
        }
        if let Type::Union(id)=ty {
            let mut output=None;let mut errors=Vec::new();
            for member in self.unions.members(id){let (result,error)=self.future_shape(member,span)?;if output.is_some_and(|old|old!=result){return Err(Diagnostic::new("future-result","Task groups require the same result type for every input.",span));}output=Some(result);errors.push(error);}
            return Ok((output.unwrap(),self.unions.intern(errors,span)?));
        }
        if !self.is_task_type(ty){return Err(Diagnostic::new("future-input","Expected a lazy Task or a started Job.",span));}
        let (_,output)=self.checked_generic_callable_shape(ty).ok_or_else(||Self::pending_result(span))?;
        let effects=self.discovery_error_effects();
        let failures=match ty {
            Type::Closure(id)=>effects.closure_errors[id].clone(),
            Type::Callback(id)=>effects.callback_errors.get(&id).cloned().unwrap_or_default(),
            Type::StoredCallable(id) if self.callables.storage.origins(id).is_some_and(|origins|origins.is_empty())=>crate::error_effects::FailureSet::default(),
            Type::StoredCallable(id)=>effects.storage_errors.get(&id).cloned().unwrap_or_default(),
            _=>return Err(Diagnostic::new("future-input","This task representation cannot be scheduled.",span)),
        };
        let runtime=failures.runtime.then(||self.materialize_discovery_runtime_error());
        let error=failures.catch_type(&self.unions,runtime,span)?.unwrap_or(Type::Undefined);
        Ok((output,error))
    }
    pub(super) fn future_start_transfer(&self,root:Type,span:Span)->Result<(),Diagnostic>{
        let mut pending=vec![root];let mut seen=HashSet::new();
        while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}
            match ty {
                Type::Closure(id) if self.is_task_type(ty)=>{
                    let closure=&self.closures[id];
                    if closure.program_scoped || !closure.program_captures.is_empty(){return Err(Diagnostic::new("task-transfer","Started tasks must own their inputs and cannot borrow program state.",span));}
                    for capture in &closure.captures {let ty=closure.function.bindings[capture.inner].ty;if capture.storage!=Mode::Consume && !(capture.storage==Mode::Copy && ty.is_copy()){return Err(Diagnostic::new("task-transfer","Started tasks cannot retain caller loans. Move owned inputs into the async function.",span));}pending.push(ty);}
                },
                Type::Array(element)=>pending.push(element.ty()),
                Type::Union(id)=>pending.extend(self.unions.members(id)),
                Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::Job)=>{},
                Type::Record(id) if self.program.records[id].native_type.is_none()=>pending.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty)),
                _=>thread_transfer::require_owned_data(ty,&self.program.records,&|id|self.unions.members(id)).map_err(|_|Diagnostic::new("task-transfer","Started tasks require concrete, owned transferable inputs and results.",span))?,
            }
        }Ok(())
    }
    pub(super) fn future_channel_constructor(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::NativeConstructor(args,Some(annotation))=&mut expression.kind else{return Ok(None);};
        let ty=self.resolve_value_annotation(annotation,false)?;
        let Type::Record(id)=ty else{return Ok(None);};if self.program.records[id].native_type!=Some(NativeTypeId::AsyncChannel){return Ok(None);}
        if args.len()!=1{return Err(Diagnostic::new("arity","new Channel<T> requires capacity: int (1..1000000).",expression.span));}
        let payload=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        thread_transfer::require_channel_payload(payload,&self.program.records,&|id|self.unions.members(id)).map_err(|_|Diagnostic::new("channel-transfer","Channel messages must be owned transferable data without endpoints or jobs.",expression.span))?;
        self.expression(context,&mut args[0],Mode::Read,loans,Some(Type::Int))?;
        self.require(ty,expected,expression.span)?;
        expression.target=Some(CallTarget::NativeBinding(crate::native_bindings::by_symbol("@native:std:futures#__channel").unwrap().0));Ok(Some(ty))
    }
    pub(super) fn future_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};
        if let ExprKind::Field(base,member)=&callee.kind {
            let hint=self.expression_type_hint(context,base);
            let identity=hint.and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
            let operation=match (identity,member.as_str()) {
                (_,"start") if hint.is_none_or(|ty|self.is_task_type(ty) || self.job_parameters(ty).is_some())=>Some("__start"),(_,"timeout") if hint.is_none_or(|ty|self.is_task_type(ty) || self.job_parameters(ty).is_some())=>Some("__timeout"),
                (Some(NativeTypeId::Job),"cancel")=>Some("__cancel"),
                (Some(NativeTypeId::AsyncChannel),"sender")=>Some("__sender"),
                (Some(NativeTypeId::AsyncChannel),"receive")=>Some("__receive"),
                (Some(NativeTypeId::AsyncChannel),"close")=>Some("__close"),
                (Some(NativeTypeId::AsyncChannel|NativeTypeId::AsyncSender),"send")=>Some("__send"),_=>None,
            };
            if let Some(name)=operation {
                args.insert(0,(**base).clone());
                if matches!(name,"__timeout"|"__send"|"__receive") {let args=std::mem::take(args);surface::lazy(expression,name,args);return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);}
                callee.kind=ExprKind::Name(format!("@native:std:futures#{name}"));
            }
        }
        let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
        let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:futures") else{return Ok(None);};
        let span=expression.span;
        if !binding.accepts_arity(args.len()){return Err(Diagnostic::new("arity",format!("{} expects {} arguments including its receiver.",binding.name,binding.parameters.len()),span));}
        if matches!(binding.name,"__join"|"__timeout"|"__delay"|"__yield"|"__race"|"__allSettled"|"__send"|"__receive") && !context.task_body {return Err(Diagnostic::new("async-context","This operation must run inside an async task.",span));}
        let result=match binding.name {
            "__start"=>{
                if !context.task_body{return Err(Diagnostic::new("async-context","Start jobs inside an async function driven by the runtime.",span));}
                let ty=self.future_input(context,&mut args[0],loans)?;
                if !self.is_task_type(ty){return Err(Diagnostic::new("task-start","start consumes an unstarted Task exactly once.",span));}
                self.future_start_transfer(ty,span)?;
                let (output,error)=self.future_shape(ty,span)?;
                for ty in [output,error]{self.future_start_transfer(ty,span)?;}
                self.thread_application(NativeTypeId::Job,&[output,error],span)?
            },
            "__cancel"=>{let ty=self.expression(context,&mut args[0],Mode::Read,loans,None)?;if self.job_parameters(ty).is_none(){return Err(Diagnostic::new("job-cancel","cancel requires a started Job; await it afterward to account for cleanup.",span));}Type::Boolean},
            "__join"|"__timeout"|"__race"|"__allSettled"=>{
                let (output,error)=if args.is_empty(){(Type::Void,Type::Undefined)}else{
                let ty=self.future_input(context,&mut args[0],loans)?;
                let item=if matches!(binding.name,"__race"|"__allSettled") {match ty {Type::Array(item)=>item.ty(),Type::Record(_)=>ty,_=>return Err(Diagnostic::new("future-input","Task groups take an owned array of tasks or jobs.",span))}}else{ty};
                self.future_shape(item,span)?};
                if binding.name=="__timeout" {self.expression(context,&mut args[1],Mode::Read,loans,Some(Type::Int))?;}
                let failure=self.thread_application(NativeTypeId::AsyncError,&[error],span)?;
                let result=self.thread_result(output,failure,span)?;
                if binding.name=="__allSettled" {Type::Array(crate::syntax::ArrayElement::from_value_type(result).ok_or_else(||Diagnostic::new("future-result","Unsupported settlement result element.",span))?)}else{result}
            },
            "__delay"|"__yield"=>{if !args.is_empty(){self.expression(context,&mut args[0],Mode::Read,loans,Some(Type::Int))?;}let failure=self.thread_application(NativeTypeId::AsyncError,&[Type::Undefined],span)?;self.thread_result(Type::Void,failure,span)?},
            "__sender"|"__close"|"__send"|"__receive"=>{
                let access=if binding.name=="__close"{Mode::Mutate}else{Mode::Read};
                let ty=self.expression(context,&mut args[0],access,loans,None)?;
                let Type::Record(record)=ty else{return Err(Diagnostic::new("channel-receiver","Expected an async channel endpoint.",span));};
                if !matches!(self.program.records[record].native_type,Some(NativeTypeId::AsyncChannel|NativeTypeId::AsyncSender)){return Err(Diagnostic::new("channel-receiver","Expected an async channel endpoint.",span));}
                let payload=self.program.records[record].application.as_ref().unwrap().resolved_arguments[0];
                if binding.name=="__sender"{self.thread_application(NativeTypeId::AsyncSender,&[payload],span)?}else if binding.name=="__close"{Type::Void}else{
                    if binding.name=="__send"{let mut active=loans.clone();for root in Self::loan_roots(&args[0],access){active.entry(root).or_default().push((access,args[0].span));}let actual=self.expression(context,&mut args[1],Mode::Consume,&active,Some(payload))?;self.require(actual,Some(payload),args[1].span)?;}
                    let failure=self.thread_application(NativeTypeId::AsyncError,&[Type::Undefined],span)?;self.thread_result(if binding.name=="__receive"{payload}else{Type::Void},failure,span)?
                }
            },
            _=>return Err(Diagnostic::new("future-call","Call this future operation directly.",span)),
        };
        self.require(result,expected,span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
    }
}

// Error effects settle after body checking; never emit a stale typed failure transport.
pub(super) fn validate(program:&CheckedProgram)->Result<(),Diagnostic>{
    let runtime=program.records.iter().position(|record|record.name.text=="#RuntimeError").map(Type::Record);
    let unions=crate::unions::UnionArena::from_snapshot_with_records(program.unions.clone(),&program.records);
    for function in program.functions.iter().chain(program.closures.iter().map(|closure|&closure.function)).filter(|function|!function.template){
        for expression in crate::error_effects::checked_expressions(function){
            let Some(CallTarget::NativeBinding(id))=expression.target else{continue;};
            let binding=&crate::native_bindings::BINDINGS[id];
            if binding.module!="std:futures" || !matches!(binding.name,"__start"|"__timeout"|"__race"|"__allSettled"){continue;}
            let ExprKind::Call(_,args)=&expression.kind else{continue;};let Some(input)=args.first()else{continue;};
            let result=expression.ty.unwrap();
            let expected=if binding.name=="__start"{
                let Type::Record(id)=result else{unreachable!()};program.records[id].application.as_ref().unwrap().resolved_arguments[1]
            }else{
                let item=if let Type::Array(item)=result{item.ty()}else{result};let Type::Union(id)=item else{unreachable!()};
                program.unions[id.index()].members.iter().find_map(|ty|{
                    let Type::Record(id)=ty else{return None;};let record=&program.records[*id];
                    if record.application.as_ref()?.declaration.text!="Result#Err"{return None;}
                    let Type::Record(id)=record.fields[0].ty? else{return None;};Some(program.records[id].application.as_ref()?.resolved_arguments[0])
                }).unwrap()
            };
            let mut pending=vec![input.ty.unwrap()];
            while let Some(ty)=pending.pop(){
                let failures=match ty{
                    Type::Closure(id)=>program.error_effects.closure_errors[id].clone(),
                    Type::Callback(id)=>program.error_effects.callback_errors.get(&id).cloned().unwrap_or_default(),
                    Type::StoredCallable(id) if program.stored_contracts.origins(id).is_some_and(|origins|origins.is_empty())=>crate::error_effects::FailureSet::default(),
                    Type::StoredCallable(id)=>program.error_effects.storage_errors.get(&id).cloned().unwrap_or_default(),
                    Type::Array(item)=>{pending.push(item.ty());continue;},
                    Type::Union(id)=>{pending.extend(&program.unions[id.index()].members);continue;},
                    Type::Record(id)=>{if program.records[id].native_type!=Some(NativeTypeId::Job){pending.extend(program.records[id].fields.iter().filter_map(|field|field.ty));}continue;},
                    _=>continue,
                };
                // An empty failure set needs no payload conversion. A thrown
                // undefined remains Some(Undefined) and must still be checked.
                let Some(actual)=failures.catch_type(&unions,runtime,expression.span)? else{continue;};
                let members=if let Type::Union(id)=actual{program.unions[id.index()].members.clone()}else{vec![actual]};
                if actual!=expected && !matches!(expected,Type::Union(id) if members.iter().all(|ty|program.unions[id.index()].members.contains(ty))){
                    return Err(Diagnostic::new("future-errors","Task error inference changed after checking; use a task with a resolved completion contract.",expression.span));
                }
            }
        }
    }Ok(())
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn empty_failure_set_is_not_a_thrown_undefined_payload(){
  let source="async function fail():Task<int>{throw 'original';}async function main():Task<void>{await fail().start();}";
  let mut program=crate::check(source).unwrap().program;
  let affected=program.error_effects.closure_errors.iter().enumerate().filter(|(_,error)|error.explicit.contains(&Type::String)).map(|(id,_)|id).collect::<Vec<_>>();
  assert!(!affected.is_empty());
  for &id in &affected {program.error_effects.closure_errors[id]=crate::error_effects::FailureSet::default();}
  assert!(validate(&program).is_ok());
  for id in affected {program.error_effects.closure_errors[id].explicit.push(Type::Undefined);}
  assert_eq!(validate(&program).unwrap_err().code,"future-errors");
 }
}
