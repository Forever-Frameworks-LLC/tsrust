//! Adapt only registered test/hook factories; ordinary callable results stay exact.
use super::*;

impl Analyzer {
    pub(super) fn testing_callback_adapter(&mut self,context:&Context,expression:&mut Expr,actual:Type,expected:Option<Type>)->Result<Type,Diagnostic>{
        let Some(Type::Callback(wanted))=expected else{return Ok(actual);};
        let contract=&self.callables.callbacks[wanted];
        if contract.kind!=CallbackKind::Program || contract.result!=Type::Void || !contract.params.is_empty(){return Ok(actual);}
        if self.is_task_type(actual){return Err(Diagnostic::new("test-callback-task","Register a callback returning Task<void>, rather than a constructed task.",expression.span));}
        let Some((parameters,task))=self.checked_generic_callable_shape(actual) else{return Ok(actual);};
        if !parameters.is_empty() || !self.is_task_type(task){return Ok(actual);}
        let Some((parameters,result))=self.checked_generic_callable_shape(task) else{return Ok(actual);};
        if !parameters.is_empty() || result!=Type::Void {
            return Err(Diagnostic::new("test-callback-result","A test or hook callback must return void or Task<void>.",expression.span));
        }
        // Reuse the ownership/environment proof before constructing the adapter;
        // no borrowed local capture gains an escaping lifetime at this boundary.
        crate::callable_compatibility::Contracts{storage:&self.callables.storage,signatures:&self.callables.shapes,
            callbacks:&self.callables.callbacks,results:&self.callables.results,closures:&self.closures,effects_only:self.effects_only}
            .matching(actual,Type::Callback(wanted),expression.span)?;
        if let Type::Closure(origin)=actual {
            self.program_callback_sites.insert(self.closures[origin].span.start,CallbackKind::Program);
        }
        let key=(actual,Type::Callback(wanted));
        let id=if let Some(&id)=self.testing_adapters.get(&key){id}else{
            let span=expression.span;
            let (invocation,borrowed,lifetime_count)=match actual {
                Type::Closure(id)=>(self.closures[id].invocation,self.closures[id].borrowed,self.closures[id].lifetime_count),
                Type::Callback(id)=>(self.callables.callbacks[id].kind.invocation(),false,0),
                Type::StoredCallable(id)=>{
                    let declaration=self.callables.storage.declaration(id).expect("checked callback family");
                    let lifetimes=self.callables.storage.scoped_native_family(id).map_or_else(
                        ||self.callables.storage.origins(id).map_or(0,|origins|origins.iter().map(|id|self.closures[*id].lifetime_count).max().unwrap_or(0)),
                        crate::native_view_callable_storage::Family::lifetimes);
                    (self.callables.callbacks[declaration].kind.invocation(),lifetimes!=0 || self.borrows_local_storage(actual),lifetimes)
                },
                _=>return Ok(actual),
            };
            let binding=Binding{rest:false,name:"test_factory".into(),ty:actual,parameter:false,input_ty:None,
                omission:crate::ParameterOmission::Required,declaration_span:None,initializer:None,rebindable:false,
                capture_reference:false,borrowed_element:None,borrowed_place:None,mutable_place:false,view_roots:Vec::new(),
                native_view_loans:None,callback_invocation:Some(invocation),callback_reference:false,callback_kind:None,
                mode:Mode::Consume,mutable:invocation==Mode::Mutate,transferred:None,moved_fields:BTreeMap::new(),used:Some(span)};
            let mut receiver=Expr::new(ExprKind::Name("test_factory".into()),span);receiver.binding=Some(0);receiver.ty=Some(actual);
            let mut factory=Expr::new(ExprKind::Call(Box::new(receiver),Vec::new()),span);factory.ty=Some(task);
            factory.target=Some(match actual{Type::Closure(_)=>CallTarget::Closure(0),Type::Callback(_)=>CallTarget::Callback(0),Type::StoredCallable(_)=>CallTarget::StoredCallable(0),_=>unreachable!()});
            factory.callable_origin=Some(match actual{Type::Closure(id)=>CallableOrigin::Closure(id),Type::Callback(id)=>CallableOrigin::Callback(id),Type::StoredCallable(id)=>CallableOrigin::Storage(id),_=>unreachable!()});
            let mut receiver=Expr::new(ExprKind::Unary{op:UnaryOp::AwaitCallee,value:Box::new(factory),operator:span},span);receiver.ty=Some(task);
            let mut poll=Expr::new(ExprKind::Call(Box::new(receiver),Vec::new()),span);poll.ty=Some(Type::Void);
            poll.target=Some(match task{Type::Closure(id)=>CallTarget::ConditionalClosure(id),Type::Callback(id)=>CallTarget::ConditionalCallback(id),Type::StoredCallable(id)=>CallTarget::StoredCallableField(id),_=>return Ok(actual)});
            poll.callable_origin=Some(match task{Type::Closure(id)=>CallableOrigin::Closure(id),Type::Callback(id)=>CallableOrigin::Callback(id),Type::StoredCallable(id)=>CallableOrigin::Storage(id),_=>unreachable!()});
            let body=vec![Stmt{labels:Vec::new(),control_target:None,span,kind:StmtKind::Return(Some(poll))}];
            let id=self.closures.len();
            self.closures.push(CheckedClosure{program_async:true,program_scoped:true,program_captures:Vec::new(),
                polymorphic:false,methods:Vec::new(),forward:None,borrowed,lifetime_count,depth:context.closure_depth+1,
                parameters:Vec::new(),function:CheckedFunction{storage_exits:Default::default(),native_returns:None,native_mutations:None,
                    template:context.template,specialization:false,name:context.function_name.clone(),bindings:vec![binding],parameter_count:0,result:Type::Void,body},
                captures:vec![Capture{restored:false,outer:expression.binding.unwrap_or(0),inner:0,storage:Mode::Consume,access:invocation,span}],span,escapes:true,invocation});
            self.testing_adapters.insert(key,id);id
        };
        if !self.discovering_results {self.validated_closures.insert(id);}
        let span=expression.span;let mut source=std::mem::replace(expression,Expr::new(ExprKind::Undefined,span));source.ty=Some(actual);
        // Reuse the one-capture callable-constructor ABI used by rest adapters.
        *expression=Expr::new(ExprKind::Unary{op:UnaryOp::CallableRestAdapter,value:Box::new(source),operator:span},span);
        expression.ty=Some(Type::Closure(id));
        Ok(Type::Closure(id))
    }
}
