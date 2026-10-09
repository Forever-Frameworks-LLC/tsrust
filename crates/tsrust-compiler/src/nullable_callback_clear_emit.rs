//! Prove absent callback payloads on every normal exit before native reconstruction.
use super::*;

impl Emitter<'_> {
    fn nullable_clear_path(
        &self,
        function: &CheckedFunction,
        value: &Expr,
    ) -> Option<(usize, Vec<String>)> {
        let (root, steps) = self.storage_steps(value);
        let binding = root.binding?;
        let mut ty = function.bindings[binding].ty;
        let mut names = Vec::new();
        for step in steps {
            match step {
                StorageStep::Field(index) => {
                    let Type::Record(record) = ty else {
                        return None;
                    };
                    let field = &self.program.records[record].fields[index];
                    names.push(field.name.text.clone());
                    ty = field.ty?;
                }
                StorageStep::Discard(_) => {}
                _ => return None,
            }
        }
        Some((binding, names))
    }

    pub(super) fn definitely_clears_nullable_callback(
        &self,
        callee: usize,
        parameter: usize,
        path: &[String],
        active: &mut HashSet<usize>,
    ) -> bool {
        if !active.insert(callee) {
            return false;
        }
        let function = &self.program.functions[callee];
        let mut exits = Vec::new();
        if let Some(state) = self.nullable_clear_body(
            function,
            &function.body,
            parameter,
            path,
            false,
            &mut exits,
            active,
        ) {
            exits.push(state);
        }
        active.remove(&callee);
        !exits.is_empty() && exits.into_iter().all(|absent| absent)
    }

    fn nullable_clear_body(
        &self,
        function: &CheckedFunction,
        body: &[Stmt],
        parameter: usize,
        path: &[String],
        mut absent: bool,
        exits: &mut Vec<bool>,
        active: &mut HashSet<usize>,
    ) -> Option<bool> {
        for statement in body {
            match &statement.kind {
                StmtKind::Assign {
                    target,
                    value,
                    operator: None,
                } => {
                    absent = self
                        .nullable_clear_path(function, target)
                        .is_some_and(|(root, names)| root == parameter && names == path)
                        && matches!(&value.kind, ExprKind::Inject { value, .. } if matches!(value.ty, Some(Type::Null | Type::Undefined)));
                }
                StmtKind::Block(body) => {
                    absent = self.nullable_clear_body(
                        function, body, parameter, path, absent, exits, active,
                    )?
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    // Evaluating the predicate can mutate the input. Each branch
                    // must establish absence independently of that evaluation.
                    let left = self.nullable_clear_body(
                        function, then_body, parameter, path, false, exits, active,
                    );
                    let right = self.nullable_clear_body(
                        function, else_body, parameter, path, false, exits, active,
                    );
                    absent = match (left, right) {
                        (Some(left), Some(right)) => left && right,
                        (Some(state), None) | (None, Some(state)) => state,
                        (None, None) => return None,
                    };
                }
                StmtKind::Return(value) => {
                    exits.push(absent && value.is_none());
                    return None;
                }
                StmtKind::Throw(_) => return None,
                StmtKind::Expr(value) => {
                    absent = false;
                    if let (Some(CallTarget::Function(callee)), ExprKind::Call(_, args)) =
                        (value.target, &value.kind)
                    {
                        for (index, argument) in args.iter().enumerate() {
                            let Some((root, prefix)) = self.nullable_clear_path(function, argument)
                            else {
                                continue;
                            };
                            if root == parameter
                                && path.starts_with(&prefix)
                                && self.definitely_clears_nullable_callback(
                                    callee,
                                    index,
                                    &path[prefix.len()..],
                                    active,
                                )
                            {
                                absent = true;
                            }
                        }
                    }
                }
                // Unknown effects and loops do not establish a definite clear.
                _ => absent = false,
            }
        }
        Some(absent)
    }
}
