//! Exact singleton identities retain the scalar's native representation and ownership.
use crate::{Diagnostic,Span,syntax::Type,type_syntax::LiteralAnnotation};
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum LiteralKind { String,Int,Number,Boolean }
impl LiteralKind {
    pub fn base(self)->Type {match self{Self::String=>Type::String,Self::Int=>Type::Int,Self::Number=>Type::Number,Self::Boolean=>Type::Boolean}}
}
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum LiteralValue { String(String),Int(i64),Number(u64),Boolean(bool) }
impl LiteralValue {
    pub fn kind(&self)->LiteralKind {match self{Self::String(_)=>LiteralKind::String,Self::Int(_)=>LiteralKind::Int,Self::Number(_)=>LiteralKind::Number,Self::Boolean(_)=>LiteralKind::Boolean}}
    pub fn checked(value:&LiteralAnnotation,span:Span)->Result<Self,Diagnostic>{
        Ok(match value {
            LiteralAnnotation::String(value)=>Self::String(value.clone()),
            LiteralAnnotation::Boolean(value)=>Self::Boolean(*value),
            LiteralAnnotation::Int(raw)=>{
                let (negative,digits)=raw.strip_prefix('-').map_or((false,raw.as_str()),|digits|(true,digits));
                let digits=crate::syntax::numeric_literals::integer_spelling(digits);
                let text=if negative{format!("-{digits}")}else{digits};
                Self::Int(text.parse().map_err(|_|Diagnostic::new("integer-range","Integer literal type is outside the signed 64-bit range.",span))?)
            },
            LiteralAnnotation::Number(raw)=>{
                let value=raw.replace('_',"").parse::<f64>().map_err(|_|Diagnostic::new("literal-type","Invalid number literal type.",span))?;
                if !value.is_finite(){return Err(Diagnostic::new("literal-type","A number literal type must be finite.",span));}
                Self::Number(if value==0.0 {0.0f64.to_bits()}else{value.to_bits()})
            }
        })
    }
    pub fn annotation(&self)->LiteralAnnotation {match self {
        Self::String(value)=>LiteralAnnotation::String(value.clone()),Self::Int(value)=>LiteralAnnotation::Int(value.to_string()),Self::Boolean(value)=>LiteralAnnotation::Boolean(*value),
        Self::Number(bits)=>{let text=format!("{:?}",f64::from_bits(*bits));LiteralAnnotation::Number(text)}
    }}
    pub fn text(&self)->String {self.annotation().text()}
    pub(crate) fn key(&self)->String {format!("@literal:{:?}:{}",self.kind(),self.text())}
}

impl LiteralValue {
    pub(crate) fn expression(value:&crate::syntax::Expr,expected:LiteralKind)->Option<Result<Self,Diagnostic>> {
        use crate::syntax::{ExprKind,UnaryOp};
        let annotation=match &value.kind {
            ExprKind::String(text)=>LiteralAnnotation::String(text.clone()),
            ExprKind::Boolean(value)=>LiteralAnnotation::Boolean(*value),
            ExprKind::Integer(text)=>if expected==LiteralKind::Number{LiteralAnnotation::Number(text.clone())}else{LiteralAnnotation::Int(text.clone())},
            ExprKind::Number(value)=>LiteralAnnotation::Number(format!("{value:?}")),
            ExprKind::Unary{op:UnaryOp::Negate,value:inner,..}=>match &inner.kind {
                ExprKind::Integer(text)=>if expected==LiteralKind::Number{LiteralAnnotation::Number(format!("-{text}"))}else{LiteralAnnotation::Int(format!("-{text}"))},
                ExprKind::Number(number)=>LiteralAnnotation::Number(format!("{:?}",-number)),
                _=>return None,
            },
            _=>return None,
        };
        Some(Self::checked(&annotation,value.span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn checked(value:LiteralAnnotation)->Result<LiteralValue,Diagnostic>{LiteralValue::checked(&value,Span::default())}
    #[test]
    fn literal_identity_normalizes_spelling_without_rounding_integers() {
        for spelling in ["0x10","0b10000","0o20","1_6"] {
            assert_eq!(checked(LiteralAnnotation::Int(spelling.into())).unwrap(),LiteralValue::Int(16));
        }
        assert_eq!(checked(LiteralAnnotation::Int("9007199254740993".into())).unwrap(),LiteralValue::Int(9007199254740993));
        assert_eq!(checked(LiteralAnnotation::Int("-9223372036854775808".into())).unwrap(),LiteralValue::Int(i64::MIN));
        assert!(checked(LiteralAnnotation::Int("9223372036854775808".into())).is_err());
        assert!(checked(LiteralAnnotation::Int("-9223372036854775809".into())).is_err());
    }
    #[test]
    fn number_identity_is_finite_and_canonical_but_distinct_from_int() {
        assert_eq!(checked(LiteralAnnotation::Number("-0.0".into())).unwrap(),checked(LiteralAnnotation::Number("0.0".into())).unwrap());
        assert_eq!(checked(LiteralAnnotation::Number("2.50".into())).unwrap(),checked(LiteralAnnotation::Number("2.5".into())).unwrap());
        assert_ne!(checked(LiteralAnnotation::Number("1.0".into())).unwrap(),LiteralValue::Int(1));
        assert!(checked(LiteralAnnotation::Number("1e999".into())).is_err());
    }
}

/// A closed scalar literal family has one decoder and exact, disjoint identities.
pub(crate) fn family(values:&[Type])->Option<LiteralKind>{
    let Type::Literal(_,kind)=*values.first()? else{return None;};
    values.iter().all(|value|matches!(value,Type::Literal(_,other) if *other==kind)).then_some(kind)
}
