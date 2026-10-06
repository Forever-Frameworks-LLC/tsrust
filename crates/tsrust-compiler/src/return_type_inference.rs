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
