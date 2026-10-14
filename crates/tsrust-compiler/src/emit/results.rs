//! Result propagation reuses ordinary source throw lowering.
use super::*;
impl Emitter<'_> {
    pub(super) fn result_unwrap(&self, fun: &CheckedFunction, expr: &Expr, argument: &Expr, error: Type) -> String {
        let value = self.expr(fun, argument, Mode::Consume);
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let thrown = self.exception_regions.throw_value("payload", error, &unions, expr.span).expect("checked Result propagation");
        let members = match argument.ty.unwrap() { Type::Union(id) => self.program.unions[id.index()].members.clone(), ty => vec![ty] };
        let arms = members.iter().enumerate().map(|(tag, ty)| {
            let Type::Record(id) = ty else { unreachable!() };
            let record = &self.program.records[*id];
            let ok = record.application.as_ref().unwrap().declaration.text == "Result#Ok";
            let payload = if record.fields.is_empty() { "" } else { "field0:payload," };
            let pattern = format!("Record{}{{{payload}..}}", self.native_record_declaration(*id));
            let pattern = match argument.ty.unwrap() { Type::Union(id) => format!("Union{}::V{tag}({pattern})", id.index()), _ => pattern };
            let body = if ok { if record.fields.is_empty() { "()".into() } else { "payload".into() } } else { thrown.clone() };
            format!("{pattern}=>{{{body}}}")
        }).collect::<Vec<_>>().join(",");
        format!("match {value}{{{arms}}}")
    }
}
