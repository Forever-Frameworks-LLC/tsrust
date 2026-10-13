//! Readonly nullable record payloads share the ordinary nominal record bridge.
use super::*;

impl Emitter<'_> {
    pub(super) fn nullable_record_parameter(&self, fun: &CheckedFunction, id: usize) -> Option<(UnionId, usize)> {
        let binding = fun.bindings.get(id)?;
        if id >= fun.parameter_count || binding.mode != Mode::Read { return None; }
        let input = binding.input_ty.unwrap_or(binding.ty);
        let union = input.union()?;
        let members = &self.program.unions[union.index()].members;
        let record = members.iter().find_map(|ty| if let Type::Record(record) = ty { Some(*record) } else { None })?;
        if members.iter().any(|ty| !matches!(ty, Type::Record(_) | Type::Null | Type::Undefined))
            || !self.borrowing_records.contains(&self.program.scoped_records.declaration(record)) { return None; }
        Some((union, record))
    }

    pub(super) fn nullable_record_parameter_bound(&self, fun: &CheckedFunction, id: usize) -> Option<String> {
        let (_, record) = self.nullable_record_parameter(fun, id)?;
        let layout = self.record_borrow_layout(record)?;
        let fields = layout.leaves.iter().map(|leaf| {
            let associated = leaf.path.iter().map(|field| format!("Field{field}")).collect::<String>();
            format!("{associated}={}", self.native_type(leaf.ty))
        }).collect::<Vec<_>>().join(",");
        Some(format!("N{id}:TsrReadRecord{}<{fields}>", self.program.scoped_records.declaration(record)))
    }

    pub(super) fn nullable_record_parameter_field(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> Option<String> {
        let (root, method) = self.nullable_record_parameter_field_work(fun, expr, mode)?;
        let receiver = self.expr(fun, root, Mode::Read);
        Some(Self::finish_nullable_record_parameter_field(expr, &method, receiver))
    }

    pub(super) fn nullable_record_parameter_field_work<'a>(&self, fun: &CheckedFunction, expr: &'a Expr, mode: Mode) -> Option<(&'a Expr, String)> {
        if mode == Mode::Mutate || !matches!(expr.kind, ExprKind::Field(_, _)) { return None; }
        let mut root = expr;
        let mut path = Vec::new();
        while let ExprKind::Field(base, name) = &root.kind {
            let Type::Record(record) = base.ty? else { return None; };
            path.push(*self.fields[record].get(name.as_str())?);
            root = base;
        }
        let Type::Record(record) = root.ty? else { return None; };
        if !(0..fun.parameter_count).any(|id| self.nullable_record_parameter(fun, id).is_some_and(|(_, payload)| payload == record)) { return None; }
        path.reverse();
        let layout = self.record_borrow_layout(record)?;
        if !layout.leaves.iter().any(|leaf| leaf.path == path) { return None; }
        let method = path.iter().map(|field| format!("field{field}")).collect::<Vec<_>>().join("_");
        expr.ty?;
        Some((root, method))
    }

    pub(super) fn finish_nullable_record_parameter_field(expr: &Expr, method: &str, receiver: String) -> String {
        let selected = format!("({receiver}).tsr_{method}()");
        if expr.ty.expect("checked field").is_copy() { format!("*({selected})") } else { selected }
    }
}
