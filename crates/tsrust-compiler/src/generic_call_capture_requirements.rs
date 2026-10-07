//! Arrow capture requirements use lexical identities without inventing environments.
use super::*;
pub(super) fn arrow(body: &[Stmt], parameters: &[Parameter], effects: &Index) -> Result<HashSet<BindingId>, Diagnostic> {
    let mut owned = parameters.iter().map(|parameter| parameter.binding.clone()).collect::<Vec<_>>();
    collect(body, &mut owned, effects);
    let owned = owned.iter().filter_map(|binding| binding.identity).collect::<HashSet<_>>();
    let mut probe = body.to_vec();
    for parameter in parameters {
        if let ParameterInitializer::Default { value, .. } = &parameter.initializer {
            probe.push(Stmt { kind: StmtKind::Expr((**value).clone()), span: value.span, labels: Vec::new(), control_target: None });
        }
    }
    let mut captures = HashSet::new();
    crate::source_walk::body(&mut probe, &mut |_| Ok(()), &mut |expression| {
        if let ExprKind::Closure { body, params, .. } = &mut expression.kind {
            // A nested body has its own invocation requirements and frame.
            body.clear(); params.clear();
        }
        if let ExprKind::Name(name) = &expression.kind && let Some(binding) = effects.binding(name, expression.span) && !owned.contains(&binding) {
            captures.insert(binding);
        }
        Ok(())
    })?;
    Ok(captures)
}
