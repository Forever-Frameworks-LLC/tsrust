//! Heap continuations for generic annotation and record-field materialization.
use super::*;

struct Expansion { index: usize, fields: Vec<RecordField>, bindings: HashMap<String, ValueAnnotation> }

impl Context<'_> {
    fn finish_annotation(&mut self, value: &mut ValueAnnotation) -> Result<Option<Expansion>, Diagnostic> {
        match &mut value.kind {
            Kind::Literal(literal) => {
                let definition=crate::literal_types::LiteralValue::checked(literal,value.span)?;
                let name=definition.key();let kind=definition.kind();
                let id=if let Some(&id)=self.names.get(&name){id}else{
                    let id=self.records.len();self.names.insert(name.clone(),id);
                    self.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:Some(definition),application:None,name:Token{text:name,span:value.span,kind:TokenKind::Symbol},type_parameters:Vec::new(),fields:Vec::new()});id
                };
                self.unions.register_literal_type(id,kind);
            },
            Kind::Polymorphic { .. } => {},
            Kind::Named(name) => {
                if name.text == "Result" {
                    return Err(Diagnostic::new("arity", "Result requires success and error types, for example Result<int, string>.", value.span));
                } else if self.templates.contains_key(&name.text) {
                    return Err(Diagnostic::new(
                        "arity",
                        "Generic record types require explicit type arguments.",
                        value.span,
                    ));
                }
            }
            Kind::Group(_) | Kind::Array(_) | Kind::Union(_) | Kind::Callable { .. } => {},
            Kind::Tuple(positions) => {
                let types = positions.iter().map(|position| crate::analysis::resolve_value_type(position, false, &self.names, self.unions)).collect::<Result<Vec<_>, _>>()?;
                let label = format!("[{}]", types.iter().map(|ty|self.label(*ty)).collect::<Vec<_>>().join(","));
                let id = if let Some(&id) = self.names.get(&label) { id } else {
                    let id = self.records.len();
                    let fields = positions.iter().enumerate().map(|(index, annotation)| RecordField {
                        name: Token { kind: TokenKind::Symbol, text: index.to_string(), span: annotation.span },
                        annotation: annotation.clone(), ty: None, callable_declaration: None, optional: false,
                    }).collect();
                    self.records.push(Record { native_type: None, map_definition: None, set_definition: None, enum_definition: None,literal_definition:None, application: None, name: Token { kind: TokenKind::Symbol, text:label.clone(), span:value.span }, type_parameters: Vec::new(), fields });
                    self.names.insert(label.clone(), id);
                    id
                };
                value.kind = Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span });
            }
            Kind::Applied { name, arguments } => {
                if name.text == "Result" { self.finish_result_annotation(value)?; return Ok(None); }
                if name.text == "Map" {
                    if arguments.len() != 2 { return Err(Diagnostic::new("arity", "Map requires key and value types.", value.span)); }
                    let key = crate::analysis::resolve_value_type(&arguments[0], false, &self.names, self.unions)?;
                    let result = crate::analysis::resolve_value_type(&arguments[1], false, &self.names, self.unions)?;
                    let id = self.map_instance(key, result, arguments, value.span)?;
                    value.kind = Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span });
                    return Ok(None);
                }
                if name.text == "Set" {
                    if arguments.len() != 1 { return Err(Diagnostic::new("arity", "Set requires one key type.", value.span)); }
                    let key = crate::analysis::resolve_value_type(&arguments[0], false, &self.names, self.unions)?;
                    let id = self.set_instance(key, arguments, value.span)?;
                    value.kind = Kind::Named(TypeName { text: self.records[id].name.text.clone(), span: value.span });
                    return Ok(None);
                }
                let types = arguments
                    .iter().enumerate()
                    .map(|(position, argument)| {
                        crate::analysis::resolve_value_type(
                            argument,
                            position == 0 && self.templates.get(&name.text).is_some_and(|record| matches!(record.native_type, Some(crate::native_signatures::NativeTypeId::Thread | crate::native_signatures::NativeTypeId::ScopedThread))),
                            &self.names,
                            self.unions,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let (index, fields) = self.begin_instance(&name.text, arguments, types, value.span)?;
                if let Some((fields, bindings)) = fields { return Ok(Some(Expansion { index, fields, bindings })); }
                value.kind = Kind::Named(TypeName {
                    text: self.records[index].name.text.clone(),
                    span: value.span,
                });
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Copy)]
enum Child { Inner, Member(usize), Parameter(usize), Result }
enum Frame {
    Node { parent: ValueAnnotation, child: Child },
    Record { application: ValueAnnotation, expansion: Expansion, field: usize },
}
#[derive(Clone, Copy)]
enum Phase { Enter, Finish, Return }

fn take(value: &mut ValueAnnotation) -> ValueAnnotation {
    let span = value.span;
    std::mem::replace(value, ValueAnnotation { kind: Kind::Tuple(Vec::new()), span })
}
fn child(value: &mut ValueAnnotation, path: Child) -> &mut ValueAnnotation {
    match (&mut value.kind, path) {
        (Kind::Group(inner) | Kind::Array(inner), Child::Inner) => inner,
        (Kind::Union(values) | Kind::Tuple(values) | Kind::Applied { arguments: values, .. }, Child::Member(index)) => &mut values[index],
        (Kind::Callable { parameters, .. }, Child::Parameter(index)) => &mut parameters[index].annotation,
        (Kind::Callable { result, .. }, Child::Result) => result,
        _ => unreachable!("materialization child address"),
    }
}
fn next_child(value: &ValueAnnotation, previous: Option<Child>) -> Option<Child> {
    match (&value.kind, previous) {
        (Kind::Group(_) | Kind::Array(_), None) => Some(Child::Inner),
        (Kind::Union(values) | Kind::Tuple(values) | Kind::Applied { arguments: values, .. }, previous) => {
            let index = match previous { None => 0, Some(Child::Member(index)) => index + 1, _ => unreachable!() };
            (index < values.len()).then_some(Child::Member(index))
        }
        (Kind::Callable { parameters, .. }, previous) => {
            let index = match previous { None => 0, Some(Child::Parameter(index)) => index + 1, Some(Child::Result) => return None, _ => unreachable!() };
            Some(if index < parameters.len() { Child::Parameter(index) } else { Child::Result })
        }
        _ => None,
    }
}
fn descend(current: &mut ValueAnnotation, path: Child, frames: &mut Vec<Frame>) {
    let mut parent = take(current);
    *current = take(child(&mut parent, path));
    frames.push(Frame::Node { parent, child: path });
}

impl Context<'_> {
    fn enter_annotation(&self, value: &ValueAnnotation) -> Result<(), Diagnostic> {
        let Kind::Applied { name, arguments } = &value.kind else { return Ok(()); };
        let arity_error = match name.text.as_str() {
            "Result" => {
                (arguments.len() != 2).then_some("Result requires success and error types.")
            }
            "Map" => (arguments.len() != 2).then_some("Map requires key and value types."),
            "Set" => (arguments.len() != 1).then_some("Set requires one key type."),
            _ => {
                let Some(template) = self.templates.get(&name.text) else {
                    return Err(Diagnostic::new("type", "Unknown generic record type.", name.span));
                };
                if arguments.len() != template.type_parameters.len() {
                    return Err(Diagnostic::new("arity", format!("Generic record expects {} type arguments, received {}.", template.type_parameters.len(), arguments.len()), value.span));
                }
                None
            }
        };
        if let Some(message) = arity_error { return Err(Diagnostic::new("arity", message, value.span)); }
        Ok(())
    }

    pub(super) fn annotation(&mut self, value: &mut ValueAnnotation, bindings: &HashMap<String, ValueAnnotation>) -> Result<(), Diagnostic> {
        crate::type_substitution::apply(value, bindings)?;
        let mut current = take(value);
        let mut frames = Vec::new();
        let mut phase = Phase::Enter;
        let outcome = loop {
            let step = (|| -> Result<bool, Diagnostic> {
                match phase {
                    Phase::Enter => {
                        self.enter_annotation(&current)?;
                        if let Some(path) = next_child(&current, None) {
                            descend(&mut current, path, &mut frames);
                        } else { phase = Phase::Finish; }
                    }
                    Phase::Finish => {
                        if let Some(mut expansion) = self.finish_annotation(&mut current)? {
                            if expansion.fields.is_empty() {
                                current.kind = Kind::Named(TypeName { text: self.records[expansion.index].name.text.clone(), span: current.span });
                                phase = Phase::Return;
                            } else {
                                let application = take(&mut current);
                                current = take(&mut expansion.fields[0].annotation);
                                frames.push(Frame::Record { application, expansion, field: 0 });
                                let Frame::Record { expansion, .. } = frames.last().unwrap() else { unreachable!() };
                                crate::type_substitution::apply(&mut current, &expansion.bindings)?;
                                phase = Phase::Enter;
                            }
                        } else {
                            phase = Phase::Return;
                        }
                    }
                    Phase::Return => {
                        let Some(frame) = frames.pop() else { return Ok(true); };
                        match frame {
                            Frame::Node { mut parent, child: path } => {
                                *child(&mut parent, path) = take(&mut current);
                                current = parent;
                                if let Some(next) = next_child(&current, Some(path)) {
                                    descend(&mut current, next, &mut frames);
                                    phase = Phase::Enter;
                                } else { phase = Phase::Finish; }
                            }
                            Frame::Record { application, mut expansion, field } => {
                                expansion.fields[field].annotation = take(&mut current);
                                let next = field + 1;
                                if next < expansion.fields.len() {
                                    current = take(&mut expansion.fields[next].annotation);
                                    frames.push(Frame::Record { application, expansion, field: next });
                                    let Frame::Record { expansion, .. } = frames.last().unwrap() else { unreachable!() };
                                    crate::type_substitution::apply(&mut current, &expansion.bindings)?;
                                    phase = Phase::Enter;
                                } else {
                                    self.records[expansion.index].fields = expansion.fields;
                                    current = application;
                                    current.kind = Kind::Named(TypeName { text: self.records[expansion.index].name.text.clone(), span: current.span });
                                }
                            }
                        }
                    }
                }
                Ok(false)
            })();
            match step { Ok(false) => {}, Ok(true) => break Ok(()), Err(error) => break Err(error) }
        };
        // Restore the original caller's partially materialized annotation. An
        // unfinished record retains its published identity but no partial fields.
        while let Some(frame) = frames.pop() {
            match frame {
                Frame::Node { mut parent, child: path } => {
                    *child(&mut parent, path) = current;
                    current = parent;
                }
                Frame::Record { application, .. } => {
                    current = application;
                }
            }
        }
        *value = current;
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_materialization_uses_parent_continuations_and_production_disposal() {
        let unions = crate::unions::UnionArena::default();
        let mut callables = crate::callable_resolution::Tables::default();
        let mut context = Context { templates: BTreeMap::new(), records: Vec::new(), names: HashMap::new(), instances: HashMap::new(), counts: HashMap::new(), unions: &unions, callables: &mut callables };
        let mut annotation = crate::type_syntax::parse("int").unwrap();
        for level in 0..16_384 {
            let span = annotation.span;
            annotation = if level % 3 == 0 {
                ValueAnnotation { kind: Kind::Union(vec![annotation, crate::type_syntax::parse("null").unwrap()]), span }
            } else if level % 3 == 1 {
                ValueAnnotation { kind: Kind::Group(Box::new(annotation)), span }
            } else {
                let mut wrapper = crate::type_syntax::parse("(value: int) => int").unwrap();
                let Kind::Callable { result, .. } = &mut wrapper.kind else { unreachable!() };
                **result = annotation;
                wrapper
            };
        }
        let original = annotation.clone();
        context.annotation(&mut annotation, &HashMap::new()).unwrap();
        assert!(annotation == original);
        drop(annotation);
        drop(original);
    }

    #[test]
    fn failed_materialization_restores_parents_and_stops_before_later_members() {
        let unions = crate::unions::UnionArena::default();
        let mut callables = crate::callable_resolution::Tables::default();
        let mut context = Context { templates: BTreeMap::new(), records: Vec::new(), names: HashMap::new(), instances: HashMap::new(), counts: HashMap::new(), unions: &unions, callables: &mut callables };
        let mut annotation = crate::type_syntax::parse("Result<['first'], Missing<int>> | 'later'").unwrap();
        let error = context.annotation(&mut annotation, &HashMap::new()).unwrap_err();
        assert_eq!(error.code, "type");
        assert_eq!(error.span.start, 18);
        assert_eq!(context.records.len(), 2); // First literal and its tuple only.
        let Kind::Union(members) = &annotation.kind else { panic!("parent union restored") };
        let Kind::Applied { arguments, .. } = &members[0].kind else { panic!("failed Result remains an application") };
        assert!(matches!(&arguments[0].kind, Kind::Named(_)));
        assert!(matches!(&members[1].kind, Kind::Literal(_)));

        let mut program = crate::syntax::parse("type Box<T>={first:[T];second:Missing<T>;};function main(){}").unwrap();
        let record = program.records.remove(0);
        context.templates.insert(record.name.text.clone(), record);
        let mut application = crate::type_syntax::parse("Box<int>").unwrap();
        let error = context.annotation(&mut application, &HashMap::new()).unwrap_err();
        assert_eq!(error.code, "type");
        let index = context.instances[&("Box".to_owned(), vec![Type::Int])];
        assert!(context.records[index].fields.is_empty());
        assert!(matches!(application.kind, Kind::Applied { .. }));
    }
}
