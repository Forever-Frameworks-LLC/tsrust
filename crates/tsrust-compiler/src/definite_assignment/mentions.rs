//! Read-only mention discovery lets completion frames revisit cleanup safely.
use super::*;
use crate::syntax::ParameterInitializer;

pub(super) fn expression_mention(root: &Expr, variable: Variable) -> Option<crate::Span> {
    enum Work<'a> { Statement(&'a Stmt), Expression(&'a Expr) }
    let mut work = vec![Work::Expression(root)];
    let mut found = None;
    while let Some(next) = work.pop() {
        match next {
            Work::Statement(statement) => match &statement.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checked traversal"),
                StmtKind::Try(region) => work.extend(region.statements().map(Work::Statement)),
                StmtKind::Throw(value) | StmtKind::Let { value, .. } | StmtKind::Return(Some(value)) | StmtKind::Expr(value) => work.push(Work::Expression(value)),
                StmtKind::Block(body) => work.extend(body.iter().map(Work::Statement)),
                StmtKind::If { condition, then_body, else_body } => { work.push(Work::Expression(condition)); work.extend(then_body.iter().chain(else_body).map(Work::Statement)); },
                StmtKind::While { condition, body } => { work.push(Work::Expression(condition)); work.extend(body.iter().map(Work::Statement)); },
                StmtKind::Loop { condition, body, update, .. } => {
                    if let Some(condition) = condition { work.push(Work::Expression(condition)); }
                    work.extend(body.iter().chain(update).map(Work::Statement));
                },
                StmtKind::ForOf { iterable, body, .. } => { work.push(Work::Expression(iterable)); work.extend(body.iter().map(Work::Statement)); },
                StmtKind::Assign { target, value, .. } => { work.push(Work::Expression(target)); work.push(Work::Expression(value)); },
                StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {},
            },
            Work::Expression(value) => {
                let named = match (&variable.named, &value.kind) {
                    (Some(required), ExprKind::Name(text)) => required.functions.contains(text),
                    (None, ExprKind::Name(text)) => text == variable.name && variable.binding.is_none_or(|binding| value.binding == Some(binding)),
                    _ => false,
                };
                let captured = matches!(value.kind, ExprKind::Closure { .. }) && variable.binding.is_some_and(|binding| {
                    variable.captures.iter().any(|(span, outer)| span.start >= value.span.start && span.end <= value.span.end && outer.contains(&binding))
                });
                if (named || captured) && found.is_none_or(|span: crate::Span| value.span.start < span.start) { found = Some(value.span); }
                match &value.kind {
                    ExprKind::SpreadCall(plan) => work.extend(plan.expressions().map(Work::Expression)),
                    ExprKind::Match { body, .. } => work.extend(body.iter().map(Work::Statement)),
                    ExprKind::Closure { params, body, .. } => {
                        for parameter in params { if let ParameterInitializer::Default { value, .. } = &parameter.initializer { work.push(Work::Expression(value)); } }
                        work.extend(body.iter().map(Work::Statement));
                    },
                    ExprKind::Conditional { predicate, consequent, alternate } => work.extend([Work::Expression(predicate), Work::Expression(consequent), Work::Expression(alternate)]),
                    ExprKind::Optional { base: left, body: right, .. } | ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right)
                    | ExprKind::CompoundAssign { target: left, value: right, .. } | ExprKind::Assign { target: left, value: right, .. }
                    | ExprKind::CoalesceAssign { target: left, value: right, .. } | ExprKind::LogicalAssign { target: left, value: right, .. } => work.extend([Work::Expression(left), Work::Expression(right)]),
                    ExprKind::OptionalChain(value) | ExprKind::Unary { value, .. } | ExprKind::Update { target: value, .. } | ExprKind::Field(value, _)
                    | ExprKind::Inject { value, .. } | ExprKind::Project { value, .. } | ExprKind::TypeAssertion { value, .. } => work.push(Work::Expression(value)),
                    ExprKind::Record(fields) => work.extend(fields.iter().map(|(_, value)| Work::Expression(value))),
                    ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _)
                    | ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values) => work.extend(values.iter().map(Work::Expression)),
                    ExprKind::Call(callee, arguments) => { work.push(Work::Expression(callee)); work.extend(arguments.iter().map(Work::Expression)); },
                    ExprKind::SpreadValue(_) | ExprKind::Selected { .. } | ExprKind::Name(_) | ExprKind::String(_) | ExprKind::Integer(_)
                    | ExprKind::EnumMember { .. } | ExprKind::Number(_) | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => {},
                }
            },
        }
    }
    found
}
