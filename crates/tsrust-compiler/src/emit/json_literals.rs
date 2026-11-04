//! Exact scalar literal decoding shares primitive precision and input-budget checks.
use super::*;
use crate::literal_types::{LiteralKind,LiteralValue};
fn read(kind:LiteralKind)->&'static str{
    match kind{LiteralKind::String=>"tsrust_native_json::source::Text::deserialize(decoder)?.0",LiteralKind::Int=>"tsrust_native_json::typed::ExactInt::deserialize(decoder)?.0",LiteralKind::Number=>"tsrust_native_json::typed::ExactNumber::deserialize(decoder)?.0",LiteralKind::Boolean=>"bool::deserialize(decoder)?"}
}
impl Emitter<'_>{
    fn json_literal_condition(&self,id:usize)->String{
        match self.program.records[id].literal_definition.as_ref().expect("literal identity"){
            LiteralValue::String(value)=>format!("value.as_ref()=={value:?}"),
            LiteralValue::Int(value)=>format!("value=={value}i64"),
            LiteralValue::Number(bits)=>format!("value==f64::from_bits({bits}u64)"),
            LiteralValue::Boolean(value)=>format!("value=={value}"),
        }
    }
    fn json_literal_payload(&self,id:usize)->String {
        match self.program.records[id].literal_definition.as_ref().expect("literal identity") {
            LiteralValue::String(value)=>format!("String::Borrowed({value:?})"),
            _=>"value".into(),
        }
    }
    pub(super) fn json_literal_leaf(&self,id:usize,kind:LiteralKind)->String{
        format!("let value={};if {}{{Ok(Self({}))}}else{{Err(serde::de::Error::custom(\"JSON literal does not match its declared type\"))}}",read(kind),self.json_literal_condition(id),self.json_literal_payload(id))
    }
    pub(super) fn json_literal_union(&self,id:UnionId,kind:LiteralKind,declaration:&str)->String{
        let members=&self.program.unions[id.index()].members;
        let mut choices=String::new();
        for (tag,&ty) in members.iter().enumerate(){if let Type::Literal(record,_)=ty{
            let stored=self.union_store_payload(id,ty,&self.json_literal_payload(record));
            choices.push_str(&format!("if {}{{return Ok({declaration}(Union{}::V{tag}({stored})));}}",self.json_literal_condition(record),id.index()));
        }}
        let absent=members.iter().position(|ty|*ty==Type::Null).map(|tag|format!("Ok({declaration}(Union{}::V{tag}(TsrNull)))",id.index())).unwrap_or_else(||"Err(serde::de::Error::custom(\"Expected a non-null declared JSON literal\"))".into());
        format!("struct Choices;impl<'de> serde::de::Visitor<'de> for Choices{{type Value={declaration};fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{{f.write_str(\"a declared scalar literal\")}}fn visit_none<E:serde::de::Error>(self)->Result<Self::Value,E>{{{absent}}}fn visit_some<D:serde::Deserializer<'de>>(self,decoder:D)->Result<Self::Value,D::Error>{{let value={};{choices}Err(serde::de::Error::custom(\"JSON literal does not match its declared union\"))}}}}decoder.deserialize_option(Choices)",read(kind))
    }
}
