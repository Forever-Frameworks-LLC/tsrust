//! Owned parameter patterns extract array positions without copying elements.
use super::*;

pub(super) fn lower(body: &mut [Stmt], source: &str, source_type: Type) {
    let mut arrays = HashMap::from([(source.to_owned(), (source_type, 0usize))]);
    for statement in body {
        let StmtKind::Let { name, value, .. } = &mut statement.kind else { continue; };
        let root = selection_root(value).map(str::to_owned);
        let Some(root) = root else { continue; };
        let Some((Type::Array(element), removed)) = arrays.get_mut(&root) else { continue; };
        let selected_type = if matches!(value.kind, ExprKind::Call(..)) { Type::Array(*element) } else { element.ty() };
        lower_selection(value, &root, removed, false);
        if name.text.starts_with("pattern#") && matches!(selected_type, Type::Array(_)) {
            arrays.insert(name.text.clone(), (selected_type, 0));
        }
    }
}

fn selection_root(value: &Expr) -> Option<&str> {
    match &value.kind {
        ExprKind::Binary { op: BinaryOp::PatternDefault, left, .. }
        | ExprKind::Unary { op: UnaryOp::PatternSelection, value: left, .. } => selection_root(left),
        ExprKind::Index(base, _) => match &base.kind { ExprKind::Name(name) => Some(name), _ => None },
        ExprKind::Call(callee, _) => match &callee.kind {
            ExprKind::Field(base, name) if name == "pattern#rest" => match &base.kind {
                ExprKind::Name(name) => Some(name), _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn lower_selection(value: &mut Expr, source: &str, removed: &mut usize, optional: bool) {
    match &mut value.kind {
        ExprKind::Binary { op: BinaryOp::PatternDefault, left, .. } => {
            lower_selection(left, source, removed, true);
        }
        ExprKind::Unary { op: UnaryOp::PatternSelection, value, .. } => {
            lower_selection(value, source, removed, optional);
        }
        ExprKind::Index(base, index) if matches!(&base.kind, ExprKind::Name(name) if name == source) => {
            let ExprKind::Integer(position) = &index.kind else { return; };
            let position = position.parse::<usize>().expect("parser pattern position").saturating_sub(*removed);
            let span = value.span;
            let index = Expr::new(ExprKind::Integer(position.to_string()), index.span);
            let call = Expr::new(ExprKind::Call(
                Box::new(Expr::new(ExprKind::Field(base.clone(), "remove".into()), span)), vec![index.clone()]), span);
            value.kind = if optional {
                let length = Expr::new(ExprKind::Field(base.clone(), "length".into()), span);
                let condition = Expr::new(ExprKind::Binary { op: BinaryOp::Greater, operator: span,
                    left: Box::new(length), right: Box::new(index) }, span);
                ExprKind::Conditional { predicate: Box::new(condition), consequent: Box::new(call),
                    alternate: Box::new(Expr::new(ExprKind::Undefined, span)) }
            } else { call.kind };
            *removed += 1;
        }
        ExprKind::Call(callee, arguments) if matches!(&callee.kind,
            ExprKind::Field(base, name) if name == "pattern#rest" && matches!(&base.kind, ExprKind::Name(name) if name == source)) => {
            if let Some(index) = arguments.first_mut() && let ExprKind::Integer(position) = &mut index.kind {
                *position = position.parse::<usize>().expect("parser rest position").saturating_sub(*removed).to_string();
            }
        }
        _ => {}
    }
}
