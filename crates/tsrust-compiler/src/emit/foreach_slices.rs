//! Read-only traversal can retain a source range instead of cloning its data.
use super::*;
use crate::fixed_storage::{visit_function_expressions,visit_statements};

#[derive(Default)]
pub(super) struct Index {
    bindings:HashMap<usize,HashSet<usize>>,
    callbacks:HashMap<usize,usize>,
    fixed_results:HashMap<usize,(usize,usize)>,
}

fn leaf(mut ty:Type)->Type {
    while let Type::Array(element)=ty {ty=element.ty();}
    ty
}

fn plain_records(program:&CheckedProgram)->Vec<bool> {
    let mut plain=vec![true;program.records.len()];
    let mut parents=vec![Vec::new();program.records.len()];
    let mut work=Vec::new();
    for (id,record) in program.records.iter().enumerate() {
        if record.native_type.is_some(){plain[id]=false;}
        for field in &record.fields {
            let Some(ty)=field.ty else {plain[id]=false;continue;};
            match leaf(ty) {
                Type::Record(child)=>parents[child].push(id),
                Type::Int|Type::Number|Type::Boolean|Type::String|Type::Null|Type::Undefined|Type::Literal(..)|Type::Enum(..)=>{},
                _=>plain[id]=false,
            }
        }
        if !plain[id]{work.push(id);}
    }
    while let Some(id)=work.pop() {
        for &parent in &parents[id] {if plain[parent]{plain[parent]=false;work.push(parent);}}
    }
    plain
}

fn length_parameter(program:&CheckedProgram,id:usize)->Option<usize> {
    let closure=&program.closures[id];
    if closure.polymorphic || closure.forward.is_some() || closure.function.template || closure.program_scoped || closure.program_async || program.task_origins.contains(&id)
        || closure.parameters.len()!=3 || program.task_type(closure.function.result) || !matches!(closure.invocation,Mode::Read|Mode::Mutate){return None;}
    for &id in &closure.parameters {
        let binding=&closure.function.bindings[id];
        if binding.rest || binding.omission.is_omittable() || binding.initializer.is_some() || !matches!(binding.mode,Mode::Read|Mode::Copy){return None;}
    }
    let source=closure.parameters[2];
    if !matches!(closure.function.bindings[source].ty,Type::Array(_)) || receiver_moves::captured(&closure.function,&program.closures)?.contains(&source){return None;}
    let mut names=HashSet::new();let mut lengths=HashSet::new();
    visit_function_expressions(&closure.function,&mut |expr| {
        if matches!(expr.kind,ExprKind::Name(_)) && expr.binding==Some(source){names.insert(expr as *const Expr as usize);}
        if let ExprKind::Field(base,name)=&expr.kind && name=="length" && matches!(base.kind,ExprKind::Name(_)) && base.binding==Some(source){lengths.insert(base.as_ref() as *const Expr as usize);}
    });
    names.is_subset(&lengths).then_some(source)
}

// Discarded traversals need no outer heap container; ordinary calls rebuild it.
fn fixed_result_width(fun:&CheckedFunction)->Option<usize> {
    if crate::analysis::has_exception_syntax(&fun.body){return None;}
    let Type::Array(element)=fun.result else{return None;};
    let Type::Array(inner)=element.ty() else{return None;};
    let bytes=crate::storage::array_storage_bound(inner);
    let mut width=None;let mut valid=true;
    visit_statements(&fun.body,&mut |stmt| {
        let StmtKind::Return(value)=&stmt.kind else{return;};
        let Some(Expr{kind:ExprKind::Array(values),..})=value else{valid=false;return;};
        if values.iter().any(|value|matches!(value.kind,ExprKind::Unary{op:UnaryOp::Spread,..}))
            || values.len().checked_mul(bytes).is_none_or(|bytes|bytes>crate::fixed_storage::MAX_STACK_BYTES)
            || width.is_some_and(|width|width!=values.len()){valid=false;}
        width=Some(values.len());
    });
    valid.then_some(width).flatten()
}

impl Index {
    pub(super) fn new(program:&CheckedProgram,fixed:&crate::fixed_storage::FixedStorage,mut ordinary:impl FnMut(usize)->bool)->Self {
        let mut index=Self::default();let mut plain=None;
        let mut specialized=HashMap::<usize,HashSet<usize>>::new();
        for plan in &fixed.plans {if !plan.bindings.is_empty(){specialized.entry(&program.functions[plan.function] as *const CheckedFunction as usize).or_default().extend(plan.bindings.keys().copied());}}
        let mut callbacks=HashMap::new();let mut widths=HashMap::new();
        for fun in program.functions.iter().chain(program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
            if fun.template || crate::analysis::has_exception_syntax(&fun.body){continue;}
            let readonly=|id:usize| {let b=&fun.bindings[id];!b.parameter && !b.rebindable && !b.mutable && !b.capture_reference && !b.callback_reference && b.borrowed_place.is_none() && b.borrowed_element.is_none() && !b.mutable_place && b.view_roots.is_empty() && b.native_view_loans.is_none()};
            let mut candidates=HashMap::new();
            visit_statements(&fun.body,&mut |stmt| {
                let StmtKind::Let{binding:Some(id),value,..}=&stmt.kind else {return;};
                if !readonly(*id) || value.target!=Some(CallTarget::ArraySlice){return;}
                let Type::Array(element)=fun.bindings[*id].ty else {return;};
                if !match leaf(element.ty()){Type::Record(id)=>plain.get_or_insert_with(||plain_records(program))[id],Type::Int|Type::Number|Type::Boolean|Type::String|Type::Null|Type::Undefined|Type::Literal(..)|Type::Enum(..)=>true,_=>false}{return;}
                let ExprKind::Call(callee,_)=&value.kind else {return;};let ExprKind::Field(base,_)=&callee.kind else {return;};
                if matches!(base.kind,ExprKind::Name(_)) && let Some(source)=base.binding.filter(|&id|readonly(id))
                    && !specialized.get(&(fun as *const CheckedFunction as usize)).is_some_and(|ids|ids.contains(&source)){candidates.insert(*id,source);}
            });
            if candidates.is_empty(){continue;}
            let Some(captured)=receiver_moves::captured(fun,&program.closures) else {continue;};
            candidates.retain(|id,source|!captured.contains(id) && !captured.contains(source));
            if candidates.is_empty(){continue;}
            let sources=candidates.values().copied().collect::<HashSet<_>>();let mut names=HashMap::<usize,HashSet<usize>>::new();
            let mut allowed=HashSet::new();let mut source_reads=HashSet::new();let mut uses=HashMap::<usize,HashSet<usize>>::new();
            visit_function_expressions(fun,&mut |expr| {
                if matches!(expr.kind,ExprKind::Name(_)) && let Some(id)=expr.binding.filter(|id|candidates.contains_key(id)||sources.contains(id)){names.entry(id).or_default().insert(expr as *const Expr as usize);}
                let ExprKind::Call(callee,args)=&expr.kind else {return;};let ExprKind::Field(base,_)=&callee.kind else {return;};
                if !matches!(base.kind,ExprKind::Name(_)){return;}
                if expr.target==Some(CallTarget::ArraySlice){source_reads.insert(base.as_ref() as *const Expr as usize);}
                if expr.target!=Some(CallTarget::ArrayForEach){return;}
                let Some(binding)=base.binding.filter(|id|candidates.contains_key(id)) else {return;};let Some(callback)=args.first() else {return;};
                if callback.traversal_method.is_some(){return;}let Some(Type::Closure(id))=callback.ty else {return;};
                let parameter=*callbacks.entry(id).or_insert_with(||length_parameter(program,id).filter(|_|ordinary(id)));
                if parameter.is_some(){allowed.insert(base.as_ref() as *const Expr as usize);uses.entry(binding).or_default().insert(id);}
            });
            // An exited branch can hide a move in final binding metadata.
            candidates.retain(|id,source|names.get(id).is_some_and(|uses|!uses.is_empty()&&uses.is_subset(&allowed)) && names.get(source).is_some_and(|uses|uses.is_subset(&source_reads)));
            for &id in candidates.keys() {for &callback in &uses[&id]{
                index.callbacks.insert(callback,callbacks[&callback].unwrap());
                let fun=&program.closures[callback].function;
                if let Some(width)=*widths.entry(callback).or_insert_with(||fixed_result_width(fun)){index.fixed_results.insert(fun as *const CheckedFunction as usize,(callback,width));}
            }}
            if !candidates.is_empty(){index.bindings.insert(fun as *const CheckedFunction as usize,candidates.into_keys().collect());}
        }
        index
    }
    pub(super) fn binding(&self,fun:&CheckedFunction,id:usize)->bool {self.bindings.get(&(fun as *const CheckedFunction as usize)).is_some_and(|ids|ids.contains(&id))}
    pub(super) fn callback(&self,id:usize)->Option<usize>{self.callbacks.get(&id).copied()}
    pub(super) fn fixed_result(&self,fun:&CheckedFunction)->Option<(usize,usize)>{self.fixed_results.get(&(fun as *const CheckedFunction as usize)).copied()}
}
