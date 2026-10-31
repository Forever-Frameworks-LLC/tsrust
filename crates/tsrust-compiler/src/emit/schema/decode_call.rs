//! Public fused decoder boundary separates JSON errors from validation failures.
use super::*;

impl Emitter<'_> {
    pub(in crate::emit) fn schema_text_decode_call(&self,fun:&CheckedFunction,expr:&Expr,args:&[Expr])->String{
        let Type::Record(root)=args[0].ty.unwrap() else{unreachable!()};
        let mut generated=String::from("fn tsrust_schema_numeric_code(code:&str)->tsrust_schema_issues::Code{use tsrust_schema_issues::Code;match code{\"integer-range\"=>Code::IntegerRange,\"integer-fractional\"=>Code::IntegerFractional,\"number-precision\"=>Code::NumberPrecision,\"number-nonfinite\"=>Code::NumberNonfinite,\"too-small\"=>Code::TooSmall,\"too-large\"=>Code::TooLarge,_=>Code::Type}}");
        let errors=expr.schema_invocations.as_deref().unwrap_or_default().iter().fold(crate::error_effects::FailureSet::default(),|errors,family|errors|self.program.error_effects.storage_errors.get(family).cloned().unwrap_or_default());
        let fallible=errors.runtime||errors.opaque||!errors.explicit.is_empty();
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError"&&record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let (declaration,error_name)=if fallible{self.completion_error_layout("SchemaFactoryError",&errors,runtime,expr.span)}else{("enum SchemaFactoryError {}".into(),"SchemaFactoryError".into())};
        let declaration=format!("{declaration}type SchemaFactoryFailure={error_name};enum PreparedField<P,T>{{Present(P),Default(T)}}enum PreparedPresence<T,P>{{Absent(T),Present(P)}}");
        let validator=self.schema_text_validator(root,&mut generated,&mut HashSet::new(),&errors);
        let Type::Union(result)=expr.ty.unwrap() else{unreachable!()};
        let members=&self.program.unions[result.index()].members;
        let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
        let ok_tag=members.iter().position(|ty|*ty==Type::Record(ok)).unwrap();
        let err_tag=members.iter().position(|ty|*ty==Type::Record(err)).unwrap();
        let Type::Union(failures)=self.program.records[err].fields[0].ty.unwrap() else{unreachable!()};
        let ok_order=self.schema_record_order(ok);
        let err_order=self.schema_record_order(err);
        let error_value=|kind:&str,payload:&str|{
            let(tag,id)=self.program.unions[failures.index()].members.iter().enumerate().find_map(|(tag,ty)|{
                let Type::Record(id)=ty else{return None};
                self.program.records[*id].name.text.starts_with(&format!("@schema:decode-error-{kind}:")).then_some((tag,*id))
            }).unwrap();
            let order=self.schema_record_order(id);
            format!("Union{}::V{err_tag}(Record{err}{{field0:Union{}::V{tag}(Record{}{{field0:String::Borrowed({kind:?}),field1:{payload}{order}}}){err_order}}})",result.index(),failures.index(),self.native_record_declaration(id))
        };
        let json=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::JsonError)).unwrap();
        let json_failure=error_value("json",&format!("Record{}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64{order}}}",self.native_record_declaration(json),order=self.schema_record_order(json)));
        let validation_failure=error_value("validation","tsrust_schema_error(issues)");
        let success=format!("Union{}::V{ok_tag}(Record{ok}{{field0:value{ok_order}}})",result.index());
        let mut direct_generated=String::from("use tsrust_native_json::typed::serde::{self,Deserialize};");
        let direct=self.schema_direct_decoder(root,&mut direct_generated,&mut HashSet::new());
        let (attempt,finish)=if let Some(seed)=direct {
            generated.push_str(&direct_generated);
            (format!("if let Some(Some(value))=tsrust_native_json::typed::try_decode_seed(text.as_ref(),{seed}(schema)){{Ok({success})}}else{{"),"}")
        }else{(String::new(),"")};
        let body=format!("{{{generated}let schema={schema};let text={text};{attempt}match tsrust_native_json::source::document_plain(text.as_ref()){{Err(error)=>Ok({json_failure}),Ok(input)=>{{let mut issues=tsrust_schema_issues::Issues::new(64,32,1024);let mut path=Vec::new();let mut failure=None;let validated={validator}(schema,input,&mut path,&mut issues,&mut failure,None,&mut None,&mut false);if let Some(error)=failure{{Err(error)}}else{{Ok(match validated{{Some(value)=>{success},None=>{validation_failure}}})}}}}}}{finish}}}",schema=self.expr(fun,&args[0],Mode::Read),text=self.expr(fun,&args[1],Mode::Read));
        if fallible{
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
            let call=self.exception_regions.result_call(&body,&error_name,&errors,&unions,expr.span).expect("schema decoder callback exception transport");
            format!("{{{declaration}{call}}}")
        }else{format!("{{{declaration}match {body}{{Ok(value)=>value,Err(error)=>match error{{}}}}}}")}
    }
}
