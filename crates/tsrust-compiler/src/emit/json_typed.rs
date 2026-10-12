//! Generate direct owned visitors. No intermediate dynamic JSON tree is built.
use super::*;
impl Emitter<'_> {
    pub(super) fn json_typed_decode(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        args: &[Expr],
    ) -> String {
        let result = expr.ty.unwrap().union().unwrap();
        let members = &self.program.unions[result.index()].members;
        let [ok, err] = crate::result_identity::variants(&self.program.records, members).unwrap();
        let root = self.program.records[ok].fields[0].ty.unwrap();
        let error = self
            .program
            .records
            .iter()
            .position(|record| {
                record.native_type == Some(if args.len()==1 {crate::native_signatures::NativeTypeId::JsonlError}else{crate::native_signatures::NativeTypeId::JsonError})
            })
            .unwrap();
        let mut types = vec![root];
        let mut index = 0;
        while index < types.len() {
            let children = match types[index] {
                Type::Array(element) => vec![element.ty()],
                Type::Record(id) => self.program.records[id]
                    .fields
                    .iter()
                    .map(|field| field.ty.unwrap())
                    .collect(),
                Type::Union(id) => self.program.unions[id.index()]
                    .members
                    .iter()
                    .copied()
                    .filter(|ty| *ty != Type::Undefined)
                    .collect(),
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
                "Decode{}",
                types.iter().position(|value| *value == ty).unwrap()
            )
        };
        let mut generated =
            String::from("use tsrust_native_json::typed::serde::{self,Deserialize};");
        for (index, &ty) in types.iter().enumerate() {
            let native = self.native_type(ty);
            let body = match ty {
                Type::Int => {
                    "Ok(Self(tsrust_native_json::typed::ExactInt::deserialize(decoder)?.0))".into()
                }
                Type::Number => {
                    "Ok(Self(tsrust_native_json::typed::ExactNumber::deserialize(decoder)?.0))"
                        .into()
                }
                Type::Boolean => "Ok(Self(bool::deserialize(decoder)?))".into(),
                Type::String => {
                    "Ok(Self(String::Owned(std::string::String::deserialize(decoder)?)))".into()
                }
                Type::Null => "<()>::deserialize(decoder)?;Ok(Self(TsrNull))".into(),
                Type::Array(element) => {
                    let item = helper(element.ty());
                    format!(
                        "struct Items;impl<'de> serde::de::Visitor<'de> for Items{{type Value=Decode{index};fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{{f.write_str(\"an array\")}}fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut seq:A)->Result<Self::Value,A::Error>{{let mut values=<{native}>::from([]);while let Some(value)=seq.next_element::<{item}>()?{{values.push(value.0);}}Ok(Decode{index}(values))}}}}decoder.deserialize_seq(Items)"
                    )
                }
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    let mut fields = String::new();
                    let mut values = Vec::new();
                    for (position, field) in record.fields.iter().enumerate() {
                        let ty = field.ty.unwrap();
                        fields.push_str(&format!(
                            "let mut field{position}:Option<{}>=None;",
                            helper(ty)
                        ));
                        let missing = ty.union().and_then(|union| {
                            self.program.unions[union.index()]
                                .members
                                .iter()
                                .position(|ty| *ty == Type::Undefined)
                                .map(|tag| format!("Union{}::V{tag}(TsrUndefined)", union.index()))
                        });
                        let value = if let Some(missing) = missing {
                            format!(
                                "field{position}.map(|value|value.0).unwrap_or_else(||{missing})"
                            )
                        } else {
                            format!(
                                "field{position}.ok_or_else(||serde::de::Error::missing_field({:?}))?.0",
                                field.name.text
                            )
                        };
                        values.push(format!("field{position}:{value}"));
                    }
                    if self.record_runtime.ordered.contains(&id) {
                        values.push(format!(
                            "key_order:[{}]",
                            (0..record.fields.len())
                                .map(|i| i.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                    }
                    let construct =
                        format!("Ok(Decode{index}(Record{id}{{{}}}))", values.join(","));
                    if record.is_tuple() {
                        let reads = record
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(position, _)| format!("field{position}=seq.next_element()?;"))
                            .collect::<String>();
                        format!(
                            "struct Fields;impl<'de> serde::de::Visitor<'de> for Fields{{type Value=Decode{index};fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{{f.write_str(\"a tuple\")}}fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut seq:A)->Result<Self::Value,A::Error>{{{fields}{reads}if seq.next_element::<serde::de::IgnoredAny>()?.is_some(){{return Err(serde::de::Error::custom(\"tuple length\"));}}{construct}}}}}decoder.deserialize_seq(Fields)"
                        )
                    } else {
                        let arms = record
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(position, field)| {
                                format!(
                                    "{:?}=>field{position}=Some(map.next_value()?),",
                                    field.name.text
                                )
                            })
                            .collect::<String>();
                        format!(
                            "struct Fields;impl<'de> serde::de::Visitor<'de> for Fields{{type Value=Decode{index};fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{{f.write_str(\"an object\")}}fn visit_map<A:serde::de::MapAccess<'de>>(self,mut map:A)->Result<Self::Value,A::Error>{{{fields}while let Some(key)=map.next_key::<std::string::String>()?{{match key.as_str(){{{arms}_=>{{let _:serde::de::IgnoredAny=map.next_value()?;}}}}}}{construct}}}}}decoder.deserialize_map(Fields)"
                        )
                    }
                }
                Type::Union(id) => {
                    let members = &self.program.unions[id.index()].members;
                    if members.iter().all(|ty|matches!(ty,Type::Null|Type::Undefined)) {
                        let null=members.iter().position(|ty|*ty==Type::Null).unwrap();
                        generated.push_str(&format!("struct Decode{index}({native});impl<'de> serde::Deserialize<'de> for Decode{index}{{fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{{let ()=serde::Deserialize::deserialize(decoder)?;Ok(Self(Union{}::V{null}(TsrNull)))}}}}",id.index()));
                        continue;
                    }
                    let (tag, value) = members
                        .iter()
                        .enumerate()
                        .find(|(_, ty)| !matches!(ty, Type::Null | Type::Undefined))
                        .unwrap();
                    let stored = self.union_store_payload(id, *value, "value.0");
                    let present = format!("Union{}::V{tag}({stored})", id.index());
                    let payload = helper(*value);
                    if let Some(null) = members.iter().position(|ty| *ty == Type::Null) {
                        format!(
                            "Ok(Self(match Option::<{payload}>::deserialize(decoder)?{{Some(value)=>{present},None=>Union{}::V{null}(TsrNull)}}))",
                            id.index()
                        )
                    } else {
                        format!("let value={payload}::deserialize(decoder)?;Ok(Self({present}))")
                    }
                }
                _ => unreachable!("validated typed JSON shape"),
            };
            generated.push_str(&format!("struct Decode{index}({native});impl<'de> serde::Deserialize<'de> for Decode{index}{{fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{{{body}}}}}"));
        }
        if args.len()==1 {
            let reader=self.expr(fun,&args[0],Mode::Mutate);
            let optional=root.union().unwrap();
            let absent=self.program.unions[optional.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
            let os=self.program.records[error].fields[5].ty.unwrap().union().unwrap();
            let os_members=&self.program.unions[os.index()].members;
            let integer=os_members.iter().position(|ty|*ty==Type::Int).unwrap();
            let missing=os_members.iter().position(|ty|*ty==Type::Undefined).unwrap();
            let ok_tag=members.iter().position(|ty|*ty==Type::Record(ok)).unwrap();
            let err_tag=members.iter().position(|ty|*ty==Type::Record(err)).unwrap();
            let result=result.index();let optional=optional.index();let os=os.index();
            return format!("{{{generated}let reader={reader};match reader.handle.next_with(tsrust_native_json::typed::decode::<Decode0>){{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:match value{{Some(value)=>value.0,None=>Union{optional}::V{absent}(TsrUndefined)}}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.error.code),field1:String::Borrowed(error.error.message),field2:String::Borrowed(\"read\"),field3:error.line as i64,field4:error.error.offset as i64,field5:match error.os_code{{Some(code)=>Union{os}::V{integer}(i64::from(code)),None=>Union{os}::V{missing}(TsrUndefined)}}}}}})}}}}");
        }
        let text = self.expr(fun, &args[0], Mode::Read);
        let limits = self.expr(fun, args.last().unwrap(), Mode::Read);
        let (decoder,flag)=if args.len()==3 {("decode_jsonc",format!(",{}",self.expr(fun,&args[1],Mode::Copy)))}else{("decode",String::new())};
        let ok_tag = members
            .iter()
            .position(|ty| *ty == Type::Record(ok))
            .unwrap();
        let err_tag = members
            .iter()
            .position(|ty| *ty == Type::Record(err))
            .unwrap();
        let union = result.index();
        format!(
            "{{{generated}let text={text};let limits={limits};let decoded=(||{{let convert=|value:i64|usize::try_from(value).map_err(|_|tsrust_native_json::invalid_limits());tsrust_native_json::typed::{decoder}::<Decode0>(text.as_ref().as_bytes(),tsrust_native_json::Limits{{bytes:convert(limits.field0)?,depth:convert(limits.field1)?,entries:convert(limits.field2)?,nodes:convert(limits.field3)?}}{flag})}})();match decoded{{Ok(value)=>Union{union}::V{ok_tag}(Record{ok}{{field0:value.0}}),Err(error)=>Union{union}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}"
        )
    }
}
