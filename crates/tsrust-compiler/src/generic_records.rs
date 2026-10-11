//! Source record specialization. Concrete records enter the ordinary checker/emitter.
use crate::syntax::*;
use crate::type_syntax::{TypeName, ValueAnnotation, ValueAnnotationKind as Kind};
use crate::{Diagnostic, Span};
use std::collections::{BTreeMap, HashMap};
#[path = "result_types.rs"]
mod result_types;

#[derive(Default)]
pub(crate) struct Cache {
    templates: BTreeMap<String, Record>,
    instances: HashMap<(String, Vec<Type>), usize>,
    counts: HashMap<String, usize>,
}

impl Cache {
    pub(crate) fn declaration(&self, name: &str) -> Option<Span> {
        self.templates.get(name).map(|record| record.name.span)
    }

    pub(crate) fn materialize(
        &mut self,
        annotation: &mut ValueAnnotation,
        records: &mut Vec<Record>,
        names: &mut HashMap<String, usize>,
        unions: &crate::unions::UnionArena,
        callables: &mut crate::callable_resolution::Tables,
    ) -> Result<(), Diagnostic> {
        let mut context = Context {
            templates: std::mem::take(&mut self.templates),
            instances: std::mem::take(&mut self.instances),
            counts: std::mem::take(&mut self.counts),
            records: std::mem::take(records),
            names: std::mem::take(names),
            depth: 0,
            unions,
            callables,
        };
        let result = context.annotation(annotation, &HashMap::new());
        // Return all state on errors too; callers never lose existing checked records.
        *records = context.records;
        *names = context.names;
        self.templates = context.templates;
        self.instances = context.instances;
        self.counts = context.counts;
        result
    }
}

#[cfg(test)]
pub(crate) fn specialize(program: &mut Program) -> Result<(), Diagnostic> {
    specialize_in(program, &crate::unions::UnionArena::default(), &mut crate::callable_resolution::Tables::default()).map(|_| ())
}

pub(crate) fn specialize_in(
    program: &mut Program,
    unions: &crate::unions::UnionArena,
    callables: &mut crate::callable_resolution::Tables,
) -> Result<Cache, Diagnostic> {
    let mut context = Context {
        templates: BTreeMap::new(),
        records: Vec::new(),
        names: HashMap::new(),
        instances: HashMap::new(),
        counts: HashMap::new(),
        depth: 0,
        unions,
        callables,
    };
    let mut declarations = HashMap::new();
    for record in std::mem::take(&mut program.records) {
        if matches!(
            record.name.text.as_str(),
            "int" | "number" | "boolean" | "string" | "null" | "undefined" | "void" | "Set" | "Map" | "Result"
        ) {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type {:?} is already defined.", record.name.text),
                record.name.span,
            ));
        }
        if declarations
            .insert(record.name.text.clone(), record.name.span)
            .is_some()
        {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Type {:?} is already defined.", record.name.text),
                record.name.span,
            ));
        }
        if record.type_parameters.is_empty() {
            context
                .names
                .insert(record.name.text.clone(), context.records.len());
            context.records.push(record);
        } else {
            context.templates.insert(record.name.text.clone(), record);
        }
    }
    for template in context.templates.values() {
        let mut fields = HashMap::new();
        for field in &template.fields {
            if fields.insert(&field.name.text, field.name.span).is_some() {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Field {:?} is already defined.", field.name.text),
                    field.name.span,
                ));
            }
            context.validate_template(&field.annotation, &template.type_parameters)?;
        }
    }
    unions.register_named_types(&context.records);
    let initial = context.records.len();
    for index in 0..initial {
        let mut fields = std::mem::take(&mut context.records[index].fields);
        for field in &mut fields {
            context.annotation(&mut field.annotation, &HashMap::new())?;
        }
        context.records[index].fields = fields;
    }
    for argument in &mut program.type_argument_checks { context.annotation(argument, &HashMap::new())?; }
    for function in &mut program.functions {
        if !function.type_parameters.is_empty() {
            continue;
        }
        if let Some(application) = &mut function.application {
            for argument in &mut application.arguments { context.annotation(argument,&HashMap::new())?; }
        }
        for (_, annotation, callback) in &mut function.params {
            context.annotation(annotation, &HashMap::new())?;
            if let Some(callback) = callback {
                context.callback(callback)?;
            }
        }
        if let Some(result) = &mut function.result {
            context.result(result)?;
        }
        context.body(&mut function.body, &mut function.parameter_initializers)?;
    }
    program.records = context.records;
    Ok(Cache { templates: context.templates, instances: context.instances, counts: context.counts })
}

struct Context<'a> {
    templates: BTreeMap<String, Record>,
    records: Vec<Record>,
    names: HashMap<String, usize>,
    instances: HashMap<(String, Vec<Type>), usize>,
    counts: HashMap<String, usize>,
    depth: usize,
    unions: &'a crate::unions::UnionArena,
    callables: &'a mut crate::callable_resolution::Tables,
}
impl Context<'_> {
    fn validate_template(
        &self,
        value: &ValueAnnotation,
        parameters: &[TypeName],
    ) -> Result<(), Diagnostic> {
        match &value.kind {
            Kind::Polymorphic { .. } => Ok(()),
            Kind::Named(name) => {
                if parameters
                    .iter()
                    .any(|parameter| parameter.text == name.text)
                    || self.names.contains_key(&name.text)
                    || matches!(
                        name.text.as_str(),
                        "int" | "number" | "boolean" | "string" | "null" | "undefined"
                    )
                {
                    return Ok(());
                }
                if self.templates.contains_key(&name.text) {
                    return Err(Diagnostic::new(
                        "arity",
                        "Generic record types require explicit type arguments.",
                        name.span,
                    ));
                }
                Err(Diagnostic::new(
                    "type",
                    "Unknown or unsupported type in generic record declaration.",
                    name.span,
                ))
            }
            Kind::Group(inner) | Kind::Array(inner) => self.validate_template(inner, parameters),
            Kind::Union(members) | Kind::Tuple(members) => {
                for member in members {
                    self.validate_template(member, parameters)?;
                }
                Ok(())
            }
            Kind::Applied { name, arguments } => {
                if name.text == "Result" {
                    if arguments.len() != 2 { return Err(Diagnostic::new("arity", "Result requires success and error types.", value.span)); }
                    for argument in arguments {
                        if !matches!(&argument.kind, Kind::Named(name) if name.text == "void") {
                            self.validate_template(argument, parameters)?;
                        }
                    }
                    return Ok(());
                }
                if name.text == "Map" {
                    if arguments.len() != 2 { return Err(Diagnostic::new("arity", "Map requires key and value types.", value.span)); }
                    for argument in arguments { self.validate_template(argument, parameters)?; }
                    return Ok(());
                }
                if name.text == "Set" {
                    if arguments.len() != 1 { return Err(Diagnostic::new("arity", "Set requires one key type.", value.span)); }
                    return self.validate_template(&arguments[0], parameters);
                }
                if name.text == "@testing:element" {
                    if arguments.len() != 1 { return Err(Diagnostic::new("arity", "Internal containment element projection requires one type argument.", value.span)); }
                    return self.validate_template(&arguments[0], parameters);
                }
                if parameters
                    .iter()
                    .any(|parameter| parameter.text == name.text)
                {
                    return Err(Diagnostic::new(
                        "type",
                        "A type parameter cannot be used as a generic type constructor.",
                        name.span,
                    ));
                }
                let Some(template) = self.templates.get(&name.text) else {
                    return Err(Diagnostic::new(
                        "type",
                        "Unknown generic record type.",
                        name.span,
                    ));
                };
                if template.type_parameters.len() != arguments.len() {
                    return Err(Diagnostic::new(
                        "arity",
                        "Wrong number of generic record type arguments.",
                        value.span,
                    ));
                }
                for argument in arguments {
                    self.validate_template(argument, parameters)?;
                }
                Ok(())
            }
            Kind::Callable { parameters: arguments, result, .. } => {
                for argument in arguments {
                    self.validate_template(&argument.annotation, parameters)?;
                }
                let mut result = result.as_ref();
                while let Kind::Group(inner) = &result.kind { result = inner; }
                if !matches!(&result.kind, Kind::Named(name) if name.text == "void") {
                    self.validate_template(result, parameters)?;
                }
                Ok(())
            },
        }
    }

    fn annotation(
        &mut self,
        value: &mut ValueAnnotation,
        bindings: &HashMap<String, ValueAnnotation>,
    ) -> Result<(), Diagnostic> {
        if !bindings.is_empty() {
            crate::type_substitution::apply(value, bindings)?;
            return self.annotation(value, &HashMap::new());
        }
        if matches!(&value.kind, Kind::Applied { name, .. } if name.text == "Result") {
            return self.result_annotation(value);
        }
        match &mut value.kind {
            Kind::Polymorphic { .. } => {},
            Kind::Named(name) => {
                if name.text == "Result" {
                    return Err(Diagnostic::new("arity", "Result requires success and error types, for example Result<int, string>.", value.span));
                } else if let Some(argument) = bindings.get(&name.text) {
                    *value = argument.clone();
                } else if self.templates.contains_key(&name.text) {
                    return Err(Diagnostic::new(
                        "arity",
                        "Generic record types require explicit type arguments.",
                        value.span,
                    ));
                }
            }
            Kind::Group(inner) | Kind::Array(inner) => self.annotation(inner, bindings)?,
            Kind::Union(members) => {
                for member in members {
                    self.annotation(member, bindings)?;
                }
            }
            Kind::Tuple(positions) => {
                for position in positions.iter_mut() { self.annotation(position, bindings)?; }
                let types = positions.iter().map(|position| crate::analysis::resolve_value_type(position, false, &self.names, self.unions)).collect::<Result<Vec<_>, _>>()?;
                let label = format!("[{}]", types.iter().map(|ty|self.label(*ty)).collect::<Vec<_>>().join(","));
                let id = if let Some(&id) = self.names.get(&label) { id } else {
                    let id = self.records.len();
                    let fields = positions.iter().enumerate().map(|(index, annotation)| RecordField {
                        name: Token { kind: TokenKind::Symbol, text: index.to_string(), span: annotation.span },
                        annotation: annotation.clone(), ty: None, callable_declaration: None, optional: false,
                    }).collect();
                    self.records.push(Record { native_type: None, map_definition: None, set_definition: None, enum_definition: None, application: None, name: Token { kind: TokenKind::Symbol, text:label.clone(), span:value.span }, type_parameters: Vec::new(), fields });
                    self.names.insert(label.clone(), id);
                    id
                };
                value.kind = Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span });
            }
            Kind::Callable {
                parameters, result, ..
            } => {
                for parameter in parameters {
                    self.annotation(&mut parameter.annotation, bindings)?;
                }
                self.annotation(result, bindings)?;
            }
            Kind::Applied { name, arguments } => {
                if name.text == "Map" {
                    if arguments.len() != 2 { return Err(Diagnostic::new("arity", "Map requires key and value types.", value.span)); }
                    for argument in arguments.iter_mut() { self.annotation(argument, bindings)?; }
                    let key = crate::analysis::resolve_value_type(&arguments[0], false, &self.names, self.unions)?;
                    let result = crate::analysis::resolve_value_type(&arguments[1], false, &self.names, self.unions)?;
                    let id = self.map_instance(key, result, arguments, value.span)?;
                    value.kind = Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span });
                    return Ok(());
                }
                if name.text == "Set" {
                    if arguments.len() != 1 { return Err(Diagnostic::new("arity", "Set requires one key type.", value.span)); }
                    self.annotation(&mut arguments[0], bindings)?;
                    let key = crate::analysis::resolve_value_type(&arguments[0], false, &self.names, self.unions)?;
                    let id = self.set_instance(key, arguments, value.span)?;
                    value.kind = Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span });
                    return Ok(());
                }
                if bindings.contains_key(&name.text) {
                    return Err(Diagnostic::new(
                        "type",
                        "A type parameter cannot be used as a generic type constructor.",
                        name.span,
                    ));
                }
                let Some(template) = self.templates.get(&name.text) else {
                    return Err(Diagnostic::new(
                        "type",
                        "Unknown generic record type.",
                        name.span,
                    ));
                };
                if arguments.len() != template.type_parameters.len() {
                    return Err(Diagnostic::new(
                        "arity",
                        format!(
                            "Generic record expects {} type arguments, received {}.",
                            template.type_parameters.len(),
                            arguments.len()
                        ),
                        value.span,
                    ));
                }
                for argument in arguments.iter_mut() {
                    self.annotation(argument, bindings)?;
                }
                let types = arguments
                    .iter()
                    .map(|argument| {
                        crate::analysis::resolve_value_type(
                            argument,
                            false,
                            &self.names,
                            self.unions,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let index = self.instance(&name.text, arguments, types, value.span)?;
                value.kind = Kind::Named(TypeName {
                    text: self.records[index].name.text.clone(),
                    span: value.span,
                });
            }
        }
        Ok(())
    }
    #[inline(never)]
    fn set_instance(&mut self, key: Type, arguments: &[ValueAnnotation], span: Span) -> Result<usize, Diagnostic> {
        let members = key.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![key]);
        if !members.iter().all(|member| matches!(member, Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined | Type::String | Type::Enum(_, _))) {
            return Err(Diagnostic::new("set-key", "Set keys require numeric, boolean, string, null, undefined, or enum values; owned object identity is not supported.", span));
        }
        let label = format!("Set<{}>", self.label(key));
        if let Some(&id) = self.names.get(&label) { return Ok(id); }
        let id = self.records.len();
        self.records.push(Record { native_type: None, map_definition: None, set_definition: Some(SetDefinition { key }), enum_definition: None,
            name: Token { text: label.clone(), span, kind: TokenKind::Symbol }, type_parameters: Vec::new(), fields: Vec::new(),
            application: Some(Box::new(RecordApplication { resolved_arguments: vec![key], declaration: Token { text: "Set".into(), span, kind: TokenKind::Name }, arguments: arguments.to_vec() })) });
        self.names.insert(label, id);
        self.unions.register_set_type(id);
        Ok(id)
    }
    #[inline(never)]
    fn map_instance(&mut self, key: Type, value: Type, arguments: &[ValueAnnotation], span: Span) -> Result<usize, Diagnostic> {
        let members = key.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![key]);
        if !members.iter().all(|member| matches!(member, Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined | Type::String | Type::Enum(_, _))) {
            return Err(Diagnostic::new("map-key", "Map keys require scalar or enum values; owned object identity is unsupported.", span));
        }
        let label = format!("Map<{}, {}>", self.label(key), self.label(value));
        if let Some(&id) = self.names.get(&label) { return Ok(id); }
        let id = self.records.len();
        self.records.push(Record { native_type: None, map_definition: Some(MapDefinition { key, value, entry: None }), set_definition: None, enum_definition: None,
            name: Token { text: label.clone(), span, kind: TokenKind::Symbol }, type_parameters: Vec::new(), fields: Vec::new(),
            application: Some(Box::new(RecordApplication { resolved_arguments: vec![key, value], declaration: Token { text: "Map".into(), span, kind: TokenKind::Name }, arguments: arguments.to_vec() })) });
        self.names.insert(label, id); self.unions.register_map_type(id); Ok(id)
    }
    fn instance(
        &mut self,
        name: &str,
        arguments: &[ValueAnnotation],
        types: Vec<Type>,
        span: Span,
    ) -> Result<usize, Diagnostic> {
        let key = (name.to_owned(), types.clone());
        if let Some(&index) = self.instances.get(&key) {
            return Ok(index);
        }
        if self.depth >= 32
            || self.instances.len() >= 1024
            || self.counts.get(name).copied().unwrap_or(0) >= 128
        {
            return Err(Diagnostic::new(
                "generic-limit",
                "Generic record specialization exceeds its depth (32), per-declaration (128), or program (1024) limit.",
                span,
            ));
        }
        let template = self.templates[name].clone();
        let index = self.records.len();
        let label = format!(
            "{}<{}>",
            name,
            types
                .iter()
                .map(|ty| self.label(*ty))
                .collect::<Vec<_>>()
                .join(",")
        );
        let label = if label.len() > 512 {
            format!("{name}<...>#{index}")
        } else {
            label
        };
        self.names.insert(label.clone(), index);
        self.instances.insert(key, index);
        *self.counts.entry(name.to_owned()).or_default() += 1;
        self.records.push(Record {
            native_type: template.native_type, map_definition: None, set_definition: None,                enum_definition: None,
            application: Some(Box::new(RecordApplication { resolved_arguments: types, declaration: template.name.clone(), arguments: arguments.to_vec() })),
            name: Token {
                text: label,
                ..template.name.clone()
            },
            type_parameters: Vec::new(),
            fields: Vec::new(),
        });
        let bindings = template
            .type_parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.text.clone(), argument.clone()))
            .collect();
        self.depth += 1;
        let mut fields = template.fields;
        for field in &mut fields {
            self.annotation(&mut field.annotation, &bindings)?;
        }
        self.depth -= 1;
        self.records[index].fields = fields;
        Ok(index)
    }
    fn label(&self, ty: Type) -> String {
        match ty {
            Type::Int => "int".into(),
            Type::Number => "number".into(),
            Type::Boolean => "boolean".into(),
            Type::String => "string".into(),
            Type::Null => "null".into(),
            Type::Undefined => "undefined".into(),
            Type::Record(id) | Type::Enum(id, _) | Type::Set(id) | Type::Map(id) => self.records[id].name.text.clone(),
            Type::Union(id) => self
                .unions
                .members(id)
                .into_iter()
                .map(|member| self.label(member))
                .collect::<Vec<_>>()
                .join("|"),
            Type::Array(element) => {
                let base = match element {
                    ArrayElement::Int(_) => "int".into(),
                    ArrayElement::Number(_) => "number".into(),
                    ArrayElement::Boolean(_) => "boolean".into(),
                    ArrayElement::String(_) => "string".into(),
                    ArrayElement::Null(_) => "null".into(),
                    ArrayElement::Undefined(_) => "undefined".into(),
                    ArrayElement::Record(id, _) | ArrayElement::Enum(id, _, _) | ArrayElement::Set(id, _) | ArrayElement::Map(id, _) => self.records[id].name.text.clone(),
                    ArrayElement::StoredCallable(id, _) => format!("callable{id}"),
                    ArrayElement::Union(id, _) => format!("({})", self.label(Type::Union(id))),
                };
                format!("{base}{}", "[]".repeat(element.depth() as usize + 1))
            }
            _ => unreachable!("validated value type argument"),
        }
    }
    fn parameter(&mut self, parameter: &mut CallableParameter) -> Result<(), Diagnostic> {
        match &mut parameter.ty {
            ParameterAnnotation::Named(value) => self.annotation(value, &HashMap::new()),
            ParameterAnnotation::Callback(callback) => self.callback(callback),
            ParameterAnnotation::Inferred(_) => Ok(()),
        }
    }
    fn callback(&mut self, callback: &mut CallbackAnnotation) -> Result<(), Diagnostic> {
        for parameter in &mut callback.params {
            self.parameter(parameter)?;
        }
        self.result(&mut callback.result)
    }
    fn result(&mut self, result: &mut ResultAnnotation) -> Result<(), Diagnostic> {
        match result {
            ResultAnnotation::Inferred(_) => Ok(()),
            ResultAnnotation::Named(value) => self.annotation(value, &HashMap::new()),
            ResultAnnotation::Callable { params, result, .. } => {
                for parameter in params {
                    self.parameter(parameter)?;
                }
                self.result(result)
            }
        }
    }
    fn body(&mut self, body: &mut [Stmt], initializers: &mut [ParameterInitializer]) -> Result<(), Diagnostic> {
        enum Work<'a> {
            Statement(&'a mut Stmt),
            Expression(&'a mut Expr),
        }
        let mut work = body.iter_mut().map(Work::Statement).collect::<Vec<_>>();
        for initializer in initializers {if let ParameterInitializer::Default{value,..}=initializer{work.push(Work::Expression(value));}}
        while let Some(node) = work.pop() {
            match node {
                Work::Statement(statement) => match &mut statement.kind {
                    StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                    StmtKind::Try(region) => work.extend(region.statements_mut().map(Work::Statement)),
                    StmtKind::Throw(value) => work.push(Work::Expression(value)),
                    StmtKind::Let {
                        annotation, value, ..
                    } => {
                        if let Some(annotation) = annotation {
                            self.annotation(annotation, &HashMap::new())?;
                        }
                        work.push(Work::Expression(value));
                    }
                    StmtKind::Block(body) => work.extend(body.iter_mut().map(Work::Statement)),
                    StmtKind::If {
                        condition,
                        then_body,
                        else_body,
                    } => {
                        work.push(Work::Expression(condition));
                        work.extend(then_body.iter_mut().chain(else_body).map(Work::Statement));
                    }
                    StmtKind::While { condition, body } => {
                        work.push(Work::Expression(condition));
                        work.extend(body.iter_mut().map(Work::Statement));
                    }
                    StmtKind::Loop {
                        condition,
                        body,
                        update,
                        ..
                    } => {
                        if let Some(condition) = condition {
                            work.push(Work::Expression(condition));
                        }
                        work.extend(body.iter_mut().chain(update).map(Work::Statement));
                    }
                    StmtKind::ForOf { iterable, body, .. } => {
                        work.push(Work::Expression(iterable));
                        work.extend(body.iter_mut().map(Work::Statement));
                    }
                    StmtKind::Assign { target, value, .. } => {
                        work.push(Work::Expression(target));
                        work.push(Work::Expression(value));
                    }
                    StmtKind::Return(Some(value)) | StmtKind::Expr(value) => {
                        work.push(Work::Expression(value))
                    }
                    StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {}
                },
                Work::Expression(expression) => match &mut expression.kind {
                    ExprKind::SpreadCall(plan) => work.extend(plan.expressions_mut().map(Work::Expression)),
                    ExprKind::SpreadValue(_) => (),
                    ExprKind::Closure {
                        params,
                        result,
                        body,
                        ..
                    } => {
                        for parameter in params {
                            self.parameter(parameter)?;
                            if let ParameterInitializer::Default{value,..}=&mut parameter.initializer {work.push(Work::Expression(value));}
                        }
                        self.result(result)?;
                        work.extend(body.iter_mut().map(Work::Statement));
                    }
                    ExprKind::Conditional {
                        predicate,
                        consequent,
                        alternate,
                    } => {
                        work.push(Work::Expression(predicate));
                        work.push(Work::Expression(consequent));
                        work.push(Work::Expression(alternate));
                    }
                    ExprKind::Optional {
                        base: left,
                        body: right,
                        ..
                    }
                    | ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => {
                        work.push(Work::Expression(left));
                        work.push(Work::Expression(right));
                    }
                    ExprKind::CompoundAssign { target, value, .. }
                    | ExprKind::Assign { target, value, .. }
                    | ExprKind::CoalesceAssign { target, value, .. }
                    | ExprKind::LogicalAssign { target, value, .. } => {
                        work.push(Work::Expression(target));
                        work.push(Work::Expression(value));
                    }
                    ExprKind::OptionalChain(value)
                    | ExprKind::Unary { value, .. }
                    | ExprKind::Update { target: value, .. }
                    | ExprKind::Field(value, _)
                    | ExprKind::Inject { value, .. }
                    | ExprKind::Project { value, .. } => work.push(Work::Expression(value)),
                    ExprKind::Record(fields) => {
                        work.extend(fields.iter_mut().map(|(_, value)| Work::Expression(value)))
                    }
                    ExprKind::TypeAssertion {value,annotation,..} => {
                        self.annotation(annotation,&HashMap::new())?;
                        work.push(Work::Expression(value));
                    }
                    ExprKind::ArrayOf(values, element) | ExprKind::SetConstructor(values, element) | ExprKind::MapConstructor(values, element) | ExprKind::NativeConstructor(values, element) => {
                        if let Some(element) = element { self.annotation(element, &HashMap::new())?; }
                        work.extend(values.iter_mut().map(Work::Expression));
                    }
                    ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                        work.extend(values.iter_mut().map(Work::Expression))
                    }
                    ExprKind::Call(callee, arguments) => {
                        work.push(Work::Expression(callee));
                        work.extend(arguments.iter_mut().map(Work::Expression));
                    }
                    ExprKind::Selected { .. }
                    | ExprKind::Name(_)
                    | ExprKind::String(_)
                    | ExprKind::Integer(_)
                    | ExprKind::EnumMember { .. } | ExprKind::Number(_)
                    | ExprKind::Boolean(_)
                    | ExprKind::Null
                    | ExprKind::Undefined => {}
                },
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_materialization_appends_without_changing_record_or_union_ids() {
        let mut program = crate::syntax::parse("type Original={value:int|null;};type Box<T>={value:T;};type Tree<T>={value:T;children:Tree<T>[];};type Holder={value:Box<int|null>;};function main(){}").unwrap();
        let unions = crate::unions::UnionArena::default();
        let mut callables = crate::callable_resolution::Tables::default();
        let mut cache = specialize_in(&mut program, &unions, &mut callables).unwrap();
        let mut names = program.records.iter().enumerate().map(|(id, record)| {
            (record.name.text.clone(), id)
        }).collect::<HashMap<_, _>>();
        let mut fields = Vec::new();
        crate::analysis::append_checked_record_fields(&mut program.records, &names, &unions, &mut fields, &mut crate::callable_resolution::Tables::default()).unwrap();
        let checked_before = fields.clone();
        let original = program.records.iter().map(|record| {
            (record.name.text.clone(), record.fields.iter().map(|field| field.annotation.clone()).collect::<Vec<_>>())
        }).collect::<Vec<_>>();
        let before = crate::analysis::resolve_value_type(&program.records[0].fields[0].annotation, false, &names, &unions).unwrap();
        let initial = program.records.len();
        for source in ["Box<string|null>", "Box<(null|string)>"] {
            let mut annotation = crate::type_syntax::parse(source).unwrap();
            cache.materialize(&mut annotation, &mut program.records, &mut names, &unions, &mut callables).unwrap();
            assert_eq!(crate::analysis::resolve_value_type(&annotation, false, &names, &unions).unwrap(), Type::Record(initial));
        }
        assert_eq!(program.records.len(), initial + 1);
        assert_eq!(cache.counts["Box"], 2);
        let mut nested = crate::type_syntax::parse("Box<Box<int>>").unwrap();
        cache.materialize(&mut nested, &mut program.records, &mut names, &unions, &mut callables).unwrap();
        assert_eq!(program.records.len(), initial + 3);
        let mut tree = crate::type_syntax::parse("Tree<int>").unwrap();
        cache.materialize(&mut tree, &mut program.records, &mut names, &unions, &mut callables).unwrap();
        assert_eq!(program.records.len(), initial + 4);
        assert_eq!(cache.counts["Tree"], 1, "recursive array fields reuse the published instance");
        crate::analysis::append_checked_record_fields(&mut program.records, &names, &unions, &mut fields, &mut crate::callable_resolution::Tables::default()).unwrap();
        assert_eq!(&fields[..initial], checked_before.as_slice());
        assert_eq!(fields.len(), program.records.len());
        assert!(program.records.iter().flat_map(|record| &record.fields).all(|field| field.ty.is_some()));
        let tree_id = initial + 3;
        let children = crate::analysis::resolve_value_type(&program.records[tree_id].fields[1].annotation, false, &names, &unions).unwrap();
        assert!(matches!(children, Type::Array(element) if element.ty() == Type::Record(tree_id)));
        for (id, (name, fields)) in original.iter().enumerate() {
            assert_eq!(&program.records[id].name.text, name);
            assert_eq!(names[name], id);
            assert_eq!(program.records[id].fields.iter().map(|field| field.annotation.clone()).collect::<Vec<_>>(), *fields);
        }
        assert_eq!(crate::analysis::resolve_value_type(&program.records[0].fields[0].annotation, false, &names, &unions).unwrap(), before);
        let length = program.records.len();
        let mut invalid = crate::type_syntax::parse("Box<int,string>").unwrap();
        assert_eq!(cache.materialize(&mut invalid, &mut program.records, &mut names, &unions, &mut callables).unwrap_err().code, "arity");
        assert_eq!(program.records.len(), length);
        assert_eq!(names["Original"], 0);
    }
    #[test]
    fn appended_fields_use_ordinary_recursive_layout_rejection() {
        let mut program = crate::syntax::parse("type Original={value:int;};type Bad<T>={value:Bad<T>;};function main(){}").unwrap();
        let unions = crate::unions::UnionArena::default();
        let mut callables = crate::callable_resolution::Tables::default();
        let mut cache = specialize_in(&mut program, &unions, &mut callables).unwrap();
        let mut names = program.records.iter().enumerate().map(|(id, record)| (record.name.text.clone(), id)).collect();
        let mut fields = Vec::new();
        crate::analysis::append_checked_record_fields(&mut program.records, &names, &unions, &mut fields, &mut crate::callable_resolution::Tables::default()).unwrap();
        let mut annotation = crate::type_syntax::parse("Bad<int>").unwrap();
        cache.materialize(&mut annotation, &mut program.records, &mut names, &unions, &mut callables).unwrap();
        let error = crate::analysis::append_checked_record_fields(&mut program.records, &names, &unions, &mut fields, &mut crate::callable_resolution::Tables::default()).unwrap_err();
        assert_eq!(error.code, "recursive-layout");
        assert_eq!(fields[0]["value"], Type::Int);
        assert_eq!(names["Original"], 0);
        assert_eq!(program.records.len(), 2);
    }

    #[test]
    fn late_materialization_preserves_consumed_declaration_budget() {
        let mut source = "type Box<T>={value:T;};".to_owned();
        for id in 0..128 {
            source.push_str(&format!("type Argument{id}={{value:int;}};type Holder{id}={{value:Box<Argument{id}>;}};"));
        }
        source.push_str("function main(){}");
        let mut program = crate::syntax::parse(&source).unwrap();
        let unions = crate::unions::UnionArena::default();
        let mut callables = crate::callable_resolution::Tables::default();
        let mut cache = specialize_in(&mut program, &unions, &mut callables).unwrap();
        let mut names = program.records.iter().enumerate().map(|(id, record)| (record.name.text.clone(), id)).collect();
        let initial = program.records.len();
        let mut existing = crate::type_syntax::parse("Box<Argument0>").unwrap();
        cache.materialize(&mut existing, &mut program.records, &mut names, &unions, &mut callables).unwrap();
        assert_eq!(program.records.len(), initial);
        let mut next = crate::type_syntax::parse("Box<int>").unwrap();
        let error = cache.materialize(&mut next, &mut program.records, &mut names, &unions, &mut callables).unwrap_err();
        assert_eq!(error.code, "generic-limit");
        assert_eq!(cache.counts["Box"], 128);
        assert_eq!(program.records.len(), initial);
    }

}
