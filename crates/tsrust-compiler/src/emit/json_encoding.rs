//! Borrow checked values through generated serializers without constructing JSON trees.
use super::*;
impl Emitter<'_> {
    pub(super) fn json_typed_encode(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        args: &[Expr],
    ) -> String {
        let root = match args[0].ty.unwrap() {
            Type::Union(id) if !id.copy => Type::BorrowedUnion(id),
            ty => ty,
        };
        let mut types = vec![root];
        let mut index = 0;
        while index < types.len() {
            let children = match types[index] {
                Type::Array(element) => vec![element.ty()],
                Type::Record(id) if self.program.records[id].native_type.is_none() => {
                    self.program.records[id]
                        .fields
                        .iter()
                        .map(|field| field.ty.unwrap())
                        .collect()
                }
                Type::Union(id) | Type::BorrowedUnion(id) => {
                    self.program.unions[id.index()].members.clone()
                }
                _ => vec![],
            };
            for child in children {
                if !types.contains(&child) {
                    types.push(child);
                }
            }
            index += 1;
        }
        let helper = |ty: Type| {
            format!(
                "Encode{}",
                types.iter().position(|value| *value == ty).unwrap()
            )
        };
        let mut generated = String::from("use tsrust_native_json::typed::serde::{self,Serialize};");
        for (index, &ty) in types.iter().enumerate() {
            let native = self.native_type(ty).replace("'_", "'a");
            let body = match ty {
                Type::Int => "serializer.serialize_i64(*self.0)".into(),
                Type::Number => {
                    "tsrust_native_json::typed::FiniteNumber(*self.0).serialize(serializer)".into()
                }
                Type::Boolean => "serializer.serialize_bool(*self.0)".into(),
                Type::String => "serializer.serialize_str(self.0.as_ref())".into(),
                Type::Null => "serializer.serialize_unit()".into(),
                Type::Undefined => {
                    "tsrust_native_json::typed::Undefined.serialize(serializer)".into()
                }
                Type::Array(element) => {
                    let item = helper(element.ty());
                    format!(
                        "use serde::ser::SerializeSeq;let depth=tsrust_native_json::typed::encoding_container::<S::Error>(self.1)?;let mut sequence=serializer.serialize_seq(Some(self.0.len()))?;for index in 0..self.0.len(){{sequence.serialize_element(&{item}(self.0.get(index).expect(\"array index\"),depth))?;}}sequence.end()"
                    )
                }
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    match record.native_type {
      Some(crate::native_signatures::NativeTypeId::JsonValue|crate::native_signatures::NativeTypeId::JsonView)=>"tsrust_native_json::typed::Dynamic(&self.0.value,self.1).serialize(serializer)".into(),
      Some(crate::native_signatures::NativeTypeId::StringView)=>"serializer.serialize_str(self.0.text)".into(),
      None=>{
       let mut fields:Vec<_>=record.fields.iter().enumerate().collect();
       if !record.is_tuple(){fields.sort_by(|a,b|a.1.name.text.cmp(&b.1.name.text));}
       let mut writes=String::new();
       for (position,field) in fields {
        let ty=field.ty.unwrap();let item=helper(ty);let value=format!("&{item}(&self.0.field{position},depth)");
        if record.is_tuple(){writes.push_str(&format!("output.serialize_element({value})?;"));}else{
         let absent=if ty==Type::Undefined{Some("true".into())}else{ty.union().and_then(|union|self.program.unions[union.index()].members.iter().position(|ty|*ty==Type::Undefined).map(|tag|format!("matches!(&self.0.field{position},Union{}::V{tag}(_))",union.index())))};
         let write=format!("output.serialize_entry({:?},{value})?;",field.name.text);
         writes.push_str(&absent.map_or(write.clone(),|absent|format!("if !({absent}){{{write}}}")));
        }
       }
       if record.is_tuple(){format!("use serde::ser::SerializeSeq;let depth=tsrust_native_json::typed::encoding_container::<S::Error>(self.1)?;let mut output=serializer.serialize_seq(Some({}))?;{writes}output.end()",record.fields.len())}else{format!("use serde::ser::SerializeMap;let depth=tsrust_native_json::typed::encoding_container::<S::Error>(self.1)?;let mut output=serializer.serialize_map(None)?;{writes}output.end()")}
      },
      _=>unreachable!("checked JSON encoding native type"),
     }
                }
                Type::Union(id) | Type::BorrowedUnion(id) => {
                    let prefix = if matches!(ty, Type::BorrowedUnion(_)) {
                        "ViewUnion"
                    } else {
                        "Union"
                    };
                    let arms = self.program.unions[id.index()]
                        .members
                        .iter()
                        .enumerate()
                        .map(|(tag, &member)| {
                            format!(
                                "{prefix}{}::V{tag}(value)=>{}(value,self.1).serialize(serializer)",
                                id.index(),
                                helper(member)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("match self.0{{{arms}}}")
                }
                _ => unreachable!("checked JSON encoding type"),
            };
            generated.push_str(&format!("struct Encode{index}<'a>(&'a {native},usize);impl serde::Serialize for Encode{index}<'_>{{fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{{{body}}}}}"));
        }
        let union = expr.ty.unwrap().union().unwrap();
        let members = &self.program.unions[union.index()].members;
        let [ok, err] = crate::result_identity::variants(&self.program.records, members).unwrap();
        let error = self
            .program
            .records
            .iter()
            .position(|record| {
                record.native_type == Some(crate::native_signatures::NativeTypeId::JsonError)
            })
            .unwrap();
        let ok_tag = members
            .iter()
            .position(|ty| *ty == Type::Record(ok))
            .unwrap();
        let err_tag = members
            .iter()
            .position(|ty| *ty == Type::Record(err))
            .unwrap();
        let union = union.index();
        let value = self.expr(fun, &args[0], Mode::Read);
        let limit = self.expr(fun, &args[1], Mode::Copy);
        format!(
            "{{{generated}let value={value};let limit={limit};let encoded=usize::try_from(limit).map_err(|_|tsrust_native_json::invalid_limits()).and_then(|limit|tsrust_native_json::typed::encode(&Encode0(&value,0),limit));match encoded{{Ok(text)=>Union{union}::V{ok_tag}(Record{ok}{{field0:String::Owned(text)}}),Err(error)=>Union{union}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}"
        )
    }
}
