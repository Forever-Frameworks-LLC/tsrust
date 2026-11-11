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


type EmptyKey=(usize,bool);
#[derive(Default)]
struct EmptyWalk{
 indices:HashMap<EmptyKey,usize>,low:HashMap<EmptyKey,usize>,path:Vec<EmptyKey>,stack:Vec<EmptyKey>,active:HashSet<EmptyKey>,
 plans:HashMap<EmptyKey,Vec<Type>>,resolved:HashMap<EmptyKey,Type>,
}
impl EmptyWalk{
 fn enter(&mut self,key:EmptyKey)->bool{
  if let Some(&index)=self.indices.get(&key){
   if self.active.contains(&key)&&let Some(parent)=self.path.last(){let low=self.low.get_mut(parent).unwrap();*low=(*low).min(index);}
   return false;
  }
  let index=self.indices.len();self.indices.insert(key,index);self.low.insert(key,index);self.path.push(key);self.stack.push(key);self.active.insert(key);true
 }
 fn finish(&mut self,key:EmptyKey,types:Vec<Type>)->Option<Vec<(EmptyKey,Vec<Type>)>>{
  self.plans.insert(key,types);assert_eq!(self.path.pop(),Some(key));
  if let Some(parent)=self.path.last(){let child=self.low[&key];let low=self.low.get_mut(parent).unwrap();*low=(*low).min(child);}
  if self.low[&key]!=self.indices[&key]{return None;}
  let mut component=Vec::new();loop{let member=self.stack.pop().unwrap();self.active.remove(&member);component.push((member,self.plans.remove(&member).unwrap()));if member==key{break;}}
  Some(component)
 }
}


impl Analyzer {
    pub(super) fn empty_scoped_map_layout(&mut self,ty:Type)->Result<Type,Diagnostic>{
        self.empty_environment_work(EmptyTask::MapLayout(ty))
    }
    pub(super) fn empty_scoped_record_environment(&mut self,id:usize)->Result<Type,Diagnostic>{
        self.empty_environment_work(EmptyTask::Record(id,false))
    }
    fn empty_environment_work(&mut self,root:EmptyTask)->Result<Type,Diagnostic>{
        let mut work=vec![root];let mut result=Type::Void;let mut walk=EmptyWalk::default();
        while let Some(task)=work.pop(){match task{
            EmptyTask::MapLayout(ty)=>{
                result=ty;
                match ty{
                    Type::Map(id)=>{
                        let payload=self.program.records[id].map_definition.as_ref().unwrap().value;
                        let Type::Record(record)=payload else{continue;};
                        if self.scoped_records.declaration(record)!=record || !self.aggregate_callable_facts(payload).scoped{continue;}
                        let key=(id,false);
                        if let Some(&known)=walk.resolved.get(&key){result=known;continue;}
                        if !walk.enter(key){continue;}
                        work.push(EmptyTask::MapValue(id));work.push(EmptyTask::Record(record,false));
                    }
                    Type::Record(id)=>work.push(EmptyTask::Record(id,true)),
                    _=>{}
                }
            }
            EmptyTask::MapValue(id)=>{
                let key=self.program.records[id].map_definition.as_ref().unwrap().key;
                result=self.finish_empty_component(&mut walk,(id,false),vec![key,result])?;
            },
            EmptyTask::Record(id,maps_only)=>{
                let key=(id,maps_only);
                if let Some(&known)=walk.resolved.get(&key){result=known;continue;}
                if !walk.enter(key){result=Type::Record(id);continue;}
                let fields=self.program.records[id].fields.iter().map(|field|field.ty.unwrap()).collect::<Vec<_>>();
                work.push(EmptyTask::Fields{id,fields,types:Vec::new(),index:0,maps_only});
            }
            EmptyTask::Fields{id,fields,mut types,index,maps_only}=>{
                if index>0{types.push(result);}
                if let Some(&field)=fields.get(index){
                    let child=if maps_only{EmptyTask::MapLayout(field)}else{EmptyTask::Field(field,self.program.records[id].name.span)};
                    work.push(EmptyTask::Fields{id,fields,types,index:index+1,maps_only});work.push(child);
                }else{result=self.finish_empty_component(&mut walk,(id,maps_only),types)?;}
            }
            EmptyTask::Field(ty,span)=>{
                result=ty;
                match ty{
                    Type::Callback(id)=>result=if self.absent_callable_type(ty){ty}else{self.absent_callback(id)},
                    Type::StoredCallable(family)=>{
                        if let Some(id)=self.callables.storage.declaration(family) && self.callables.callbacks[id].kind.is_owned(){result=self.absent_callback(id);}else{result=ty;}
                    }
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
                        Type::Callback(id)=>result=if self.absent_callable_type(member){member}else{self.absent_callback(id)},
                        Type::StoredCallable(family)=>{
                            if let Some(id)=self.callables.storage.declaration(family) && self.callables.callbacks[id].kind.is_owned(){result=self.absent_callback(id);}else{result=member;}
                        }
                        Type::Record(id)=>work.push(EmptyTask::Record(id,false)),
                        other=>result=other,
                    }
                }else{result=self.unions.intern(types,span)?;}
            }
        }}
        Ok(result)
    }

    fn finish_empty_component(&mut self,walk:&mut EmptyWalk,key:EmptyKey,types:Vec<Type>)->Result<Type,Diagnostic>{
        let original=if self.program.records[key.0].map_definition.is_some(){Type::Map(key.0)}else{Type::Record(key.0)};
        let Some(component)=walk.finish(key,types)else{return Ok(original);};
        let unchanged=component.iter().all(|((id,_),types)|{
            if let Some(map)=&self.program.records[*id].map_definition{types.as_slice()==[map.key,map.value]}
            else{self.program.records[*id].fields.iter().map(|field|field.ty.unwrap()).eq(types.iter().copied())}
        });
        if unchanged{for((id,mode),_)in component{let ty=if self.program.records[id].map_definition.is_some(){Type::Map(id)}else{Type::Record(id)};walk.resolved.insert((id,mode),ty);}return Ok(original);}
        if component.len()==1{
            let ((id,mode),types)=component.into_iter().next().unwrap();
            // A self edge still needs identity reservation/remapping below.
            let recursive=types.iter().any(|ty|self.empty_type_mentions(*ty,id));
            if !recursive{let ty=if self.program.records[id].map_definition.is_some(){Type::Map(self.scoped_map_value_instance(id,types[1]))}else{self.empty_environment_record(id,types)};walk.resolved.insert((id,mode),ty);return Ok(ty);}
            return self.finish_empty_recursive_component(walk,key,vec![((id,mode),types)]);
        }
        self.finish_empty_recursive_component(walk,key,component)
    }
    fn empty_type_mentions(&self,ty:Type,id:usize)->bool{
        let mut pending=vec![ty];let mut seen=HashSet::new();while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty{Type::Record(record)|Type::Map(record) if record==id=>return true,Type::Array(element)=>pending.push(element.with_depth(0).ty()),Type::Union(union)|Type::BorrowedUnion(union)=>pending.extend(self.unions.members(union)),_=>{}}}false
    }
    fn finish_empty_recursive_component(&mut self,walk:&mut EmptyWalk,key:EmptyKey,component:Vec<(EmptyKey,Vec<Type>)>)->Result<Type,Diagnostic>{
        let start=self.program.records.len();let fields=component.iter().map(|((id,_),types)|(*id,types.clone())).collect();
        let remap=self.scoped_records.materialize_empty_component(fields,&mut self.program.records,&self.unions)?;
        for id in start..self.program.records.len(){self.fields.push(self.program.records[id].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());if self.program.records[id].map_definition.is_some(){self.unions.register_map_type(id);}}
        for((id,mode),_)in component{let mapped=remap[&id];let ty=if self.program.records[mapped].map_definition.is_some(){Type::Map(mapped)}else{Type::Record(mapped)};walk.resolved.insert((id,mode),ty);}
        Ok(walk.resolved[&key])
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

#[cfg(test)]
#[test]
fn empty_owned_record_fields_use_explicit_absent_callback_identity(){
 let source=crate::format("type Holder={run:Owned<Function<int>>;};function main(){}").unwrap();assert_eq!(crate::format(&source).unwrap(),source);
 let mut analyzer=Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();let record=analyzer.record_names["Holder"];
 let source=*analyzer.fields[record].get("run").unwrap();assert!(matches!(source,Type::StoredCallable(_)));
 let empty=analyzer.empty_scoped_record_environment(record).unwrap();assert_eq!(analyzer.empty_scoped_record_environment(record).unwrap(),empty);
 let Type::Record(id)=empty else{panic!("empty record carrier")};let field=*analyzer.fields[id].get("run").unwrap();
 assert_ne!(field,source);assert!(analyzer.absent_callable_type(field));assert!(analyzer.finite_owned_shape(empty));
 assert_eq!(analyzer.scoped_records.declaration(id),record);
}

#[cfg(test)]
mod recursive_empty_environment_tests{
 use super::*;
 fn analyzed(source:&str)->Analyzer{let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap()}
 fn field(analyzer:&Analyzer,id:usize,name:&str)->Type{analyzer.fields[id][name]}
 #[test]
 fn self_recursive_empty_record_reuses_finite_absent_layout(){
  let mut analyzer=analyzed("type Holder={run:Once<Function<int>>;next?:Holder;};function main(){}");let source=analyzer.record_names["Holder"];
  let original=field(&analyzer,source,"run");let empty=analyzer.empty_scoped_record_environment(source).unwrap();let Type::Record(id)=empty else{panic!("empty holder")};
  assert!(analyzer.absent_callable_type(field(&analyzer,id,"run")));assert_eq!(analyzer.unions.payload(field(&analyzer,id,"next")),Some(Type::Record(id)));assert_eq!(field(&analyzer,source,"run"),original);
  let count=analyzer.program.records.len();assert_eq!(analyzer.empty_scoped_record_environment(source).unwrap(),empty);assert_eq!(analyzer.program.records.len(),count);assert_eq!(analyzer.empty_scoped_record_environment(id).unwrap(),empty);assert_eq!(analyzer.program.records.len(),count);
 }
 #[test]
 fn both_members_of_mutual_empty_component_keep_their_absent_callbacks(){
  let mut analyzer=analyzed("type A={first:Once<Function<int>>;next?:B;};type B={second:Once<Function<int>>;next?:A;};function main(){}");let a=analyzer.record_names["A"];let b=analyzer.record_names["B"];
  let Type::Record(empty_a)=analyzer.empty_scoped_record_environment(a).unwrap()else{panic!("empty A")};let Some(Type::Record(empty_b))=analyzer.unions.payload(field(&analyzer,empty_a,"next"))else{panic!("empty B edge")};
  assert!(analyzer.absent_callable_type(field(&analyzer,empty_a,"first")));assert!(analyzer.absent_callable_type(field(&analyzer,empty_b,"second")));assert_eq!(analyzer.unions.payload(field(&analyzer,empty_b,"next")),Some(Type::Record(empty_a)));
  let count=analyzer.program.records.len();assert_eq!(analyzer.empty_scoped_record_environment(b).unwrap(),Type::Record(empty_b));assert_eq!(analyzer.program.records.len(),count);assert_eq!(analyzer.scoped_records.declaration(empty_a),a);assert_eq!(analyzer.scoped_records.declaration(empty_b),b);
 }
 #[test]
 fn recursive_map_edges_preserve_maps_only_callback_policy(){
  let mut analyzer=analyzed("type Holder={run:ReadOnly<Function<int>>;children:Map<int,Holder>;other?:Map<int,Holder>;};function main(){}");let holder=analyzer.record_names["Holder"];let original=field(&analyzer,holder,"run");
  let Type::Record(empty)=analyzer.empty_scoped_record_environment(holder).unwrap()else{panic!("empty holder")};assert!(analyzer.absent_callable_type(field(&analyzer,empty,"run")));
  let Type::Map(map)=field(&analyzer,empty,"children")else{panic!("empty map edge")};assert_eq!(analyzer.program.records[map].map_definition.as_ref().unwrap().value,Type::Record(empty));assert_eq!(analyzer.unions.named_type(map),Type::Map(map));let optional=field(&analyzer,empty,"other").union().unwrap();assert!(analyzer.unions.members(optional).contains(&Type::Map(map)));
  let Type::Record(outer)=analyzer.empty_scoped_map_layout(Type::Record(holder)).unwrap()else{panic!("map-only holder")};assert_eq!(field(&analyzer,outer,"run"),original);let Type::Map(map)=field(&analyzer,outer,"children")else{panic!("map-only child")};let Type::Record(payload)=analyzer.program.records[map].map_definition.as_ref().unwrap().value else{panic!("map-only payload")};assert!(analyzer.absent_callable_type(field(&analyzer,payload,"run")));
  let count=analyzer.program.records.len();assert_eq!(analyzer.empty_scoped_map_layout(Type::Record(holder)).unwrap(),Type::Record(outer));assert_eq!(analyzer.program.records.len(),count);
 }
 #[test]
 fn full_component_cache_cannot_alias_an_ordinary_root_only_plan(){
  let mut analyzer=analyzed("type A={first:Once<Function<int>>;next?:B;};type B={second:Once<Function<string>>;next?:A;};function main(){}");let a=analyzer.record_names["A"];let b=analyzer.record_names["B"];
  let Type::StoredCallable(family)=field(&analyzer,a,"first")else{panic!("owned field")};let callback=analyzer.callables.storage.declaration(family).unwrap();let absent=analyzer.absent_callback(callback);let next=field(&analyzer,a,"next");
  let start=analyzer.program.records.len();let root_only=analyzer.scoped_records.materialize_recursive(a,vec![absent,next],&mut analyzer.program.records,&analyzer.unions,&|_|false).unwrap();
  for id in start..analyzer.program.records.len(){analyzer.fields.push(analyzer.program.records[id].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
  let Type::Record(empty)=analyzer.empty_scoped_record_environment(a).unwrap()else{panic!("complete empty A")};assert_ne!(empty,root_only);
  let Some(Type::Record(child))=analyzer.unions.payload(field(&analyzer,empty,"next"))else{panic!("complete empty B")};assert!(analyzer.absent_callable_type(field(&analyzer,child,"second")));
  let count=analyzer.program.records.len();assert_eq!(analyzer.empty_scoped_record_environment(b).unwrap(),Type::Record(child));assert_eq!(analyzer.program.records.len(),count);
  assert_eq!(analyzer.scoped_records.materialize_recursive(a,vec![absent,next],&mut analyzer.program.records,&analyzer.unions,&|_|false).unwrap(),root_only);
 }

}
