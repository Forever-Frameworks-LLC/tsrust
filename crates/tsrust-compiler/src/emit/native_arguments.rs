//! Convert checked optional inputs to the native Option ABI without copying payloads.
use super::*;
impl Emitter<'_> {
    pub(super) fn native_optional_argument(&self,parameter:&crate::native_bindings::NativeParameter,value:String,ty:Type)->String {
        if !parameter.optional {return value;}
        if let Some(id)=ty.union() {
            let prefix=if !id.copy && parameter.access==Mode::Read {"ViewUnion"}else{"Union"};
            let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,member)| {
                let (binding,result)=if *member==Type::Undefined {("_","None")}else{("value","Some(value)")};
                format!("{prefix}{}::V{tag}({binding})=>{result}",id.index())
            }).collect::<Vec<_>>().join(",");
            format!("match {value}{{{arms}}}")
        }else if ty==Type::Undefined {
            format!("{{let _={value};None}}")
        }else{format!("Some({value})")}
    }
    pub(super) fn native_sql_parameters(&self,parameter:&crate::native_bindings::NativeParameter,value:String,ty:Type,convert:impl Fn(Type,&str)->String)->String {
        let array=|ty:Type,value:&str|{
            if ty==Type::Undefined{return format!("{{let _={value};Vec::new()}}");}
            let Type::Array(array)=ty else{unreachable!("checked SQL parameter array")};let element=array.ty();
            format!("({value}).iter().map(|value: &{}|{}).collect::<Vec<_>>()",self.native_type(element),convert(element,"value"))
        };
        if let Some(id)=ty.union(){
            let prefix=if !id.copy && parameter.access==Mode::Read{"ViewUnion"}else{"Union"};
            let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("{prefix}{}::V{tag}(parameters)=>{}",id.index(),array(*ty,"parameters"))).collect::<Vec<_>>().join(",");
            format!("match {value}{{{arms}}}")
        }else{array(ty,&value)}
    }

}
