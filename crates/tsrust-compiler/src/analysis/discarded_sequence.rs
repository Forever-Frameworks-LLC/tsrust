//! Lexical temporary boundaries and final-result transport for comma removals.
use super::*;
use super::storage_exit_availability::Exits;

#[derive(Clone, Debug, Default)]
pub(super) struct Sites {
    pub(super) sequences: HashSet<(usize, usize)>,
    pub(super) bound_sequences: HashSet<(usize, usize)>,
    // Synthetic native blocks export the source declaration into its original scope.
    pub(super) exports: HashMap<usize, usize>,
    pub(super) operands: HashSet<(usize, usize)>,
}

impl Analyzer {
    /// Only a direct final removal receives the selected slot owner. Other final
    /// values still use ordinary sequence analysis until their transport is planned.
    pub(super) fn prepare_bound_removal_sequence(&self, context: &Context, value: &mut Expr) -> bool {
        let mut last = &*value;
        if !matches!(last.kind, ExprKind::Sequence(_)) { return false; }
        while let ExprKind::Sequence(values) = &last.kind { let Some(value)=values.last() else{return false;};last=value; }
        if Self::known_array_removal_place(context,last).is_none() { return false; }
        self.prepare_removal_sequence(context,value)
    }

    pub(super) fn prepare_removal_sequence(&self, context: &Context, value: &mut Expr) -> bool {
        let mut pending = vec![&*value];
        let mut removal = false;
        let mut safe = true;
        while let Some(operand) = pending.pop() {
            if let ExprKind::Sequence(values) = &operand.kind {
                pending.extend(values.iter().rev());
            } else if Self::known_array_removal_place(context, operand).is_some() {
                removal = true;
            } else if !self.safe_discarded_scalar(context, operand) {
                safe = false;
            }
        }
        let precise = safe && removal;
        if precise {
            let ExprKind::Sequence(values) = &mut value.kind else { unreachable!() };
            let mut pending = std::mem::take(values);
            pending.reverse();
            while let Some(operand) = pending.pop() {
                if matches!(operand.kind, ExprKind::Sequence(_)) {
                    let ExprKind::Sequence(inner) = operand.into_kind() else { unreachable!() };
                    pending.extend(inner.into_iter().rev());
                } else {
                    values.push(operand);
                }
            }
        }
        precise
    }

    fn safe_discarded_scalar(&self, context: &Context, value: &Expr) -> bool {
        let scalar = |value: &Expr| self.expression_type_hint(context, value).is_some_and(|ty| ty.is_copy() || ty == Type::String);
        let mut pending = vec![value];
        while let Some(value) = pending.pop() {
            match &value.kind {
                ExprKind::Integer(_) | ExprKind::Number(_) | ExprKind::Boolean(_) | ExprKind::String(_) | ExprKind::Null | ExprKind::Undefined => {},
                ExprKind::Name(_) if scalar(value) => {},
                ExprKind::Field(_, _) if Self::direct_fields(value).is_some() && scalar(value) => {},
                ExprKind::Index(base, index) if Self::direct_fields(base).is_some() && scalar(value) => pending.push(index),
                ExprKind::Unary { value, .. } | ExprKind::Update { target: value, .. } => pending.push(value),
                ExprKind::Binary { left, right, .. }
                | ExprKind::Assign { target: left, value: right, .. }
                | ExprKind::CompoundAssign { target: left, value: right, .. } => { pending.push(right); pending.push(left); },
                ExprKind::Call(callee, arguments) if matches!(&callee.kind, ExprKind::Field(base, method) if method == "log" && matches!(&base.kind, ExprKind::Name(name) if name == "console" && !context.names.contains_key(name))) => pending.extend(arguments.iter().rev()),
                _ => return false,
            }
        }
        true
    }
}

pub(super) fn lower(body: &mut [Stmt], exits: &mut Exits) {
    let mut pending = body.iter_mut().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        if matches!(&statement.kind, StmtKind::Let { value, binding: Some(_), .. } if exits.bound_sequence(value.span)) {
            let old=std::mem::replace(&mut statement.kind,StmtKind::Break);
            let StmtKind::Let { rebindable,name,annotation,mut value,binding }=old else{unreachable!()};
            let ExprKind::Sequence(values)=&mut value.kind else{unreachable!()};
            let mut values=std::mem::take(values);
            let last=values.pop().expect("nonempty checked sequence");
            let mut body=values.into_iter().map(|value|{
                exits.record_comma_operand(value.span);
                Stmt {labels:vec![],control_target:None,span:value.span,kind:StmtKind::Expr(value)}
            }).collect::<Vec<_>>();
            body.push(Stmt{labels:vec![],control_target:None,span:last.span,kind:StmtKind::Let{rebindable,name,annotation,value:last,binding}});
            exits.record_sequence_export(statement.span.start,binding.unwrap());
            statement.kind=StmtKind::Block(body);
        }
        if matches!(&statement.kind, StmtKind::Assign { target,value,operator:None } if matches!(target.kind,ExprKind::Name(_)) && exits.bound_sequence(value.span)) {
            let old=std::mem::replace(&mut statement.kind,StmtKind::Break);
            let StmtKind::Assign { target,mut value,operator }=old else{unreachable!()};
            let ExprKind::Sequence(values)=&mut value.kind else{unreachable!()};
            let mut values=std::mem::take(values);
            let last=values.pop().expect("nonempty checked sequence");
            let mut body=values.into_iter().map(|value|{
                exits.record_comma_operand(value.span);
                Stmt{labels:vec![],control_target:None,span:value.span,kind:StmtKind::Expr(value)}
            }).collect::<Vec<_>>();
            body.push(Stmt{labels:vec![],control_target:None,span:last.span,kind:StmtKind::Assign{target,value:last,operator}});
            statement.kind=StmtKind::Block(body);
        }
        if let StmtKind::Expr(value) = &mut statement.kind
            && exits.discarded_sequence(value.span)
            && let ExprKind::Sequence(values) = &mut value.kind {
            let operands = std::mem::take(values).into_iter().map(|value| {
                exits.record_comma_operand(value.span);
                Stmt { labels: vec![], control_target: None, span: value.span, kind: StmtKind::Expr(value) }
            }).collect();
            statement.kind = StmtKind::Block(operands);
        }
        match &mut statement.kind {
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => pending.extend(body.iter_mut()),
            StmtKind::Loop { body, update, .. } => { pending.extend(body.iter_mut()); pending.extend(update.iter_mut()); }
            StmtKind::If { then_body, else_body, .. } => { pending.extend(then_body.iter_mut()); pending.extend(else_body.iter_mut()); }
            StmtKind::Try(region) => pending.extend(region.statements_mut()),
            _ => {}
        }
    }
}
