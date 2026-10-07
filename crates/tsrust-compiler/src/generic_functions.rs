//! Bounded explicit function-template materialization before ordinary ownership checking.
#[path = "generic_value_signatures.rs"]
mod value_signatures;
use crate::syntax::*;
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use crate::{Diagnostic, Span};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Named(String),
    Array(Box<Key>),
    Applied(String, Vec<Key>),
    Union(Vec<Key>),
    Tuple(Vec<Key>),
    Callable(CallbackKind, Vec<(bool, crate::Mode, bool, Key)>, Box<Key>),
}
fn key(value: &ValueAnnotation) -> Result<Key, Diagnostic> {
    Ok(match &value.kind {
        Kind::Named(name) => Key::Named(name.text.clone()),
        Kind::Group(inner) => return key(inner),
        Kind::Array(inner) => Key::Array(Box::new(key(inner)?)),
        Kind::Applied { name, arguments } => Key::Applied(name.text.clone(),arguments.iter().map(key).collect::<Result<_,_>>()?),
        Kind::Tuple(members) => Key::Tuple(members.iter().map(key).collect::<Result<_,_>>()?),
        Kind::Union(members) => {
            let mut keys=Vec::new();
            for member in members {
                match key(member)? { Key::Union(inner)=>keys.extend(inner), other=>keys.push(other) }
            }
            keys.sort_by_key(|key|match key { Key::Named(name) if name=="null"=>1,Key::Named(name) if name=="undefined"=>2,_=>0 });
            keys.dedup();
            if keys.len()==1 {return Ok(keys.remove(0));}
            Key::Union(keys)
        }
        Kind::Callable { parameters, result, .. } => Key::Callable(crate::callable_annotations::ownership(value).expect("callable ownership"),
            parameters.iter().map(|parameter| Ok((parameter.rest,
                parameter.access.as_ref().map_or(crate::Mode::Copy, |access| access.mode),
                parameter.omission.is_omittable(), key(&parameter.annotation)?))).collect::<Result<_, Diagnostic>>()?,
            Box::new(key(result)?)),
    })
}
pub(crate) fn same_annotation_contract(left: &ValueAnnotation, right: &ValueAnnotation) -> bool {
    matches!((key(left), key(right)), (Ok(left), Ok(right)) if left == right)
}

struct Pending {
    function: Function,
    bindings: HashMap<String, ValueAnnotation>,
    depth: usize,
}
#[derive(Default)]
pub(crate) struct Cache {
    templates: BTreeMap<String, Function>,
    instances: HashMap<(String, Vec<Key>, Vec<Type>), String>,
    origins: HashMap<String, String>,
    counts: HashMap<String, usize>,
    checked_instances: HashMap<(String, Vec<Type>), String>,
    captured_checked_instances: HashMap<(String, Vec<Type>, Vec<Type>), String>,
    pending: Vec<Pending>,
    argument_checks: Vec<ValueAnnotation>,
    calls: HashMap<(usize, usize), Vec<ValueAnnotation>>,
}
impl Cache {
    pub(crate) fn template(&self, name: &str) -> Option<&Function> {
        self.templates.get(name)
    }

    fn argument_hints(&self,name:&str,hints:&[Option<Type>],types:&crate::generic_inference::Types<'_>)->Result<Vec<Option<Type>>,Diagnostic> {
        let template=&self.templates[name];
        let minimum=crate::parameter_arity::minimum_arguments(&template.parameter_initializers[..template.params.len()-usize::from(template.rest_parameter)],|initializer|initializer.omission().is_omittable());
        if hints.len()<minimum||hints.len()>template.params.len(){return Err(Diagnostic::new("arity",format!("Expected {minimum}..={} arguments, found {}.",template.params.len(),hints.len()),template.name.span));}
        let mut result=hints.to_vec();result.resize(template.params.len(),None);
        for (hint,initializer) in result.iter_mut().zip(&template.parameter_initializers) {
            // Undefined selects omission/default handling, not a type-variable
            // constraint. Preserve null and every other supplied union member.
            if initializer.omission().is_omittable() && let Some(actual) = *hint {
                let members = actual.union().map(|id| types.unions.members(id)).unwrap_or_else(|| vec![actual]);
                let supplied = members.into_iter().filter(|ty| *ty != Type::Undefined).collect::<Vec<_>>();
                *hint = if supplied.is_empty() { None } else { Some(types.unions.intern(supplied, template.name.span)?) };
            }
        }
        Ok(result)
    }

    pub(crate) fn parameter_contexts(
        &self,
        name: &str,
        expected: Option<Type>,
        types: &crate::generic_inference::Types<'_>,
        hints: Option<&[Option<Type>]>,
    ) -> Result<Vec<Option<Type>>, Diagnostic> {
        let template = &self.templates[name];
        let parameters = template.params.iter().map(|(_, annotation, _)| annotation.clone()).collect::<Vec<_>>();
        let unknown = vec![None; parameters.len()];
        let result = template.result.as_ref().and_then(|result| match result {
            ResultAnnotation::Named(value) => Some(value.as_ref()),
            _ => None,
        });
        let mut bindings = HashMap::new();
        match types.solve(crate::generic_inference::Constraints {
            variables: &template.type_parameters,
            parameters: &parameters,
            arguments: &self.argument_hints(name, hints.unwrap_or(&unknown), types)?,
            result,
            expected,
            span: template.name.span,
        }, &mut bindings) {
            Ok(_) => {},
            Err(error) if error.code == "generic-inference" => {},
            Err(error) => return Err(error),
        }
        Ok(parameters.iter().map(|annotation| types.contextual_type(annotation, &bindings)).collect())
    }

    pub(crate) fn infer(
        &self,
        name: &str,
        hints: &[Option<Type>],
        expected: Option<Type>,
        types: &crate::generic_inference::Types<'_>,
        span: Span,
    ) -> Result<Vec<Type>, Diagnostic> {
        let template = &self.templates[name];
        let parameters = template.params.iter().map(|(_, annotation, _)| annotation.clone()).collect::<Vec<_>>();
        let result = template.result.as_ref().and_then(|result| match result {
            ResultAnnotation::Named(value) => Some(value.as_ref()),
            ResultAnnotation::Callable { .. } | ResultAnnotation::Inferred(_) => None,
        });
        types.solve(crate::generic_inference::Constraints {
            variables: &template.type_parameters,
            parameters: &parameters,
            arguments: &self.argument_hints(name, hints, types)?,
            result,
            expected,
            span,
        }, &mut HashMap::new())
    }

    pub(crate) fn request_checked(
        &mut self,
        name: &str,
        arguments: Vec<Type>,
        types: &crate::generic_inference::Types<'_>,
        span: Span,
        depth: usize,
    ) -> Result<String, Diagnostic> {
        let identity = (name.to_owned(), arguments.clone());
        if let Some(concrete) = self.checked_instances.get(&identity) {
            return Ok(concrete.clone());
        }
        let annotations = arguments.iter().map(|ty| types.annotation(*ty, span)).collect::<Result<_, _>>()?;
        let concrete = self.instance(name, annotations, span, depth)?;
        self.checked_instances.insert(identity, concrete.clone());
        Ok(concrete)
    }

    pub(crate) fn request_captured_checked(
        &mut self, name: &str, arguments: Vec<Type>, captures: Vec<Type>,
        types: &crate::generic_inference::Types<'_>, span: Span, depth: usize,
    ) -> Result<String, Diagnostic> {
        if captures.is_empty() { return self.request_checked(name, arguments, types, span, depth); }
        let identity = (name.to_owned(), arguments.clone(), captures.clone());
        if let Some(concrete) = self.captured_checked_instances.get(&identity) { return Ok(concrete.clone()); }
        let annotations = arguments.iter().map(|ty| types.annotation(*ty, span)).collect::<Result<_, _>>()?;
        let concrete = self.instance_with_captures(name, annotations, captures, span, depth)?;
        self.captured_checked_instances.insert(identity, concrete.clone());
        Ok(concrete)
    }

    pub(crate) fn take_argument_checks(&mut self) -> Vec<ValueAnnotation> {
        std::mem::take(&mut self.argument_checks)
    }

    pub(crate) fn seed_checked(
        &mut self,
        program: &Program,
        names: &HashMap<String, usize>,
        unions: &crate::unions::UnionArena,
    ) -> Result<(), Diagnostic> {
        self.seed_functions(&program.functions, names, unions)
    }

    pub(crate) fn seed_functions(
        &mut self,
        functions: &[Function],
        names: &HashMap<String, usize>,
        unions: &crate::unions::UnionArena,
    ) -> Result<(), Diagnostic> {
        // Source function order chooses a stable representative if checked identities merge.
        for function in functions {
            let Some(declaration) = self.origins.get(function.name.text.as_str()) else {
                continue;
            };
            let application = function.application.as_ref().expect("published generic instance");
            // Callable applications already retain their full source-key instance.
            // Data-only bootstrap cannot invent checked callable Type identities;
            // checked requests publish those identities when contracts resolve.
            let mut pending = application.arguments.iter().collect::<Vec<_>>();
            let mut callable = false;
            while let Some(annotation) = pending.pop() {
                match &annotation.kind {
                    Kind::Callable { .. } => { callable = true; break; }
                    Kind::Array(inner) | Kind::Group(inner) => pending.push(inner),
                    Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => pending.extend(arguments),
                    Kind::Named(_) => {},
                }
            }
            if callable { continue; }
            let arguments = application.arguments.iter().map(|argument| {
                crate::analysis::resolve_value_type(argument, false, names, unions)
            }).collect::<Result<Vec<_>, _>>()?;
            self.checked_instances.entry((declaration.clone(), arguments))
                .or_insert_with(|| function.name.text.clone());
        }
        Ok(())
    }

    fn instance(
        &mut self,
        name: &str,
        arguments: Vec<ValueAnnotation>,
        span: Span,
        depth: usize,
    ) -> Result<String, Diagnostic> {
        self.instance_with_captures(name, arguments, Vec::new(), span, depth)
    }

    fn instance_with_captures(
        &mut self, name: &str, arguments: Vec<ValueAnnotation>, captures: Vec<Type>,
        span: Span, depth: usize,
    ) -> Result<String, Diagnostic> {
        let template = &self.templates[name];
        if arguments.len() != template.type_parameters.len() {
            return Err(Diagnostic::new(
                "arity",
                format!(
                    "Generic function expects {} type arguments, received {}.",
                    template.type_parameters.len(),
                    arguments.len()
                ),
                span,
            ));
        }
        let identity = (
            name.to_owned(),
            arguments.iter().map(key).collect::<Result<Vec<_>, _>>()?,
            captures,
        );
        if let Some(name) = self.instances.get(&identity) {
            return Ok(name.clone());
        }
        if depth > 32
            || self.instances.len() >= 1024
            || self.counts.get(name).copied().unwrap_or(0) >= 128
        {
            return Err(Diagnostic::new("generic-limit","Generic function specialization exceeds its depth (32), per-declaration (128), or program (1024) limit.",span));
        }
        let concrete = format!("{name}<specialization#{}>", self.instances.len());
        let bindings = template
            .type_parameters
            .iter()
            .zip(&arguments)
            .map(|(parameter, argument)| (parameter.text.clone(), argument.clone()))
            .collect();
        let mut function = template.clone();
        function.application = Some(Box::new(FunctionApplication {
            declaration: template.name.clone(),
            variables: template.type_parameters.clone(),
            arguments: arguments.clone(),
            patterns: template.params.iter().map(|(_,annotation,_)|annotation.clone()).collect(),
        }));
        function.name.text = concrete.clone();
        function.type_parameters.clear();
        self.argument_checks.extend(arguments);
        self.instances.insert(identity, concrete.clone());
        self.origins.insert(concrete.clone(), name.to_owned());
        *self.counts.entry(name.to_owned()).or_default() += 1;
        self.pending.push(Pending {
            function,
            bindings,
            depth,
        });
        Ok(concrete)
    }
}

impl Cache {
    pub(crate) fn drain(&mut self, allow_implicit: bool) -> Result<Vec<Function>, Diagnostic> {
    let mut output = Vec::new();
    while let Some(Pending {
        mut function,
        bindings,
        depth,
    }) = self.pending.pop()
    {
        let mut substitute =
            |value: &mut ValueAnnotation| crate::type_substitution::apply(value, &bindings);
        crate::source_walk::function_annotations(&mut function, &mut substitute)?;
        // A generic data slot can specialize to a callable. Rebuild the source
        // callable view after substitution before signature/escape analysis.
        for (_, annotation, callback) in &mut function.params {
            if callback.is_none() { *callback = crate::callable_annotations::callback(annotation); }
        }
        if let Some(ResultAnnotation::Named(annotation)) = &function.result {
            if crate::callable_annotations::callback(annotation).is_some() {
                function.result = Some(crate::callable_annotations::result_annotation(annotation));
            }
        }
        let end = function.scope_span.end;
        let mut shadows = function
            .params
            .iter()
            .map(|(name, _, _)| (name.text.clone(), Span::new(function.name.span.start, end)))
            .collect::<Vec<_>>();
        locals(&function.body, &mut shadows);
        crate::source_walk::function_body(&mut function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Closure { params, body, .. } = &expression.kind {
                for parameter in params {
                    shadows.push((parameter.name.text.clone(), expression.span));
                }
                locals(body, &mut shadows);
            }
            Ok(())
        })?;
        crate::source_walk::function_body(&mut function, &mut substitute, &mut |expression| {
            let ExprKind::Call(callee, _) = &mut expression.kind else {
                return Ok(());
            };
            let ExprKind::Name(name) = &mut callee.kind else {
                return Ok(());
            };
            let explicit = self.calls.get(&(expression.span.start, expression.span.end));
            let shadowed = shadows.iter().any(|(local, span)| {
                local == name && span.start <= callee.span.start && callee.span.end <= span.end
            });
            if !self.templates.contains_key(name) || shadowed {
                if let Some(arguments) = explicit {
                    return Err(Diagnostic::new(
                        "type",
                        "Explicit type arguments require a declared generic function.",
                        arguments
                            .first()
                            .map_or(expression.span, |argument| argument.span),
                    ));
                }
                return Ok(());
            }
            let Some(arguments) = explicit else {
                if allow_implicit { return Ok(()); }
                return Err(Diagnostic::new("unsupported","Generic function argument inference is not yet implemented; provide explicit type arguments.",callee.span));
            };
            let mut arguments = arguments.clone();
            for argument in &mut arguments {
                crate::type_substitution::apply(argument, &bindings)?;
            }
            *name = self.instance(name, arguments, expression.span, depth + 1)?;
            Ok(())
        })?;
        output.push(function);
    }
    Ok(output)
    }
}

fn locals(body: &[Stmt], result: &mut Vec<(String, Span)>) {
    let mut pending = vec![body];
    while let Some(body) = pending.pop() {
        let end = crate::source_walk::body_end(body);
        for statement in body {
            match &statement.kind {
                StmtKind::Let { name, value, .. } => {
                    result.push((name.text.clone(), Span::new(value.span.end, end)))
                }
                StmtKind::Block(body) | StmtKind::While { body, .. } => pending.push(body),
                StmtKind::Try(region) => {
                    pending.push(&region.body);
                    if let Some(handler)=&region.catch {
                        if let (Some(name),Some(first))=(&handler.name,handler.body.first()) {
                            result.push((name.text.clone(),Span::new(first.span.start,crate::source_walk::body_end(&handler.body))));
                        }
                        pending.push(&handler.body);
                    }
                    if let Some(body)=&region.finally {pending.push(body);}
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.push(then_body);
                    pending.push(else_body);
                }
                StmtKind::Loop { body, update, .. } => {
                    pending.push(body);
                    pending.push(update);
                }
                StmtKind::ForOf { name, body, .. } => {
                    if let Some(first) = body.first() {
                        result.push((
                            name.text.clone(),
                            Span::new(first.span.start, crate::source_walk::body_end(body)),
                        ));
                    }
                    pending.push(body);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod exception_scope_tests {
    use super::*;
    #[test]
    fn exception_locals_and_catch_binding_have_distinct_scope_ranges() {
        let source="function main(){try{const item=1;item;}catch(item){item;}finally{const cleanup=2;cleanup;}item;}";
        let program=crate::syntax::parse(source).unwrap();
        let mut found=Vec::new();locals(&program.functions[0].body,&mut found);
        let try_use=source.find("item;}").unwrap();
        let catch_use=source.find("{item;}").unwrap()+1;
        let outer_use=source.rfind("item;").unwrap();
        let item_scopes=found.iter().filter(|(name,_)|name=="item").map(|(_,span)|*span).collect::<Vec<_>>();
        assert_eq!(item_scopes.len(),2);
        assert_eq!(item_scopes.iter().filter(|span|span.start<=try_use && try_use<span.end).count(),1);
        assert_eq!(item_scopes.iter().filter(|span|span.start<=catch_use && catch_use<span.end).count(),1);
        assert!(item_scopes.iter().all(|span|outer_use>=span.end));
        assert!(found.iter().any(|(name,_)|name=="cleanup"));
    }
}

pub(crate) fn specialize(program: &mut Program) -> Result<Cache, Diagnostic> {
    if !program
        .functions
        .iter()
        .any(|function| !function.type_parameters.is_empty())
    {
        return Ok(Cache::default());
    }
    crate::generic_defaults::validate(program)?;
    let mut declarations = HashMap::new();
    let mut cache = Cache {
        captured_checked_instances: HashMap::new(),
        templates: BTreeMap::new(),
        instances: HashMap::new(),
        origins: HashMap::new(),
        counts: HashMap::new(),
        checked_instances: HashMap::new(),
        pending: Vec::new(),
        argument_checks: Vec::new(),
        calls: HashMap::new(),
    };
    let records = program
        .records
        .iter()
        .map(|record| (record.name.text.clone(), record.type_parameters.len()))
        .collect();
    for mut function in std::mem::take(&mut program.functions) {
        if declarations
            .insert(function.name.text.clone(), function.name.span)
            .is_some()
        {
            return Err(Diagnostic::new(
                "duplicate",
                "Function is already defined.",
                function.name.span,
            ));
        }
        if function.type_parameters.is_empty() {
            cache.pending.push(Pending {
                function,
                bindings: HashMap::new(),
                depth: 0,
            });
        } else {
            if matches!(
                function.name.text.as_str(),
                "main"
                    | "clone"
                    | "String"
                    | "Boolean"
                    | "Array"
                    | "Object"
                    | "console"
                    | "Number"
                    | "Math"
                    | "null"
                    | "typeof"
                    | "void"
            ) {
                return Err(Diagnostic::new(
                    "duplicate",
                    "Generic function name is unavailable.",
                    function.name.span,
                ));
            }
            crate::generic_validation::function(
                &mut function,
                &records,
                &program.call_type_arguments,
            )?;
            cache.templates.insert(function.name.text.clone(), function);
        }
    }
    cache.calls = program.call_type_arguments.iter()
        .map(|(span, args)| ((span.start, span.end), args.clone()))
        .collect();
    let output = cache.drain(true)?;
    program.functions = output;
    program.call_type_arguments.clear();
    program.type_argument_checks.extend(std::mem::take(&mut cache.argument_checks));
    // Preserve lexical templates, published explicit instances and consumed budgets.
    // Checked inference must extend this cache rather than starting a second one.
    Ok(cache)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_handoff_retains_templates_instances_and_consumed_budgets() {
        let source = "function identity<T>(value:T):T{return value;}function main(){identity<int>(7);identity<(int)>(8);identity<string>(clone(\"Pat\"));}";
        let mut program = crate::syntax::parse(source).unwrap();
        let declaration = program.functions[0].name.clone();
        let mut cache = specialize(&mut program).unwrap();
        cache.seed_checked(&program, &HashMap::new(), &crate::unions::UnionArena::default()).unwrap();
        assert_eq!(cache.checked_instances.len(), 2);
        assert!(cache.checked_instances.contains_key(&("identity".to_owned(), vec![Type::Int])));
        assert!(cache.checked_instances.contains_key(&("identity".to_owned(), vec![Type::String])));
        assert_eq!(cache.templates.len(), 1);
        let template = &cache.templates["identity"];
        assert_eq!(template.name.span, declaration.span);
        assert_eq!(template.type_parameters[0].text, "T");
        assert!(template.application.is_none());
        assert_eq!(cache.instances.len(), 2);
        assert_eq!(cache.counts["identity"], 2);
        assert!(cache.pending.is_empty());
        assert!(cache.argument_checks.is_empty());
        assert!(program.call_type_arguments.is_empty());
        assert_eq!(program.type_argument_checks.len(), 2);
        assert_eq!(program.functions.len(), 3);
        for concrete in cache.instances.values() {
            assert!(program.functions.iter().any(|function| {
                function.name.text == *concrete && function.application.is_some()
            }));
        }
    }
    #[test]
    fn checked_cache_preserves_declaration_and_nominal_identity() {
        let source = "type Left={value:int;};type Right={value:int;};type Box<T>={value:T;};function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function main(){identity<Box<Left>>({value:{value:1}});identity<Box<Right>>({value:{value:2}});other<Box<Left>>({value:{value:3}});identity<int|null>(7);identity<null|int>(8);}";
        let mut program = crate::syntax::parse(source).unwrap();
        let mut cache = specialize(&mut program).unwrap();
        crate::generic_records::specialize(&mut program).unwrap();
        let names = program.records.iter().enumerate().map(|(id, record)| {
            (record.name.text.clone(), id)
        }).collect();
        let unions = crate::unions::UnionArena::default();
        cache.seed_checked(&program, &names, &unions).unwrap();
        assert_eq!(cache.checked_instances.len(), 4);
        let nominal_arguments = cache.checked_instances.keys().filter_map(|(name, args)| {
            match args.as_slice() {
                [Type::Record(id)] if name == "identity" => Some(*id),
                _ => None,
            }
        }).collect::<std::collections::HashSet<_>>();
        assert_eq!(nominal_arguments.len(), 2, "equal-shaped nominal arguments stay distinct");
        assert_eq!(cache.checked_instances.keys().filter(|(name, _)| name == "other").count(), 1);
        assert_eq!(cache.checked_instances.keys().filter(|(_, args)| {
            matches!(args.as_slice(), [Type::Union(_)])
        }).count(), 1, "reordered nullable syntax shares checked identity");
    }

    #[test]
    fn retained_pending_drain_substitutes_recursive_explicit_calls() {
        let source = "function recursive<T>(value:T):T{return recursive<T>(value);}function main(){}";
        let mut program = crate::syntax::parse(source).unwrap();
        let mut cache = specialize(&mut program).unwrap();
        assert_eq!(program.functions.len(), 1);
        let annotation = crate::type_syntax::parse("int").unwrap();
        let name = cache.instance("recursive", vec![annotation], Span::new(0, 0), 1).unwrap();
        let output = cache.drain(true).unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(cache.instances.len(), 1);
        assert!(cache.pending.is_empty());
        let function = &output[0];
        assert_eq!(function.name.text, name);
        assert!(matches!(&function.params[0].1.kind, Kind::Named(ty) if ty.text == "int"));
        let StmtKind::Return(Some(expression)) = &function.body[0].kind else { panic!("return") };
        let ExprKind::Call(callee, arguments) = &expression.kind else { panic!("recursive call") };
        assert!(matches!(&callee.kind, ExprKind::Name(callee) if *callee == name));
        assert!(matches!(&arguments[0].kind, ExprKind::Name(argument) if argument == "value"));
        assert!(arguments[0].ty.is_none(), "materialization does not check argument expressions");
    }

}

#[cfg(test)]
#[test]
fn callable_payload_keys_normalize_equivalent_ownership_and_omission() {
    use crate::type_syntax::CallableOwnership;
    let program = crate::syntax::parse("function inspect(action:Function<int,[value?:int]>):void{}function main(){}").unwrap();
    let original = program.functions[0].params[0].1.clone();
    let mut equivalent = original.clone();
    let Kind::Callable { ownership, parameters, .. } = &mut equivalent.kind else { panic!("callable") };
    *ownership = CallableOwnership::Owned;
    parameters[0].name.text = "renamed".into();
    parameters[0].span = Span::default();
    parameters[0].omission = crate::ParameterOmission::Defaulted;
    assert_eq!(key(&original).unwrap(), key(&equivalent).unwrap());
    let Kind::Callable { ownership, .. } = &mut equivalent.kind else { panic!("callable") };
    *ownership = CallableOwnership::OwnedOnce;
    assert_ne!(key(&original).unwrap(), key(&equivalent).unwrap());
    let Kind::Callable { ownership, parameters, .. } = &mut equivalent.kind else { panic!("callable") };
    *ownership = CallableOwnership::Owned;
    parameters[0].omission = crate::ParameterOmission::Required;
    assert_ne!(key(&original).unwrap(), key(&equivalent).unwrap());
}
