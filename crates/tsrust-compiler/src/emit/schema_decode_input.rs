//! Decode the explicit owned input type requested by a coercion callback.
use super::*;

impl Emitter<'_> {
    pub(super) fn schema_text_input(&self,ty:Type,generated:&mut String)->String {
        let name=format!("decode_input_{ty:?}").chars().map(|ch|if ch.is_ascii_alphanumeric(){ch}else{'_'}).collect::<String>();
        let marker=format!("/*{name}*/");if generated.contains(&marker){return name;}generated.push_str(&marker);
        let native=self.native_type(ty);
        let mismatch=|kind:&str|format!("issues.push(path,Code::Type,Kind::{kind},observed);None");
        let body=match ty {
            Type::String|Type::Int|Type::Number|Type::Boolean|Type::Literal(_,_)|Type::Enum(_,_)=>{
                let base=match ty{Type::Enum(_,crate::syntax::EnumKind::String)=>Type::String,Type::Enum(_,_)=>Type::Number,_=>ty.scalar_base()};
                let read=match base{Type::String=>"input.string().map(|value|String::Owned(value.into_owned()))",Type::Int=>"input.int()",Type::Number=>"input.number()",Type::Boolean=>"input.boolean()",_=>unreachable!()};
                let kind=self.schema_kind(ty);
                let mut body=format!("let value=match {read}{{Ok(value)=>value,Err(error)=>{{issues.push(path,tsrust_schema_numeric_code(error.code),Kind::{kind},observed);return None;}}}};");
                if let Type::Literal(id,_)=ty{
                    use crate::literal_types::LiteralValue;
                    let expected=match self.program.records[id].literal_definition.as_ref().unwrap(){LiteralValue::String(value)=>format!("{value:?}"),LiteralValue::Int(value)=>format!("{value}i64"),LiteralValue::Number(bits)=>float_literal(f64::from_bits(*bits)),LiteralValue::Boolean(value)=>value.to_string()};
                    body.push_str(&format!("if value!={expected}{{issues.push(path,Code::Literal,Kind::{kind},observed);return None;}}"));
                }
                if let Type::Enum(id,_)=ty{
                    for(tag,member)in self.program.records[id].enum_definition.as_ref().unwrap().members.iter().enumerate(){
                        let expected=match &member.value{crate::syntax::EnumValue::String(value)=>format!("{value:?}"),crate::syntax::EnumValue::Number(value)=>float_literal(*value)};
                        body.push_str(&format!("if value=={expected}{{return Some(Enum{id}::V{tag});}}"));
                    }
                    body.push_str(&format!("issues.push(path,Code::Literal,Kind::{kind},observed);None"));
                }else{body.push_str("Some(value)");}
                body
            },
            Type::Null=>format!("if input.kind()==\"null\"{{Some(TsrNull)}}else{{{}}}",mismatch("Null")),
            Type::Undefined=>mismatch("Missing"),
            Type::Array(element)=>{
                let child=self.schema_text_input(element.ty(),generated);
                format!("if input.kind()!=\"array\"{{{};return None;}}let mut values=Some(<{native}>::from([]));input.elements(|index,input|{{path.push(tsrust_schema_issues::Segment::Index(index));let value={child}(input,path,issues);path.pop();match(value,&mut values){{(Some(value),Some(values))=>values.push(value),(None,_)=>values=None,_=>{{}}}}values.is_some()}}).expect(\"validated array cursor\");values",mismatch("Array").replace(";None",""))
            },
            Type::Union(union)=>{
                let mut body=String::from("let checkpoint=issues.checkpoint();");
                for(tag,member)in self.program.unions[union.index()].members.iter().enumerate(){
                    if *member==Type::Undefined{continue;}
                    let child=self.schema_text_input(*member,generated);
                    body.push_str(&format!("if let Some(value)={child}(input,path,issues){{issues.restore(checkpoint);return Some(Union{}::V{tag}(value));}}",union.index()));
                }
                body.push_str("issues.reject();None");body
            },
            Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)=>format!("Some(Record{}{{value:input.retain()}})",self.native_record_declaration(id)),
            Type::Record(id) if self.program.records[id].native_type.is_none()&&!self.program.records[id].is_tuple()=>{
                let mut body=String::from("let mut fields=match input.fields(){Ok(fields)=>fields,Err(_)=>{issues.push(path,Code::Type,Kind::Object,observed);return None;}};");
                let mut values=Vec::new();
                for(index,field)in self.program.records[id].fields.iter().enumerate(){
                    let ty=field.ty.unwrap();let child=self.schema_text_input(ty,generated);
                    let missing=if let Type::Union(union)=ty && self.program.unions[union.index()].members.contains(&Type::Undefined){format!("Some({})",self.schema_output_injection(Type::Undefined,ty,"TsrUndefined"))}
                        else{format!("issues.push(path,Code::Missing,Kind::{},Kind::Missing);None",self.schema_kind(ty))};
                    body.push_str(&format!("path.push(tsrust_schema_issues::Segment::Field({key:?}));let field{index}=match fields.remove({key:?}){{Some(input)=>{child}(input,path,issues),None=>{{{missing}}}}};path.pop();let field{index}=field{index}?;",key=field.name.text));
                    values.push(format!("field{index}"));
                }
                body.push_str("if !fields.is_empty(){for(key,value)in &fields{let mut unknown_path=path.clone();unknown_path.push(tsrust_schema_issues::Segment::Field(key));issues.push(&unknown_path,Code::UnknownField,Kind::Missing,tsrust_source_kind(*value));}return None;}");
                body.push_str(&format!("Some(Record{}{{{}}})",self.native_record_declaration(id),values.join(",")));body
            },
            _=>mismatch("Object"),
        };
        generated.push_str(&format!("fn {name}(input:tsrust_native_json::source::Node<'_>,path:&mut Vec<tsrust_schema_issues::Segment<'_>>,issues:&mut tsrust_schema_issues::Issues)->Option<{native}>{{use tsrust_schema_issues::{{Code,Kind}};let observed=tsrust_source_kind(input);{body}}}"));
        name
    }
}
