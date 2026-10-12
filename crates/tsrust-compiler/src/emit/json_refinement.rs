//! Native payload access after a stable JSON boolean proof.
use super::*;
impl Emitter<'_> {
  pub(super) fn json_boolean_projection(&self, fun: &CheckedFunction, value: &Expr) -> String {
    let emitted = self.expr(fun, value, Mode::Read);
    let extraction = |ty: Type, name: &str| match ty {
      Type::Boolean => format!("*{name}"),
      Type::Record(id)
        if matches!(
          self.program.records[id].native_type,
          Some(
            crate::native_signatures::NativeTypeId::JsonValue
              | crate::native_signatures::NativeTypeId::JsonView
          )
        ) =>
      {
        format!("{name}.value.as_bool().expect(\"checked JSON boolean proof\")")
      }
      _ => "unreachable!(\"checked JSON boolean alternative\")".into(),
    };
    if let Some(union) = value.ty.and_then(Type::union) {
      let arms = self.program.unions[union.index()]
        .members
        .iter()
        .enumerate()
        .map(|(i, ty)| {
          format!(
            "{}{}::V{i}(payload)=>{}",
            if union.copy { "Union" } else { "ViewUnion" },
            union.index(),
            extraction(*ty, "payload")
          )
        })
        .collect::<Vec<_>>()
        .join(",");
      format!("match &({emitted}){{{arms}}}")
    } else {
      format!(
        "{{let payload={emitted};{}}}",
        extraction(value.ty.unwrap(), "payload")
      )
    }
  }
}
