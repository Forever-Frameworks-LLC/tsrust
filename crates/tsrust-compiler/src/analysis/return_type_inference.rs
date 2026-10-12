//! Shared checked return inference for arrows and named function declarations.
use super::*;

pub(super) fn has_value_return(body: &[Stmt]) -> bool {
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Return(Some(_)) => return true,
            StmtKind::Block(body)
            | StmtKind::While { body, .. }
            | StmtKind::ForOf { body, .. }
            | StmtKind::Loop { body, .. } => pending.extend(body),
            StmtKind::Try(region) => pending.extend(region.statements()),
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                pending.extend(then_body);
                pending.extend(else_body);
            }
            _ => {}
        }
    }
    false
}

/// Whether a body has an exception region, at any statement depth.
pub(super) fn has_exception_region(body: &[Stmt]) -> bool {
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Try(_) => return true,
            StmtKind::Block(body)
            | StmtKind::While { body, .. }
            | StmtKind::ForOf { body, .. }
            | StmtKind::Loop { body, .. } => pending.extend(body),
            StmtKind::If { then_body, else_body, .. } => {
                pending.extend(then_body);
                pending.extend(else_body);
            }
            _ => {}
        }
    }
    false
}

impl Analyzer {
    /// Finish an inferred instantiated result before outer generic constraints
    /// inspect its type. This checks the body, without evaluating the source call.
    pub(super) fn prepare_named_result_hint(
        &mut self,
        context: &Context,
        index: usize,
    ) -> Result<(), Diagnostic> {
        if self.signatures[index].inferred_result && !self.discovering_results {
            self.named_captures.prepare(index, context, &self.program)?;
            self.function(index)?;
        }
        Ok(())
    }

    /// The type of the first return whose value can be typed from parameters,
    /// literals and already known functions alone. A recursive function uses
    /// it as its result while its own body is still being checked.
    #[inline(never)]
    pub(super) fn provisional_result(&self, context: &Context, body: &[Stmt]) -> Option<Type> {
        let mut returns = Vec::new();
        let mut pending = body.iter().rev().collect::<Vec<_>>();
        while let Some(statement) = pending.pop() {
            match &statement.kind {
                StmtKind::Return(Some(value)) => returns.push(value),
                StmtKind::Block(statements)
                | StmtKind::While { body: statements, .. }
                | StmtKind::ForOf { body: statements, .. }
                | StmtKind::Loop { body: statements, .. } => pending.extend(statements.iter().rev()),
                StmtKind::Try(region) => pending.extend(region.statements().collect::<Vec<_>>().into_iter().rev()),
                StmtKind::If { then_body, else_body, .. } => {
                    pending.extend(else_body.iter().rev());
                    pending.extend(then_body.iter().rev());
                }
                _ => {}
            }
        }
        returns.into_iter().find_map(|value| self.independent_value_hint(context, value))
    }

    /// A branch or operand that does not need the unknown result still tells
    /// the type of the whole value: both branches of a conditional agree, and
    /// arithmetic keeps its operand type.
    fn independent_value_hint(&self, context: &Context, value: &Expr) -> Option<Type> {
        let data = |ty: Type| matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::String).then_some(ty);
        match &value.kind {
            ExprKind::Conditional { consequent, alternate, .. } => self
                .independent_value_hint(context, consequent)
                .or_else(|| self.independent_value_hint(context, alternate)),
            ExprKind::Binary { op: BinaryOp::Add, left, right, .. } => {
                let left = self.independent_value_hint(context, left);
                let right = self.independent_value_hint(context, right);
                if left == Some(Type::String) || right == Some(Type::String) { Some(Type::String) } else { left.or(right) }
            }
            ExprKind::Binary { op: BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder, left, right, .. } => self
                .independent_value_hint(context, left)
                .or_else(|| self.independent_value_hint(context, right)),
            // A call has no type until its target is checked.
            ExprKind::Call(_, _) => None,
            _ => self.expression_type_hint(context, value).and_then(data),
        }
    }

    #[inline(never)]
    pub(super) fn inferred_body_result(&self, body: &[Stmt]) -> Result<Type, Diagnostic> {
        let mut pending = body.iter().collect::<Vec<_>>();
        let mut inferred = None;
        while let Some(statement) = pending.pop() {
            match &statement.kind {
                StmtKind::Return(value) => {
                    let actual = value
                        .as_ref()
                        .and_then(|value| value.ty)
                        .unwrap_or(Type::Void);
                    if let Some(previous) = inferred {
                        self.require(actual, Some(previous), statement.span)?;
                    } else {
                        inferred = Some(actual);
                    }
                }
                StmtKind::Block(statements)
                | StmtKind::While {
                    body: statements, ..
                }
                | StmtKind::ForOf {
                    body: statements, ..
                }
                | StmtKind::Loop {
                    body: statements, ..
                } => pending.extend(statements),
                StmtKind::Try(region) => pending.extend(region.statements()),
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend(then_body);
                    pending.extend(else_body);
                }
                _ => {}
            }
        }
        Ok(inferred.unwrap_or(Type::Void))
    }
}
