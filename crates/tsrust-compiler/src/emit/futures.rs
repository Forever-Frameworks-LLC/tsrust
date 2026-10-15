//! Native futures retain typed errors and move their inputs exactly once.
use super::*;
use crate::Span;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
    fn future_owned(&self,root:Type)->bool{
        let mut pending=vec![root];let mut seen=std::collections::HashSet::new();
        while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty{
            Type::Closure(id)=>{let closure=&self.program.closures[id];if closure.program_scoped || !closure.program_captures.is_empty(){return false;}for capture in &closure.captures {if capture.storage!=Mode::Consume && capture.storage!=Mode::Copy{return false;}pending.push(closure.function.bindings[capture.inner].ty);}},
            Type::Record(id)=>pending.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty)),
            Type::Array(element)=>pending.push(element.ty()),
            Type::StoredCallable(_)|Type::Callback(_)|Type::BorrowedCallable(_)=>return false,
            _=>{}
        }}true
    }

    fn future_inject(&self,source:Type,destination:Type,value:&str)->String{
        if source==destination{return value.into();}
        let Type::Union(target)=destination else{return value.into();};
        let members=&self.program.unions[target.index()].members;
        if let Type::Union(source)=source {
            let arms=self.program.unions[source.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(payload)=>{}",source.index(),self.future_inject(*ty,destination,"payload"))).collect::<Vec<_>>().join(",");format!("match {value}{{{arms}}}")
        }else{let tag=members.iter().position(|ty|*ty==source).expect("checked future error alternative");format!("Union{}::V{tag}({value})",target.index())}
    }
    fn future_error_type(&self,ty:Type,span:Span)->Type{
        if let Type::Record(id)=ty && self.program.records[id].native_type==Some(NativeTypeId::Job){return self.program.records[id].application.as_ref().unwrap().resolved_arguments[1];}
        let contract=match ty {Type::Closure(id)=>self.program.error_effects.closure_errors[id].clone(),Type::Callback(id)=>self.program.error_effects.callback_errors.get(&id).cloned().unwrap_or_default(),Type::StoredCallable(id)=>self.program.error_effects.storage_errors.get(&id).cloned().unwrap_or_default(),_=>return Type::Undefined};
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError").map(Type::Record);
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
        contract.catch_type(&unions,runtime,span).expect("checked future errors").unwrap_or(Type::Undefined)
    }
    fn future_poll(&self,ty:Type,value:&str,error:Type,span:Span)->String{
        if let Type::StoredCallable(id)=ty && self.program.stored_contracts.origins(id).is_some_and(|origins|origins.is_empty()){return format!("match {value}{{}}");}
        if let Type::Union(id)=ty {
            let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(task)=>{}",id.index(),self.future_poll(*ty,"task",error,span))).collect::<Vec<_>>().join(",");return format!("match {value}{{{arms}}}");
        }
        let own_error=self.future_error_type(ty,span);
        if let Type::Record(id)=ty && self.program.records[id].native_type==Some(NativeTypeId::Job){
            let convert=self.future_inject(own_error,error,"cause");return format!("({value}).join().await.map_err(|failure|failure.map(|cause|{convert}))");
        }
        let registered=match ty {
            Type::Closure(id)=>self.result_closures.borrow().get(&id).cloned().map(|name|(name,self.program.error_effects.closure_errors[id].clone())),
            Type::Callback(id)=>self.result_callbacks.borrow().get(&id).cloned(),
            Type::StoredCallable(id)=>self.result_storage.borrow().get(&id).cloned(),_=>unreachable!("checked task")
        };
        let call=if let Type::Callback(id)=ty {format!("TaskCallback::<(),{}>::invoke_once({value},()).await",self.callback_result_type(id,self.program.callback_contracts[id].result))}else{format!("({value}).call().await")};
        if let Some((name,contract))=registered {
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError").map(Type::Record);
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
            let cause=exception_contract_emit::checked_catch_value("error",&name,&contract,&unions,runtime,span).expect("checked future error transport");
            let cause=self.future_inject(own_error,error,&cause);
            format!("({call}).map_err(|error|tsrust_native_futures::Failure::Worker({cause}))")
        }else{format!("Ok::<_,tsrust_native_futures::Failure<{}>>({call})",self.native_type(error))}
    }
    fn future_result(&self,ty:Type,call:&str)->String{
        let Type::Union(union)=ty else{unreachable!("future result")};
        let mut success=String::new();let mut failure=String::new();
        for (tag,member) in self.program.unions[union.index()].members.iter().enumerate(){
            let Type::Record(record)=*member else{unreachable!()};let declaration=&self.program.records[record];
            if declaration.application.as_ref().unwrap().declaration.text=="Result#Ok"{
                let fields=if declaration.fields.is_empty(){String::new()}else{"field0:value,".into()};
                success=format!("Union{}::V{tag}(Record{}{{{fields}{}}})",union.index(),self.native_record_declaration(record),self.schema_record_order(record).trim_start_matches(','));
            }else{
                let Type::Record(error)=declaration.fields[0].ty.unwrap() else{unreachable!()};let cause=self.program.records[error].fields[1].ty.unwrap();let own=self.program.records[error].application.as_ref().unwrap().resolved_arguments[0];
                let absent=self.future_inject(Type::Undefined,cause,"TsrUndefined");let present=self.future_inject(own,cause,"cause");
                let failed=format!("Record{}{{field0:String::Borrowed(code),field1:cause{}}}",self.native_record_declaration(error),self.schema_record_order(error));
                let wrapped=format!("Union{}::V{tag}(Record{}{{field0:{failed}{}}})",union.index(),self.native_record_declaration(record),self.schema_record_order(record));
                failure=format!("{{let(code,cause)=match failure{{tsrust_native_futures::Failure::Worker(cause)=>(\"worker\",{present}),tsrust_native_futures::Failure::Cancelled=>(\"cancelled\",{absent}),tsrust_native_futures::Failure::Panic=>(\"panic\",{absent}),tsrust_native_futures::Failure::Timeout=>(\"timeout\",{absent}),tsrust_native_futures::Failure::Range=>(\"range\",{absent}),tsrust_native_futures::Failure::Empty=>(\"empty\",{absent}),tsrust_native_futures::Failure::Closed=>(\"closed\",{absent})}};{wrapped}}}");
            }
        }
        format!("match {call}{{Ok(value)=>{success},Err(failure)=>{failure}}}")
    }
    pub(super) fn future_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
        let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:futures"{return None;}
        self.native_bindings.borrow_mut().insert(id);
        let result=expr.ty.unwrap();
        if matches!(binding.name,"__cancel"|"__sender"|"__close"){
            let receiver=self.expr(fun,&args[0],if binding.name=="__close"{Mode::Mutate}else{Mode::Read});
            return Some(format!("({receiver}).{}()",match binding.name{"__cancel"=>"cancel","__sender"=>"sender",_=>"close"}));
        }
        if binding.name=="__start"{
            let Type::Record(record)=result else{unreachable!()};let error=self.program.records[record].application.as_ref().unwrap().resolved_arguments[1];
            let value=self.expr(fun,&args[0],Mode::Consume);let poll=self.future_poll(args[0].ty.unwrap(),"task",error,expr.span);
            // Strip only the runtime wrapper; source failures remain typed payloads.
            return Some(format!("{{let task={value};tsrust_native_futures::start(async move{{{poll}}})}}"));
        }
        let result_item=if let Type::Array(item)=result{item.ty()}else{result};
        let Type::Union(union)=result_item else{unreachable!("future operation result")};
        let error=self.program.unions[union.index()].members.iter().find_map(|ty|{let Type::Record(record)=ty else{return None;};let record=&self.program.records[*record];if record.application.as_ref()?.declaration.text!="Result#Err"{return None;}let Type::Record(error)=record.fields[0].ty? else{return None;};Some(self.program.records[error].application.as_ref().unwrap().resolved_arguments[0])}).unwrap();
        let call=match binding.name{
            "__delay"=>format!("tsrust_native_futures::delay({}).await",self.expr(fun,&args[0],Mode::Read)),
            "__yield"=>"tsrust_native_futures::yield_now().await".into(),
            "__join"=>format!("({}).join().await",self.expr(fun,&args[0],Mode::Consume)),
            "__timeout"=>{let value=self.expr(fun,&args[0],Mode::Consume);let duration=self.expr(fun,&args[1],Mode::Read);let poll=self.future_poll(args[0].ty.unwrap(),"task",error,expr.span);format!("{{let task={value};let milliseconds={duration};tsrust_native_futures::timeout(async move{{{poll}}},milliseconds).await}}")},
            "__race"|"__allSettled"=>{
                let operation=if binding.name=="__race"{"race"}else{"all_settled"};
                if args.is_empty(){format!("tsrust_native_futures::{operation}(Vec::<std::future::Ready<Result<(),tsrust_native_futures::Failure<{}>>>>::new()).await",self.native_type(error))}else{
                let values=self.expr(fun,&args[0],Mode::Consume);
                if let Type::Array(item)=args[0].ty.unwrap(){
                    let poll=self.future_poll(item.ty(),"task",error,expr.span);
                    format!("tsrust_native_futures::{operation}(({values}).into_iter().map(|task|async move{{{poll}}}).collect()).await")
                }else{
                    let Type::Record(id)=args[0].ty.unwrap()else{unreachable!()};
                    let fields=&self.program.records[id].fields;
                    let result_type=self.program.unions[union.index()].members.iter().find_map(|ty|{let Type::Record(id)=ty else{return None;};let record=&self.program.records[*id];(record.application.as_ref()?.declaration.text=="Result#Ok").then(||record.fields.first().and_then(|field|field.ty).unwrap_or(Type::Void))}).unwrap();
                    let send=if self.future_owned(args[0].ty.unwrap()){ "+Send" }else{""};
                    let items=fields.iter().enumerate().map(|(index,field)|{
                        let poll=self.future_poll(field.ty.unwrap(),"task",error,expr.span);
                        format!("{{let task=inputs.field{index};Box::pin(async move{{{poll}}}) as std::pin::Pin<Box<dyn std::future::Future<Output=Result<{},tsrust_native_futures::Failure<{}>>>{send}+'_>>}}",self.native_type(result_type),self.native_type(error))
                    }).collect::<Vec<_>>().join(",");
                    format!("{{let inputs={values};tsrust_native_futures::{operation}(vec![{items}]).await}}")
                }}
            },
            "__send"=>format!("({}).send({}).await",self.expr(fun,&args[0],Mode::Read),self.expr(fun,&args[1],Mode::Consume)),
            "__receive"=>format!("({}).receive().await",self.expr(fun,&args[0],Mode::Read)),_=>unreachable!("future operation")
        };
        Some(if binding.name=="__allSettled"{format!("({call}).into_iter().map(|result|{}).collect::<Vec<_>>()",self.future_result(result_item,"result"))}else{self.future_result(result,&call)})
    }
    pub(super) fn future_record(&self,output:&mut String,id:usize)->bool{
        let record=&self.program.records[id];let name=match record.native_type{Some(NativeTypeId::Job)=>"Job",Some(NativeTypeId::AsyncChannel)=>"Channel",Some(NativeTypeId::AsyncSender)=>"Sender",_=>return false};
        if let Some(application)=&record.application{let args=application.resolved_arguments.iter().map(|ty|self.native_type(*ty)).collect::<Vec<_>>().join(",");writeln!(output,"type Record{id}=tsrust_native_futures::{name}<{args}>;").unwrap();}true
    }
}
