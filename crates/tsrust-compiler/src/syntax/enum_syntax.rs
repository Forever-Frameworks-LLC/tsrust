//! Closed, homogeneous enum declarations and compile-time discriminants.
use super::*;

impl Parser<'_> {
    #[inline(never)]
    pub(super) fn enum_declaration(&mut self, records: &[Record]) -> Result<Record, Diagnostic> {
        let name = self.name()?;
        self.expect("{")?;
        let mut members: Vec<EnumMember> = Vec::new();
        let mut kind = None;
        let mut next = Some(0.0);
        while !self.accept("}") {
            let member = self.name()?;
            if members.iter().any(|prior| prior.name.text == member.text) {
                return Err(Diagnostic::new("duplicate", "An enum member name may only be declared once.", member.span));
            }
            let value = if self.accept("=") {
                let expression = self.expression()?;
                constant(&expression, &name.text, &members, records)?
            } else {
                EnumValue::Number(next.ok_or_else(|| Diagnostic::new("enum", "A member after a string value needs an explicit string initializer.", member.span))?)
            };
            let actual = match &value { EnumValue::Number(_) => EnumKind::Number, EnumValue::String(_) => EnumKind::String };
            if kind.is_some_and(|kind| kind != actual) {
                return Err(Diagnostic::new("enum", "Enum members must share a numeric or string value type.", member.span));
            }
            kind = Some(actual);
            next = match &value { EnumValue::Number(value) => Some(value + 1.0), EnumValue::String(_) => None };
            members.push(EnumMember { name: member, value });
            if !self.accept(",") { self.expect("}")?; break; }
        }
        self.accept(";");
        Ok(Record { native_type: None, map_definition: None, set_definition: None, literal_definition:None,enum_definition: Some(EnumDefinition { kind: kind.unwrap_or(EnumKind::Number), members }),
            application: None, name, type_parameters: Vec::new(), fields: Vec::new() })
    }
}

/// Evaluate one constant initializer. Operands are queued, so the native
/// call depth does not grow with the initializer.
fn constant(expression: &Expr, owner: &str, members: &[EnumMember], records: &[Record]) -> Result<EnumValue, Diagnostic> {
    enum Work<'a> {
        Visit(&'a Expr),
        Unary(&'a Expr),
        Binary(&'a Expr),
    }
    let invalid = |expression: &Expr| Diagnostic::new("enum", "An enum initializer requires a constant numeric/string expression or a previous enum member.", expression.span);
    let mut work = vec![Work::Visit(expression)];
    let mut values: Vec<EnumValue> = Vec::new();
    while let Some(step) = work.pop() {
        match step {
            Work::Visit(expression) => {
                let value = match &expression.kind {
                    ExprKind::Integer(text) => EnumValue::Number(text.parse().map_err(|_| invalid(expression))?),
                    ExprKind::Number(value) => EnumValue::Number(*value),
                    ExprKind::String(value) => EnumValue::String(value.clone()),
                    ExprKind::Name(name) if name == "Infinity" => EnumValue::Number(f64::INFINITY),
                    ExprKind::Name(name) if name == "NaN" => EnumValue::Number(f64::NAN),
                    ExprKind::Name(name) => members.iter().find(|member| &member.name.text == name).map(|member| member.value.clone()).ok_or_else(|| invalid(expression))?,
                    ExprKind::Field(base, name) => {
                        let ExprKind::Name(enumeration) = &base.kind else { return Err(invalid(expression)); };
                        if enumeration == owner {
                            members.iter().find(|member| &member.name.text == name).map(|member| member.value.clone()).ok_or_else(|| invalid(expression))?
                        } else {
                            records.iter().find(|record| &record.name.text == enumeration)
                                .and_then(|record| record.enum_definition.as_ref())
                                .and_then(|definition| definition.members.iter().find(|member| &member.name.text == name))
                                .map(|member| member.value.clone()).ok_or_else(|| invalid(expression))?
                        }
                    }
                    ExprKind::Unary { op: UnaryOp::Negate | UnaryOp::Positive | UnaryOp::BitNot, value, .. } => {
                        work.push(Work::Unary(expression));
                        work.push(Work::Visit(value));
                        continue;
                    }
                    ExprKind::Binary { left, right, .. } => {
                        work.push(Work::Binary(expression));
                        work.push(Work::Visit(right));
                        work.push(Work::Visit(left));
                        continue;
                    }
                    _ => return Err(invalid(expression)),
                };
                values.push(value);
            }
            Work::Unary(expression) => {
                let EnumValue::Number(value) = values.pop().expect("unary operand") else { return Err(invalid(expression)); };
                values.push(EnumValue::Number(match expression.kind { ExprKind::Unary { op: UnaryOp::Negate, .. } => -value, ExprKind::Unary { op: UnaryOp::BitNot, .. } => (!uint32(value) as i32) as f64, _ => value }));
            }
            Work::Binary(expression) => {
                let ExprKind::Binary { op, .. } = &expression.kind else { unreachable!("binary initializer") };
                let right = values.pop().expect("right operand");
                let left = values.pop().expect("left operand");
                values.push(match (left, right) {
                    (EnumValue::Number(left), EnumValue::Number(right)) => EnumValue::Number(match op {
                        BinaryOp::Add => left + right, BinaryOp::Subtract => left - right,
                        BinaryOp::Multiply => left * right, BinaryOp::Divide => left / right,
                        BinaryOp::Remainder => left % right, BinaryOp::Exponent => left.powf(right),
                        BinaryOp::BitAnd => ((uint32(left) & uint32(right)) as i32) as f64,
                        BinaryOp::BitOr => ((uint32(left) | uint32(right)) as i32) as f64,
                        BinaryOp::BitXor => ((uint32(left) ^ uint32(right)) as i32) as f64,
                        BinaryOp::ShiftLeft => (uint32(left).wrapping_shl(uint32(right) & 31) as i32) as f64,
                        BinaryOp::ShiftRight => ((uint32(left) as i32) >> (uint32(right) & 31)) as f64,
                        BinaryOp::ShiftRightUnsigned => (uint32(left) >> (uint32(right) & 31)) as f64,
                        _ => return Err(invalid(expression)),
                    }),
                    (EnumValue::String(mut left), EnumValue::String(right)) if *op == BinaryOp::Add => { left.push_str(&right); EnumValue::String(left) }
                    _ => return Err(invalid(expression)),
                });
            }
        }
    }
    Ok(values.pop().expect("constant initializer value"))
}

fn uint32(value: f64) -> u32 {
    if !value.is_finite() { return 0; }
    value.trunc().rem_euclid(4294967296.0) as u32
}
