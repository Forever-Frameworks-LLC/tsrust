//! Array join registrations follow nominal native record declarations.
use super::*;
impl Emitter<'_> {
    pub(super) fn record_join_registration(&self,id:usize)->Option<String> {
        if self.native_record_declaration(id)!=id{return None;}
        let record=&self.program.records[id];
        if record.literal_definition.is_some(){return None;}
        let view_lifetime = native_view_emit::access(self.program, id).is_some() || self.native_views.recursive(id);
        let parameters = if view_lifetime { vec!["'view".to_owned()] } else { record.fields.iter().enumerate().filter_map(|(index,field)|self.scoped_record_native_field(id,index,field.ty.expect("checked field")).then(||format!("P{index}"))).collect::<Vec<_>>() };
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
