//! Checked error effects are separate from scheduling and native representation.
mod generic_invocation_effects;
use crate::analysis::{CheckedProgram,CheckedFunction,ClosureForward};
use crate::syntax::*;

/// Region-local operation proofs. Unknown or failing duplicates must not be erased.
#[derive(Default)]
pub(crate) struct OperationEffects {
    points:std::collections::HashMap<(usize,usize),Option<bool>>,
}
impl OperationEffects {
    #[inline(never)]
    pub(crate) fn for_body(body:&[Stmt])->Self {
        enum Node<'a>{Statement(&'a Stmt),Expression(&'a Expr)}
        let mut output=Self::default();
        let mut pending=body.iter().map(Node::Statement).collect::<Vec<_>>();
        while let Some(node)=pending.pop() {
            match node {
                Node::Statement(statement)=>{
                    if let Some((span,effect))=statement_operation_effect(statement) {output.insert(span,Some(effect));}
                    else if let StmtKind::Assign{operator:Some((_,span)),..}=&statement.kind {output.insert(*span,None);}
                    match &statement.kind {
                        StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                        StmtKind::Try(region)=>pending.extend(region.statements().map(Node::Statement)),
                        StmtKind::Let{value,..}|StmtKind::Expr(value)|StmtKind::Throw(value)=>pending.push(Node::Expression(value)),
                        StmtKind::Return(value)=>pending.extend(value.iter().map(Node::Expression)),
                        StmtKind::Assign{target,value,..}=>pending.extend([Node::Expression(target),Node::Expression(value)]),
                        StmtKind::Block(body)=>pending.extend(body.iter().map(Node::Statement)),
                        StmtKind::If{condition,then_body,else_body}=>{
                            pending.push(Node::Expression(condition));pending.extend(then_body.iter().chain(else_body).map(Node::Statement));
                        }
                        StmtKind::While{condition,body}=>{pending.push(Node::Expression(condition));pending.extend(body.iter().map(Node::Statement));}
                        StmtKind::ForOf{iterable,body,..}=>{pending.push(Node::Expression(iterable));pending.extend(body.iter().map(Node::Statement));}
                        StmtKind::Loop{condition,body,update,..}=>{
                            pending.extend(condition.iter().map(Node::Expression));pending.extend(body.iter().chain(update).map(Node::Statement));
                        }
                        StmtKind::Break|StmtKind::Continue=>{}
                    }
                }
                Node::Expression(value)=>{
                    let span=match &value.kind {ExprKind::Update{operator,..}|ExprKind::CompoundAssign{operator,..}=>*operator,_=>value.span};
                    output.insert(span,expression_operation_effect(value).map(|(_,effect)|effect));
                    match &value.kind {
                        ExprKind::SpreadCall(plan)=>pending.extend(plan.expressions().map(Node::Expression)),
                        ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value)|ExprKind::Unary{value,..}|ExprKind::Inject{value,..}|ExprKind::Project{value,..}|ExprKind::Update{target:value,..}|ExprKind::Field(value,_)=>pending.push(Node::Expression(value)),
                        ExprKind::Optional{base,body,..}=>pending.extend([Node::Expression(base),Node::Expression(body)]),
                        ExprKind::Index(base,index)|ExprKind::Binary{left:base,right:index,..}=>pending.extend([Node::Expression(base),Node::Expression(index)]),
                        ExprKind::Assign{target,value,..}|ExprKind::CompoundAssign{target,value,..}|ExprKind::CoalesceAssign{target,value,..}|ExprKind::LogicalAssign{target,value,..}=>pending.extend([Node::Expression(target),Node::Expression(value)]),
                        ExprKind::Conditional{predicate,consequent,alternate}=>pending.extend([Node::Expression(predicate),Node::Expression(consequent),Node::Expression(alternate)]),
                        ExprKind::Call(callee,args)=>{pending.push(Node::Expression(callee));pending.extend(args.iter().map(Node::Expression));}
                        ExprKind::Array(values)|ExprKind::ArrayOf(values,_)|ExprKind::SetConstructor(values,_)|ExprKind::MapConstructor(values,_) | ExprKind::NativeConstructor(values,_)|ExprKind::Template(values) | ExprKind::Sequence(values)=>pending.extend(values.iter().map(Node::Expression)),
                        ExprKind::Record(fields)=>pending.extend(fields.iter().map(|(_,value)|Node::Expression(value))),
                        // Source closure bodies execute in their own invocation context.
                        ExprKind::Closure{..}|ExprKind::SpreadValue(_)|ExprKind::Selected{..}|ExprKind::Name(_)|ExprKind::String(_)|ExprKind::Integer(_)|ExprKind::EnumMember { .. } | ExprKind::Number(_)|ExprKind::Boolean(_)|ExprKind::Null|ExprKind::Undefined=>{}
                    }
                }
            }
        }
        output
    }
    pub(crate) fn insert(&mut self,span:crate::Span,effect:Option<bool>) {
        self.points.entry((span.start,span.end)).and_modify(|previous|{
            *previous=match (*previous,effect) {
                (Some(true),_)|(_,Some(true))=>Some(true),
                (None,_)|(_,None)=>None,
                _=>Some(false),
            };
        }).or_insert(effect);
    }
    pub(crate) fn get(&self,span:crate::Span)->Option<bool> {
        self.points.get(&(span.start,span.end)).copied().flatten()
    }
}

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub(crate) struct FailureSet {
    pub explicit:Vec<Type>,
    pub runtime:bool,
    pub opaque:bool,
}
impl FailureSet {
    pub(crate) fn catch_type(&self,unions:&crate::unions::UnionArena,runtime:Option<Type>,span:crate::Span)->Result<Option<Type>,crate::Diagnostic> {
        if self.opaque {
            return Err(crate::Diagnostic::new("error-effect",UNRESOLVED_CATCH,span));
        }
        let mut members=self.explicit.clone();
        if self.runtime {
            let Some(record @ Type::Record(_))=runtime else {
                return Err(crate::Diagnostic::new("error-effect","Runtime error record must be materialized before catch typing.",span));
            };
            members.push(record);
        }
        if members.is_empty() {return Ok(None);}
        unions.intern(members,span).map(Some)
    }
    /// Resolve canonical alternatives for native error transport using catch typing rules.
    pub(crate) fn native_payloads(&self,unions:&crate::unions::UnionArena,runtime:Option<Type>,span:crate::Span)->Result<Vec<Type>,crate::Diagnostic> {
        Ok(match self.catch_type(unions,runtime,span)? {
            None=>vec![],
            Some(Type::Union(id))=>unions.members(id),
            Some(ty)=>vec![ty],
        })
    }
    /// Map each concrete source alternative into a containing destination contract.
    pub(crate) fn native_injection(&self,destination:&Self,unions:&crate::unions::UnionArena,runtime:Option<Type>,span:crate::Span)->Result<Vec<usize>,crate::Diagnostic> {
        let source=self.native_payloads(unions,runtime,span)?;
        let destination=destination.native_payloads(unions,runtime,span)?;
        source.into_iter().map(|payload|destination.iter().position(|candidate|*candidate==payload)
            .ok_or_else(||crate::Diagnostic::new("error-effect","Propagated error payload is absent from the destination contract.",span))).collect()
    }
    fn may_fail(&self)->bool {self.runtime || self.opaque || !self.explicit.is_empty()}
    fn runtime()->Self {Self{runtime:true,..Self::default()}}
    fn unknown()->Self {Self{opaque:true,..Self::default()}}
    fn thrown(ty:Option<Type>)->Self {
        match ty {Some(ty)=>Self{explicit:vec![ty],..Self::default()},None=>Self::unknown()}
    }
    fn join(&mut self,other:Self)->bool {
        let mut changed=false;
        if other.runtime && !self.runtime {self.runtime=true;changed=true;}
        if other.opaque && !self.opaque {self.opaque=true;changed=true;}
        for ty in other.explicit {if !self.explicit.contains(&ty) {self.explicit.push(ty);changed=true;}}
        changed
    }
}
impl std::ops::BitOr for FailureSet {
    type Output=Self;
    fn bitor(mut self,other:Self)->Self {self.join(other);self}
}
impl std::ops::BitOrAssign for FailureSet {
    fn bitor_assign(&mut self,other:Self) {self.join(other);}
}
#[derive(Clone,Default)]
pub(crate) struct ErrorEffects {
    pub functions:Vec<bool>,pub closures:Vec<bool>,
    pub function_errors:Vec<FailureSet>,pub closure_errors:Vec<FailureSet>,
    pub runtime_schema:Option<Record>,
    pub storage_errors:std::collections::HashMap<usize,FailureSet>,
    pub callback_errors:std::collections::HashMap<usize,FailureSet>,
    pub method_errors:std::collections::HashMap<(usize,usize),FailureSet>,
    pub invocation_errors:Vec<FailureSet>,
}

pub(crate) fn runtime_error_schema()->Record {
            use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind,TypeName};
    let span=crate::Span::new(0,0);
    let fields=[("name",Type::String),("code",Type::String),("message",Type::String),("source",Type::String),("start",Type::Int),("end",Type::Int)]
        .into_iter().map(|(name,ty)|RecordField {
            name:Token{text:name.to_owned(),span,kind:TokenKind::Name},
            annotation:ValueAnnotation{kind:ValueAnnotationKind::Named(TypeName{text:if ty==Type::String {"string"}else{"int"}.to_owned(),span}),span},
            ty:Some(ty),callable_declaration:None,optional:false,
        }).collect();
    Record {native_type: None, map_definition:None,set_definition:None,enum_definition:None,application:None,name:Token{text:"#RuntimeError".to_owned(),span,kind:TokenKind::Symbol},type_parameters:vec![],fields}
}

pub(crate) fn materialize_runtime_record(program:&mut CheckedProgram)->Type {
    if let Some(id)=program.records.iter().position(|record|record.name.kind==TokenKind::Symbol && record.name.text=="#RuntimeError") {
        return Type::Record(id);
    }
    let id=program.records.len();
    program.records.push(program.error_effects.runtime_schema.take().unwrap_or_else(runtime_error_schema));
    Type::Record(id)
}

fn checked_statements(program:&mut CheckedProgram,visit:&mut impl FnMut(&mut Stmt)->Result<(),crate::Diagnostic>)->Result<(),crate::Diagnostic> {
    let functions=program.functions.iter_mut().chain(program.closures.iter_mut().flat_map(|closure|std::iter::once(&mut closure.function).chain(closure.methods.iter_mut().map(|method|&mut method.function))));
    for function in functions.filter(|function|!function.template) {
        let mut pending=function.body.iter_mut().collect::<Vec<_>>();
        while let Some(statement)=pending.pop() {
            visit(statement)?;
            match &mut statement.kind {
                StmtKind::Try(region)=>pending.extend(region.statements_mut()),
                StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
                StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);}
                StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);}
                _=>{},
            }
        }
    }
    Ok(())
}

pub(crate) fn materialize_catch_types(program:&mut CheckedProgram)->Result<(),crate::Diagnostic> {
    // A handler whose try body cannot fail never runs. No failure state reached
    // it, so its statements were not checked and there is nothing to emit.
    checked_statements(program,&mut |statement| {
        if let StmtKind::Try(region)=&mut statement.kind
            && let Some(handler)=&mut region.catch
            && handler.error_contract.as_deref().is_some_and(|contract|!contract.runtime && !contract.opaque && contract.explicit.is_empty())
        {
            handler.body.clear();
            handler.binding=None;
        }
        Ok(())
    })?;
    let mut named=false;let mut needs_runtime=false;
    checked_statements(program,&mut |statement| {
        if let StmtKind::Try(region)=&statement.kind {
            if let Some(handler)=&region.catch {
                if let Some(name)=&handler.name {
                    named=true;
                    let contract=handler.error_contract.as_ref().ok_or_else(||crate::Diagnostic::new("error-effect","Catch error contract is missing.",name.span))?;
                    if contract.opaque {return Err(crate::Diagnostic::new("error-effect",UNRESOLVED_CATCH,name.span));}
                    needs_runtime |= contract.runtime;
                }
            }
        }
        Ok(())
    })?;
    if !named {return Ok(());}
    let runtime=needs_runtime.then(||materialize_runtime_record(program));
    let unions=crate::unions::UnionArena::from_snapshot(program.unions.clone());
    checked_statements(program,&mut |statement| {
        if let StmtKind::Try(region)=&mut statement.kind {
            if let Some(handler)=&mut region.catch {
                if let Some(name)=&handler.name {
                    let contract=handler.error_contract.as_ref().expect("validated catch contract");
                    handler.error_type=contract.catch_type(&unions,runtime,name.span)?;
                }
            }
        }
        Ok(())
    })?;
    program.unions=unions.snapshot();
    Ok(())
}

pub(crate) fn annotate(program:&mut CheckedProgram)->ErrorEffects {
    let families=program.stored_contracts.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
    annotate_parts_with_storage(&mut program.functions.iter_mut().collect::<Vec<_>>(),&mut program.closures,&families,&program.polymorphic_invocations)
}
/// Solve finite callback-origin relationships retained by source checking.
/// Unknown source kinds and declarations without callers remain opaque.
#[inline(never)]
// Checked invocation alternatives own their parameter identities; receiver storage
// cannot identify the callback contracts of a replacement method.
fn selected_callback_arguments(callee:&Expr,arguments:&[Expr],closures:&[crate::analysis::CheckedClosure],invocations:&crate::polymorphic_invocation_plans::Registry)->Option<Vec<(usize,Type)>> {
    let id=callee.generic_invocation?;
    let plan=invocations.checked(id,closures,callee.span).expect("validated generic invocation");
    let mut origins=Vec::new();
    for case in plan.cases {
        for (&parameter,argument) in case.body.parameters.iter().zip(arguments) {
            if let Type::Callback(id)=case.body.function.bindings[parameter].ty {
                origins.push((id,argument.ty.unwrap_or(Type::Void)));
            }
        }
    }
    Some(origins)
}

pub(crate) const UNRESOLVED_CATCH:&str="Catch value requires a resolved error contract for this callable. For a callback field of a record parameter, the program must construct a record of that type so the failures of the callback are known.";

/// A callback field of a record declaration receives the concrete callbacks
/// that its constructed instances hold. A function that calls the field of a
/// record parameter fails exactly as those callbacks can.
pub(crate) fn record_field_origins(unions:&crate::unions::UnionArena,records:&[crate::syntax::Record],instances:&crate::scoped_record_layout::Registry)->Vec<(usize,Type)> {
    let mut origins=Vec::new();
    for (instance,record) in records.iter().enumerate() {
        let declaration=instances.declaration(instance);
        if declaration==instance {continue;}
        for (declared,held) in records[declaration].fields.iter().zip(&record.fields) {
            let declared = declared.ty.map(|ty|unions.payload(ty).unwrap_or(ty));
            let actual = held.ty.map(|ty|unions.payload(ty).unwrap_or(ty));
            if let (Some(Type::Callback(id)),Some(actual))=(declared,actual) && actual!=Type::Callback(id) {
                if instances.parameter_contract(instance) {
                    // A fresh parameter variable has the declaration's failure
                    // contract, without becoming a constructed runtime origin.
                    if let Type::Callback(parameter) = actual { origins.push((parameter,Type::Callback(id))); }
                } else { origins.push((id,actual)); }
            }
        }
    }
    origins
}

pub(crate) fn annotate_inferred_callbacks(program:&mut CheckedProgram)->ErrorEffects {
    use std::collections::HashMap;
    let (active_functions,active_closures)=executable_callers(program);
    let mut origins:HashMap<usize,Vec<Type>>=HashMap::new();
    // An absent environment is uninhabited: it cannot execute or throw.
    // Ground only these distinct contracts, never the populated declaration.
    for id in &program.absent_callback_payloads {
        origins.entry(*id).or_default().push(Type::Undefined);
    }
    for (instance,source) in &program.function_specializations {
        if !active_functions.contains(instance) {continue;}
        let Some(source)=program.functions.get(*source) else {continue;};
        let Some(instance)=program.functions.get(*instance) else {continue;};
        for (formal,actual) in source.bindings.iter().take(source.parameter_count).zip(&instance.bindings) {
            if let Type::Callback(id)=formal.ty {origins.entry(id).or_default().push(actual.ty);}
        }
    }
    for (id,closure) in program.closures.iter().enumerate().filter(|(id,_)|active_closures.contains(id)) {
        let _=id;
        for method in &closure.methods {
            for (source,instance) in closure.parameters.iter().zip(&method.parameters) {
                if let Type::Callback(id)=closure.function.bindings[*source].ty {
                    origins.entry(id).or_default().push(method.function.bindings[*instance].ty);
                }
            }
        }
    }
    for function in program.functions.iter().enumerate().filter(|(id,_)|active_functions.contains(id)).map(|(_,function)|function).chain(program.closures.iter().enumerate().filter(|(id,_)|active_closures.contains(id)).flat_map(|(_,closure)|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))) {
        for call in checked_expressions(function) {
            let ExprKind::Call(callee,args)=&call.kind else {continue;};
            if matches!(call.target,Some(CallTarget::ClosureMethod(_))) && let Some(selected)=selected_callback_arguments(callee,args,&program.closures,&program.polymorphic_invocations) {
                for (id,ty) in selected {origins.entry(id).or_default().push(ty);}
                continue;
            }
            let parameters=match call.target {
                Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=>program.functions.get(id).map(|target|target.bindings.iter().take(target.parameter_count).collect::<Vec<_>>()),
                Some(CallTarget::Closure(_)|CallTarget::ConditionalClosure(_))=>{
                    let origin=match call.target {Some(CallTarget::ConditionalClosure(id))=>Some(id),_=>call.closure_origin()};
                    origin.and_then(|id|program.closures.get(id)).map(|closure|closure.parameters.iter().map(|id|&closure.function.bindings[*id]).collect())
                }
                Some(CallTarget::ClosureMethod(method))=>{
                    let origin=match callee.ty {Some(Type::Closure(id))=>Some(id),_=>None};
                    origin.and_then(|id|program.closures.get(id)).and_then(|closure|{
                        let closure=if let Some(ClosureForward::Once(id))=closure.forward {program.closures.get(id)?}else{closure};
                        let method=closure.methods.get(method)?;
                        Some(method.parameters.iter().map(|id|&method.function.bindings[*id]).collect())
                    })
                }
                _=>None,
            };
            let Some(parameters)=parameters else {continue;};
            for (formal,actual) in parameters.into_iter().zip(args) {
                if let Type::Callback(id)=formal.ty {
                    // Missing checked metadata cannot silently omit an origin.
                    origins.entry(id).or_default().push(actual.ty.unwrap_or(Type::Void));
                }
            }
        }
    }
    let unions=crate::unions::UnionArena::from_snapshot(program.unions.clone());
    for (id,actual) in record_field_origins(&unions,&program.records,&program.scoped_records) {
        origins.entry(id).or_default().push(actual);
    }
    for types in origins.values_mut() {
        *types=types.iter().flat_map(|ty|if let Type::Union(id)=ty {unions.members(*id)}else{vec![*ty]}).collect();
    }
    // A callback-only cycle has no closed-world source witness. It must not
    // acquire a pure contract merely because the least fixed point is empty.
    let mut grounded=std::collections::HashSet::new();
    loop {
        let mut changed=false;
        for (id,types) in &origins {
            if types.iter().any(|ty|match ty {Type::Callback(source)=>grounded.contains(source),_=>true}) {
                changed |= grounded.insert(*id);
            }
        }
        if !changed {break;}
    }
    let mut callbacks=origins.keys().map(|id|(*id,if grounded.contains(id){FailureSet::default()}else{FailureSet::unknown()})).collect::<HashMap<_,_>>();
    loop {
        let effects=annotate_with_callbacks(program,&callbacks);
        let mut changed=false;
        for (id,types) in &origins {
            let mut found=FailureSet::default();
            for ty in types {
                found |= match ty {
                    Type::Closure(id)=>effects.closure_errors.get(*id).cloned().unwrap_or_else(FailureSet::unknown),
                    Type::Callback(id)=>effects.callback_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
                    Type::StoredCallable(id)|Type::BorrowedCallable(id)=>effects.storage_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
                    Type::Null|Type::Undefined=>FailureSet::default(),
                    _=>FailureSet::unknown(),
                };
            }
            changed |= callbacks.get_mut(id).expect("initialized callback origin").join(found);
        }
        if !changed {return effects;}
    }
}

#[inline(never)]
pub(crate) fn executable_callers(program:&CheckedProgram)->(std::collections::HashSet<usize>,std::collections::HashSet<usize>) {
    use std::collections::HashSet;
    // A function without callback parameters has one native signature and is
    // emitted whether or not the program calls it. Its calls supply callback
    // origins exactly like calls reached from main.
    let mut functions=program.functions.iter().enumerate().filter_map(|(id,function)|{
        let fixed=!function.template && !function.bindings.iter().take(function.parameter_count).any(|binding|matches!(binding.ty,Type::Callback(_)));
        (function.name=="main" || fixed).then_some(id)
    }).collect::<HashSet<_>>();
    let mut closures:HashSet<usize>=HashSet::new();
    loop {
        let before=(functions.len(),closures.len());
        let bodies=functions.iter().filter_map(|id|program.functions.get(*id)).chain(closures.iter().filter_map(|id|program.closures.get(*id)).flat_map(|closure:&crate::analysis::CheckedClosure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))).collect::<Vec<_>>();
        let mut next_functions=Vec::new();let mut next_closures=Vec::new();
        for function in bodies {
            for value in checked_expressions(function) {
                if let Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=value.target {next_functions.push(id);}
                if let Some(id)=value.closure_origin() {next_closures.push(id);}
                if let Some(Type::Closure(id))=value.ty {next_closures.push(id);}
                if let Some(Type::StoredCallable(id)|Type::BorrowedCallable(id))=value.ty {
                    if let Some((_,_,origins))=program.stored_contracts.families().find(|(family,_,_)|*family==id) {next_closures.extend(origins.iter().copied());}
                }
            }
        }
        for id in &closures {
            match program.closures[*id].forward {
                Some(ClosureForward::Function(id))=>next_functions.push(id),
                Some(ClosureForward::Once(id))=>next_closures.push(id),
                _=>{},
            }
        }
        functions.extend(next_functions);closures.extend(next_closures);
        if before==(functions.len(),closures.len()) {return (functions,closures);}
    }
}
#[inline(never)]
/// Unreferenced unresolved definitions have no concrete error ABI to emit.
/// Retain definitions referenced by emitted callers for normal diagnostics.
pub(crate) fn defer_unreferenced_error_definitions(program:&mut CheckedProgram) {
    let candidates=program.functions.iter().enumerate().filter_map(|(id,function)|{
        let unresolved=program.error_effects.function_errors.get(id).is_some_and(|effects|effects.opaque)
            || function.bindings.iter().take(function.parameter_count).any(|binding|matches!(binding.ty,Type::Callback(callback) if program.error_effects.callback_errors.get(&callback).is_none_or(|effects|effects.opaque)));
        unresolved.then_some(id)
    }).collect::<std::collections::HashSet<_>>();
    let mut required=program.functions.iter().enumerate().filter_map(|(id,function)|(function.name=="main").then_some(id)).collect::<std::collections::HashSet<_>>();
    for (id,function) in program.functions.iter().enumerate() {
        if candidates.contains(&id) || function.template {continue;}
        for expression in checked_expressions(function) {
            if let Some(CallTarget::Function(target)|CallTarget::ForwardFunction(target))=expression.target {required.insert(target);}
        }
    }
    for closure in program.closures.iter().filter(|closure|!closure.function.template) {
        if let Some(ClosureForward::Function(target))=closure.forward {required.insert(target);}
        for function in std::iter::once(&closure.function).chain(closure.methods.iter().filter(|method|!method.function.template).map(|method|&method.function)) {
            for expression in checked_expressions(function) {
                if let Some(CallTarget::Function(target)|CallTarget::ForwardFunction(target))=expression.target {required.insert(target);}
            }
        }
    }
    loop {
        let before=required.len();
        for id in candidates.iter().filter(|id|required.contains(id)).copied().collect::<Vec<_>>() {
            for expression in checked_expressions(&program.functions[id]) {
                if let Some(CallTarget::Function(target)|CallTarget::ForwardFunction(target))=expression.target {required.insert(target);}
            }
        }
        if required.len()==before {break;}
    }
    for id in candidates.difference(&required) {program.functions[*id].template=true;}
}

pub(crate) fn checked_expressions(function:&CheckedFunction)->Vec<&Expr> {
    walk_checked_expressions(&function.body,function.bindings.iter().filter_map(|binding|binding.initializer.as_deref()))
}
pub(crate) fn body_expressions(body:&[Stmt])->Vec<&Expr> {
    walk_checked_expressions(body,std::iter::empty())
}
fn walk_checked_expressions<'a>(body:&'a [Stmt],initializers:impl Iterator<Item=&'a Expr>)->Vec<&'a Expr> {
        enum Node<'n>{Statement(&'n Stmt),Expression(&'n Expr)}
        let mut output=Vec::new();
        let mut pending=body.iter().map(Node::Statement).chain(initializers.map(Node::Expression)).collect::<Vec<_>>();
        while let Some(node)=pending.pop() {
            match node {
                Node::Statement(statement)=>{
                    match &statement.kind {
                        StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                        StmtKind::Try(region)=>pending.extend(region.statements().map(Node::Statement)),
                        StmtKind::Let{value,..}|StmtKind::Expr(value)|StmtKind::Throw(value)=>pending.push(Node::Expression(value)),
                        StmtKind::Return(value)=>pending.extend(value.iter().map(Node::Expression)),
                        StmtKind::Assign{target,value,..}=>pending.extend([Node::Expression(target),Node::Expression(value)]),
                        StmtKind::Block(body)=>pending.extend(body.iter().map(Node::Statement)),
                        StmtKind::If{condition,then_body,else_body}=>{
                            pending.push(Node::Expression(condition));pending.extend(then_body.iter().chain(else_body).map(Node::Statement));
                        }
                        StmtKind::While{condition,body}=>{pending.push(Node::Expression(condition));pending.extend(body.iter().map(Node::Statement));}
                        StmtKind::ForOf{iterable,body,..}=>{pending.push(Node::Expression(iterable));pending.extend(body.iter().map(Node::Statement));}
                        StmtKind::Loop{condition,body,update,..}=>{
                            pending.extend(condition.iter().map(Node::Expression));pending.extend(body.iter().chain(update).map(Node::Statement));
                        }
                        StmtKind::Break|StmtKind::Continue=>{}
                    }
                }
                Node::Expression(value)=>{
                    output.push(value);
                    match &value.kind {
                        ExprKind::SpreadCall(plan)=>pending.extend(plan.expressions().map(Node::Expression)),
                        ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value)|ExprKind::Unary{value,..}|ExprKind::Inject{value,..}|ExprKind::Project{value,..}|ExprKind::Update{target:value,..}|ExprKind::Field(value,_)=>pending.push(Node::Expression(value)),
                        ExprKind::Optional{base,body,..}=>pending.extend([Node::Expression(base),Node::Expression(body)]),
                        ExprKind::Index(base,index)|ExprKind::Binary{left:base,right:index,..}=>pending.extend([Node::Expression(base),Node::Expression(index)]),
                        ExprKind::Assign{target,value,..}|ExprKind::CompoundAssign{target,value,..}|ExprKind::CoalesceAssign{target,value,..}|ExprKind::LogicalAssign{target,value,..}=>pending.extend([Node::Expression(target),Node::Expression(value)]),
                        ExprKind::Conditional{predicate,consequent,alternate}=>pending.extend([Node::Expression(predicate),Node::Expression(consequent),Node::Expression(alternate)]),
                        ExprKind::Call(callee,args)=>{pending.push(Node::Expression(callee));pending.extend(args.iter().map(Node::Expression));}
                        ExprKind::Array(values)|ExprKind::ArrayOf(values,_)|ExprKind::SetConstructor(values,_)|ExprKind::MapConstructor(values,_) | ExprKind::NativeConstructor(values,_)|ExprKind::Template(values) | ExprKind::Sequence(values)=>pending.extend(values.iter().map(Node::Expression)),
                        ExprKind::Record(fields)=>pending.extend(fields.iter().map(|(_,value)|Node::Expression(value))),
                        // Source closure bodies execute in their own invocation context.
                        ExprKind::Closure{..}|ExprKind::SpreadValue(_)|ExprKind::Selected{..}|ExprKind::Name(_)|ExprKind::String(_)|ExprKind::Integer(_)|ExprKind::EnumMember { .. } | ExprKind::Number(_)|ExprKind::Boolean(_)|ExprKind::Null|ExprKind::Undefined=>{}
                    }
                }
            }
        }
    output
}
/// Resolve a checked call graph with callback contracts supplied by specialization.
/// Missing declarations remain opaque; supplied contracts are not inferred from types.
#[inline(never)]
pub(crate) fn annotate_with_callbacks(program:&mut CheckedProgram,callbacks:&std::collections::HashMap<usize,FailureSet>)->ErrorEffects {
    let families=program.stored_contracts.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
    let mut functions=program.functions.iter_mut().map(Some).collect::<Vec<_>>();
    annotate_partial_with_contracts(&mut functions,&mut program.closures,&families,callbacks,&program.polymorphic_invocations)
}
pub(crate) fn annotate_parts_with_storage(functions:&mut [&mut CheckedFunction],closures:&mut [crate::analysis::CheckedClosure],families:&[(usize,Vec<usize>)],invocations:&crate::polymorphic_invocation_plans::Registry)->ErrorEffects {
    annotate_partial_with_storage(&mut functions.iter_mut().map(|function|Some(&mut **function)).collect::<Vec<_>>(),closures,families,invocations)
}

pub(crate) fn annotate_partial_with_storage(functions:&mut [Option<&mut CheckedFunction>],closures:&mut [crate::analysis::CheckedClosure],families:&[(usize,Vec<usize>)],invocations:&crate::polymorphic_invocation_plans::Registry)->ErrorEffects {
    annotate_partial_with_contracts(functions,closures,families,&Default::default(),invocations)
}

/// Include calls in the currently checked region before its callable is stored.
/// Callback-only cycles retain an unknown contract until a concrete origin exists.
#[inline(never)]
pub(crate) fn annotate_partial_with_callers(functions:&mut [Option<&mut CheckedFunction>],closures:&mut [crate::analysis::CheckedClosure],families:&[(usize,Vec<usize>)],caller:&CheckedFunction,unions:&crate::unions::UnionArena,invocations:&crate::polymorphic_invocation_plans::Registry,fields:&[(usize,Type)])->ErrorEffects {
    use std::collections::{HashMap,HashSet};
    let mut origins:HashMap<usize,Vec<Type>>=HashMap::new();
    for &(id,actual) in fields {origins.entry(id).or_default().push(actual);}
    for function in functions.iter().filter_map(|function|function.as_deref()).chain(std::iter::once(caller)).chain(closures.iter().map(|closure|&closure.function)).chain(closures.iter().flat_map(|closure|closure.methods.iter().map(|method|&method.function))) {
        for expression in checked_expressions(function) {
            let ExprKind::Call(callee,arguments)=&expression.kind else {continue;};
            if matches!(expression.target,Some(CallTarget::ClosureMethod(_))) && let Some(selected)=selected_callback_arguments(callee,arguments,closures,invocations) {
                for (id,ty) in selected {origins.entry(id).or_default().extend(if let Type::Union(id)=ty {unions.members(id)}else{vec![ty]});}
                continue;
            }
            let parameters=match expression.target {
                Some(CallTarget::Function(target)|CallTarget::ForwardFunction(target))=>functions.get(target).and_then(Option::as_deref).map(|target|target.bindings.iter().take(target.parameter_count).collect::<Vec<_>>()),
                Some(CallTarget::Closure(_)|CallTarget::ConditionalClosure(_))=>{
                    let origin=match expression.target {Some(CallTarget::ConditionalClosure(id))=>Some(id),_=>expression.closure_origin()};
                    origin.and_then(|id|closures.get(id)).map(|closure|closure.parameters.iter().map(|id|&closure.function.bindings[*id]).collect())
                }
                Some(CallTarget::ClosureMethod(method))=>{
                    let origin=match callee.ty {Some(Type::Closure(id))=>Some(id),_=>None};
                    origin.and_then(|id|closures.get(id)).and_then(|closure|{
                        let closure=if let Some(ClosureForward::Once(id))=closure.forward {closures.get(id)?}else{closure};
                        let method=closure.methods.get(method)?;
                        Some(method.parameters.iter().map(|id|&method.function.bindings[*id]).collect())
                    })
                }
                _=>None,
            };
            let Some(parameters)=parameters else {continue;};
            for (formal,actual) in parameters.into_iter().zip(arguments) {
                if let Type::Callback(id)=formal.ty {
                    let ty=actual.ty.unwrap_or(Type::Void);
                    origins.entry(id).or_default().extend(if let Type::Union(id)=ty {unions.members(id)}else{vec![ty]});
                }
            }
        }
    }
    let mut grounded=HashSet::new();
    loop {
        let mut changed=false;
        for (id,types) in &origins {
            if types.iter().any(|ty|match ty {Type::Callback(id)=>grounded.contains(id),_=>true}) {changed|=grounded.insert(*id);}
        }
        if !changed {break;}
    }
    let mut callbacks=origins.keys().map(|id|(*id,if grounded.contains(id){FailureSet::default()}else{FailureSet::unknown()})).collect::<HashMap<_,_>>();
    loop {
        let effects=annotate_partial_with_contracts(functions,closures,families,&callbacks,invocations);
        let mut changed=false;
        for (id,types) in &origins {
            let mut found=FailureSet::default();
            for ty in types {
                found|=match ty {
                    Type::Closure(id)=>effects.closure_errors.get(*id).cloned().unwrap_or_else(FailureSet::unknown),
                    Type::Callback(id)=>effects.callback_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
                    Type::StoredCallable(id)|Type::BorrowedCallable(id)=>effects.storage_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
                    Type::Null|Type::Undefined=>FailureSet::default(),
                    _=>FailureSet::unknown(),
                };
            }
            changed|=callbacks.get_mut(id).expect("callback origin").join(found);
        }
        if !changed {return effects;}
    }
}
#[inline(never)]
fn annotate_partial_with_contracts(functions:&mut [Option<&mut CheckedFunction>],closures:&mut [crate::analysis::CheckedClosure],families:&[(usize,Vec<usize>)],callbacks:&std::collections::HashMap<usize,FailureSet>,invocations:&crate::polymorphic_invocation_plans::Registry)->ErrorEffects {
    let mut effects=ErrorEffects{invocation_errors:vec![FailureSet::default();invocations.entries().count()],method_errors:closures.iter().enumerate().flat_map(|(id,closure)|closure.methods.iter().enumerate().map(move |(method,_)|((id,method),FailureSet::default()))).collect(),callback_errors:callbacks.clone(),functions:vec![false;functions.len()],closures:vec![false;closures.len()],function_errors:vec![FailureSet::default();functions.len()],closure_errors:vec![FailureSet::default();closures.len()],runtime_schema:Some(runtime_error_schema()),storage_errors:families.iter().map(|(id,origins)|(*id,if origins.is_empty(){FailureSet::unknown()}else{FailureSet::default()})).collect()};
    for (id,function) in functions.iter().enumerate() {
        if function.is_none() {effects.function_errors[id]=FailureSet::unknown();effects.functions[id]=true;}
    }
    // Payload types are checked identities. Every finite coordinate grows
    // monotonically; the final unchanged pass refreshes expression metadata.
    loop {
        let mut changed=false;
        for (id,function) in functions.iter_mut().enumerate() {
            let Some(function)=function else {continue;};
            let next=function_effect(function,&effects);
            changed |= effects.function_errors[id].join(next);
            effects.functions[id]=effects.function_errors[id].may_fail();
        }
        for (id,closure) in closures.iter_mut().enumerate() {
            let mut next=function_effect(&mut closure.function,&effects);
            for (method_id,method) in closure.methods.iter_mut().enumerate() {
                let found=function_effect(&mut method.function,&effects);
                changed|=effects.method_errors.get_mut(&(id,method_id)).expect("method effects").join(found.clone());
                next |= found;
            }
            next |= match closure.forward {
                Some(ClosureForward::Function(origin))=>effects.function_errors.get(origin).cloned().unwrap_or_else(FailureSet::unknown),
                Some(ClosureForward::Once(origin))=>effects.closure_errors.get(origin).cloned().unwrap_or_else(FailureSet::unknown),
                Some(ClosureForward::Abstract(contract))=>effects.callback_errors.get(&contract).cloned().unwrap_or_else(FailureSet::unknown),
                Some(ClosureForward::RestAdapter{..})=>match closure.function.bindings.first().map(|binding|binding.ty) {
                    Some(Type::Closure(id))=>effects.closure_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
                    Some(Type::Callback(id))=>effects.callback_errors.get(&id).cloned().unwrap_or_else(FailureSet::unknown),
                    Some(Type::StoredCallable(id)|Type::BorrowedCallable(id))=>effects.storage_errors.get(&id).cloned().unwrap_or_else(FailureSet::unknown),
                    _=>FailureSet::unknown(),
                },
                Some(ClosureForward::Native(id))=>native_binding_effect(id),
                None=>FailureSet::default(),
            };
            changed |= effects.closure_errors[id].join(next);
            effects.closures[id]=effects.closure_errors[id].may_fail();
        }
        for (id,origins) in families {
            let mut next=if origins.is_empty(){FailureSet::unknown()}else{FailureSet::default()};
            for origin in origins {next |= effects.closure_errors.get(*origin).cloned().unwrap_or_else(FailureSet::unknown);}
            changed |= effects.storage_errors.get_mut(id).expect("initialized family effects").join(next);
        }
        // A consuming wrapper calls the selected method on its inner environment.
        // Preserve that identity for effect lookup without merging other methods.
        for (id,closure) in closures.iter().enumerate() {
            if let Some(ClosureForward::Once(inner))=closure.forward {
                let methods=effects.method_errors.iter().filter_map(|((origin,method),contract)|(*origin==inner).then_some((*method,contract.clone()))).collect::<Vec<_>>();
                for (method,contract) in methods {
                    changed|=effects.method_errors.entry((id,method)).or_default().join(contract);
                }
            }
        }
        changed |= effects.refresh_invocation_errors(invocations);
        if !changed {break;}
    }
    effects
}
fn function_effect(function:&mut CheckedFunction,effects:&ErrorEffects)->FailureSet {
    let mut failure=FailureSet::default();
    for binding in &mut function.bindings {
        if let Some(value)=&mut binding.initializer {failure |= expression_effect(value,effects,0);}
    }
    failure | body_effect(&mut function.body,effects,0)
}
#[inline(never)]
pub(crate) fn annotate_region_body(body:&mut [Stmt],effects:&ErrorEffects)->FailureSet {body_effect(body,effects,0)}

#[inline(never)]
fn body_effect(body:&mut [Stmt],effects:&ErrorEffects,depth:usize)->FailureSet {
    if depth>=128 {return FailureSet::unknown();}
    let mut failure=FailureSet::default();
    for statement in body {
        let operation_failure=statement_operation_effect(statement).is_some_and(|(_,fails)|fails);
        failure |= match &mut statement.kind {
            StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
            StmtKind::Throw(value)=>expression_effect(value,effects,depth+1)|FailureSet::thrown(value.ty),
            StmtKind::Try(region)=>{
                let mut found=body_effect(&mut region.body,effects,depth+1);
                // The body must still be annotated at each failure point for
                // catch-entry state. Its errors leave this region only when
                // there is no handler; errors in the handler escape normally.
                if let Some(handler)=&mut region.catch {
                    let reachable=found.may_fail();
                    handler.error_contract=Some(Box::new(found));
                    let handled=body_effect(&mut handler.body,effects,depth+1);
                    found=if reachable {handled}else{FailureSet::default()};
                }
                if let Some(body)=&mut region.finally {
                    let cleanup=body_effect(body,effects,depth+1);
                    if crate::completion::definitely_abrupt(body,0) {found=cleanup;}else{found |= cleanup;}
                }
                found
            }
            StmtKind::Let{value,..}|StmtKind::Expr(value)=>expression_effect(value,effects,depth+1),
            StmtKind::Return(value)=>value.as_mut().map(|value|expression_effect(value,effects,depth+1)).unwrap_or_default(),
            StmtKind::Assign{target,value,..}=>{
                (if operation_failure {FailureSet::runtime()}else{FailureSet::default()}) | expression_effect(target,effects,depth+1) | expression_effect(value,effects,depth+1)
            }
            StmtKind::Block(body)=>body_effect(body,effects,depth+1),
            StmtKind::If{condition,then_body,else_body}=>expression_effect(condition,effects,depth+1)|body_effect(then_body,effects,depth+1)|body_effect(else_body,effects,depth+1),
            StmtKind::While{condition,body}=>expression_effect(condition,effects,depth+1)|body_effect(body,effects,depth+1),
            StmtKind::ForOf{iterable,body,..}=>expression_effect(iterable,effects,depth+1)|body_effect(body,effects,depth+1),
            StmtKind::Loop{condition,body,update,..}=>{
                let mut found=condition.as_mut().map(|value|expression_effect(value,effects,depth+1)).unwrap_or_default();
                found |= body_effect(body,effects,depth+1);found | body_effect(update,effects,depth+1)
            }
            StmtKind::Break|StmtKind::Continue=>FailureSet::default(),
        };
    }
    failure
}
/// Map only annotated own effects, using mutation operator spans when applicable.
pub(crate) fn expression_operation_effect(value:&Expr)->Option<(crate::Span,bool)> {
    if !value.effects_checked {return None;}
    Some(mutation_operation_effect(value).unwrap_or((value.span,value.own_may_fail)))
}

/// Own update or compound-expression failure at its recorded operator span.
pub(crate) fn mutation_operation_effect(value:&Expr)->Option<(crate::Span,bool)> {
    let ty=value.ty?;
    match &value.kind {
        ExprKind::Update{operator,..}=>Some((*operator,ty==Type::Int)),
        ExprKind::CompoundAssign{operator,op,..}=>Some((*operator,ty==Type::Int && integer_operation(*op))),
        _=>None,
    }
}

/// Own compound-assignment failure, at its recorded operator span; unknown types stay unmapped.
pub(crate) fn statement_operation_effect(statement:&Stmt)->Option<(crate::Span,bool)> {
    let StmtKind::Assign{target,operator:Some((op,span)),..}=&statement.kind else {return None;};
    let ty=target.ty?;
    Some((*span,ty==Type::Int && integer_operation(*op)))
}

/// Failure before a statement finishes its assignment or chooses its branch.
pub(crate) fn statement_entry_may_fail(statement:&Stmt)->bool {
    match &statement.kind {
        StmtKind::Let{value,..}|StmtKind::Expr(value)=>value.may_fail,
        StmtKind::Return(value)=>value.as_ref().is_some_and(|value|value.may_fail),
        StmtKind::Assign{target,value,..}=>target.may_fail|value.may_fail|statement_operation_effect(statement).is_some_and(|(_,fails)|fails),
        StmtKind::If{condition,..}|StmtKind::While{condition,..}=>condition.may_fail,
        StmtKind::ForOf{iterable,..}=>iterable.may_fail,
        StmtKind::Loop{condition,post_test:false,..}=>condition.as_ref().is_some_and(|value|value.may_fail),
        _=>false,
    }
}
pub(crate) fn integer_operation(op:BinaryOp)->bool {
    matches!(op,BinaryOp::Add|BinaryOp::Subtract|BinaryOp::Multiply|BinaryOp::Divide|BinaryOp::Remainder|BinaryOp::Exponent|BinaryOp::ShiftLeft|BinaryOp::ShiftRight|BinaryOp::ShiftRightUnsigned)
}
/// Built-in evaluation and callback invocation are distinct error sources.
/// Unrefined built-ins conservatively retain a runtime-error category; an
/// unknown callable contract retains an opaque category instead.
/// Native APIs need an explicit audited contract; unresolved APIs stay opaque.
fn native_binding_effect(id:usize)->FailureSet {
    match crate::native_bindings::BINDINGS.get(id) {
        // Schema construction and validation return data; neither invokes user callbacks.
        Some(binding) if binding.module=="std:schema" && matches!(binding.name,"__string"|"__boolean"|"__safeParseString"|"__safeParseOwnedString"|"__safeParseBoolean"|"__safeParseBooleanValue"|"__int"|"__safeParseInt"|"__safeParseIntValue"|"__number"|"__safeParseNumber"|"__safeParseNumberValue"|"__safeParseNumberInt"|"__null"|"__safeParseNull"|"__safeParseNullValue"|"__minString"|"__maxString"|"__minInt"|"__maxInt")=>FailureSet::default(),
        // Byte operations carry validation/reservation failures through Result.
        // Length observes checked storage, and a view borrows a validated range.
        // These bindings neither throw nor call user code.
        Some(binding) if binding.module=="std:json" && matches!(binding.name,"borrowRaw"|"parse"|"parseJsonc"|"stringify"|"kind"|"fromArray"|"fromEntries"|"nullValue"|"fromBoolean"|"fromInt"|"fromNumber"|"fromString"|"asString"|"asBoolean"|"asInt"|"asNumber"|"takeString"|"get"|"at"|"openJsonl"|"borrowJsonl"|"readJsonl"|"closeJsonl"|"createJsonl"|"borrowJsonlWriter"|"writeJsonl"|"flushJsonl"|"closeJsonlWriter")=>FailureSet::default(),
        Some(binding) if binding.module=="std:env" && binding.name=="get"=>FailureSet::default(),
        Some(binding) if binding.module=="std:bytes" && matches!(binding.name,"fromArray"|"length"|"at"|"set"|"alloc"|"resize"|"append"|"copy"|"fromUtf8"|"toUtf8"|"view"|"viewMut")=>FailureSet::default(),
        Some(binding) if binding.module=="std:fs" && matches!(binding.name,"open"|"read"|"close"|"create"|"write"|"writeAll"|"readFile"|"writeFile"|"bufferReader"|"bufferWriter"|"flush")=>FailureSet::default(),
        // Sharing does not invoke the environment. Observations read checked values,
        // returning NaN/false for unsupported numeric/length shapes.
        // Allocation failures are outside recoverable checked error transport.
        Some(binding) if binding.module=="std:testing-runtime" && matches!(binding.name,"shareCallback"|"assertionDetails"|"numericValue"|"lengthValue"|"hasLength")=>FailureSet::default(),
        // Target constants have no recoverable runtime failures.
        Some(binding) if binding.module=="std:os" && matches!(binding.name,"platform"|"arch"|"endianness")=>FailureSet::default(),
        Some(binding) if binding.module=="std:path" && binding.name=="relative"=>FailureSet::runtime(),
        // Lexical UTF-8 processing only: no I/O or recoverable runtime failure.
        Some(binding) if binding.module=="std:path" && matches!(binding.name,"basename"|"dirname"|"extname"|"isAbsolute"|"normalize"|"join")=>FailureSet::default(),
        Some(binding) if binding.module=="std:testing-runtime" && binding.name=="checkAssertion"=>FailureSet::runtime(),
        Some(binding) if binding.module=="std:testing-runtime" && matches!(binding.name,"runTests"|"finishTests")=>FailureSet::runtime(),
        Some(binding) if binding.module=="std:testing-runtime" && binding.name=="registerTest"=>FailureSet::runtime(),
        Some(binding) if binding.module=="std:testing-runtime" && matches!(binding.name,"registerHook"|"beginSuite"|"beginFile"|"endSuite")=>FailureSet::runtime(),
        _=>FailureSet::unknown(),
    }
}
fn native_effect(target:CallTarget,value:&Expr,effects:&ErrorEffects)->FailureSet {
    use CallTarget::*;
    match target {
        Callback(_)=>value.callback_contract().and_then(|id|effects.callback_errors.get(&id).cloned()).unwrap_or_else(FailureSet::unknown),
        ConditionalCallback(id)=>effects.callback_errors.get(&id).cloned().unwrap_or_else(FailureSet::unknown),
        NativeBinding(id)=>native_binding_effect(id),
        ClosureMethod(method)=>{
            if let ExprKind::Call(callee,_)=&value.kind && let Some(id)=callee.generic_invocation {
                return effects.invocation_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown);
            }
            let origin=match &value.kind {ExprKind::Call(callee,_)=>match callee.ty {Some(Type::Closure(id))=>Some(id),_=>None},_=>None};
            origin.and_then(|id|effects.method_errors.get(&(id,method)).cloned()).unwrap_or_else(FailureSet::unknown)
        }
        StoredCallable(_)|StoredCallableField(_)=>FailureSet::unknown(),
        MathSqrt|MathFloor|MathCeil|MathTrunc|MathRound|MathSign|MathFround|MathF16round|
        NumberIsFinite|NumberIsNaN|NumberIsInteger|NumberIsSafeInteger|
        JsonEncode|JsonDecode|ArrayIsArray|ObjectIs|BooleanConversion|JsonBoolean|JsonStringLength|JsonStringView=>FailureSet::default(),
        ArrayMap|MapGroupBy|ArrayFlatMap|ArrayFilter|ArrayForEach|ArraySome|ArrayEvery|
        ArrayFind|ArrayFindLast|ArrayFindIndex|ArrayFindLastIndex|ArrayReduce|
        ArrayReduceRight|ArraySort|ArrayToSorted=>{
            let callback=match &value.kind {ExprKind::Call(_,args)=>args.first(),_=>None};
            let invoked=match callback.and_then(|value|value.ty) {
                Some(Type::Closure(id))=>effects.closure_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
                Some(Type::Callback(id))=>effects.callback_errors.get(&id).cloned().unwrap_or_else(FailureSet::unknown),
                Some(Type::StoredCallable(id)|Type::BorrowedCallable(id))=>effects.storage_errors.get(&id).cloned().unwrap_or_else(FailureSet::unknown),
                None if matches!(target,ArraySort|ArrayToSorted)=>FailureSet::default(),
                _=>FailureSet::unknown(),
            };
            FailureSet::runtime()|invoked
        }
        _=>FailureSet::runtime(),
    }
}
#[inline(never)]
fn expression_effect(value:&mut Expr,effects:&ErrorEffects,depth:usize)->FailureSet {
    if depth>=128 {value.may_fail=true;value.own_may_fail=true;value.effects_checked=true;return FailureSet::unknown();}
    let runtime=match &value.kind {
        ExprKind::SpreadCall(plan)=>plan.segments.iter().any(|segment|segment.length.is_none()) && (plan.minimum>0 || plan.maximum.is_some()),
        ExprKind::Index(_,_)=>true,
        ExprKind::Unary{op:UnaryOp::NonNull,..}=>true,
        ExprKind::Update{..}|ExprKind::CompoundAssign{..}=>mutation_operation_effect(value).is_some_and(|(_,fails)|fails),
        ExprKind::Unary{op:UnaryOp::Negate,..}=>value.ty==Some(Type::Int),
        ExprKind::Binary{op,..}=>value.ty==Some(Type::Int) && integer_operation(*op),
        _=>false,
    };
    let own=if runtime {FailureSet::runtime()}else {
        match value.target {
            Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=>effects.function_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
            Some(CallTarget::Closure(_))=>value.closure_origin().and_then(|origin|effects.closure_errors.get(origin).cloned()).unwrap_or_else(FailureSet::unknown),
            Some(CallTarget::ConditionalClosure(id))=>effects.closure_errors.get(id).cloned().unwrap_or_else(FailureSet::unknown),
            Some(CallTarget::StoredCallable(_))=>value.storage_family().and_then(|family|effects.storage_errors.get(&family).cloned()).unwrap_or_else(FailureSet::unknown),
            Some(CallTarget::StoredCallableField(family))=>effects.storage_errors.get(&family).cloned().unwrap_or_else(FailureSet::unknown),
            Some(target)=>native_effect(target,value,effects),
            None=>FailureSet::default(),
        }
    };
    value.own_may_fail=own.may_fail();
    value.effects_checked=true;
    let next=depth+1;
    let children=match &mut value.kind {
        ExprKind::SpreadCall(plan)=>plan.expressions_mut().fold(FailureSet::default(),|found,value|expression_effect(value,effects,next)|found),
        ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value)|ExprKind::Unary{value,..}|ExprKind::Inject{value,..}|ExprKind::Project{value,..}|ExprKind::Update{target:value,..}|ExprKind::Field(value,_)=>expression_effect(value,effects,next),
        ExprKind::Optional{base,body,branch,..}=>{
            let found=expression_effect(base,effects,next);
            if *branch==NullishBranch::Absent {found}else{found|expression_effect(body,effects,next)}
        }
        ExprKind::Index(base,index)|ExprKind::Binary{left:base,right:index,..}=>expression_effect(base,effects,next)|expression_effect(index,effects,next),
        ExprKind::Assign{target,value,..}|ExprKind::CompoundAssign{target,value,..}|ExprKind::CoalesceAssign{target,value,..}|ExprKind::LogicalAssign{target,value,..}=>expression_effect(target,effects,next)|expression_effect(value,effects,next),
        ExprKind::Conditional{predicate,consequent,alternate}=>expression_effect(predicate,effects,next)|expression_effect(consequent,effects,next)|expression_effect(alternate,effects,next),
        ExprKind::Call(callee,args)=>{
            let mut found=expression_effect(callee,effects,next);
            for value in args {found |= expression_effect(value,effects,next);}found
        }
        ExprKind::Array(values)|ExprKind::ArrayOf(values,_)|ExprKind::SetConstructor(values,_)|ExprKind::MapConstructor(values,_) | ExprKind::NativeConstructor(values,_)|ExprKind::Template(values) | ExprKind::Sequence(values)=>values.iter_mut().fold(FailureSet::default(),|found,value|expression_effect(value,effects,next)|found),
        ExprKind::Record(fields)=>fields.iter_mut().fold(FailureSet::default(),|found,(_,value)|expression_effect(value,effects,next)|found),
        // An environment is created here; its body runs only on invocation.
        ExprKind::Closure{..}|ExprKind::SpreadValue(_)|ExprKind::Selected{..}|ExprKind::Name(_)|ExprKind::String(_)|ExprKind::Integer(_)|ExprKind::EnumMember { .. } | ExprKind::Number(_)|ExprKind::Boolean(_)|ExprKind::Null|ExprKind::Undefined=>FailureSet::default(),
    };
    let failures=own|children;value.may_fail=failures.may_fail();failures
}

#[cfg(test)]
mod tests {
    #[test]
    fn deferred_error_definitions_preserve_live_callees() {
        let mut checked=crate::check("function used():void{}function unused():void{}function relay():void{used();}function main(){used();}").unwrap();
        let used=checked.program.functions.iter().position(|function|function.name=="used").unwrap();
        let unused=checked.program.functions.iter().position(|function|function.name=="unused").unwrap();
        let relay=checked.program.functions.iter().position(|function|function.name=="relay").unwrap();
        for id in [used,unused,relay] {checked.program.error_effects.function_errors[id]=super::FailureSet::unknown();}
        super::defer_unreferenced_error_definitions(&mut checked.program);
        assert!(!checked.program.functions[used].template,"live calls must retain unresolved definitions for diagnostics");
        assert!(checked.program.functions[unused].template);
        assert!(checked.program.functions[relay].template);
        assert!(checked.program.error_effects.function_errors[used].opaque);
        assert!(checked.program.error_effects.function_errors[unused].opaque);
    }
    #[test]
    fn relative_native_binding_has_checked_runtime_failure_contract() {
        let id=crate::native_bindings::BINDINGS.iter().position(|binding|binding.module=="std:path" && binding.name=="relative").unwrap();
        let effect=super::native_binding_effect(id);
        assert!(effect.runtime);
        assert!(!effect.opaque);
        assert!(effect.explicit.is_empty());
    }
    #[test]
    fn lexical_native_path_origins_have_resolved_pure_contracts() {
        let mut checked=crate::check("function main(){const action=(path:string):string=>\"unused\";}").unwrap();
        let native=crate::native_bindings::BINDINGS.iter().position(|binding|binding.module=="std:path" && binding.name=="basename").unwrap();
        checked.program.closures[0].forward=Some(crate::analysis::ClosureForward::Native(native));
        let crate::syntax::StmtKind::Return(Some(value))=&checked.program.closures[0].function.body[0].kind else {panic!("arrow result");};
        let value=value.clone();
        checked.program.closures[0].function.body.clear();
        let effects=super::annotate(&mut checked.program);
        assert!(!effects.closure_errors[0].opaque,"lexical basename has no unresolved invocation");
        assert!(!effects.closures[0]);
        assert_eq!(super::native_effect(crate::syntax::CallTarget::NativeBinding(native),&value,&effects),super::FailureSet::default());
        for (id,binding) in crate::native_bindings::BINDINGS.iter().enumerate() {
            // Audited testing-runtime and target identity bindings have explicit contracts; every other binding stays unresolved.
            let audited=(binding.module=="std:json" && matches!(binding.name,"borrowRaw"|"parse"|"parseJsonc"|"stringify"|"kind"|"fromArray"|"fromEntries"|"nullValue"|"fromBoolean"|"fromInt"|"fromNumber"|"fromString"|"asString"|"asBoolean"|"asInt"|"asNumber"|"takeString"|"get"|"at"|"openJsonl"|"borrowJsonl"|"readJsonl"|"closeJsonl"|"createJsonl"|"borrowJsonlWriter"|"writeJsonl"|"flushJsonl"|"closeJsonlWriter")) || (binding.module=="std:schema" && matches!(binding.name,"__string"|"__boolean"|"__safeParseString"|"__safeParseOwnedString"|"__safeParseBoolean"|"__safeParseBooleanValue"|"__int"|"__safeParseInt"|"__safeParseIntValue"|"__number"|"__safeParseNumber"|"__safeParseNumberValue"|"__safeParseNumberInt"|"__null"|"__safeParseNull"|"__safeParseNullValue"|"__minString"|"__maxString"|"__minInt"|"__maxInt")) || (binding.module=="std:env" && binding.name=="get") || (binding.module=="std:fs" && matches!(binding.name,"open"|"read"|"close"|"create"|"write"|"writeAll"|"readFile"|"writeFile"|"bufferReader"|"bufferWriter"|"flush")) || (binding.module=="std:bytes" && matches!(binding.name,"fromArray"|"length"|"at"|"set"|"alloc"|"resize"|"append"|"copy"|"fromUtf8"|"toUtf8"|"view"|"viewMut")) || binding.module=="std:path" || (binding.module=="std:os" && matches!(binding.name,"platform"|"arch"|"endianness")) || (binding.module=="std:testing-runtime" && matches!(binding.name,"checkAssertion"|"shareCallback"|"assertionDetails"|"numericValue"|"lengthValue"|"hasLength"|"runTests"|"finishTests"|"registerTest"|"registerHook"|"beginSuite"|"beginFile"|"endSuite"));
            if binding.module=="std:os" {assert_eq!(super::native_binding_effect(id),super::FailureSet::default());}
            if !audited {assert!(super::native_binding_effect(id).opaque,"unreviewed binding {} must remain unresolved",binding.name);}
        }
        checked.program.closures[0].forward=Some(crate::analysis::ClosureForward::Native(usize::MAX));
        assert!(super::annotate(&mut checked.program).closure_errors[0].opaque);
    }
    #[test]
    fn native_error_injection_maps_broader_contract_by_payload_identity() {
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span::new(7,13);
        let source=FailureSet{explicit:vec![Type::String],runtime:true,opaque:false};
        let destination=FailureSet{explicit:vec![Type::Boolean,Type::Int,Type::String,Type::Record(0)],runtime:true,opaque:false};
        assert_eq!(source.native_injection(&destination,&unions,Some(Type::Record(0)),span).unwrap(),vec![2,3]);
        assert_eq!(source.native_injection(&source,&unions,Some(Type::Record(0)),span).unwrap(),vec![0,1]);
        let missing=FailureSet{explicit:vec![Type::String],..Default::default()};
        let error=source.native_injection(&missing,&unions,Some(Type::Record(0)),span).unwrap_err();
        assert_eq!(error.code,"error-effect");
        assert_eq!(error.span,span);
    }
    #[test]
    fn native_error_payload_layout_flattens_and_deduplicates_runtime_record() {
        let unions=crate::unions::UnionArena::default();
        let span=crate::Span::new(7,13);
        let union=unions.intern([Type::String,Type::Int],span).unwrap();
        let failures=FailureSet{explicit:vec![union,Type::Record(0),Type::Int],runtime:true,opaque:false};
        let payloads=failures.native_payloads(&unions,Some(Type::Record(0)),span).unwrap();
        assert_eq!(payloads,vec![Type::Int,Type::Record(0),Type::String]);
        let reordered=FailureSet{explicit:vec![Type::Record(0),Type::Int,union],runtime:true,opaque:false};
        assert_eq!(reordered.native_payloads(&unions,Some(Type::Record(0)),span).unwrap(),payloads);
        assert!(FailureSet::default().native_payloads(&unions,None,span).unwrap().is_empty());
        assert_eq!(FailureSet{opaque:true,..Default::default()}.native_payloads(&unions,None,span).unwrap_err().code,"error-effect");
        assert_eq!(FailureSet{runtime:true,..Default::default()}.native_payloads(&unions,None,span).unwrap_err().span,span);
    }
    #[test]
    fn region_operation_map_visits_branches_and_nested_assignment_operands() {
        let program=crate::check("function main(){let values=[1];let n:int=0;if(true){n += values[0];}else{n += 2;}}").unwrap().program;
        let main=program.functions.iter().find(|function|function.name=="main").unwrap();
        let points=OperationEffects::for_body(&main.body);
        let StmtKind::If{then_body,else_body,..}=&main.body[2].kind else {panic!("branches");};
        for body in [then_body,else_body] {
            let StmtKind::Assign{operator:Some((_,span)),..}=&body[0].kind else {panic!("assignment");};
            assert_eq!(points.get(*span),Some(true));
        }
        let StmtKind::Assign{value,..}=&then_body[0].kind else {panic!("assignment");};
        let ExprKind::Index(base,_) =&value.kind else {panic!("bounds");};
        assert_eq!(points.get(value.span),Some(true));
        assert_eq!(points.get(base.span),Some(false));
    }
    #[test]
    fn synthetic_wrapper_cannot_erase_same_span_integer_failure() {
        let program=crate::check("function negate(value:int):int|undefined{return -value;}function main(){}").unwrap().program;
        let function=program.functions.iter().find(|function|function.name=="negate").unwrap();
        let StmtKind::Return(Some(value))=&function.body[0].kind else {panic!("return");};
        let ExprKind::Inject{value:child,..}=&value.kind else {panic!("union wrapper");};
        assert_eq!(value.span,child.span);
        assert_eq!(expression_operation_effect(value),Some((value.span,false)));
        assert_eq!(expression_operation_effect(child),Some((child.span,true)));
        for reversed in [false,true] {
            let mut points=OperationEffects::default();
            let values=if reversed {[child.as_ref(),value]}else{[value,child.as_ref()]};
            for expression in values {let (span,effect)=expression_operation_effect(expression).unwrap();points.insert(span,Some(effect));}
            assert_eq!(points.get(value.span),Some(true));
        }
        let mut unknown=OperationEffects::default();
        unknown.insert(value.span,Some(false));unknown.insert(value.span,None);
        assert_eq!(unknown.get(value.span),None);
    }
    #[test]
    fn operation_mapping_requires_effect_annotation_and_preserves_mutation_span() {
        let mut value=Expr::new(ExprKind::Integer("1".to_owned()),crate::Span::new(1,2));
        value.ty=Some(Type::Int);
        assert_eq!(expression_operation_effect(&value),None);
        let effects=ErrorEffects::default();
        expression_effect(&mut value,&effects,0);
        assert_eq!(expression_operation_effect(&value),Some((value.span,false)));
        let program=crate::check("function main(){let value:int=1;const result=value++;}").unwrap().program;
        let main=program.functions.iter().find(|function|function.name=="main").unwrap();
        let StmtKind::Let{value,..}=&main.body[1].kind else {panic!("declaration");};
        let ExprKind::Update{operator,..}=value.kind else {panic!("update");};
        assert_eq!(expression_operation_effect(value),Some((operator,true)));
    }
    #[test]
    fn mutation_expression_mapping_matches_recorded_operator_spans() {
        for (ty,expected) in [("int",true),("number",false)] {
            for expression in ["value++","++value","value += 2"] {
                let program=crate::check(&format!("function main(){{let value:{ty}=1;const result=({expression});}}")).unwrap().program;
                let main=program.functions.iter().find(|function|function.name=="main").unwrap();
                let StmtKind::Let{value,..}=&main.body[1].kind else {panic!("declaration");};
                let operator=match &value.kind {ExprKind::Update{operator,..}|ExprKind::CompoundAssign{operator,..}=>*operator,_=>panic!("mutation")};
                assert_eq!(mutation_operation_effect(value),Some((operator,expected)));
                assert_ne!(operator,value.span);
            }
        }
        let parsed=crate::syntax::parse("function main(){value++;}").unwrap();
        let StmtKind::Expr(value)=&parsed.functions[0].body[0].kind else {panic!("update");};
        assert_eq!(mutation_operation_effect(value),None);
    }
    #[test]
    fn compound_statement_failure_mapping_uses_operator_span_and_checked_type() {
        for (ty,expected) in [("int",true),("number",false)] {
            let program=crate::check(&format!("function main(){{let value:{ty}=1;value += 2;}}")).unwrap().program;
            let main=program.functions.iter().find(|function|function.name=="main").unwrap();
            let statement=&main.body[1];
            let StmtKind::Assign{operator:Some((_,span)),..}=statement.kind else {panic!("compound");};
            assert_eq!(statement_operation_effect(statement),Some((span,expected)));
            assert_ne!(span,statement.span);
        }
        let parsed=crate::syntax::parse("function main(){value += 2;}").unwrap();
        assert_eq!(statement_operation_effect(&parsed.functions[0].body[0]),None);
    }
    #[test]
    fn checked_invocation_alternatives_supply_their_own_failure_contracts() {
        let source = crate::format("function first<T>(value:T):T{return value;}function second<T>(value:T):T{throw 'alternative';}function main(){const one=first;const two=second;one<int>(7);two<int>(9);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let mut program = crate::check(&source).unwrap().program;
        let pure = program.closures.iter().position(|closure| closure.methods[0].function.body.iter().any(|statement| matches!(statement.kind, StmtKind::Return(_)))).unwrap();
        let throwing = program.closures.iter().position(|closure| closure.methods[0].function.body.iter().any(|statement| matches!(statement.kind, StmtKind::Throw(_)))).unwrap();
        for (origins, fails) in [(vec![throwing], true), (vec![pure, throwing], true), (vec![pure], false)] {
            let id = program.polymorphic_invocations.publish(origins.into_iter().map(|origin| crate::polymorphic_invocation_plans::Pair { origin, method: 0 }), &program.closures, crate::Span::default()).unwrap();
            let main = program.functions.iter_mut().find(|function| function.name == "main").unwrap();
            let call = main.body.iter_mut().find_map(|statement| match &mut statement.kind { StmtKind::Expr(call) => Some(call), _ => None }).unwrap();
            let ExprKind::Call(callee, _) = &mut call.kind else { panic!("source invocation"); };
            assert_eq!(callee.ty, Some(Type::Closure(pure)));
            callee.generic_invocation = Some(id);
            let effects = annotate(&mut program);
            assert!(!effects.method_errors[&(pure,0)].may_fail());
            let main = program.functions.iter().find(|function| function.name == "main").unwrap();
            let call = main.body.iter().find_map(|statement| match &statement.kind { StmtKind::Expr(call) => Some(call), _ => None }).unwrap();
            assert_eq!(call.own_may_fail, fails, "only selected invocation methods contribute failure");
            assert_eq!(effects.invocation_errors[id].explicit, if fails { vec![Type::String] } else { vec![] });
            assert!(!effects.invocation_errors[id].opaque);
        }
    }

    #[test]
    fn own_call_failures_exclude_errors_from_argument_evaluation() {
        let mut program=crate::check("function fail():int{return 1/0;}function outer(n:int):int{return n;}function main(){outer(fail());}").unwrap().program;
        annotate(&mut program);
        let main=program.functions.iter().find(|function|function.name=="main").unwrap();
        let StmtKind::Expr(value)=&main.body[0].kind else {panic!("call");};
        assert!(value.may_fail);
        assert!(!value.own_may_fail);
        let ExprKind::Call(_,args)=&value.kind else {panic!("outer call");};
        assert!(args[0].own_may_fail);
        let fail=program.functions.iter().find(|function|function.name=="fail").unwrap();
        let StmtKind::Return(Some(operation))=&fail.body[0].kind else {panic!("arithmetic");};
        assert!(operation.own_may_fail);
    }
    #[test]
    fn indirect_errors_activate_handlers_after_initial_empty_contracts() {
        let mut program=crate::check("function main(){relay();}function relay():string{return fail();}function fail():string{return \"bad\";}function replacement():int{return 7;}").unwrap().program;
        let id=|name:&str|program.functions.iter().position(|function|function.name==name).unwrap();
        let main=id("main");let relay=id("relay");let fail=id("fail");let replacement=id("replacement");
        assert!(main<relay && relay<fail,"witness requires forward propagation across iterations");
        let StmtKind::Return(Some(value))=program.functions[fail].body[0].kind.clone() else {panic!("return");};
        program.functions[fail].body[0].kind=StmtKind::Throw(value);
        let mut thrown=program.functions[replacement].body[0].clone();
        let StmtKind::Return(Some(value))=thrown.kind else {panic!("return");};
        thrown.kind=StmtKind::Throw(value);
        let mut parsed=crate::syntax::parse("function main(){try{}catch{}}").unwrap();
        let mut statement=parsed.functions[0].body.remove(0);
        let StmtKind::Try(region)=&mut statement.kind else {panic!("region");};
        region.body=std::mem::take(&mut program.functions[main].body);
        region.catch.as_mut().unwrap().body.push(thrown);
        program.functions[main].body.push(statement);
        let effects=annotate(&mut program);
        assert_eq!(effects.function_errors[main].explicit,vec![Type::Int]);
        assert_eq!(effects.function_errors[relay].explicit,vec![Type::String]);
        let StmtKind::Try(region)=&program.functions[main].body[0].kind else {panic!("region");};
        assert_eq!(region.catch.as_ref().unwrap().error_contract.as_ref().unwrap().explicit,vec![Type::String]);
        let StmtKind::Expr(call)=&region.body[0].kind else {panic!("call");};
        assert!(call.may_fail);
    }
    #[test]
    fn unreachable_handler_does_not_add_an_outgoing_throw_contract() {
        let mut program=crate::check("function value():string{return \"bad\";}function main(){}").unwrap().program;
        let mut thrown=program.functions.iter().find(|function|function.name=="value").unwrap().body[0].clone();
        let StmtKind::Return(Some(value))=thrown.kind else {panic!("return");};
        thrown.kind=StmtKind::Throw(value);
        let mut parsed=crate::syntax::parse("function main(){try{}catch{}}").unwrap();
        let mut statement=parsed.functions[0].body.remove(0);
        let StmtKind::Try(region)=&mut statement.kind else {panic!("region");};
        region.catch.as_mut().unwrap().body.push(thrown);
        let main=program.functions.iter().position(|function|function.name=="main").unwrap();
        program.functions[main].body.push(statement);
        let effects=annotate(&mut program);
        assert_eq!(effects.function_errors[main],FailureSet::default());
    }
    use super::*;
    #[test]
    fn catch_types_extend_checked_arena_without_changing_existing_ids() {
        let mut program=crate::check("type User={value:int;};function main(){}").unwrap().program;
        let arena=crate::unions::UnionArena::default();
        let old=arena.intern([Type::Int,Type::Undefined],crate::Span::new(0,0)).unwrap();
        program.unions=arena.snapshot();
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){}}").unwrap();
        let mut statement=parsed.functions[0].body.remove(0);
        let StmtKind::Try(region)=&mut statement.kind else {panic!("region");};
        region.catch.as_mut().unwrap().error_contract=Some(Box::new(FailureSet{explicit:vec![Type::String],runtime:true,opaque:false}));
        program.functions[0].body.push(statement);
        materialize_catch_types(&mut program).unwrap();
        let StmtKind::Try(region)=&program.functions[0].body[0].kind else {panic!("region");};
        let Type::Union(id)=region.catch.as_ref().unwrap().error_type.unwrap() else {panic!("catch union");};
        assert_eq!(id.index(),1);
        assert_eq!(program.unions[id.index()].members,vec![Type::Record(1),Type::String]);
        assert!(!Type::Union(id).is_copy());
        let Type::Union(old)=old else {panic!("old union");};
        assert_eq!(program.unions[old.index()].members,vec![Type::Int,Type::Undefined]);
        materialize_catch_types(&mut program).unwrap();
        assert_eq!(program.records.len(),2);assert_eq!(program.unions.len(),2);
    }
    #[test]
    fn opaque_named_catch_is_rejected_before_allocating_record_or_union() {
        let mut program=crate::check("function main(){}").unwrap().program;
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){}}").unwrap();
        let mut statement=parsed.functions[0].body.remove(0);
        let StmtKind::Try(region)=&mut statement.kind else {panic!("region");};
        region.catch.as_mut().unwrap().error_contract=Some(Box::new(FailureSet{runtime:true,opaque:true,explicit:vec![]}));
        program.functions[0].body.push(statement);
        assert_eq!(materialize_catch_types(&mut program).unwrap_err().code,"error-effect");
        assert!(program.records.is_empty());assert!(program.unions.is_empty());
        let StmtKind::Try(region)=&mut program.functions[0].body[0].kind else {panic!("region");};
        region.catch.as_mut().unwrap().name=None;
        materialize_catch_types(&mut program).unwrap();
        assert!(program.records.is_empty());assert!(program.unions.is_empty());
    }
    #[test]
    fn runtime_record_materialization_is_stable_and_preserves_source_ids() {
        let mut program=crate::check("type RuntimeError={value:int;};function main(){}").unwrap().program;
        let ty=materialize_runtime_record(&mut program);
        assert_eq!(ty,Type::Record(1));
        assert_eq!(program.records.len(),2);
        assert_eq!(program.records[0].name.text,"RuntimeError");
        assert_eq!(program.records[1].name.text,"#RuntimeError");
        assert_eq!(materialize_runtime_record(&mut program),ty);
        program.error_effects=annotate(&mut program);
        assert_eq!(materialize_runtime_record(&mut program),ty);
        assert_eq!(program.records.len(),2);
    }
    #[test]
    fn catch_payload_materialization_preserves_owned_union_identity() {
        let arena=crate::unions::UnionArena::default();
        let span=crate::Span::new(4,9);
        let payload=FailureSet{explicit:vec![Type::String,Type::Int,Type::String],runtime:true,opaque:false};
        let ty=payload.catch_type(&arena,Some(Type::Record(0)),span).unwrap().unwrap();
        let Type::Union(id)=ty else {panic!("catch union");};
        assert_eq!(arena.members(id),vec![Type::Int,Type::Record(0),Type::String]);
        assert!(!ty.is_copy());
        assert_eq!(payload.catch_type(&arena,Some(Type::Record(0)),span).unwrap(),Some(ty));
        assert_eq!(FailureSet::default().catch_type(&arena,None,span).unwrap(),None);
        assert_eq!(FailureSet::thrown(Some(Type::Int)).catch_type(&arena,None,span).unwrap(),Some(Type::Int));
    }
    #[test]
    fn unresolved_catch_payloads_do_not_silently_become_any() {
        let arena=crate::unions::UnionArena::default();
        let span=crate::Span::new(4,9);
        for payload in [FailureSet::unknown(),FailureSet::runtime()] {
            let error=payload.catch_type(&arena,None,span).unwrap_err();
            assert_eq!(error.code,"error-effect");assert_eq!(error.span,span);
        }
    }
    #[test]
    fn retained_runtime_schema_has_checked_fields_and_no_source_name_collision() {
        let program=crate::check("type RuntimeError={value:int;};function main(){}").unwrap().program;
        let schema=program.error_effects.runtime_schema.as_ref().unwrap();
        assert_eq!(schema.name.kind,TokenKind::Symbol);
        assert_eq!(schema.name.text,"#RuntimeError");
        let fields=schema.fields.iter().map(|field|(field.name.text.as_str(),field.ty.unwrap())).collect::<Vec<_>>();
        assert_eq!(fields,vec![("name",Type::String),("code",Type::String),("message",Type::String),("source",Type::String),("start",Type::Int),("end",Type::Int)]);
        assert_eq!(program.records.len(),1,"schema metadata does not inject a source record");
    }
    #[test]
    fn exception_finite_family_keeps_pure_and_unknown_contracts_distinct() {
        let checked=crate::check("function value():void{const actions:((n:int)=>int)[]=[(n:int):int=>n,(n:int):int=>64];}function main(){}").unwrap();
        let (family,_,_)=checked.program.stored_contracts.families().find(|(_,_,origins)|origins.len()==2).unwrap();
        assert_eq!(checked.program.error_effects.storage_errors[&family],FailureSet::default());
        let mut functions=[];let mut closures=[];
        let unknown=annotate_partial_with_storage(&mut functions,&mut closures,&[(7,vec![99]),(8,vec![])],&Default::default());
        assert!(unknown.storage_errors[&7].opaque);
        assert!(unknown.storage_errors[&8].opaque);
        assert!(unknown.storage_errors.get(&9).is_none());
    }
    #[test]
    fn finite_stored_family_contracts_propagate_without_opaque_errors() {
        let checked=crate::check("function value(a:int,field:boolean):int{const actions:((n:int)=>int)[]=[(n:int):int=>64/n,(n:int):int=>n];if(field){return actions[0](a);}let selected=actions.remove(0);return selected(a);}function caller(a:int):int{return value(a,true);}function main(){}").unwrap();
        let effects=&checked.program.error_effects;
        for name in ["value","caller"] {
            let id=checked.program.functions.iter().position(|function|function.name==name).unwrap();
            assert!(effects.function_errors[id].runtime);
            assert!(!effects.function_errors[id].opaque,"{name} retains opaque stored effect");
        }
        let (family,_,_)=checked.program.stored_contracts.families().find(|(_,_,origins)|origins.len()==2).unwrap();
        assert!(effects.storage_errors[&family].runtime);
        assert!(!effects.storage_errors[&family].opaque);
    }
    #[test]
    fn closure_call_contract_uses_origin_not_local_binding_index() {
        let (program,effects)=effects("function failing(a:int):int{const leaf=(n:int):int=>64/n;return leaf(a);}function pure(a:int):int{const leaf=(n:int):int=>n;return leaf(a);}function main(){}");
        let failing=program.functions.iter().position(|function|function.name=="failing").unwrap();
        let pure=program.functions.iter().position(|function|function.name=="pure").unwrap();
        assert!(effects.function_errors[failing].runtime);
        assert!(!effects.function_errors[failing].opaque);
        assert_eq!(effects.function_errors[pure],FailureSet::default());
    }
    #[test]
    fn ieee_math_and_checked_builtin_ranges_have_distinct_contracts() {
        let (program,effects)=effects("function ieee(n:number):number{return Math.sqrt(n);}function points(code:int):string{return String.fromCodePoint(code);}function main(){}");
        let ieee=program.functions.iter().position(|function|function.name=="ieee").unwrap();
        let points=program.functions.iter().position(|function|function.name=="points").unwrap();
        assert_eq!(effects.function_errors[ieee],FailureSet::default());
        assert!(effects.function_errors[points].runtime);
        assert!(!effects.function_errors[points].opaque);
    }
    #[test]
    fn array_callback_invocation_propagates_its_checked_throw_payload() {
        let mut program=crate::check("function mapped():string[]{return [1].map((value:int):string=>\"bad\");}function main(){}").unwrap().program;
        let mut replaced=0;
        for closure in &mut program.closures {
            for function in std::iter::once(&mut closure.function).chain(closure.methods.iter_mut().map(|method|&mut method.function)) {
                for statement in &mut function.body {
                    if let StmtKind::Return(Some(value))=&statement.kind {
                        if value.ty==Some(Type::String) {statement.kind=StmtKind::Throw(value.clone());replaced+=1;}
                    }
                }
            }
        }
        assert!(replaced>0);
        let effects=annotate(&mut program);
        let mapped=program.functions.iter().position(|function|function.name=="mapped").unwrap();
        assert!(effects.function_errors[mapped].explicit.contains(&Type::String));
        assert!(effects.function_errors[mapped].runtime);
        assert!(!effects.function_errors[mapped].opaque);
    }
    #[test]
    fn checked_program_retains_error_contracts_for_later_phases() {
        let program=crate::check("function index(values:int[],i:int):int{return values[i];}function main(){}").unwrap().program;
        assert_eq!(program.error_effects.function_errors.len(),program.functions.len());
        assert_eq!(program.error_effects.closure_errors.len(),program.closures.len());
        let index=program.functions.iter().position(|function|function.name=="index").unwrap();
        assert!(program.error_effects.function_errors[index].runtime);
    }
    fn effects(source:&str)->(CheckedProgram,ErrorEffects) {
        let mut program=crate::check(source).unwrap().program;
        let effects=annotate(&mut program);(program,effects)
    }
    fn named(program:&CheckedProgram,effects:&ErrorEffects,name:&str)->bool {
        let id=program.functions.iter().position(|function|function.name==name).unwrap();effects.functions[id]
    }
    #[test]
    fn checked_bounds_propagate_through_calls_but_number_division_does_not() {
        let (program,effects)=effects("function index(values:int[],i:int):int{return values[i];}function relay(values:int[],i:int):int{return index(values,i);}function ieee(a:number,b:number):number{return a/b;}function main(){}");
        assert!(named(&program,&effects,"index"));assert!(named(&program,&effects,"relay"));assert!(!named(&program,&effects,"ieee"));
        let id=program.functions.iter().position(|function|function.name=="relay").unwrap();
        let crate::syntax::StmtKind::Return(Some(value))=&program.functions[id].body[0].kind else{panic!("return expression");};assert!(value.may_fail);
        assert!(effects.function_errors[id].runtime);
        assert!(effects.function_errors[id].explicit.is_empty());
        assert!(!effects.function_errors[id].opaque);
    }
    #[test]
    fn recursive_effects_reach_a_fixed_point() {
        let (program,effects)=effects("function left(n:int):int{if(n===0){return 1;}return right(n);}function right(n:int):int{if(n===1){return 10/n;}return left(n);}function main(){}");
        assert!(named(&program,&effects,"left"));assert!(named(&program,&effects,"right"));
    }
    #[test]
    fn closure_creation_does_not_inherit_invocation_failure() {
        let (program,effects)=effects("function make():Function<int>{return ():int=>10/0;}function invoke():int{return make()();}function main(){}");
        assert!(!named(&program,&effects,"make"));assert!(named(&program,&effects,"invoke"));assert!(effects.closures.iter().any(|effect|*effect));
    }
    #[test]
    fn handled_errors_do_not_escape_but_handler_and_cleanup_errors_do() {
        fn escaping(source:&str)->bool {
            let mut program=crate::syntax::parse(source).unwrap();
            body_effect(&mut program.functions[0].body,&ErrorEffects::default(),0).may_fail()
        }
        assert!(!escaping("function main(){try{throw 1;}catch{}}"));
        assert!(!escaping("function main(){try{throw 1;}catch(error){}finally{}}"));
        assert!(!escaping("function main(){try{throw 1;}catch{try{throw 2;}catch{}}}"));
        assert!(escaping("function main(){try{throw 1;}finally{}}"));
        assert!(escaping("function main(){try{throw 1;}catch(error){throw error;}}"));
        assert!(escaping("function main(){try{}catch{}finally{throw 2;}}"));
        assert!(!escaping("function main(){try{throw 1;}finally{return;}}"));
        assert!(!escaping("function main(){try{throw 1;}finally{if(true){return;}else{return;}}}"));
        assert!(escaping("function main(){try{throw 1;}finally{if(true){return;}}}"));
        assert!(escaping("function main(){try{throw 1;}finally{done:{break done;}}}"));
    }
    #[test]
    fn returning_a_function_keeps_its_invocation_effect() {
        let (program,effects)=effects("function risky():int{return 10/0;}function make():Function<int>{return risky;}function invoke():int{return make()();}function main(){}");
        assert!(!named(&program,&effects,"make"));
        assert!(named(&program,&effects,"invoke"));
        let risky=program.functions.iter().position(|function|function.name=="risky").unwrap();
        let forwarded=program.closures.iter().position(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::Function(id)) if id==risky)).unwrap();
        assert!(effects.closures[forwarded]);
    }
    #[test]
    fn checked_throw_payloads_propagate_without_becoming_runtime_errors() {
        let mut program=crate::check("function fail():string{return \"bad\";}function relay():string{return fail();}function main(){}").unwrap().program;
        let fail=program.functions.iter().position(|function|function.name=="fail").unwrap();
        let relay=program.functions.iter().position(|function|function.name=="relay").unwrap();
        let StmtKind::Return(Some(value))=program.functions[fail].body[0].kind.clone() else {panic!("checked return");};
        program.functions[fail].body[0].kind=StmtKind::Throw(value);
        let effects=annotate(&mut program);
        for id in [fail,relay] {
            assert_eq!(effects.function_errors[id].explicit,vec![Type::String]);
            assert!(!effects.function_errors[id].runtime);
            assert!(!effects.function_errors[id].opaque);
        }
    }
    #[test]
    fn handled_payloads_leave_body_failure_annotations_intact() {
        let mut program=crate::check("function fail():string{return \"bad\";}function relay():string{return fail();}function main(){}").unwrap().program;
        let fail=program.functions.iter().position(|function|function.name=="fail").unwrap();
        let relay=program.functions.iter().position(|function|function.name=="relay").unwrap();
        let StmtKind::Return(Some(value))=program.functions[fail].body[0].kind.clone() else {panic!("checked return");};
        program.functions[fail].body[0].kind=StmtKind::Throw(value);
        let mut parsed=crate::syntax::parse("function main(){try{}catch{}}").unwrap();
        let StmtKind::Try(mut region)=parsed.functions[0].body.remove(0).kind else {panic!("region");};
        region.body=std::mem::take(&mut program.functions[relay].body);
        let statement=Stmt{labels:vec![],control_target:None,span:crate::Span::new(0,0),kind:StmtKind::Try(region)};
        program.functions[relay].body.push(statement);
        let effects=annotate(&mut program);
        assert!(!effects.functions[relay]);
        assert_eq!(effects.function_errors[relay],FailureSet::default());
        let StmtKind::Try(region)=&program.functions[relay].body[0].kind else {panic!("region");};
        let incoming=region.catch.as_ref().unwrap().error_contract.as_ref().unwrap();
        assert_eq!(incoming.explicit,vec![Type::String]);
        assert!(!incoming.runtime);
        assert!(!incoming.opaque);
        let StmtKind::Return(Some(value))=&region.body[0].kind else {panic!("return");};
        assert!(value.may_fail);
    }
    #[test]
    fn nested_catches_keep_incoming_and_handler_payloads_separate() {
        let mut program=crate::check("function text():string{return \"bad\";}function count():int{return 9;}function main(){}").unwrap().program;
        let mut throws=Vec::new();
        for name in ["text","count"] {
            let function=program.functions.iter().find(|function|function.name==name).unwrap();
            let mut statement=function.body[0].clone();
            let StmtKind::Return(Some(value))=statement.kind else {panic!("return");};
            statement.kind=StmtKind::Throw(value);throws.push(statement);
        }
        let mut parsed=crate::syntax::parse("function main(){try{try{}catch{}}catch{}}").unwrap();
        let mut outer=parsed.functions[0].body.remove(0);
        let StmtKind::Try(region)=&mut outer.kind else {panic!("outer region");};
        let StmtKind::Try(inner)=&mut region.body[0].kind else {panic!("inner region");};
        inner.body.push(throws.remove(0));inner.catch.as_mut().unwrap().body.push(throws.remove(0));
        let main=program.functions.iter().position(|function|function.name=="main").unwrap();
        program.functions[main].body.push(outer);
        let effects=annotate(&mut program);
        assert!(!effects.functions[main]);
        let StmtKind::Try(region)=&program.functions[main].body[0].kind else {panic!("outer region");};
        assert_eq!(region.catch.as_ref().unwrap().error_contract.as_ref().unwrap().explicit,vec![Type::Int]);
        let StmtKind::Try(inner)=&region.body[0].kind else {panic!("inner region");};
        assert_eq!(inner.catch.as_ref().unwrap().error_contract.as_ref().unwrap().explicit,vec![Type::String]);
    }
    #[test]
    fn resolved_callback_contracts_refresh_direct_conditional_and_builtin_effects() {
        let mut program=crate::check("function direct(f:Function<int,[n:int]>,n:int):int{return f(n);}function conditional(f:Function<int,[n:int]>,n:int):int{return(n>0?f:f)(n);}function mapped(f:Function<int,[n:int]>,values:int[]):int[]{return values.map(f);}function main(){}").unwrap().program;
        let unknown=annotate(&mut program);
        for name in ["direct","conditional","mapped"] {
            let id=program.functions.iter().position(|function|function.name==name).unwrap();
            assert!(unknown.function_errors[id].opaque,"{name}");
        }
        let contracts=(0..program.callback_contracts.len()).map(|id|(id,FailureSet::default())).collect();
        let pure=annotate_with_callbacks(&mut program,&contracts);
        for name in ["direct","conditional","mapped"] {
            let id=program.functions.iter().position(|function|function.name==name).unwrap();
            assert!(!pure.function_errors[id].opaque,"{name}");
            assert_eq!(pure.function_errors[id].runtime,name=="mapped","{name}");
        }
        let thrown=FailureSet{explicit:vec![Type::String],..Default::default()};
        let contracts=(0..program.callback_contracts.len()).map(|id|(id,thrown.clone())).collect();
        let failing=annotate_with_callbacks(&mut program,&contracts);
        for name in ["direct","conditional","mapped"] {
            let id=program.functions.iter().position(|function|function.name==name).unwrap();
            assert!(!failing.function_errors[id].opaque,"{name}");
            assert_eq!(failing.function_errors[id].explicit,vec![Type::String],"{name}");
        }
        // Re-solving must remove a previous failure when specialization changes.
        let unknown=annotate(&mut program);
        assert!(unknown.callback_errors.is_empty());
        let id=program.functions.iter().position(|function|function.name=="direct").unwrap();
        assert!(unknown.function_errors[id].opaque);
    }

    #[test]
    fn inferred_callback_origins_join_forwarded_effects_without_crossing_declarations() {
        let mut program=crate::check("function direct(f:Function<int,[n:int]>,n:int):int{return f(n);}function relay(f:Function<int,[n:int]>,n:int):int{return direct(f,n);}function other(f:Function<int,[n:int]>,n:int):int{return f(n);}function unused(f:Function<int,[n:int]>,n:int):int{return f(n);}function borrowed(f:ReadOnly<Function<int,[n:int]>>,n:int):int{return f(n);}function main(){direct((n:int):int=>n,1);direct((n:int):int=>1/n,1);relay((n:int):int=>1/n,1);other((n:int):int=>n,1);borrowed((n:int):int=>1/n,1);}").unwrap().program;
        let effects=annotate_inferred_callbacks(&mut program);
        for name in ["direct","relay","other","unused","borrowed"] {
            let id=program.functions.iter().position(|function|function.name==name && !function.specialization).unwrap();
            let found=&effects.function_errors[id];
            assert_eq!(found.opaque,name=="unused","{name}: {found:?}");
            assert_eq!(found.runtime,matches!(name,"direct"|"relay"|"borrowed"),"{name}: {found:?}");
        }
    }

    #[test]
    fn selected_generic_method_receives_callback_argument_effects() {
        let source = crate::format("function first<T>(value:T,f:ReadOnly<Function<int,[n:int]>>):int{return f(1);}function second<T>(value:T,f:ReadOnly<Function<int,[n:int]>>):int{return f(2);}function main(){const one=first;const two=second;one<int>(7,(n:int):int=>1/n);two<int>(9,(n:int):int=>n);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let mut program=crate::check(&source).unwrap().program;
        let main=program.functions.iter().position(|function|function.name=="main").unwrap();
        let calls=program.functions[main].body.iter().filter_map(|statement|match &statement.kind {StmtKind::Expr(call)=>Some(call),_=>None}).collect::<Vec<_>>();
        let ExprKind::Call(second,_) = &calls[1].kind else {panic!("second call");};
        let Some(Type::Closure(origin))=second.ty else {panic!("method environment");};
        let method=match calls[1].target {Some(CallTarget::ClosureMethod(method))=>method,_=>panic!("method target")};
        let body=&program.closures[origin].methods[method];
        let Type::Callback(callback)=body.function.bindings[body.parameters[1]].ty else {panic!("callback contract");};
        let id=program.polymorphic_invocations.publish([crate::polymorphic_invocation_plans::Pair{origin,method}],&program.closures,crate::Span::default()).unwrap();
        let mut seen=false;
        program.functions[main].body.retain_mut(|statement| {
            let StmtKind::Expr(call)=&mut statement.kind else {return true;};
            if seen {return false;} seen=true;
            let ExprKind::Call(callee,_)=&mut call.kind else {panic!("first call");};
            callee.generic_invocation=Some(id);
            true
        });
        let unions=crate::unions::UnionArena::from_snapshot(program.unions.clone());
        let (before,remaining)=program.functions.split_at_mut(main);
        let (caller,after)=remaining.split_first_mut().unwrap();
        let mut functions=before.iter_mut().map(Some).chain(std::iter::once(None)).chain(after.iter_mut().map(Some)).collect::<Vec<_>>();
        let regional=annotate_partial_with_callers(&mut functions,&mut program.closures,&[],caller,&unions,&program.polymorphic_invocations,&[]);
        let contract=regional.callback_errors.get(&callback).expect("regional selected callback origin");
        assert!(contract.runtime && !contract.opaque,"regional selected method must receive the failing argument: {contract:?}");
        let effects=annotate_inferred_callbacks(&mut program);
        let contract=effects.callback_errors.get(&callback).expect("selected callback origin");
        assert!(contract.runtime && !contract.opaque,"selected method must receive the failing argument: {contract:?}");
    }

    #[test]
    fn inferred_callback_cycles_without_concrete_callers_remain_opaque() {
        let mut program=crate::check("function recurse(f:Function<int,[n:int]>,n:int):int{if(n===0){return f(n);}return recurse(f,n);}function main(){}").unwrap().program;
        let effects=annotate_inferred_callbacks(&mut program);
        let id=program.functions.iter().position(|function|function.name=="recurse").unwrap();
        assert!(effects.function_errors[id].opaque,"unrooted callback cycle must not be assumed pure");
    }

    #[test]
    fn inferred_callback_arguments_reach_checked_closure_parameters() {
        let mut program=crate::check("function main(){const invoke=(f:ReadOnly<Function<int,[n:int]>>,n:int):int=>f(n);invoke((n:int):int=>1/n,1);}").unwrap().program;
        let effects=annotate_inferred_callbacks(&mut program);
        assert!(effects.closure_errors.iter().all(|contract|!contract.opaque),"{:?}",effects.closure_errors);
        assert!(effects.closure_errors[0].runtime);
        assert!(effects.callback_errors.values().any(|contract|contract.runtime && !contract.opaque),"closure callback origin was not collected");
    }

    #[test]
    fn abstract_callback_forwarding_preserves_resolved_and_unknown_contracts() {
        let mut program=crate::check("function keep(f:Function<int,[n:int]>):(n:int)=>int{return f;}function main(){}").unwrap().program;
        let (origin,callback)=program.closures.iter().enumerate().find_map(|(id,closure)|match closure.forward {Some(ClosureForward::Abstract(callback))=>Some((id,callback)),_=>None}).expect("abstract callback forwarder");
        assert!(annotate(&mut program).closure_errors[origin].opaque);
        let pure=[(callback,FailureSet::default())].into_iter().collect();
        assert_eq!(annotate_with_callbacks(&mut program,&pure).closure_errors[origin],FailureSet::default());
        let failing=FailureSet{runtime:true,explicit:vec![Type::String],..Default::default()};
        let resolved=[(callback,failing.clone())].into_iter().collect();
        assert_eq!(annotate_with_callbacks(&mut program,&resolved).closure_errors[origin],failing);
        assert!(annotate(&mut program).closure_errors[origin].opaque);
    }

    #[test]
    fn rest_forwarders_preserve_finite_source_effects() {
        let mut program=crate::check("function size(...values:int[]):int{return values.length;}function main(){const action:Function<int,[left:int,right:int]>=size;action(2,3);}").unwrap().program;
        let effects=annotate(&mut program);
        let (adapter,closure)=program.closures.iter().enumerate().find(|(_,closure)|matches!(closure.forward,Some(ClosureForward::RestAdapter{..}))).unwrap();
        let Type::Closure(source)=closure.function.bindings[0].ty else {panic!("concrete rest source");};
        assert!(!effects.closure_errors[adapter].opaque,"{:?}",effects.closure_errors[adapter]);
        assert_eq!(effects.closure_errors[adapter],effects.closure_errors[source]);
    }

    #[test]
    fn rest_forwarders_preserve_resolved_callback_contracts() {
        let mut program=crate::check("function adapt(action:Function<int,[...values:int[]]>):Function<int,[value:int]>{return action;}function main(){}").unwrap().program;
        let (adapter,closure)=program.closures.iter().enumerate().find(|(_,closure)|matches!(closure.forward,Some(ClosureForward::RestAdapter{..}))).unwrap();
        let callback=match closure.function.bindings[0].ty {
            Type::Callback(id)=>id,
            Type::Closure(id)=>match program.closures[id].forward {Some(ClosureForward::Abstract(id))=>id,_=>panic!("abstract returned rest source")},
            _=>panic!("abstract rest source"),
        };
        assert!(annotate(&mut program).closure_errors[adapter].opaque);
        for contract in [FailureSet::default(),FailureSet{runtime:true,explicit:vec![Type::String],..Default::default()}] {
            let callbacks=[(callback,contract.clone())].into_iter().collect();
            let effects=annotate_with_callbacks(&mut program,&callbacks);
            assert_eq!(effects.closure_errors[adapter],contract);
        }
    }

}
