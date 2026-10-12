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

#[inline(never)]
fn constant(expression: &Expr, owner: &str, members: &[EnumMember], records: &[Record]) -> Result<EnumValue, Diagnostic> {
    let invalid = || Diagnostic::new("enum", "An enum initializer requires a constant numeric/string expression or a previous enum member.", expression.span);
    match &expression.kind {
        ExprKind::Integer(text) => Ok(EnumValue::Number(text.parse().map_err(|_| invalid())?)),
        ExprKind::Number(value) => Ok(EnumValue::Number(*value)),
        ExprKind::String(value) => Ok(EnumValue::String(value.clone())),
        ExprKind::Name(name) if name == "Infinity" => Ok(EnumValue::Number(f64::INFINITY)),
        ExprKind::Name(name) if name == "NaN" => Ok(EnumValue::Number(f64::NAN)),
        ExprKind::Name(name) => members.iter().find(|member| &member.name.text == name).map(|member| member.value.clone()).ok_or_else(invalid),
        ExprKind::Field(base, name) => {
            let ExprKind::Name(enumeration) = &base.kind else { return Err(invalid()); };
            if enumeration == owner {
                return members.iter().find(|member| &member.name.text == name).map(|member| member.value.clone()).ok_or_else(invalid);
            }
            records.iter().find(|record| &record.name.text == enumeration)
                .and_then(|record| record.enum_definition.as_ref())
                .and_then(|definition| definition.members.iter().find(|member| &member.name.text == name))
                .map(|member| member.value.clone()).ok_or_else(invalid)
        }
        ExprKind::Unary { op: UnaryOp::Negate | UnaryOp::Positive | UnaryOp::BitNot, value, .. } => {
            let EnumValue::Number(value) = constant(value, owner, members, records)? else { return Err(invalid()); };
            Ok(EnumValue::Number(match expression.kind { ExprKind::Unary { op: UnaryOp::Negate, .. } => -value, ExprKind::Unary { op: UnaryOp::BitNot, .. } => (!uint32(value) as i32) as f64, _ => value }))
        }
        ExprKind::Binary { op, left, right, .. } => {
            match (constant(left, owner, members, records)?, constant(right, owner, members, records)?) {
                (EnumValue::Number(left), EnumValue::Number(right)) => Ok(EnumValue::Number(match op {
                    BinaryOp::Add => left + right, BinaryOp::Subtract => left - right,
                    BinaryOp::Multiply => left * right, BinaryOp::Divide => left / right,
                    BinaryOp::Remainder => left % right, BinaryOp::Exponent => left.powf(right),
                    BinaryOp::BitAnd => ((uint32(left) & uint32(right)) as i32) as f64,
                    BinaryOp::BitOr => ((uint32(left) | uint32(right)) as i32) as f64,
                    BinaryOp::BitXor => ((uint32(left) ^ uint32(right)) as i32) as f64,
                    BinaryOp::ShiftLeft => (uint32(left).wrapping_shl(uint32(right) & 31) as i32) as f64,
                    BinaryOp::ShiftRight => ((uint32(left) as i32) >> (uint32(right) & 31)) as f64,
                    BinaryOp::ShiftRightUnsigned => (uint32(left) >> (uint32(right) & 31)) as f64,
                    _ => return Err(invalid()),
                })),
                (EnumValue::String(mut left), EnumValue::String(right)) if *op == BinaryOp::Add => { left.push_str(&right); Ok(EnumValue::String(left)) }
                _ => Err(invalid()),
            }
        }
        _ => Err(invalid()),
    }
}

fn uint32(value: f64) -> u32 {
    if !value.is_finite() { return 0; }
    value.trunc().rem_euclid(4294967296.0) as u32
}
