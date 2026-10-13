//! Presence builders preserve explicit missing/null alternatives in output types.
use super::*;
impl Analyzer {
    pub(super) fn schema_presence_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Field(base,method)=&callee.kind else{return Ok(None)};
        if !matches!(method.as_str(),"optional"|"nullable"){return Ok(None);}
        let hint=self.expression_type_hint(context,base);
        let mut root=&**base;let mut constructing=false;
        while let ExprKind::Call(callee,_)=&root.kind {
            match &callee.kind {
                ExprKind::Name(name)=>{constructing=name.starts_with("@schema:")||name.starts_with("@native:std:schema#");break;},
                ExprKind::Field(receiver,method) if matches!(method.as_str(),"min"|"max"|"gt"|"lt"|"optional"|"nullable"|"strip"|"retain"|"default"|"refine"|"superRefine"|"transform"|"coerce")=>{
                    if matches!(self.expression_type_hint(context,receiver),Some(Type::Record(id)) if (self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)||self.schema_pipelines.types(id).is_some())){constructing=true;break;}
                    root=receiver;
                },
                _=>break,
            }
        }
        if !constructing&&!matches!(hint,Some(Type::Record(id)) if (self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)||self.schema_pipelines.types(id).is_some())){return Ok(None);}
        if !args.is_empty(){return Err(Diagnostic::new("arity","Presence builders expect no arguments.",expr.span));}
        let mut receiver=(**base).clone();
        let ty=self.expression(context,&mut receiver,Mode::Consume,loans,None)?;
        let Type::Record(id)=ty else{return Ok(None)};
        let added=if method=="optional"{Type::Undefined}else{Type::Null};
        if self.schema_pipelines.types(id).is_some() {
            return self.schema_pipeline_wrapper(expr,id,receiver,crate::stdlib::schema::pipelines::WrapperKind::Presence(added),None,expected).map(Some);
        }
        if self.program.records[id].native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return Ok(None);}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema),span:expr.span},arguments:vec![ValueAnnotation{span:expr.span,kind:Kind::Union(vec![types.annotation(output,expr.span)?,types.annotation(added,expr.span)?])}]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,expr.span)?;
        let Type::Record(target)=result else{unreachable!()};
        let target=self.program.records[target].application.as_ref().unwrap().resolved_arguments[0];
        if target!=output {
            self.schema_choices.intern(crate::stdlib::schema::plans::Choice{output:target,branches:vec![output],selector:None,presence:Some(added)});
        }
        args.push(receiver);callee.kind=ExprKind::Name("@schema:presence".into());expr.target=Some(CallTarget::SchemaPresence);
        Ok(Some(result))
    }
}
