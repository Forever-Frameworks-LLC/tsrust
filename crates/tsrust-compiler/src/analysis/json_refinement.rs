//! JSON category proofs share ordinary place invalidation, joins and loop checks.
use super::*;
const BOOLEAN: &str = "\0json:boolean";
fn marker(mut place: crate::flow::Place) -> crate::flow::Place {
  place.fields.push(BOOLEAN.into());
  place
}
impl Analyzer {
  pub(super) fn remember_json_category(
    &self,
    context: &Context,
    span: Span,
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    yes: &mut crate::flow::Facts,
    no: &mut crate::flow::Facts,
  ) {
    let Some((place, crate::flow::Guard::Typeof(label))) =
      crate::flow::guard_pair(left, right, &context.names)
    else {
      return;
    };
    if !matches!(label.as_str(), "boolean" | "string") {
      return;
    }
    let Some(ty) = self.place_type(context, &place) else {
      return;
    };
    let dynamic = |ty: Type| matches!(ty,Type::Record(id) if matches!(self.program.records[id].native_type,Some(crate::native_signatures::NativeTypeId::JsonValue|crate::native_signatures::NativeTypeId::JsonView)));
    if !dynamic(ty)
      && !ty
        .union()
        .is_some_and(|id| self.unions.members(id).iter().copied().any(dynamic))
    {
      return;
    }
    let mut place = place;
    place.fields.push(format!("\0json:{label}"));
    let (matching, other) = if op == BinaryOp::Equal {
      (yes, no)
    } else {
      (no, yes)
    };
    matching.insert(
      place.clone(),
      crate::flow::Fact {
        members: vec![if label == "boolean" {Type::Boolean} else {Type::String}],
        origin: Some((span.start, span.end)),
      },
    );
    other.remove(&place);
  }
  pub(super) fn json_boolean_proof(
    &self,
    context: &Context,
    expr: &Expr,
  ) -> Option<crate::flow::Place> {
    let place = crate::flow::place(expr, &context.names)?;
    if context.suppressed_flow.contains(&place) || context.observer_places.contains(&place) {
      return None;
    }
    let proof = marker(place);
    context.flow.contains_key(&proof).then_some(proof)
  }
  pub(super) fn project_json_boolean(
    &mut self,
    context: &mut Context,
    expr: &mut Expr,
    loans: &Loans,
    proof: crate::flow::Place,
  ) -> Result<Type, Diagnostic> {
    let mut place = proof.clone();
    place.fields.pop();
    let fact = context.flow[&proof].clone();
    for frame in &mut context.loops {
      if frame
        .flow_entry
        .get(&proof)
        .is_some_and(|entry| entry.origin == fact.origin)
      {
        frame.flow_required.insert(proof.clone(), fact.clone());
      }
    }
    context.suppressed_flow.insert(place.clone());
    let physical = self.expression(context, expr, Mode::Read, loans, None);
    context.suppressed_flow.remove(&place);
    physical?;
    let value = expr.clone();
    let mut callee = Expr::new(ExprKind::Name("\0jsonBoolean".into()), expr.span);
    callee.ty = Some(Type::Boolean);
    expr.kind = ExprKind::Call(Box::new(callee), vec![value]);
    expr.target = Some(CallTarget::JsonBoolean);
    expr.ty = Some(Type::Boolean);
    // Optional carriers are inspected without reconstructing or consuming them.
    Ok(Type::Boolean)
  }
}

impl Analyzer {
 pub(super) fn json_string_proof(&self,context:&Context,expr:&Expr)->Option<crate::flow::Place>{
  let mut place=crate::flow::place(expr,&context.names)?;
  if context.suppressed_flow.contains(&place){return None;}
  place.fields.push("\0json:string".into());context.flow.contains_key(&place).then_some(place)
 }
 pub(super) fn use_json_string_proof(context:&mut Context,proof:crate::flow::Place){
  let fact=context.flow[&proof].clone();
  for frame in &mut context.loops {if frame.flow_entry.get(&proof).is_some_and(|entry|entry.origin==fact.origin){frame.flow_required.insert(proof.clone(),fact.clone());}}
 }
 pub(super) fn json_string_length(&mut self,context:&mut Context,expr:&mut Expr,mode:Mode,loans:&Loans)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Field(base,name)=&mut expr.kind else{return Ok(None)};
  if name!="length"{return Ok(None)}
  let Some(proof)=self.json_string_proof(context,base) else{return Ok(None)};
  if mode==Mode::Mutate{return Err(Diagnostic::new("unsupported","String length is read-only.",expr.span));}
  Self::use_json_string_proof(context,proof);
  self.expression(context,base,Mode::Read,loans,None)?;
  let value=(**base).clone();let callee=Expr::new(ExprKind::Name("\0jsonStringLength".into()),expr.span);
  expr.kind=ExprKind::Call(Box::new(callee),vec![value]);expr.target=Some(CallTarget::JsonStringLength);expr.ty=Some(Type::Int);Ok(Some(Type::Int))
 }
}
