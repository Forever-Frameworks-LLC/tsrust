//! Empty local Maps have no callback instances to provide concrete environments.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn empty_scoped_map_layout(&mut self, ty: Type) -> Result<Type, Diagnostic> {
        match ty {
            Type::Map(id) => {
                let payload = self.program.records[id]
                    .map_definition
                    .as_ref()
                    .unwrap()
                    .value;
                let Type::Record(record) = payload else {
                    return Ok(ty);
                };
                if self.scoped_records.declaration(record) != record
                    || !self.aggregate_callable_facts(payload).scoped
                {
                    return Ok(ty);
                }
                let payload = self.empty_scoped_record_environment(record)?;
                Ok(Type::Map(self.scoped_map_value_instance(id, payload)))
            }
            Type::Record(id) => {
                let fields = self.program.records[id]
                    .fields
                    .iter()
                    .map(|field| field.ty.unwrap())
                    .collect::<Vec<_>>();
                let mut types = Vec::with_capacity(fields.len());
                for field in &fields {
                    types.push(self.empty_scoped_map_layout(*field)?);
                }
                Ok(if types == fields {
                    ty
                } else {
                    self.empty_environment_record(id, types)
                })
            }
            _ => Ok(ty),
        }
    }

    #[inline(never)]
    pub(super) fn empty_scoped_record_environment(&mut self, id: usize) -> Result<Type, Diagnostic> {
        let span = self.program.records[id].name.span;
        let fields = self.program.records[id]
            .fields
            .iter()
            .map(|field| field.ty.unwrap())
            .collect::<Vec<_>>();
        let mut types = Vec::with_capacity(fields.len());
        for field in &fields {
            let field = *field;
            let ty = match field {
                Type::Callback(id) => self.absent_callback(id),
                Type::Record(id) => self.empty_scoped_record_environment(id)?,
                Type::Union(id) => {
                    let members = self.unions.members(id);
                    let mut concrete = Vec::new();
                    for member in members {
                        concrete.push(match member {
                            Type::Callback(id) => self.absent_callback(id),
                            Type::Record(id) => self.empty_scoped_record_environment(id)?,
                            other => other,
                        });
                    }
                    self.unions.intern(concrete, span)?
                }
                Type::Map(_) => self.empty_scoped_map_layout(field)?,
                other => other,
            };
            types.push(ty);
        }
        Ok(if types == fields {
            Type::Record(id)
        } else {
            self.empty_environment_record(id, types)
        })
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
