//! Function body scheduling and lexical capture dependencies.
use super::*;
mod dependencies;

struct DependencyGroup {
    members: Vec<usize>,
    completion: Vec<usize>,
    recursive: bool,
}

impl Analyzer {
    /// Schedule source dependencies with heap continuations. Runtime syntax and
    /// source function identities remain unchanged. Capturing declarations wait
    /// for a real defining context, and active recursive edges retain inference.
    pub(super) fn function(&mut self, index: usize) -> Result<(), Diagnostic> {
        if self.function_request_ready(index)? { return Ok(()); }
        continuations::driver::check_function(self, index)
    }

    /// Logical record signatures do not identify their stored callback environments.
    pub(super) fn physical_record_result_needed(&self, result:Type)->bool {
        !self.callable_result_type(result) && self.aggregate_callable_facts(result).retains
    }

    /// Owned records retain concrete environment fields, not one erased callable signature.
    pub(super) fn owned_record_parameter(&self,ty:Type)->bool {
        matches!(ty,Type::Record(_)|Type::Array(ArrayElement::Record(..))|Type::Union(_)|Type::Map(_))
            && self.physical_record_result_needed(ty) && !self.aggregate_callable_facts(ty).scoped
    }

    pub(super) fn function_request_ready(&mut self, index: usize) -> Result<bool, Diagnostic> {
        if self.checked[index].is_some() { return Ok(true); }
        if self.discovering_results {
            // A newly instantiated generic (for example typed JSON decoding)
            // can revisit an async body before a later data factory is checked.
            // Its omitted result is unknown, not void. Context-independent
            // factories can be checked normally, without caller environments.
            let physical=(self.physical_record_result_needed(self.signatures[index].result) || self.effects_only && self.assumed_modes.is_none())
                && !self.templates.contains_key(&index)
                && (!self.named_captures.needs_context(index) || self.named_captures.ready(index));
            if ((self.signatures[index].result == Type::Void && self.inferred_result_context_is_closed(index)) || physical)
                && !self.visiting[index]
                && !self.active_recursive_functions.contains(&index)
            {
                let effects = self.effects_only;
                self.discovering_results = false;
                self.effects_only = false;
                let checked = self.function(index);
                self.effects_only = effects;
                self.discovering_results = true;
                checked?;
            }
            return Ok(true);
        }
        if self.active_recursive_functions.contains(&index) {
            if self.signatures[index].inferred_result && !self.provisional_results.contains_key(&index) {
                return Err(self.missing_recursive_result(index));
            }
            return Ok(true);
        }
        if self.visiting[index] { return self.prepare_function_body(index).map(|_| true); }
        Ok(false)
    }

    fn inferred_result_context_is_closed(&self, index: usize) -> bool {
        self.signatures[index].inferred_result
            && !self.named_captures.needs_context(index) && !self.templates.contains_key(&index)
            && self.program.functions[index].parameter_initializers.iter().all(|initializer| matches!(initializer, ParameterInitializer::Required))
            && self.signatures[index].body_params.iter().all(|ty| matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::String))
    }

    fn recursive_summary_is_complete(&self, index: usize, data_types: &mut HashMap<Type, bool>) -> bool {
        let signature = &self.signatures[index];
        if self.callable_result_type(signature.result) {
            // The initial pass requests recursive inference. Its discovery
            // worklist resolves concrete environments before body scheduling.
            return self.assumed_modes.is_none() || self.known_results.contains_key(&index);
        }
        if signature.inferred_result {
            // These contexts contain only the declared scalar parameters.
            // Defaults, destructuring, captures and concrete environments still
            // require a suspended full function context, not a guessed seed.
            return self.inferred_result_context_is_closed(index);
        }
        self.dependency_data_type(signature.result, data_types)
    }

    /// Closed data graphs need no caller-created callback environment. Traverse
    /// recursive record/union graphs on the heap; cache only within one unchanged
    /// graph inspection, since checking can publish new scoped record fields.
    fn dependency_data_type(&self, root: Type, cache: &mut HashMap<Type, bool>) -> bool {
        let mut pending = vec![root];
        let mut seen = HashSet::new();
        let valid = loop {
            let Some(ty) = pending.pop() else { break true; };
            if !seen.insert(ty) { continue; }
            if let Some(&valid) = cache.get(&ty) {
                if !valid { break false; }
                continue;
            }
            match ty {
                Type::Void | Type::String | Type::Int | Type::Number | Type::Boolean
                | Type::Null | Type::Undefined | Type::Enum(..) | Type::Literal(..) => {},
                Type::Array(element) => pending.push(element.ty()),
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.unions.members(id)),
                Type::Set(id) => pending.push(self.program.records[id].set_definition.as_ref().expect("set type").key),
                Type::Map(id) => {
                    let map = self.program.records[id].map_definition.as_ref().expect("map type");
                    pending.extend([map.key, map.value]);
                }
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    if record.native_type.is_some() || record.fields.iter().any(|field| field.ty.is_none()) { break false; }
                    pending.extend(record.fields.iter().map(|field| field.ty.expect("resolved data field")));
                }
                _ => break false,
            }
        };
        if valid { cache.extend(seen.into_iter().map(|ty| (ty, true))); }
        else { cache.insert(root, false); }
        valid
    }

    fn seed_recursive_results(&mut self, members: &[usize]) -> Result<(), Diagnostic> {
        for &index in members {
            if !self.signatures[index].inferred_result || self.provisional_results.contains_key(&index) { continue; }
            let function = &self.program.functions[index];
            let mut context = Context::new(function.name.text.clone());
            for ((parameter, _, _), &ty) in function.params.iter().zip(&self.signatures[index].body_params) {
                Self::bind(&mut context, parameter, ty, true)?;
            }
            if let Some(result) = self.provisional_result(&context, &function.body) {
                self.provisional_results.insert(index, result);
                self.signatures[index].result = result;
            }
        }
        Ok(())
    }

    fn missing_recursive_result(&self, index: usize) -> Diagnostic {
        Diagnostic::new("result-inference", "The result type of this recursive function cannot be inferred: every return depends on the recursive call or on a local variable. Add a result annotation.", self.program.functions[index].name.span)
    }

    fn function_dependency_order(&self, root: usize) -> Vec<DependencyGroup> {
        enum Work { Enter(usize), Finish(usize) }
        let mut pending = vec![Work::Enter(root)];
        let mut discovery = HashMap::new();
        let mut finish = Vec::new();
        let mut parents = HashMap::<usize, Vec<usize>>::new();
        let mut data_types = HashMap::new();
        while let Some(work) = pending.pop() {
            match work {
                Work::Enter(index) => {
                    if self.checked[index].is_some() || self.scheduled_functions.contains(&index)
                        || discovery.contains_key(&index) { continue; }
                    discovery.insert(index, discovery.len());
                    pending.push(Work::Finish(index));
                    let dependencies = dependencies::references(&self.program.functions[index], &self.function_names)
                        .into_iter().filter(|&dependency| {
                            self.checked[dependency].is_none() && !self.scheduled_functions.contains(&dependency)
                                // Record callback contracts depend on environments
                                // constructed by the caller. Keep their original
                                // checking point until context suspension is explicit.
                                && self.signatures[dependency].params.iter().all(|&ty|
                                    !matches!(ty, Type::Record(_)) || self.dependency_data_type(ty, &mut data_types))
                                && (!self.named_captures.needs_context(dependency) || self.named_captures.ready(dependency))
                        }).collect::<Vec<_>>();
                    for &dependency in &dependencies { parents.entry(dependency).or_default().push(index); }
                    pending.extend(dependencies.into_iter().rev().map(Work::Enter));
                }
                Work::Finish(index) => finish.push(index),
            }
        }
        // Collapse recursive groups before reversing dependency order. Within
        // a group keep its first source traversal entry first: provisional
        // inferred results must be seeded by the same entry as direct checking.
        let mut groups = Vec::new();
        let finish_rank = finish.iter().enumerate().map(|(rank, &index)| (index, rank)).collect::<HashMap<_, _>>();
        let mut seen = HashSet::new();
        for &root in finish.iter().rev() {
            if !seen.insert(root) { continue; }
            let mut group = Vec::new();
            let mut pending = vec![root];
            while let Some(index) = pending.pop() {
                group.push(index);
                for &parent in parents.get(&index).into_iter().flatten() {
                    if seen.insert(parent) { pending.push(parent); }
                }
            }
            group.sort_unstable_by_key(|index| discovery[index]);
            let recursive = group.len() > 1 || parents.get(&root).is_some_and(|parents| parents.contains(&root));
            let mut completion = group.clone();
            completion.sort_unstable_by_key(|index| finish_rank[index]);
            groups.push(DependencyGroup { members: group, completion, recursive });
        }
        groups.into_iter().rev().collect()
    }

    /// Resolve lexical owners before installing an unused-body placeholder.
    /// The continuation list preserves the original owner-first checks; owner
    /// completion can prepare or even check a waiting descendant.
    pub(super) fn capturing_function_is_unused(&mut self, index: usize) -> Result<bool, Diagnostic> {
        let mut pending = Vec::new();
        let mut current = index;
        let mut unused = loop {
            if !self.named_captures.needs_context(current) || self.named_captures.ready(current) {
                break false;
            }
            if let Some(owner) = self.named_captures.declaring_function(current, &self.program, &self.function_names)
                && owner != current
                && self.checked[owner].is_none()
                && !self.visiting[owner]
            {
                pending.push((current, owner));
                current = owner;
            } else {
                break self.defer_unused_capturing_function(current);
            }
        };
        while let Some((current, owner)) = pending.pop() {
            if !unused {
                self.function(owner)?;
            }
            unused = self.defer_unused_capturing_function(current);
        }
        Ok(unused)
    }

    fn defer_unused_capturing_function(&mut self, index: usize) -> bool {
        if self.named_captures.ready(index) || self.checked[index].is_some() {
            return false;
        }
        self.checked[index] = Some(CheckedFunction {
            storage_exits: Default::default(), native_returns: None, native_mutations: None,
            template: true,
            specialization: false,
            name: self.program.functions[index].name.text.clone(),
            bindings: Vec::new(),
            parameter_count: 0,
            result: self.signatures[index].result,
            body: Vec::new(),
        });
        true
    }

    pub(super) fn prepare_function_body(&mut self, index: usize) -> Result<Option<(PreparedFunction, super::function_parameters::FunctionParameters)>, Diagnostic> {
        if self.discovering_results {
            return Ok(None);
        }
        if self.checked[index].is_some() {
            return Ok(None);
        }
        if self.named_captures.needs_context(index) && !self.named_captures.ready(index) {
            return Err(self.program.local_capture_plans.iter().find_map(crate::local_capture_plan::Plan::pending_diagnostic).expect("pending named capture"));
        }
        let name = self.program.functions[index].name.clone();
        if self.visiting[index] {
            if self.signatures[index].inferred_result {
                // A return that does not depend on the recursive call fixes the
                // type; the finished body is then checked against it as usual.
                let Some(&provisional) = self.provisional_results.get(&index) else {
                    return Err(self.missing_recursive_result(index));
                };
                self.signatures[index].result = provisional;
            }
            let start = self
                .function_stack
                .iter()
                .position(|&id| id == index)
                .expect("active function");
            for &id in &self.function_stack[start..] {
                if self.signatures[id]
                    .params
                    .iter()
                    .any(|ty| matches!(ty, Type::Callback(_)))
                {
                    self.recursive_callbacks.insert(id);
                }
            }
            return if self.assumed_modes.is_some() {
                Ok(None)
            } else {
                Err(Diagnostic::new(
                    "recursion-inference",
                    "Recursive access inference is required.",
                    name.span,
                ))
            };
        }
        if self.callable_result_type(self.signatures[index].result)
            && self.assumed_modes.is_some()
            && !self.known_results.contains_key(&index)
        {
            if self.discovering_function.is_some(){
                self.note_result_wait(index,name.span);
                return Err(Self::pending_result(name.span));
            }
            self.discover_results()?;
        }
        self.visiting[index] = true;
        self.function_stack.push(index);
        self.depth += 1;
        // Result discovery can now finish a closed inferred factory and its
        // helpers before run() captures retry sources. Preserve bodies eligible
        // for callback/error-contract rechecking before consuming them.
        let retain_body=self.signatures[index].params.iter().copied().any(|ty|matches!(ty,Type::Record(_)) || self.owned_record_parameter(ty))
            || return_type_inference::has_exception_region(&self.program.functions[index].body);
        if let Some(sources) = &mut self.discovery_body_sources
            && retain_body
        {
            sources.entry(index).or_insert_with(|| self.program.functions[index].body.clone());
        }
        let body = std::mem::take(&mut self.program.functions[index].body);
        let (context, parameters) = self.reserve_function_parameters(index)?;
        let result_type = self.signatures[index].result;
        Ok(Some((PreparedFunction { index, body, context, result_type }, parameters)))
    }

    pub(super) fn complete_named_parameters(&mut self, index: usize, context: &mut Context, body: &[Stmt]) -> Type {
        if self.program.local_declarations.is_empty() && self.program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::LruCache)) {
            context.cache_last_uses=cache::last_uses(body).map(std::rc::Rc::new);
        }
        for (position, access) in self.program.functions[index].parameter_access.iter().enumerate() {
            if matches!(access, Some((Mode::Read, _))) && !context.bindings[position].ty.is_copy() {
                context.readonly_parameters.insert(position);
            }
        }
        if self.signatures[index].inferred_result && !self.provisional_results.contains_key(&index)
            && let Some(provisional) = self.provisional_result(context, body)
        {
            // Type hints inside the body read the signature before the
            // recursive call is checked, so publish the result now.
            self.provisional_results.insert(index, provisional);
            self.signatures[index].result = provisional;
        }
        // Move the body into analysis rather than clone the syntax tree.
        let result_type = self.signatures[index].result;
        context.inferred_body_result = self.signatures[index].inferred_result;
        result_type
    }

    pub(super) fn finish_function_body(&mut self, prepared: PreparedFunction, returned: Result<bool, Diagnostic>) -> Result<Vec<usize>, Diagnostic> {
        let PreparedFunction { index, mut body, mut context, mut result_type } = prepared;
        let name = self.program.functions[index].name.clone();
        let returned = returned.map_err(|error| self.filter_ownership_fixes(&context, error))?;
        if context.inferred_body_result {
            result_type = self.inferred_body_result(&body)?;
            self.signatures[index].result = result_type;
        }
        // Replay skipped this function's failed return; a declared data result
        // still types its callers. A callable result needs its checked closure.
        let contract=self.program.diagnostic_contracts.contains(&name.span.start) && !self.callable_result_type(result_type);
        if result_type != Type::Void && !returned && !contract {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "Function {:?} must return {}.",
                    self.program.functions[index]
                        .application
                        .as_ref()
                        .map(|application| application.declaration.text.as_str())
                        .unwrap_or(&name.text),
                    self.type_name(result_type)
                ),
                name.span,
            ));
        }
        self.finalize_nullable_locals(&mut context.bindings, &mut body)?;
        result_type = self.physical_record_result(&mut body, result_type)?;
        let result_type = if self.callable_result_type(result_type) {
            context.returned_callable.expect("checked callable return")
        } else {
            result_type
        };
        let result_type = self.nullable_record_result(&mut body, result_type)?;
        self.finalize_nullable_returns(&mut body, result_type)?;
        self.declared_parameter_access(index, &mut context)?;
        self.validate_native_returns(&context)?;
        if self.assumed_modes.is_some() && (self.callable_result_type(self.signatures[index].result) || self.physical_record_result_needed(self.signatures[index].result))
        {
            self.known_results.insert(index, result_type);
        }
        let uncalled = self.prepare_uncalled_capturing_functions(index, &context)?;
        self.checked[index] = Some(CheckedFunction {
                storage_exits: context.storage_exits.clone(), native_returns: context.native_returns.clone(), native_mutations: context.native_mutations.clone(),
            template: context.template,
            specialization: index >= self.original_count,
            name: name.text,
            bindings: context.bindings,
            parameter_count: self.signatures[index].params.len(),
            result: result_type,
            body,
        });
        self.visiting[index] = false;
        self.function_stack.pop();
        self.depth -= 1;
        Ok(uncalled)
    }

}

pub(super) struct PreparedFunction {
    pub(super) index: usize,
    pub(super) body: Vec<Stmt>,
    pub(super) context: Context,
    pub(super) result_type: Type,
}

enum Work { Schedule(usize), Group(DependencyGroup), Check(usize), ReleaseGroup(Vec<usize>), Release(Vec<usize>) }

/// Dependency work survives while a function body runs on the checker stack.
pub(super) struct Schedule {
    pending: Vec<Work>,
    inserted: Vec<usize>,
    recursive_inserted: Vec<usize>,
}

impl Schedule {
    pub(super) fn new(index: usize) -> Self {
        Self { pending: vec![Work::Schedule(index)], inserted: Vec::new(), recursive_inserted: Vec::new() }
    }

    pub(super) fn extend(&mut self, uncalled: Vec<usize>) {
        self.pending.extend(uncalled.into_iter().rev().map(Work::Schedule));
    }

    pub(super) fn next(&mut self, analyzer: &mut Analyzer) -> Result<Option<usize>, Diagnostic> {
        while let Some(work) = self.pending.pop() {
            match work {
                Work::Schedule(index) => {
                    if analyzer.checked[index].is_some() { continue; }
                    if analyzer.scheduled_functions.contains(&index) {
                        self.pending.push(Work::Check(index));
                        continue;
                    }
                    let groups = analyzer.function_dependency_order(index);
                    let order = groups.iter().flat_map(|group| group.members.iter().copied()).collect::<Vec<_>>();
                    for &id in &order {
                        if analyzer.scheduled_functions.insert(id) { self.inserted.push(id); }
                    }
                    self.pending.push(Work::Release(order));
                    self.pending.extend(groups.into_iter().rev().map(Work::Group));
                }
                Work::Group(group) => {
                    let mut data_types = HashMap::new();
                    if group.recursive && group.members.iter().all(|&id| analyzer.recursive_summary_is_complete(id, &mut data_types)) {
                        if analyzer.assumed_modes.is_none() {
                            return Err(Diagnostic::new("recursion-inference", "Recursive access inference is required.", analyzer.program.functions[group.members[0]].name.span));
                        }
                        analyzer.seed_recursive_results(&group.members)?;
                        for &id in &group.members {
                            if analyzer.active_recursive_functions.insert(id) { self.recursive_inserted.push(id); }
                            if analyzer.signatures[id].params.iter().any(|ty| matches!(ty, Type::Callback(_))) {
                                analyzer.recursive_callbacks.insert(id);
                            }
                        }
                        self.pending.push(Work::ReleaseGroup(group.members.clone()));
                        // Actual DFS completion order also handles shared
                        // callees: reversing discovery alone can put a sibling
                        // before the inferred result that it depends on.
                        self.pending.extend(group.completion.into_iter().rev().map(Work::Check));
                    } else {
                        self.pending.extend(group.members.into_iter().rev().map(Work::Check));
                    }
                }
                Work::ReleaseGroup(ids) => {
                    for id in ids { analyzer.active_recursive_functions.remove(&id); }
                }
                Work::Check(index) => return Ok(Some(index)),
                Work::Release(ids) => {
                    for id in ids { analyzer.scheduled_functions.remove(&id); }
                }
            }
        }
        Ok(None)
    }

    pub(super) fn release(self, analyzer: &mut Analyzer) {
        for id in self.inserted { analyzer.scheduled_functions.remove(&id); }
        for id in self.recursive_inserted { analyzer.active_recursive_functions.remove(&id); }
    }
}

#[cfg(test)]
#[test]
fn specialized_record_factories_publish_physical_results_before_callers() {
    let source="type Holder={pull:Owned<Function<int>>;};function make(pull:Owned<Function<int>>):Holder{return {pull:pull};}function outer():Holder{return make(():int=>7);}function main(){const holder=outer();console.log(holder.pull());}";
    let checked=crate::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source)));
    let program=&checked.program;
    let outer=program.functions.iter().find(|function|function.name=="outer").unwrap();
    let value=outer.body.iter().find_map(|statement|if let StmtKind::Return(Some(value))=&statement.kind{Some(value)}else{None}).unwrap();
    let Some(CallTarget::Function(target))=value.target else{panic!("factory call: {:?}",value.target)};
    assert_eq!(value.ty,Some(program.functions[target].result),"call expression must retain its already checked callee shape");
    assert_eq!(outer.result,program.functions[target].result,"owned return must preserve the physical carrier");
}

#[cfg(test)]
#[test]
fn owned_record_parameters_preserve_each_callers_physical_fields(){
    let source="type Holder={pull:Owned<Function<int>>;};function useHolder(value:Owned<Holder>):int{return value.pull();}function make(pull:Owned<Function<int>>):Holder{return {pull:pull};}function main(){console.log(useHolder(make(():int=>7)),useHolder(make(():int=>9)));}";
    let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
    let checked=crate::check(&source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",&source)));
    let program=&checked.program;let mut count=0;
    for function in &program.functions {if function.template{continue;}crate::fixed_storage::visit_function_expressions(function,&mut |value|{
        if let (Some(CallTarget::Function(target)),ExprKind::Call(_,arguments))=(value.target,&value.kind){let callee=&program.functions[target];if callee.name=="useHolder"{count+=1;assert_eq!(arguments[0].ty,Some(callee.bindings[0].ty));}}
    });}assert_eq!(count,2);
}

#[cfg(test)]
#[test]
fn native_view_and_owned_record_callers_keep_both_source_orders(){
    use crate::project::{SourceId,SourceProvider,check_graph,load_project};
    struct Input<'a>(&'a str);
    impl SourceProvider for Input<'_>{
        fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
        fn read(&self,id:&SourceId)->Result<String,String>{if id.0=="main.tsr"{Ok(self.0.into())}else{Err(format!("Missing source: {}",id.0))}}
    }
    for borrowed_first in [true,false]{
        let calls=if borrowed_first{"useHolder(borrowed);useHolder(copied);useOwned(owned);"}else{"useOwned(owned);useHolder(copied);useHolder(borrowed);"};
        let source=format!("import {{alloc,view,length}} from 'std:bytes';import {{trackedResource,closeResource}} from 'std:testing';type Holder={{pull:ReadOnly<Function<int>>;}};type OwnedHolder={{pull:Once<Function<int>>;}};function useHolder(value:ReadOnly<Holder>):int{{return value.pull();}}function useOwned(value:Owned<OwnedHolder>):int{{return value.pull();}}function main(){{const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const borrowed:Holder={{pull:():int=>length(slice)}};const number=7;const copied:Holder={{pull:():int=>number}};const resource=trackedResource();const owned:OwnedHolder={{pull:():int=>{{closeResource(resource);return 9;}}}};{calls}}}");
        let source=crate::format(&source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
        let checked=check_graph(load_project(&Input(&source),"main.tsr").unwrap()).unwrap_or_else(|error|panic!("{error:?}"));
        let program=&checked.analysis.program;let mut readonly=0;let mut owned=0;
        for function in &program.functions{if function.template{continue;}crate::fixed_storage::visit_function_expressions(function,&mut |value|{
            if let (Some(CallTarget::Function(target)),ExprKind::Call(_,arguments))=(value.target,&value.kind){
                let callee=&program.functions[target];
                if callee.name.ends_with("useHolder"){readonly+=1;assert_eq!(callee.bindings[0].mode,crate::Mode::Read);}
                if callee.name.ends_with("useOwned"){owned+=1;assert_eq!(callee.bindings[0].mode,crate::Mode::Consume);assert_eq!(arguments[0].ty,Some(callee.bindings[0].ty),"Owned argument and parameter must share the actual physical carrier, whether nominal or specialized");}
            }
        });}
        assert!(readonly>=1,"Borrowed native-view callers retain their readonly contract");
        assert!(owned>=1,"The non-Clone capture reaches a consuming physical parameter");
        drop(checked.emit_rust());
    }
}

#[cfg(test)]
#[test]
fn readonly_mixed_record_arguments_keep_physical_union_layouts(){
 use crate::project::{SourceId,SourceProvider,check_graph,load_project};
 struct Input<'a>(&'a str);
 impl SourceProvider for Input<'_>{
  fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
  fn read(&self,id:&SourceId)->Result<String,String>{if id.0=="main.tsr"{Ok(self.0.into())}else{Err(format!("Missing source: {}",id.0))}}
 }
 let source=crate::format("import {Iterator} from 'std:iterator';function observe(value:ReadOnly<int[]|Iterator<int>>):boolean{return Array.isArray(value);}function main(){const value:int[]|Iterator<int>=Iterator.from([2,4]);if(!Array.isArray(value)){observe(value);observe(value);console.log(value.toArray().join(','));}const array:int[]|Iterator<int>=[5,6];if(Array.isArray(array)){observe(array);observe(array);console.log(array.join(','));}}").unwrap();assert_eq!(crate::format(&source).unwrap(),source);
 let checked=check_graph(load_project(&Input(&source),"main.tsr").unwrap()).unwrap_or_else(|errors|panic!("{errors:?}"));let program=&checked.analysis.program;
 let mut observations=0;
 for function in &program.functions{if function.template{continue;}crate::fixed_storage::visit_function_expressions(function,&mut |value|{
  if let (Some(CallTarget::Function(target)),ExprKind::Call(_,arguments))=(value.target,&value.kind){let callee=&program.functions[target];if callee.name.ends_with("observe"){
   observations+=1;assert_eq!(callee.bindings[0].mode,crate::Mode::Read);assert_eq!(arguments[0].ty.map(Type::owning),Some(callee.bindings[0].ty.owning()),"Readonly calls retain their selected physical union instead of an unrelated nominal tag");assert!(matches!(arguments[0].ty,Some(Type::BorrowedUnion(_))));
  }}
 });}assert_eq!(observations,4);
 let rust=checked.emit_rust();let main=program.functions.iter().find(|function|function.name=="main").unwrap();let mut unions=0;
 for(index,binding)in main.bindings.iter().enumerate(){if let Type::Union(union)=binding.ty{
  let marker=format!("value{index}: Union{}",union.index());let position=rust.find(&marker).expect("owning mixed union annotation");assert!(rust[position+marker.len()..].starts_with('<'),"Every physical payload slot needs its concrete type: {}",&rust[position..rust[position..].find('\n').map_or(rust.len(),|end|position+end)]);unions+=1;
 }}assert_eq!(unions,2);
}

#[cfg(test)]
#[test]
fn pending_region_checks_method_plan_origins_without_a_concrete_callee_type(){
    let source="function selected():int{return 7;}function unrelated():int{return 9;}function generic<T>(value:T):int{return selected();}function main(){const read=generic;read<int>(1);}";
    let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
    let selected=analyzer.function_names["selected"];let unrelated=analyzer.function_names["unrelated"];
    let selected_body=analyzer.program.functions[selected].body.clone();let unrelated_body=analyzer.program.functions[unrelated].body.clone();
    let main=analyzer.function_names["main"];analyzer.function(main).unwrap();
    let checked=analyzer.checked[main].as_ref().unwrap();let mut body=checked.body.clone();let result=checked.result;
    let mut found=false;
    crate::source_walk::body(&mut body,&mut |_|Ok(()),&mut |value|{
        if let Some(CallTarget::ClosureMethod(method))=value.target{
            let ExprKind::Call(callee,_)=&mut value.kind else{panic!("method invocation")};
            let Some(Type::Closure(origin))=callee.ty else{panic!("concrete control environment")};
            let plan=analyzer.polymorphic_invocations.publish([crate::polymorphic_invocation_plans::Pair{origin,method}],&analyzer.closures,callee.span)?;
            callee.generic_invocation=Some(plan);callee.ty=None;found=true;
        }
        Ok(())
    }).unwrap();assert!(found);
    analyzer.checked[selected]=None;analyzer.program.functions[selected].body=selected_body;
    analyzer.checked[unrelated]=None;analyzer.program.functions[unrelated].body=unrelated_body;
    analyzer.check_pending_region_functions(&body,result,false).unwrap();
    assert!(analyzer.checked[selected].is_some(),"the checked plan identifies the real method's callee");
    assert!(analyzer.checked[unrelated].is_none(),"preparation must not eagerly check unrelated functions");
}

#[cfg(test)]
mod cached_result_readiness_tests {
 use super::*;
 #[test]
 fn cached_callable_specialization_requeries_current_discovery_phase(){
  let source="function factory(callback:Owned<Function<int>>):Once<Function<int>>{return()=>callback();}function data(callback:Owned<Function<int>>):int{return callback();}function main(){factory(()=>7);}";
  let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let mut analyzer=Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
  let main=analyzer.function_names["main"];let mut body=analyzer.program.functions[main].body.clone();let mut context=analyzer.function_context(main,&mut body).unwrap();
  let StmtKind::Expr(expression)=&mut body[0].kind else{panic!("factory call")};
  let ExprKind::Call(_,arguments)=&mut expression.kind else{panic!("factory arguments")};
  for argument in arguments.iter_mut(){analyzer.expression(&mut context,argument,Mode::Consume,&Loans::new(),None).unwrap();}
  let span=expression.span;
  for(name,needs_origin)in[("factory",true),("data",false)]{
   let index=analyzer.function_names[name];analyzer.discovering_results=true;
   let(first,immediate)=analyzer.prepare_specialization(index,arguments,span).unwrap();assert!(!immediate,"{name}: discovery remains deferred");assert_ne!(first,index,"{name}: actual specialization required");
   let count=analyzer.specializations.len();analyzer.discovering_results=false;
   let(second,immediate)=analyzer.prepare_specialization(index,arguments,span).unwrap();assert_eq!(second,first);assert_eq!(analyzer.specializations.len(),count);assert_eq!(immediate,needs_origin,"{name}: cached readiness must match a fresh entry");
   analyzer.discovering_results=true;let(third,immediate)=analyzer.prepare_specialization(index,arguments,span).unwrap();assert_eq!(third,first);assert!(!immediate,"{name}: returning to discovery restores deferral");
  }
 }
}
