//! Named native lifetimes preserve independent input origins and returned intersections.
use super::*;
use crate::analysis::native_view_loans::{LifetimeSource, Segment, VariantGuard};
use std::collections::BTreeMap;

#[derive(Default,Clone)]
pub(super) struct Lifetimes {
    inputs: BTreeMap<(usize,LifetimeSource),String>,
    outputs: BTreeMap<(Vec<Segment>,Vec<VariantGuard>),String>,
    shared_outputs: std::collections::BTreeSet<(Vec<Segment>,Vec<VariantGuard>)>,
    pub(super) declarations: Vec<String>,
    pub(super) bounds: Vec<String>,
}

#[cfg(test)]
mod native_forward_tests {
    use super::*;

    #[test]
    fn task_factory_callback_impls_bind_storage_and_carried_slots_independently(){
        // Concrete view payloads retain independent inner lifetimes as well as
        // the two outer ReadOnly loans. An abstract Task parameter is lowered
        // through a forwarding specialization, which emits no callback impl.
        let source="type Window={};type Pair={one:Window;};async function choose(left:ReadOnly<Pair>,right:ReadOnly<Pair>,flag:boolean):Task<int>{if(flag){const value=left.one;return 1;}const value=right.one;return 2;}function main(){const action=choose;}";
        let mut parsed=crate::syntax::parse(&crate::format(source).unwrap()).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked=crate::analysis::analyze(parsed).unwrap();let mut emitter=super::super::emitter_for(&checked,"");emitter.callback_traits=true;let rust=emitter.program();let parsed=syn::parse_file(&rust).unwrap();
        struct Uses(std::collections::HashSet<String>);impl<'ast> syn::visit::Visit<'ast> for Uses{fn visit_lifetime(&mut self,lifetime:&'ast syn::Lifetime){let name=lifetime.ident.to_string();if name.starts_with("native_input"){self.0.insert(name);}}}
        let mut carried=std::collections::HashSet::new();let mut storage=std::collections::HashSet::new();
        for item in parsed.items{let syn::Item::Impl(item)=item else{continue;};let Some((path,_))=&item.trait_ else{continue;};let name=path.segments.last().unwrap().ident.to_string();if !matches!(name.as_str(),"OnceCallback"|"ReadCallback"|"Callback"){continue;}
            let declared=item.generics.lifetimes().map(|parameter|parameter.lifetime.ident.to_string()).collect::<std::collections::HashSet<_>>();let mut uses=Uses(Default::default());syn::visit::Visit::visit_item_impl(&mut uses,&item);assert!(uses.0.is_subset(&declared),"undeclared callback lifetimes: {:?}",uses.0.difference(&declared).collect::<Vec<_>>());
            if uses.0.len()>=2{carried.insert(name.clone());}if declared.contains("p0")&&declared.contains("p1"){storage.insert(name);}
        }
        for name in["OnceCallback","ReadCallback","Callback"]{assert!(carried.contains(name),"missing independent carried slots in {name}");assert!(storage.contains(name),"missing independent storage binders in {name}");}
        let mapping=std::collections::HashMap::from([("native_input0_1".into(),"p0".into())]);assert_eq!(callback_capture_emit::substitute_parameters("'native_input0_1: 'native_input0_10",&mapping),"'p0: 'native_input0_10");
    }

    #[test]
    fn consumed_task_relay_names_each_outer_reference_in_input_and_output(){
        let source="async function size(left:ReadOnly<string>,right:ReadOnly<int[]>):Task<int>{return left.length+right.length;}function relay<T>(work:Task<T>):Task<T>{return work;}async function main():Task<void>{const left='a';const right=[1];const pending=relay(size(left,right));console.log(await pending);}";
        let checked=crate::analysis::analyze(crate::syntax::parse(&crate::format(source).unwrap()).unwrap()).unwrap();let emitter=super::super::emitter_for(&checked,"");let mut count=0;
        for(index,function)in checked.functions.iter().enumerate(){if !function.name.starts_with("relay")||function.template||!function.bindings.first().is_some_and(|binding|matches!(binding.ty,Type::Closure(_))){continue;}count+=1;let signature=emitter.native_function_signature(index,&[]);assert!(!signature.params.contains("'_"));assert!(!signature.result.contains("'_"));for slot in 0..2{let name=format!("'native_input0_{slot}");assert!(signature.generics.contains(&name));assert!(signature.params.contains(&name));assert!(signature.result.contains(&name));}}
        assert!(count>0);
    }

    #[test]
    fn native_forward_signatures_name_the_checked_source_in_every_return_slot() {
        for (module,name) in [("std:bytes","view"),("std:bytes","viewMut"),("std:url","formatUrl"),("std:url","__hash"),("std:json","get")] {
            let binding=crate::native_bindings::BINDINGS.iter().find(|binding|binding.module==module && binding.name==name).unwrap();
            let source=crate::format("function main(){const action=operation;}").unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let mut parsed=crate::syntax::parse(&source).unwrap();
            crate::native_binding_types::prepare(&mut parsed.records,binding,crate::Span::default()).unwrap();
            let StmtKind::Let{value,..}=&mut parsed.functions[0].body[0].kind else {panic!("native value declaration")};
            value.kind=ExprKind::Name(format!("@native:{module}#{name}"));
            let checked=crate::analysis::analyze(parsed).unwrap();
            let emitter=super::super::emitter_for(&checked,"");
            let closure=checked.closures.iter().find(|closure|matches!(closure.forward,Some(ClosureForward::Native(_)))).unwrap();
            let signature=emitter.native_forward_borrow_signature(closure).unwrap();
            let root=closure.parameters[binding.returned_borrow.unwrap().source_parameter];
            let lifetime=&signature.storage_lifetimes[&root];
            assert!(signature.params.contains(&format!("value{root}: &{lifetime} ")) || signature.params.contains(&format!("value{root}: &{lifetime} mut ")),"{module}.{name}: {}",signature.params);
            assert!(signature.result.contains(lifetime),"{module}.{name}: {}",signature.result);
            assert!(!signature.result.contains("'_"),"{module}.{name}: {}",signature.result);
            syn::parse_file(&crate::emit::emit(&checked)).unwrap();
        }
    }
}
impl Lifetimes {
    pub(super) fn empty_output(&mut self) {
        // A checked empty component carries no owner. Quantify its output
        // lifetime independently instead of inventing a borrow or using static.
        debug_assert!(self.outputs.is_empty());
        let name = "'native_empty".to_owned();
        self.declarations.push(name.clone());
        self.outputs.insert((Vec::new(), Vec::new()), name);
        self.shared_outputs.insert((Vec::new(), Vec::new()));
    }
    pub(super) fn with_captures(function:&CheckedFunction,parameters:&[usize],captures:&BTreeMap<(usize,LifetimeSource),String>)->Self {
        let mut plan=Self::default();
        for &root in parameters {
            for loan in function.bindings[root].native_view_loans.as_deref().into_iter().flatten() {
                if !matches!(loan.source,LifetimeSource::Storage) {plan.inputs.entry((root,loan.source.clone())).or_default();}
            }
        }
        for loan in function.native_returns.as_deref().into_iter().flatten().chain(function.native_mutations.as_deref().into_iter().flatten().map(|(_,loan)|loan)) {
            plan.inputs.entry((loan.root,loan.source.clone())).or_default();
        }
        for (destination, loan) in function.native_mutations.as_deref().into_iter().flatten() {
            plan.inputs.entry((*destination, LifetimeSource::carried(loan.path.clone(), loan.guards.clone()))).or_default();
        }
        let mut slots=BTreeMap::<usize,usize>::new();
        for ((root,source),name) in &mut plan.inputs {
            if let Some(captured)=captures.get(&(*root,source.clone())) {*name=captured.clone();continue;}
            let slot=slots.entry(*root).or_default();
            *name=format!("'native_input{root}_{slot}");*slot+=1;
            plan.declarations.push(name.clone());
        }
        plan.plan_outputs(function.native_returns.as_deref().map(Vec::as_slice).unwrap_or(&[]));
        for (destination, loan) in function.native_mutations.as_deref().into_iter().flatten() {
            if function.bindings[*destination].mode != Mode::Mutate { continue; }
            let target = plan.slot(Some(*destination), &loan.path, &loan.guards);
            if let Some(source) = plan.inputs.get(&(loan.root, loan.source.clone()))
                && target != "'_" && source != target {
                let bound = format!("{source}: {target}");
                if !plan.bounds.contains(&bound) { plan.bounds.push(bound); }
            }
        }
        plan
    }

    pub(super) fn with_named_sources(inputs:BTreeMap<(usize,LifetimeSource),String>,returns:&[crate::analysis::native_view_loans::Loan])->Result<Self,crate::Diagnostic> {
        if returns.iter().any(|loan|!inputs.contains_key(&(loan.root,loan.source.clone()))) {
            return Err(crate::Diagnostic::new("native-view-family-emission","A returned family view has no named source lifetime.",crate::Span::default()));
        }
        let mut plan=Self{inputs,..Self::default()};
        plan.plan_outputs(returns);
        Ok(plan)
    }

    fn plan_outputs(&mut self,returns:&[crate::analysis::native_view_loans::Loan]) {
        let mut outputs=BTreeMap::<(Vec<Segment>,Vec<VariantGuard>),Vec<String>>::new();
        for loan in returns {
            if loan.whole { self.shared_outputs.insert((loan.path.clone(), loan.guards.clone())); }
            outputs.entry((loan.path.clone(),loan.guards.clone())).or_default().push(self.inputs[&(loan.root,loan.source.clone())].clone());
        }
        for (path,mut sources) in outputs {
            sources.sort();sources.dedup();
            let name=if sources.len()==1 {sources.remove(0)} else {
                let output=format!("'native_return{}",self.outputs.len());
                self.declarations.push(output.clone());
                self.bounds.extend(sources.into_iter().map(|source|format!("{source}: {output}")));
                output
            };
            self.outputs.insert(path,name);
        }
    }
    pub(super) fn storage(&self,root:usize)->Option<&str> {
        self.inputs.get(&(root,LifetimeSource::Storage)).map(String::as_str)
    }
    pub(super) fn slot(&self,input:Option<usize>,path:&[Segment],guards:&[VariantGuard])->&str {
        match input {
            Some(root)=>self.inputs.get(&(root,LifetimeSource::carried(path.to_vec(),guards.to_vec()))),
            None=>self.outputs.get(&(path.to_vec(),guards.to_vec())).or_else(|| {
                // An extracted field still carries its recursive component's
                // shared lifetime, including all owners joined at that boundary.
                self.shared_outputs.iter()
                    .filter(|(prefix, required)| path.starts_with(prefix) && required.iter().all(|guard| guards.contains(guard)))
                    .max_by_key(|(prefix, _)| prefix.len())
                    .and_then(|boundary| self.outputs.get(boundary))
            }),
        }.map_or("'_",String::as_str)
    }
}
impl Emitter<'_> {
    /// Trait invocation uses the same typed paths as the inherent call. Only
    /// an outer storage reference changes spelling to the existing argument
    /// binder; carried environment/payload slots keep independent lifetimes.
    pub(super) fn native_task_callback_signature(
        &self,function:&CheckedFunction,parameters:&[usize],
        record_borrows:&BTreeMap<usize,usize>,plan:&Lifetimes,replacement_where:&str,
    )->native_function_signature_emit::NativeTaskCallbackSignature {
        let mut arguments=plan.clone();
        let mut substitutions=std::collections::HashMap::<String,String>::new();
        for(position,&root)in parameters.iter().enumerate(){
            if !matches!(function.bindings[root].mode,Mode::Read|Mode::Mutate){continue;}
            let target=format!("p{position}");
            if let Some(source)=plan.storage(root){substitutions.insert(source.trim_start_matches('\'').to_owned(),target.clone());}
            else{let lifetime=format!("'{target}");arguments.inputs.insert((root,LifetimeSource::Storage),lifetime.clone());arguments.declarations.push(lifetime);}
        }
        // Existing record forwarding bounds use these same parameter binders.
        for(&root,&group)in record_borrows{if let Some(position)=parameters.iter().position(|parameter|*parameter==root){substitutions.entry(format!("record_input{group}")).or_insert_with(||format!("p{position}"));}}
        let rename=|text:&str|callback_capture_emit::substitute_parameters(text,&substitutions);
        for name in arguments.inputs.values_mut(){*name=rename(name);}
        for name in arguments.outputs.values_mut(){*name=rename(name);}
        for name in &mut arguments.declarations{*name=rename(name);}
        let mut lifetimes=arguments.declarations.clone();
        lifetimes.extend(record_borrows.values().map(|group|rename(&format!("'record_input{group}"))));lifetimes.sort();lifetimes.dedup();
        let types=parameters.iter().map(|&root|{
            let parameter=ArgumentContract::from(&function.bindings[root]);let storage=arguments.storage(root).unwrap_or("");
            let rendered=if matches!(parameter.ty,Type::Closure(_)){
                let payload=self.native_view_signature_type(parameter.ty,Some(root),&arguments);
                match parameter.mode{Mode::Read=>format!("&{storage} {payload}"),Mode::Mutate=>format!("&{storage} mut {payload}"),_=>payload}
            }else{self.native_view_family_argument_type(parameter,root,&arguments)};
            // Name only the ReadUnion owner's outer reference. Its payload
            // slots have already been rendered from independent typed paths.
            if parameter.mode==Mode::Read && let Type::Union(id)=parameter.ty && !id.copy && self.native_views.contains(parameter.ty){
                let prefix=format!("ViewUnion{}<'_,",id.index());if let Some(payload)=rendered.strip_prefix(&prefix){return format!("ViewUnion{}<{},{}",id.index(),storage,payload);}
            }
            rendered
        }).collect::<Vec<_>>();
        native_function_signature_emit::NativeTaskCallbackSignature{arguments:tuple(types.into_iter()),lifetimes,replacement_where:rename(replacement_where),substitutions}
    }

    pub(super) fn native_view_empty_result(&self, function: &CheckedFunction) -> bool {
        function.native_returns.as_deref().is_none_or(|loans| loans.is_empty())
            && matches!(function.result, Type::Record(record) if self.native_views.recursive(record) || (self.program.records[record].native_element().is_some() && self.native_views.contains(function.result)))
    }
    pub(super) fn native_view_closure_signature(&self,closure:&crate::analysis::CheckedClosure,method:Option<usize>,function:&CheckedFunction,parameters:&[usize])->Option<native_function_signature_emit::NativeSignature> {
        // A zero-argument task adapter owns the same environment as its inner task.
        if method.is_none() && parameters.is_empty()
            && let Some(ClosureForward::Once(inner)) = closure.forward
            && self.program.task_origins.contains(&inner)
        {
            let inner = &self.program.closures[inner];
            return self.native_view_closure_signature(inner, None, &inner.function, &inner.parameters);
        }
        if method.is_none() && let Some(signature)=self.native_forward_borrow_signature(closure) {return Some(signature);}
        if function.native_returns.is_none() && !self.native_views.contains(function.result) && !parameters.iter().any(|&id|self.native_views.contains(function.bindings[id].input_ty.unwrap_or(function.bindings[id].ty)) || self.record_parameter_layout(function,id).is_some()) {return None;}
        let (lifetimes,_)=self.capture_layout(closure);
        let mut next=0;
        let mut captures=BTreeMap::new();
        let mut receiver=false;
        for (position,capture) in closure.captures.iter().enumerate() {
            let root=method.map_or(capture.inner,|method|closure.methods[method].captures[position]);
            if capture.storage==Mode::Read {captures.insert((root,LifetimeSource::Storage),lifetimes[next].clone());}
            else if function.native_returns.as_deref().into_iter().flatten().any(|loan|loan.root==root && loan.source==LifetimeSource::Storage) {
                captures.insert((root,LifetimeSource::Storage),"'native_receiver".into());receiver=true;
            }
            next+=usize::from(matches!(capture.storage,Mode::Read|Mode::Mutate));
            let ty=self.native_captures.ty(&closure.function,capture.inner);
            self.native_view_capture_slots(ty,root,&mut Vec::new(),&mut Vec::new(),&lifetimes,&mut next,&mut captures);
        }
        let lifetimes=Lifetimes::with_captures(function,parameters,&captures);
        let record_bounds=parameters.iter().filter_map(|&id|self.record_parameter_bound_with_lifetimes(function,id,&lifetimes,|_|Vec::new())).collect();
        let mut signature=self.native_signature_from_plan(native_function_signature_emit::NativeSignaturePlan {
            native_capture_lifetimes:captures,
            function,
            parameters:parameters.to_vec(),
            source_parameters:parameters.to_vec(),
            borrow_lifetimes:Default::default(),
            owner_inputs:Vec::new(),
            owner_parameters:String::new(),
            record_bounds,
            replacement_bounds:Vec::new(),
        },&[]);
        if receiver {
            let invocation=method.map_or(closure.invocation,|method|closure.methods[method].invocation);
            signature.receiver=Some(invocation);
            signature.generics=if signature.generics.is_empty() {"<'native_receiver>".into()}else{format!("<'native_receiver, {}",&signature.generics[1..])};
        }
        Some(signature)
    }

    // Native forwarding bodies are described by binding metadata, rather than
    // checked source statements. Name the same source and success slot here so
    // method elision cannot accidentally borrow the callable's empty receiver.
    fn native_forward_borrow_signature(&self,closure:&crate::analysis::CheckedClosure)->Option<native_function_signature_emit::NativeSignature> {
        use crate::native_return_borrows::ReturnSlot;
        let Some(ClosureForward::Native(id))=closure.forward else {return None;};
        let binding=&crate::native_bindings::BINDINGS[id];
        let contract=binding.returned_borrow?;
        binding.validate_returned_borrow().expect("checked native return contract");
        let root=closure.parameters[contract.source_parameter];
        let lifetime=format!("'native_input{root}_0");
        let mut lifetimes=Lifetimes::default();
        lifetimes.inputs.insert((root,LifetimeSource::Storage),lifetime.clone());
        lifetimes.declarations.push(lifetime.clone());
        let mut path=Vec::new();let mut guards=Vec::new();
        if matches!(contract.slot,ReturnSlot::ResultOk|ReturnSlot::ResultOkOptional) {
            let union=closure.function.result.union().expect("checked native Result");
            let [success,_]=crate::result_identity::variants(&self.program.records,&self.program.unions[union.index()].members).expect("checked Result identities");
            guards.push(VariantGuard{path:Vec::new(),success});
            path.push(Segment::Field("payload".into()));
        }
        if matches!(contract.slot,ReturnSlot::DirectOptional|ReturnSlot::ResultOkOptional) {
            let success=self.program.records.iter().position(|record|record.native_type==Some(contract.payload)).expect("checked optional native payload");
            guards.push(VariantGuard{path:path.clone(),success});
        }
        lifetimes.outputs.insert((path,guards),lifetime.clone());
        let params=closure.parameters.iter().map(|&root| {
            let parameter=ArgumentContract::from(&closure.function.bindings[root]);
            format!("value{root}: {}",self.native_view_family_argument_type(parameter,root,&lifetimes))
        }).collect::<Vec<_>>().join(", ");
        let mut generics=lifetimes.declarations.clone();
        generics.extend(self.nullable_callback_bounds(closure.parameters.iter().map(|&root|closure.function.bindings[root].ty).chain(std::iter::once(closure.function.result))));
        Some(native_function_signature_emit::NativeSignature {
            task_callback:None,
            receiver:None,
            storage_lifetimes:BTreeMap::from([(root,lifetime)]),
            params,
            result:self.native_view_signature_type(closure.function.result,None,&lifetimes),
            generics:lifetime_parameters(&generics),
            replacement_where:String::new(),
        })
    }

    fn native_view_capture_slots(&self,ty:Type,root:usize,path:&mut Vec<Segment>,guards:&mut Vec<VariantGuard>,lifetimes:&[String],next:&mut usize,slots:&mut BTreeMap<(usize,LifetimeSource),String>) {
        match ty {
            Type::Closure(id)=>{
                let closure=&self.program.closures[id];
                for capture in &closure.captures {
                    if matches!(capture.storage,Mode::Read|Mode::Mutate){
                        let mut borrow=path.clone();borrow.push(Segment::CaptureBorrow(capture.inner));
                        slots.insert((root,LifetimeSource::carried(borrow,guards.clone())),lifetimes[*next].clone());*next+=1;
                    }
                    path.push(Segment::Field(format!("capture{}",capture.inner)));
                    let child=self.native_captures.ty(&closure.function,capture.inner);
                    self.native_view_capture_slots(child,root,path,guards,lifetimes,next,slots);
                    path.pop();
                }
            }
            Type::Record(id) if native_view_emit::access(self.program,id).is_some() || self.native_views.recursive(id)=>{
                slots.insert((root,LifetimeSource::carried(path.clone(),guards.clone())),lifetimes[*next].clone());*next+=1;
            }
            Type::StoredCallable(family) if self.program.stored_contracts.scoped_native_family(family).is_some()=>{
                // One environment lifetime per carried slot, in alternative order.
                let layout=self.program.stored_contracts.scoped_native_family(family).expect("scoped family");
                for alternative in &layout.alternatives {
                    for slot in &alternative.slots {
                        let mut prefix=path.clone();prefix.push(Segment::CallableAlternative(alternative.origin));prefix.push(Segment::Field(format!("capture{}",slot.capture)));
                        let mut slot_path=prefix.clone();slot_path.extend(slot.path.iter().cloned());
                        let mut slot_guards=guards.clone();
                        slot_guards.extend(slot.guards.iter().map(|guard|{let mut guard=guard.clone();let mut guarded=prefix.clone();guarded.extend(guard.path);guard.path=guarded;guard}));
                        slots.insert((root,LifetimeSource::carried(slot_path,slot_guards)),lifetimes[*next].clone());*next+=1;
                    }
                }
            }
            Type::Record(id) if self.program.records[id].native_element().is_some() => {
                path.push(Segment::Element);
                self.native_view_capture_slots(self.program.records[id].native_element().unwrap(),root,path,guards,lifetimes,next,slots);path.pop();
            }
            Type::Record(id) if self.native_views.contains(ty)=>{
                for field in &self.program.records[id].fields {
                    path.push(Segment::Field(field.name.text.clone()));
                    self.native_view_capture_slots(field.ty.expect("checked field"),root,path,guards,lifetimes,next,slots);path.pop();
                }
            }
            Type::Map(id) if self.native_views.contains(ty)=>{
                path.push(Segment::Element);
                self.native_view_capture_slots(self.program.records[id].map_definition.as_ref().unwrap().value,root,path,guards,lifetimes,next,slots);path.pop();
            }
            Type::Array(element) if self.native_views.contains(ty)=>{
                path.push(Segment::Element);self.native_view_capture_slots(element.ty(),root,path,guards,lifetimes,next,slots);path.pop();
            }
            Type::Union(id)|Type::BorrowedUnion(id) if self.native_views.contains(ty)=>{
                for &member in &self.program.unions[id.index()].members {
                    let variant=if let Type::Record(record)=member {guards.push(VariantGuard{path:path.clone(),success:record});true}else{false};
                    self.native_view_capture_slots(member,root,path,guards,lifetimes,next,slots);
                    if variant {guards.pop();}
                }
            }
            _=>*next+=self.native_captures.type_lifetimes(self.program,&self.borrowed_lifetime_alternatives,ty),
        }
    }

    pub(super) fn native_view_family_argument_type(&self,parameter:ArgumentContract,root:usize,plan:&Lifetimes)->String {
        let storage=plan.storage(root).unwrap_or("");
        if !self.native_views.contains(parameter.ty) {return self.native_argument_type(parameter,storage);}
        if let Type::Union(id)=parameter.ty && !id.copy {
            if parameter.mode==Mode::Read {
                return self.native_view_signature_type(Type::BorrowedUnion(id),Some(root),plan);
            }
            if parameter.mode==Mode::Mutate && (parameter.omission.is_omittable() || self.multiple_data_union(parameter.ty)) {
                let payloads=self.program.unions[id.index()].members.iter().filter(|member|!member.is_copy()).map(|&member|{
                    let mut guards=if let Type::Record(success)=member {vec![VariantGuard{path:vec![],success}]}else{vec![]};
                    let native=self.native_view_signature_slot(member,Some(root),plan,&mut Vec::new(),&mut guards);
                    format!("&{storage} mut {native}")
                }).collect::<Vec<_>>().join(",");
                return format!("Union{}<{payloads}>",id.index());
            }
        }
        let native=self.native_view_signature_type(parameter.ty,Some(root),plan);
        match parameter.mode {Mode::Read=>format!("&{storage} {native}"),Mode::Mutate=>format!("&{storage} mut {native}"),_=>native}
    }

    pub(super) fn native_view_signature_type(&self,ty:Type,input:Option<usize>,plan:&Lifetimes)->String {
        self.native_view_signature_slot(ty,input,plan,&mut Vec::new(),&mut Vec::new())
    }
    pub(super) fn native_view_signature_slot(&self,ty:Type,input:Option<usize>,plan:&Lifetimes,path:&mut Vec<Segment>,guards:&mut Vec<VariantGuard>)->String {
        if let Type::Closure(origin)=ty {
            return self.native_view_environment_type(origin,input,plan,path,guards);
        }
        if let Type::StoredCallable(family)=ty
            && let Some(layout)=self.program.stored_contracts.scoped_native_family(family) {
            let mut names=Vec::new();
            for alternative in &layout.alternatives {
                for slot in &alternative.slots {
                    let mut prefix=path.clone();prefix.push(Segment::CallableAlternative(alternative.origin));prefix.push(Segment::Field(format!("capture{}",slot.capture)));
                    let mut slot_path=prefix.clone();slot_path.extend(slot.path.iter().cloned());
                    let mut slot_guards=guards.clone();
                    slot_guards.extend(slot.guards.iter().map(|guard|{let mut guard=guard.clone();let mut guarded=prefix.clone();guarded.extend(guard.path);guard.path=guarded;guard}));
                    names.push(plan.slot(input,&slot_path,&slot_guards).to_owned());
                }
            }
            return format!("NativeViewCallable{family}<{}>",names.join(", "));
        }
        if !self.native_views.contains(ty) {return self.native_type(ty);}
        match ty {
            Type::Record(id) if native_view_emit::access(self.program,id).is_some() || self.native_views.recursive(id)=>format!("Record{id}<{}>",plan.slot(input,path,guards)),
            Type::Record(id) if self.program.records[id].native_element().is_some() => {
                path.push(Segment::Element);
                let payload=self.native_view_signature_slot(self.program.records[id].native_element().unwrap(),input,plan,path,guards);path.pop();
                format!("Record{}<{payload}>", self.native_record_declaration(id))
            }
            Type::Record(id)=>{
                let base=self.native_record_declaration(id);
                let parameters=self.program.records[id].fields.iter().enumerate().filter_map(|(position,field)|{
                    let ty=field.ty.expect("checked field");
                    if !self.scoped_record_native_field(id,position,ty) {return None;}
                    path.push(Segment::Field(field.name.text.clone()));
                    let result=self.native_view_signature_slot(ty,input,plan,path,guards);path.pop();Some(result)
                }).collect::<Vec<_>>();
                if parameters.is_empty(){self.native_type(ty)}else{format!("Record{base}<{}>",parameters.join(", "))}
            }
            Type::Map(id)=>{
                path.push(Segment::Element);
                let payload=self.native_view_signature_slot(self.program.records[id].map_definition.as_ref().unwrap().value,input,plan,path,guards);path.pop();
                format!("Map{id}<{payload}>")
            }
            Type::Array(element)=>{
                path.push(Segment::Element);
                let payload=self.native_view_signature_slot(element.ty(),input,plan,path,guards);path.pop();
                match element {
                    ArrayElement::Record(id,0)=>{
                        if let Some((_,arguments))=payload.split_once('<'){format!("RecordArray{id}<{arguments}")}else{format!("RecordArray{id}")}
                    }
                    _=>format!("Vec<{payload}>"),
                }
            }
            Type::Union(id)|Type::BorrowedUnion(id)=>{
                let arguments=self.program.unions[id.index()].members.iter().filter(|member|!member.is_copy()).map(|&member|{
                    let variant=if let Type::Record(record)=member {guards.push(VariantGuard{path:path.clone(),success:record});true}else{false};
                    let rendered=self.native_view_signature_slot(member,input,plan,path,guards);
                    if variant {guards.pop();}
                    if matches!(ty,Type::Union(_)){self.union_stored_type(id,member,rendered)}else{rendered}
                }).collect::<Vec<_>>().join(", ");
                if matches!(ty,Type::BorrowedUnion(_)){format!("ViewUnion{}<'_,{arguments}>",id.index())}else{format!("Union{}<{arguments}>",id.index())}
            }
            _=>self.native_type(ty),
        }
    }
}
