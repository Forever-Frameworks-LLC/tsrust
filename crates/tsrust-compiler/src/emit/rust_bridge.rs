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
            output.push_str(&module.source.text);output.push_str("\n}\n");
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
                    RustType::String=>format!("{value}.into_owned()"),
                    RustType::Array(inner) if matches!(inner.as_ref(),RustType::Number)=>format!("Vec::from({value})"),
                    RustType::Array(_)=>if shapes.get(index).is_some_and(Option::is_some){format!("Vec::from({value})")}else{format!("match {value}{{NumericArray::Heap(values)=>values,NumericArray::Small{{len,values}}=>values[..len].to_vec()}}")},
                    RustType::Borrow(inner,mutable)=>match inner.as_ref(){RustType::String=>format!("{value}.as_ref()"),RustType::Array(_)=>format!("{value}.{}()",if *mutable{"as_mut_slice"}else{"as_slice"}),_=>value},
                    _=>if matches!(function.bindings[index].mode,Mode::Read|Mode::Mutate){format!("*{value}")}else{value},
                }
            }).collect::<Vec<_>>().join(",");
            return Some(self.rust_output(&export.result,function.result,&format!("tsr_rust{module}::{name}({arguments})")));
        }
        None
    }
    fn rust_output(&self,ty:&RustType,checked:Type,value:&str)->String {
        match ty {
            RustType::String=>format!("String::Owned({value})"),
            RustType::Array(inner) if matches!(inner.as_ref(),RustType::Number)=>value.into(),
            RustType::Array(_)=>format!("NumericArray::Heap({value})"),
            RustType::Result(ok,err)=>{
                let Type::Union(union)=checked else{unreachable!()};let mut arms=String::new();
                for (tag,member) in self.program.unions[union.index()].members.iter().enumerate(){
                    let Type::Record(record)=member else{unreachable!()};let declaration=&self.program.records[*record];
                    let success=declaration.application.as_ref().unwrap().declaration.text=="Result#Ok";
                    let variant=if success{"Ok"}else{"Err"};
                    let payload=if declaration.fields.is_empty(){String::new()}else{format!("field0:{},",self.rust_output(if success{ok}else{err},declaration.fields[0].ty.unwrap(),"payload"))};
                    let _=write!(arms,"{variant}(payload)=>Union{}::V{tag}(Record{}{{{payload}{}}}),",union.index(),self.native_record_declaration(*record),self.schema_record_order(*record).trim_start_matches(','));
                }
                format!("match {value}{{{arms}}}")
            },
            _=>value.into(),
        }
    }
}
