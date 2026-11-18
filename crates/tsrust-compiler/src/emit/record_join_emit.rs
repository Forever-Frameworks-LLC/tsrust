//! Array join registrations follow nominal native record declarations.
use super::*;
impl Emitter<'_> {
    pub(super) fn record_join_registration(&self,id:usize)->Option<String> {
        if self.native_record_declaration(id)!=id{return None;}
        let record=&self.program.records[id];
        if record.literal_definition.is_some(){return None;}
        // A generic native handle (`Sender<Record72>`) is a foreign type: with the
        // program split into crates no crate may implement the shared trait for
        // one instantiation. One generic implementation covers all of them.
        if record.native_type.is_some() && record.application.as_ref().is_some_and(|application|!application.resolved_arguments.is_empty()) {
            let mut alias=String::new();
            if self.future_record(&mut alias,id) || self.http_record(&mut alias,id) {
                let right=alias.trim().strip_prefix(&format!("type Record{id}=")).and_then(|right|right.strip_suffix(';'))?;
                if let Some((path,arguments))=right.split_once('<') {
                    let mut depth=0usize;
                    let count=1+arguments.chars().filter(|c|{match c {'<'|'('|'['=>depth+=1,'>'|')'|']'=>depth=depth.saturating_sub(1),_=>{}} *c==','&&depth==0}).count();
                    let parameters=(0..count).map(|index|format!("T{index}")).collect::<Vec<_>>().join(",");
                    return Some(format!("impl<{parameters}> TsrJoinText for {path}<{parameters}>{{fn append_join_text(&self,output:&mut std::string::String){{output.push_str(\"[object Object]\");}}}}\n"));
                }
            }
        }
        let view_lifetime = native_view_emit::access(self.program, id).is_some() || self.native_views.recursive(id);
        let parameters = if view_lifetime { vec!["'view".to_owned()] } else if record.native_type==Some(crate::native_signatures::NativeTypeId::Deque) && record.native_element().is_some_and(|element|self.native_collection_payload_parameter(element)) { vec!["P".to_owned()] } else { record.fields.iter().enumerate().filter_map(|(index,field)|self.scoped_record_native_field(id,index,field.ty.expect("checked field")).then(||format!("P{index}"))).collect::<Vec<_>>() };
        let arguments=lifetime_parameters(&parameters);
        let bounds=if record.is_tuple() && !view_lifetime {lifetime_parameters(&parameters.iter().map(|parameter|format!("{parameter}:TsrJoinText")).collect::<Vec<_>>())}else{arguments.clone()};
        let mut output=format!("impl{bounds} TsrJoinText for Record{id}{arguments}{{");
        if record.is_tuple(){
            for method in ["append_join_text","append_locale_text"] {
                let _=write!(output,"fn {method}(&self,output:&mut std::string::String){{");
                for field in 0..record.fields.len(){
                    if field!=0{output.push_str("output.push(',');");}
                    let _=write!(output,"self.field{field}.{method}(output);");
                }
                output.push('}');
            }
        }else{output.push_str("fn append_join_text(&self,output:&mut std::string::String){output.push_str(\"[object Object]\");}");}
        output.push_str("}\n");Some(output)
    }
}
