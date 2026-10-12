//! Borrowed inspection for owned union inputs; selection must not move fields.
use super::*;
use crate::stdlib::schema::plans::Tag;

impl Emitter<'_> {
    /// Leaf state owns only checked scalar conversions. Strings stay in `input`
    /// until commitment, so a rejected alternative never copies their storage.
    pub(super) fn schema_owned_leaf_preparation(&self, output:Type, input:Type)->Option<(String,String,String)> {
        let expected=self.schema_kind(output);
        let observed=self.schema_kind(input);
        let reject=|code:&str|format!("issues.push(path,Code::{code},Kind::{expected},Kind::{observed});None");
        let incompatible=||("std::convert::Infallible".into(),reject("Type"),"match state{}".into());
        if let Type::Enum(enumeration,kind)=output {
            if input==output {return Some((self.native_type(output),"Some(*input)".into(),"state".into()));}
            let base=if kind==crate::syntax::EnumKind::String{Type::String}else{Type::Number};
            if input.scalar_base()!=base && !(base==Type::Number&&input.scalar_base()==Type::Int){return Some(incompatible());}
            let mut read=if base==Type::String {String::from("let value=AsRef::<str>::as_ref(input);")}
                else if input.scalar_base()==Type::Int {format!("let value=*input as f64;if value as i128!=*input as i128{{{};return None;}}",reject("NumberPrecision").replace(";None", ""))}
                else{format!("let value=*input;if !value.is_finite(){{{};return None;}}",reject("NumberNonfinite").replace(";None", ""))};
            for (index,member) in self.program.records[enumeration].enum_definition.as_ref().unwrap().members.iter().enumerate(){
                let value=match &member.value{crate::syntax::EnumValue::String(value)=>format!("{value:?}"),crate::syntax::EnumValue::Number(value)=>float_literal(*value)};
                read.push_str(&format!("if value=={value}{{return Some(Enum{enumeration}::V{index});}}"));
            }
            read.push_str(&reject("Literal"));
            return Some((self.native_type(output),read,"state".into()));
        }
        let literal=matches!(output,Type::Literal(_,_));
        let base=output.scalar_base();
        if !matches!(base,Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined){return None;}
        if input.scalar_base()!=base && !(base==Type::Number&&input.scalar_base()==Type::Int){return Some(incompatible());}
        let (state,mut prepare,commit,value)=match base {
            Type::String=>("()".into(),"let value=AsRef::<str>::as_ref(input);".into(),"input".into(),"()"),
            Type::Number if input.scalar_base()==Type::Int=>(self.native_type(base),format!("let value=*input as f64;if value as i128!=*input as i128{{issues.push(path,Code::NumberPrecision,Kind::Number,Kind::Integer);return None;}}"),"state".into(),"value"),
            Type::Number=>(self.native_type(base),"let value=*input;if !value.is_finite(){issues.push(path,Code::NumberNonfinite,Kind::Number,Kind::Number);return None;}".into(),"state".into(),"value"),
            Type::Int|Type::Boolean=>(self.native_type(base),"let value=*input;".into(),"state".into(),"value"),
            Type::Null|Type::Undefined=>("()".into(),String::new(),"input".into(),"()"),
            _=>unreachable!(),
        };
        if literal {
            let expected=if base==Type::String{"AsRef::<str>::as_ref(&schema.expected)"}else{"schema.expected"};
            prepare.push_str(&format!("if value!={expected}{{issues.push(path,Code::Literal,Kind::{kind},Kind::{kind});return None;}}",kind=self.schema_kind(output)));
        }else if matches!(base,Type::String|Type::Int|Type::Number){
            prepare.push_str(&format!("if let Some(code)=schema.bounds.check(value){{issues.push(path,tsrust_schema_numeric_code(code),Kind::{expected},Kind::{observed});return None;}}"));
        }
        prepare.push_str(&format!("Some({value})"));
        Some((state,prepare,commit))
    }

    /// `value` is a borrowed native value. Numeric matches preserve exact integer
    /// identity, including integers beyond the floating-point precision boundary.
    pub(super) fn schema_owned_tag_test(&self, input: Type, value: &str, tag: &Tag) -> String {
        if let Type::Union(union)=input {
            let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(index,member)|{
                let test=self.schema_owned_tag_test(*member,"value",tag);
                format!("Union{}::V{index}(value)=>{test}",union.index())
            }).collect::<Vec<_>>().join(",");
            return format!("match {value}{{{arms}}}");
        }
        if let Type::Enum(enumeration,_)=input {
            let definition=self.program.records[enumeration].enum_definition.as_ref().unwrap();
            let arms=definition.members.iter().enumerate().map(|(index,member)|{
                let actual=match &member.value {
                    crate::syntax::EnumValue::String(value)=>Tag::String(value.clone()),
                    crate::syntax::EnumValue::Number(value)=>Tag::number(*value),
                };
                format!("Enum{enumeration}::V{index}=>{}",actual==*tag)
            }).collect::<Vec<_>>().join(",");
            return format!("match {value}{{{arms}}}");
        }
        match (input.scalar_base(),tag) {
            (Type::String,Tag::String(expected))=>format!("AsRef::<str>::as_ref({value})=={expected:?}"),
            (Type::Boolean,Tag::Boolean(expected))=>format!("*({value})=={expected}"),
            (Type::Null,Tag::Null)=>String::from("true"),
            (Type::Int,Tag::Integer(expected))=>format!("*({value})=={expected}i64"),
            (Type::Number,Tag::Integer(expected)) if (*expected as f64) as i128==*expected as i128=>{
                format!("*({value})=={}",float_literal(*expected as f64))
            },
            (Type::Number,Tag::Number(bits))=>format!("*({value})=={}",float_literal(f64::from_bits(*bits))),
            // Integral floating discriminators are canonicalized to Integer.
            // A remaining Number tag therefore cannot equal an i64 input.
            _=>String::from("false"),
        }
    }

    pub(super) fn schema_owned_discriminator_test(&self, input:Type, value:&str, selector:&crate::stdlib::schema::plans::Selector, branch:usize)->String {
        if let Type::Union(union)=input {
            let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(index,member)|{
                let test=self.schema_owned_discriminator_test(*member,"value",selector,branch);
                format!("Union{}::V{index}(value)=>{test}",union.index())
            }).collect::<Vec<_>>().join(",");
            return format!("match {value}{{{arms}}}");
        }
        let Type::Record(record)=input else{return String::from("false")};
        let Some((index,field))=self.program.records[record].fields.iter().enumerate().find(|(_,field)|field.name.text==selector.field) else{return String::from("false")};
        let field_value=format!("&({value}).field{index}");
        selector.tags[branch].iter().map(|tag|format!("({})",self.schema_owned_tag_test(field.ty.unwrap(),&field_value,tag))).collect::<Vec<_>>().join("||")
    }
}
