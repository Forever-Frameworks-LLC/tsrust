//! Local Rust adapters use the same checked contracts as source functions.
use super::*;
use crate::project::rust::Type as RustType;
impl Emitter<'_> {
    pub(super) fn rust_modules(&self,output:&mut String){
        let Some(graph)=self.source_graph else{return};
        if graph.modules.values().any(|module|module.rust.is_some()){output.push_str("use rayon as tsr_rayon;\n");}
        for (index,id) in graph.order.iter().enumerate(){
            let module=&graph.modules[id];let Some(rust)=&module.rust else{continue};
            let _=writeln!(output,"mod tsr_rust{index} {{ #![forbid(unsafe_code)]");
            output.push_str("use super::tsr_rayon as rayon;\n");
            if rust.context {output.push_str("use super::tsrust_native_threads::TaskContext;\n");}
            if let Some(package)=&rust.package {
                let _=writeln!(output,"{}{}",crate::project::rust::dependencies::MARKER,serde_json::to_string(package).unwrap());
                for name in &package.crates{let _=writeln!(output,"use {name};");}
            }
            output.push_str(&rust.source.text);output.push_str("\n}\n");
        }
    }
    pub(super) fn rust_function_body(&self,index:usize,shapes:&[Option<crate::fixed_storage::Shape>])->Option<String>{
        let graph=self.source_graph?;let function=&self.program.functions[index];
        for (module,id) in graph.order.iter().enumerate(){
            let Some(rust)=&graph.modules[id].rust else{continue};
            let prefix=format!("__tsrust_module_{module}_");let Some(name)=function.name.strip_prefix(&prefix) else{continue};
            let export=rust.exports.iter().find(|export|export.name==name)?;
            let arguments=export.parameters.iter().enumerate().map(|(index,ty)|{
                let value=format!("value{index}");
                match ty {
                    RustType::Named(_)=>self.rust_input(ty,function.bindings[index].ty,&value,module,rust),
                    RustType::String=>format!("{value}.into_owned()"),
                    RustType::Array(inner) if matches!(inner.as_ref(),RustType::Number)=>format!("Vec::from({value})"),
                    RustType::Array(_)=>if shapes.get(index).is_some_and(Option::is_some){format!("Vec::from({value})")}else{format!("match {value}{{NumericArray::Heap(values)=>values,NumericArray::Small{{len,values}}=>values[..len].to_vec()}}")},
                    RustType::Borrow(inner,mutable)=>match inner.as_ref(){RustType::String=>format!("{value}.as_ref()"),RustType::Array(_)=>format!("{value}.{}()",if *mutable{"as_mut_slice"}else{"as_slice"}),_=>value},
                    _=>if matches!(function.bindings[index].mode,Mode::Read|Mode::Mutate){format!("*{value}")}else{value},
                }
            }).collect::<Vec<_>>().join(",");
            return Some(self.rust_output(&export.result,function.result,&format!("tsr_rust{module}::{name}({arguments})"),module,rust));
        }
        None
    }
    fn rust_output(&self,ty:&RustType,checked:Type,value:&str,module:usize,rust:&crate::project::rust::Module)->String {
        match ty {
            RustType::String=>format!("String::Owned({value})"),
            RustType::Array(inner) if matches!(inner.as_ref(),RustType::Number)=>value.into(),
            RustType::Array(_)=>format!("NumericArray::Heap({value})"),
            RustType::Named(name)=>self.rust_data_output(name,checked,value,module,rust),
            RustType::Result(ok,err)=>{
                let Type::Union(union)=checked else{unreachable!()};let mut arms=String::new();
                for (tag,member) in self.program.unions[union.index()].members.iter().enumerate(){
                    let Type::Record(record)=member else{unreachable!()};let declaration=&self.program.records[*record];
                    let success=declaration.application.as_ref().unwrap().declaration.text=="Result#Ok";
                    let variant=if success{"Ok"}else{"Err"};
                    let payload=if declaration.fields.is_empty(){String::new()}else{format!("field0:{},",self.rust_output(if success{ok}else{err},declaration.fields[0].ty.unwrap(),"payload",module,rust))};
                    let _=write!(arms,"{variant}(payload)=>Union{}::V{tag}(Record{}{{{payload}{}}}),",union.index(),self.native_record_declaration(*record),self.schema_record_order(*record).trim_start_matches(','));
                }
                format!("match {value}{{{arms}}}")
            },
            _=>value.into(),
        }
    }
    fn rust_input(&self,ty:&RustType,checked:Type,value:&str,module:usize,rust:&crate::project::rust::Module)->String {
        use crate::project::rust::Data;
        match ty {
            RustType::String=>format!("{value}.into_owned()"),
            RustType::Array(inner) if matches!(inner.as_ref(),RustType::Number)=>value.into(),
            RustType::Array(_)=>format!("match {value}{{NumericArray::Heap(values)=>values,NumericArray::Small{{len,values}}=>values[..len].to_vec()}}"),
            RustType::Named(name)=>match &rust.types[name] {
                Data::Struct(fields)=> {
                    let Type::Record(record)=checked else{unreachable!()};
                    let fields=fields.iter().map(|(name,ty)| {
                        let (index,field)=self.program.records[record].fields.iter().enumerate().find(|(_,field)|field.name.text==*name).unwrap();
                        format!("{name}:{}",self.rust_input(ty,field.ty.unwrap(),&format!("native.field{index}"),module,rust))
                    }).collect::<Vec<_>>().join(",");
                    format!("{{let native={value};tsr_rust{module}::{name}{{{fields}}}}}")
                },
                Data::Enum(variants)=> {
                    let members=if let Type::Union(union)=checked{self.program.unions[union.index()].members.clone()}else{vec![checked]};
                    let mut arms=Vec::new();
                    for (tag,member) in members.iter().enumerate(){
                        let Type::Record(record)=member else{unreachable!()};
                        let variant=variants.iter().find(|variant|self.program.records[*record].name.text.ends_with(&format!("_{name}{}",variant.name))).unwrap();
                        let fields=variant.fields.iter().map(|(name,ty)| {
                            let (index,field)=self.program.records[*record].fields.iter().enumerate().find(|(_,field)|field.name.text==*name).unwrap();
                            let converted=self.rust_input(ty,field.ty.unwrap(),&format!("native.field{index}"),module,rust);
                            if variant.tuple{converted}else{format!("{name}:{converted}")}
                        }).collect::<Vec<_>>().join(",");
                        let payload=if variant.tuple{format!("({fields})")}else if variant.named{format!("{{{fields}}}")}else{String::new()};
                        let pattern=if let Type::Union(union)=checked{format!("Union{}::V{tag}(native)",union.index())}else{"native".into()};
                        arms.push(format!("{pattern}=>tsr_rust{module}::{name}::{}{payload}",variant.name));
                    }
                    format!("match {value}{{{}}}",arms.join(","))
                }
            },_=>value.into()
        }
    }
    fn rust_data_output(&self,name:&str,checked:Type,value:&str,module:usize,rust:&crate::project::rust::Module)->String {
        use crate::project::rust::Data;
        let record_output=|record:usize,fields:&[(String,RustType)],tuple:bool,named:bool| {
            let fields=self.program.records[record].fields.iter().enumerate().map(|(index,field)| {
                let (position,(_,ty))=fields.iter().enumerate().find(|(_, (name,_))|*name==field.name.text).unwrap();
                let native=if tuple{format!("payload{position}")}else if named{format!("payload_{}",field.name.text)}else{format!("native.{}",field.name.text)};
                format!("field{index}:{}",self.rust_output(ty,field.ty.unwrap(),&native,module,rust))
            }).collect::<Vec<_>>().join(",");
            self.schema_output_record(record,&fields)
        };
        match &rust.types[name] {
            Data::Struct(fields)=>{let Type::Record(record)=checked else{unreachable!()};format!("{{let native={value};{}}}",record_output(record,fields,false,false))},
            Data::Enum(variants)=> {
                let members=if let Type::Union(union)=checked{self.program.unions[union.index()].members.clone()}else{vec![checked]};
                let mut arms=Vec::new();
                for (tag,member) in members.iter().enumerate(){
                    let Type::Record(record)=member else{unreachable!()};
                    let variant=variants.iter().find(|variant|self.program.records[*record].name.text.ends_with(&format!("_{name}{}",variant.name))).unwrap();
                    let pattern=if variant.tuple{format!("({})",(0..variant.fields.len()).map(|index|format!("payload{index}")).collect::<Vec<_>>().join(","))}else if variant.named{"{..}".into()}else{String::new()};
                    let output=record_output(*record,&variant.fields,variant.tuple,!variant.tuple);
                    let output=if let Type::Union(union)=checked{format!("Union{}::V{tag}({output})",union.index())}else{output};
                    // Named variant fields are bound individually to preserve moves.
                    let output=if variant.named{
                        let pattern=format!("{{{}}}",variant.fields.iter().map(|(field,_)|format!("{field}:payload_{field}")).collect::<Vec<_>>().join(","));
                        format!("tsr_rust{module}::{name}::{}{pattern}=>{output}",variant.name)
                    }else{format!("tsr_rust{module}::{name}::{}{pattern}=>{output}",variant.name)};
                    arms.push(output);
                }
                format!("match {value}{{{}}}",arms.join(","))
            }
        }
    }

}
