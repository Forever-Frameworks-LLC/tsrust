//! Callback-free typed decoding. Failed attempts use the full source validator.
use super::*;

impl Emitter<'_> {
    pub(in crate::emit) fn schema_direct_decoder(&self, id: usize, generated: &mut String, seen: &mut HashSet<usize>) -> Option<String> {
        // Pipelines, recursive plans and retained values need source-level replay.
        if self.program.schema_pipelines.types(id).is_some() { return None; }
        let name=format!("DirectSchema{id}");
        if !seen.insert(id) { return Some(name); }
        let output=self.schema_pipeline_output(id);
        let record=self.native_record_declaration(id);
        let native=self.native_type(output);
        let body=match output {
            Type::Int|Type::Number|Type::String|Type::Boolean|Type::Null=>{
                let read=match output {
                    Type::Int=>"tsrust_native_json::typed::ExactInt::deserialize(decoder)?.0",
                    Type::Number=>"tsrust_native_json::typed::ExactNumber::deserialize(decoder)?.0",
                    Type::String=>"String::Owned(std::string::String::deserialize(decoder)?)",
                    Type::Boolean=>"bool::deserialize(decoder)?",
                    Type::Null=>"{let ()=serde::Deserialize::deserialize(decoder)?;TsrNull}",
                    _=>unreachable!(),
                };
                let bounds=if matches!(output,Type::Int|Type::Number|Type::String) {
                    let value=if output==Type::String {"value.as_ref()"}else{"value"};
                    format!("if schema.bounds.check({value}).is_some(){{return Ok(None);}}")
                }else{String::new()};
                format!("let value={read};{bounds}Ok(Some(value))")
            },
            Type::Array(element)=>{
                let child=self.schema_direct_decoder(self.schema_child(element.ty()),generated,seen)?;
                format!("struct Items<'plan>(&'plan Record{record});impl<'de> serde::de::Visitor<'de> for Items<'_>{{type Value=Option<{native}>;fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{{f.write_str(\"an array\")}}fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut seq:A)->Result<Self::Value,A::Error>{{let mut values=Some(<{native}>::from([]));while let Some(value)=seq.next_element_seed({child}(&self.0.element))?{{match(value,&mut values){{(Some(value),Some(values))=>values.push(value),(None,_)=>values=None,_=>{{}}}}}}Ok(values)}}}}decoder.deserialize_seq(Items(schema))")
            },
            Type::Record(object) if self.program.records[object].native_type.is_none()&&!self.program.records[object].is_tuple()&&!self.program.records[object].name.text.starts_with("@schema:retained:")=>{
                let mut fields=String::new();let mut arms=String::new();let mut values=Vec::new();let mut missing=Vec::new();
                for (index,field) in self.program.records[object].fields.iter().enumerate() {
                    let child=self.schema_direct_decoder(self.schema_child(field.ty.unwrap()),generated,seen)?;
                    fields.push_str(&format!("let mut field{index}=None;"));
                    arms.push_str(&format!("{:?}=>field{index}=map.next_value_seed({child}(&self.0.field{index}))?,",field.name.text));
                    values.push(format!("field{index}:field{index}.unwrap()"));
                    missing.push(format!("field{index}.is_none()"));
                }
                let missing=if missing.is_empty(){"false".into()}else{missing.join("||")};
                let construct=self.schema_output_record(object,&values.join(","));
                // Every duplicate replaces its slot before validation is accepted.
                // Unknown fields force replay, including strip/retain policies.
                format!("struct Fields<'plan>(&'plan Record{record});impl<'de> serde::de::Visitor<'de> for Fields<'_>{{type Value=Option<{native}>;fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{{f.write_str(\"an object\")}}fn visit_map<A:serde::de::MapAccess<'de>>(self,mut map:A)->Result<Self::Value,A::Error>{{{fields}let mut known=true;while let Some(key)=map.next_key::<tsrust_native_json::source::Text<'de>>()?{{match key.0.as_ref(){{{arms}_=>{{known=false;let _:serde::de::IgnoredAny=map.next_value()?;}}}}}}if !known||{missing}{{return Ok(None);}}Ok(Some({construct}))}}}}decoder.deserialize_map(Fields(schema))")
            },
            _=>return None,
        };
        let choices=if self.program.schema_choices.for_output(output).next().is_some() {
            "if schema.choice.is_some(){let _:serde::de::IgnoredAny=serde::Deserialize::deserialize(decoder)?;return Ok(None);}"
        }else{""};
        generated.push_str(&format!("struct {name}<'plan>(&'plan Record{record});impl<'de> serde::de::DeserializeSeed<'de> for {name}<'_>{{type Value=Option<{native}>;fn deserialize<D:serde::Deserializer<'de>>(self,decoder:D)->Result<Self::Value,D::Error>{{let schema=self.0;{choices}{body}}}}}"));
        Some(name)
    }
}
