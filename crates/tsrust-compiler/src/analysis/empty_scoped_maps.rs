//! Empty local Maps have no callback instances to provide concrete environments.
use super::*;
enum EmptyTask {
    MapLayout(Type),
    MapValue(usize),
    Record(usize,bool),
    Field(Type,Span),
    Fields{id:usize,fields:Vec<Type>,types:Vec<Type>,index:usize,maps_only:bool},
    Union{span:Span,members:Vec<Type>,types:Vec<Type>,index:usize},
}


impl Analyzer {
    pub(super) fn empty_scoped_map_layout(&mut self,ty:Type)->Result<Type,Diagnostic>{
        self.empty_environment_work(EmptyTask::MapLayout(ty))
    }
    pub(super) fn empty_scoped_record_environment(&mut self,id:usize)->Result<Type,Diagnostic>{
        self.empty_environment_work(EmptyTask::Record(id,false))
    }
    fn empty_environment_work(&mut self,root:EmptyTask)->Result<Type,Diagnostic>{
        let mut work=vec![root];let mut result=Type::Void;
        while let Some(task)=work.pop(){match task{
            EmptyTask::MapLayout(ty)=>{
                result=ty;
                match ty{
                    Type::Map(id)=>{
                        let payload=self.program.records[id].map_definition.as_ref().unwrap().value;
                        let Type::Record(record)=payload else{continue;};
                        if self.scoped_records.declaration(record)!=record || !self.aggregate_callable_facts(payload).scoped{continue;}
                        work.push(EmptyTask::MapValue(id));work.push(EmptyTask::Record(record,false));
                    }
                    Type::Record(id)=>work.push(EmptyTask::Record(id,true)),
                    _=>{}
                }
            }
            EmptyTask::MapValue(id)=>result=Type::Map(self.scoped_map_value_instance(id,result)),
            EmptyTask::Record(id,maps_only)=>{
                let fields=self.program.records[id].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
                work.push(EmptyTask::Fields{id,fields,types:Vec::new(),index:0,maps_only});
            }
            EmptyTask::Fields{id,fields,mut types,index,maps_only}=>{
                if index>0{types.push(result);}
                if let Some(&field)=fields.get(index){
                    let child=if maps_only{EmptyTask::MapLayout(field)}else{EmptyTask::Field(field,self.program.records[id].name.span)};
                    work.push(EmptyTask::Fields{id,fields,types,index:index+1,maps_only});work.push(child);
                }else{result=if types==fields{Type::Record(id)}else{self.empty_environment_record(id,types)};}
            }
            EmptyTask::Field(ty,span)=>{
                result=ty;
                match ty{
                    Type::Callback(id)=>result=self.absent_callback(id),
                    Type::Record(id)=>work.push(EmptyTask::Record(id,false)),
                    Type::Union(id)=>work.push(EmptyTask::Union{span,members:self.unions.members(id),types:Vec::new(),index:0}),
                    Type::Map(_)=>work.push(EmptyTask::MapLayout(ty)),
                    _=>{}
                }
            }
            EmptyTask::Union{span,members,mut types,index}=>{
                if index>0{types.push(result);}
                if let Some(&member)=members.get(index){
                    work.push(EmptyTask::Union{span,members,types,index:index+1});
                    // Union members deliberately preserve map and other payloads.
                    match member{
                        Type::Callback(id)=>result=self.absent_callback(id),
                        Type::Record(id)=>work.push(EmptyTask::Record(id,false)),
                        other=>result=other,
                    }
                }else{result=self.unions.intern(types,span)?;}
            }
        }}
        Ok(result)
    }

    #[inline(never)]
    pub(super) fn empty_map_iteration_carriers(
        &mut self,
        context: &mut Context,
        body: &[Stmt],
        results: &mut HashMap<usize, Type>,
    ) -> Result<(), Diagnostic> {
        let mut pending = body.iter().collect::<Vec<_>>();
        while let Some(statement) = pending.pop() {
            match &statement.kind {
                StmtKind::ForOf {
                    iterable,
                    body,
                    binding: Some(binding),
                    ..
                } => {
                    if let Some(Type::Map(map)) = self.planned_scoped_map_result(iterable, results)
                        && let Type::Record(payload) = self.program.records[map]
                            .map_definition
                            .as_ref()
                            .unwrap()
                            .value
                        && self
                            .aggregate_callable_facts(Type::Record(
                                self.scoped_records.declaration(payload),
                            ))
                            .scoped
                    {
                        let Type::Record(pair) = context.bindings[*binding].ty else {
                            unreachable!("Map iteration entry");
                        };
                        let definition = self.program.records[map]
                            .map_definition
                            .as_ref()
                            .unwrap()
                            .clone();
                        let entry = self
                            .empty_environment_record(pair, vec![definition.key, definition.value]);
                        let Type::Record(pair) = entry else {
                            unreachable!()
                        };
                        self.program.records[map]
                            .map_definition
                            .as_mut()
                            .unwrap()
                            .entry = Some(pair);
                        results.insert(*binding, entry);
                        context.bindings[*binding].ty = entry;
                    }
                    pending.extend(body);
                }
                StmtKind::Block(body) | StmtKind::While { body, .. } => pending.extend(body),
                StmtKind::Loop { body, update, .. } => {
                    pending.extend(body);
                    pending.extend(update);
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend(then_body);
                    pending.extend(else_body);
                }
                StmtKind::Try(region) => {
                    pending.extend(&region.body);
                    if let Some(handler) = &region.catch {
                        pending.extend(&handler.body);
                    }
                    if let Some(cleanup) = &region.finally {
                        pending.extend(cleanup);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn empty_environment_record(&mut self, id: usize, types: Vec<Type>) -> Type {
        let instance = self
            .scoped_records
            .materialize(id, types, &mut self.program.records);
        if self.fields.len() <= instance {
            self.fields.push(
                self.program.records[instance]
                    .fields
                    .iter()
                    .map(|field| (field.name.text.clone(), field.ty.unwrap()))
                    .collect(),
            );
        }
        Type::Record(instance)
    }
}

/// An inner planning pass must not finalize its enclosing storage bindings.
pub(super) fn declared_storage_bindings(body: &[Stmt]) -> HashSet<usize> {
    let mut declarations = HashSet::new();
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Let {
                binding: Some(binding),
                ..
            } => {
                declarations.insert(*binding);
            }
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => {
                pending.extend(body)
            }
            StmtKind::Loop { body, update, .. } => {
                pending.extend(body);
                pending.extend(update);
            }
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                pending.extend(then_body);
                pending.extend(else_body);
            }
            StmtKind::Try(region) => pending.extend(region.statements()),
            _ => {}
        }
    }
    declarations
}

#[cfg(test)]
mod worklist_tests {
    use super::*;
    #[test]
    fn deep_empty_record_and_map_layout_walks_preserve_unchanged_types(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Item={value:int;};function main(){}").unwrap()).unwrap();
        let mut ty=Type::Int;
        for _ in 0..16384{
            let id=analyzer.scoped_records.materialize(0,vec![ty],&mut analyzer.program.records);ty=Type::Record(id);
        }
        let Type::Record(id)=ty else{unreachable!()};
        let count=analyzer.program.records.len();
        assert_eq!(analyzer.empty_scoped_record_environment(id).unwrap(),ty);
        assert_eq!(analyzer.empty_scoped_map_layout(ty).unwrap(),ty);
        assert_eq!(analyzer.program.records.len(),count);
    }
    #[test]
    fn record_environment_expands_nullable_callbacks_but_map_layout_keeps_them(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("type Holder={run:ReadOnly<Function<int>>;};type Envelope={value:Holder|undefined;};function main(){}").unwrap()).unwrap();
        let holder=analyzer.program.records.iter().position(|record|record.name.text=="Holder").unwrap();
        let envelope=analyzer.program.records.iter().position(|record|record.name.text=="Envelope").unwrap();
        assert_eq!(analyzer.empty_scoped_map_layout(Type::Record(holder)).unwrap(),Type::Record(holder));
        let original=analyzer.program.records[holder].fields[0].ty.unwrap();
        let Type::Record(empty)=analyzer.empty_scoped_record_environment(envelope).unwrap()else{panic!("empty envelope")};
        let nullable=analyzer.program.records[empty].fields[0].ty.unwrap();
        let Some(Type::Record(payload))=analyzer.unions.payload(nullable)else{panic!("nullable holder")};
        let callback=analyzer.program.records[payload].fields[0].ty.unwrap();
        assert_ne!(callback,original);assert!(analyzer.absent_callable_type(callback));
        assert!(analyzer.unions.members(nullable.union().unwrap()).contains(&Type::Undefined));
        assert_eq!(analyzer.program.records[holder].fields[0].ty,Some(original));
    }
}

#[cfg(test)]
mod absent_result_tests {
    use super::*;
    #[test]
    fn deep_absent_callback_results_keep_order_and_reuse_cached_instances(){
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut result=Type::Int;let mut source=0;
        for _ in 0..4096{
            let structural=analyzer.callables.shapes.intern([],result,Some(CallbackKind::Read));
            source=analyzer.callables.callbacks.len();
            analyzer.callables.callbacks.push(CallbackContract{structural,parameter_names:Vec::new(),params:Vec::new(),result,kind:CallbackKind::Read});
            let id=analyzer.callables.results.len();
            analyzer.callables.results.push(crate::callable_resolution::ResultContract{structural,params:Vec::new(),result});
            result=Type::Callable(id);
        }
        let empty=analyzer.absent_callback(source);
        let count=analyzer.callables.callbacks.len();
        assert_eq!(analyzer.absent_callback(source),empty);assert_eq!(analyzer.callables.callbacks.len(),count);
        let mut current=empty;let mut depth=0;
        while let Type::Callback(id)=current{
            assert_eq!(analyzer.callables.callbacks[id].kind,CallbackKind::Read);
            assert!(analyzer.absent_callback_payloads.values().any(|&empty|empty==id));
            current=analyzer.callables.callbacks[id].result;depth+=1;
        }
        assert_eq!(depth,4096);assert_eq!(current,Type::Int);
    }
}
