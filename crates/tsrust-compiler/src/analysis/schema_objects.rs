//! Owned object-schema construction uses ordinary typed record specialization.
use super::*;
impl Analyzer {
    pub(super) fn schema_object_policy(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Field(base,method)=&callee.kind else{return Ok(None)};
        if !matches!(method.as_str(),"strip"|"retain"){return Ok(None);}
        let hinted=self.expression_type_hint(context,base);
        let constructing=matches!(&base.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(name) if matches!(name.as_str(),"@schema:object"|"@schema:union"|"@schema:discriminatedUnion")) || matches!(&callee.kind,ExprKind::Field(_,method) if matches!(method.as_str(),"strip"|"retain"|"default")));
        if !constructing&&!matches!(hinted,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)||self.schema_pipelines.object(id).is_some()||self.schema_pipelines.wrapper(id).is_some_and(|plan|plan.kind==crate::stdlib::schema::pipelines::WrapperKind::Message)){return Ok(None);}
        if !args.is_empty(){return Err(Diagnostic::new("arity","Object policy methods expect no arguments.",expr.span));}
        let mut receiver=(**base).clone();
        let ty=self.expression(context,&mut receiver,Mode::Consume,loans,None)?;
        let Type::Record(id)=ty else{unreachable!()};
        if self.schema_pipelines.wrapper(id).is_some_and(|plan| plan.kind == crate::stdlib::schema::pipelines::WrapperKind::Message) {
            let retaining = method == "retain";
            return self.schema_message_policy(expr, id, receiver, retaining, expected).map(Some);
        }
        if self.schema_pipelines.object(id).is_some() {
            let retaining=method=="retain";
            return self.schema_pipeline_object_policy(expr,id,receiver,retaining,expected).map(Some);
        }
        let output = self.program.records[id].application.as_ref()
            .and_then(|application| application.resolved_arguments.first()).copied();
        let Some(output @ Type::Record(_)) = output else {
            return Err(Diagnostic::new("schema-member", "Object policy methods require a supported object schema.", expr.span));
        };
        let retaining=method=="retain";
        let Type::Record(object)=output else{unreachable!()};
        if self.program.records[object].name.text.starts_with("@schema:retained:"){return Err(Diagnostic::new("schema-member","Object policy is already retained.",expr.span));}
        let result=if retaining {self.schema_retained_type(output,expr.span)?}else{ty};
        self.require(result,expected,expr.span)?;
        args.push(receiver);callee.kind=ExprKind::Name(if retaining {"@schema:retain"}else{"@schema:strip"}.into());expr.target=Some(if retaining {CallTarget::SchemaRetain}else{CallTarget::SchemaStrip});
        Ok(Some(result))
    }

    pub(super) fn schema_retained_value_type(&mut self,output:Type,span:Span)->Result<Type,Diagnostic>{
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let json=self.schema_native_dependency(crate::native_signatures::NativeTypeId::JsonValue,span)?;
        let mut map=ValueAnnotation{span,kind:Kind::Applied{name:TypeName{text:"Map".into(),span},arguments:vec![ValueAnnotation{span,kind:Kind::Named(TypeName{text:"string".into(),span})},ValueAnnotation{span,kind:Kind::Named(TypeName{text:self.program.records[json].name.text.clone(),span})}]}};
        self.generic_records.materialize(&mut map,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let extra=self.resolve_value_annotation(&map,false)?;
        let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        let fields=[("value",output),("extra",extra)].into_iter().map(|(name,ty)|Ok(crate::syntax::RecordField{name:Token{text:name.into(),span,kind:TokenKind::Name},annotation:types.annotation(ty,span)?,ty:Some(ty),callable_declaration:None,optional:false})).collect::<Result<Vec<_>,Diagnostic>>()?;
        if let Some(wrapper)=self.program.records.iter().position(|record|record.name.text.starts_with("@schema:retained:")&&record.fields.len()==2&&record.fields[0].ty==Some(output)&&record.fields[1].ty==Some(extra)){return Ok(Type::Record(wrapper));}
        let wrapper=self.program.records.len();let name=format!("@schema:retained:{wrapper}");
        self.record_names.insert(name.clone(),wrapper);self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
        self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,name:Token{text:name.clone(),span,kind:TokenKind::Name},type_parameters:Vec::new(),fields});
        Ok(Type::Record(wrapper))
    }

    pub(super) fn schema_retained_type(&mut self,output:Type,span:Span)->Result<Type,Diagnostic>{
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let Type::Record(wrapper)=self.schema_retained_value_type(output,span)? else{unreachable!()};
        let name=self.program.records[wrapper].name.text.clone();
        let mut annotation=ValueAnnotation{span,kind:Kind::Applied{name:TypeName{text:crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema),span},arguments:vec![ValueAnnotation{span,kind:Kind::Named(TypeName{text:name,span})}]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        self.resolve_value_annotation(&annotation,false)
    }

    pub(super) fn schema_object_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:object"){return Ok(None);}
        if args.len()!=1{return Err(Diagnostic::new("arity","s.object expects one owned record of field schemas.",expr.span));}
        let previous=self.builtin_literals;self.builtin_literals=true;
        let shape=self.expression(context,&mut args[0],Mode::Consume,loans,None);
        self.builtin_literals=previous;
        let Type::Record(shape)=shape? else{return Err(Diagnostic::new("schema-type","s.object requires a record of field schemas.",args[0].span));};
        if self.program.records[shape].native_type.is_some()||self.program.records[shape].is_tuple(){return Err(Diagnostic::new("schema-type","s.object requires a record of field schemas.",args[0].span));}
        if self.program.records[shape].fields.iter().any(|field| matches!(field.ty,Some(Type::Record(id)) if self.schema_pipelines.types(id).is_some())) {
            let value=args.remove(0);
            return self.schema_pipeline_object(expr,shape,value,expected).map(Some);
        }
        let mut fields=self.program.records[shape].fields.clone();
        for field in &mut fields {
            let Some(Type::Record(child))=field.ty else{return Err(Diagnostic::new("schema-type","Every object field must contain a schema.",field.name.span));};
            let child=&self.program.records[child];
            if child.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return Err(Diagnostic::new("schema-type","Every object field must contain a schema.",field.name.span));}
            let output=child.application.as_ref().unwrap().resolved_arguments[0];
            field.optional=matches!(output,Type::Union(id) if self.unions.members(id).contains(&Type::Undefined));
            field.ty=Some(output);
        }
        let id=self.program.records.len();
        let name=format!("@schema:object:{id}");
        let output=if let Some(Type::Record(schema))=expected
            && self.program.records[schema].native_type==Some(crate::native_signatures::NativeTypeId::Schema)
            && let Type::Record(output)=self.program.records[schema].application.as_ref().unwrap().resolved_arguments[0] {
                let target=&self.program.records[output];
                if fields.len()!=target.fields.len()||fields.iter().any(|field|!target.fields.iter().any(|other|other.name.text==field.name.text&&other.ty==field.ty)){
                    return Err(Diagnostic::new("schema-type","Object schema fields do not match the expected output record.",expr.span));
                }
                output
            }else if let Some(existing)=self.program.records.iter().position(|record|record.name.text.starts_with("@schema:object:")&&record.fields.len()==fields.len()&&record.fields.iter().zip(&fields).all(|(left,right)|left.name.text==right.name.text&&left.ty==right.ty&&left.optional==right.optional)){
                // Output identity describes shape, not runtime field constraints.
                // Each constructor still owns its independently evaluated plan.
                existing
            }else{
                self.record_names.insert(name.clone(),id);
                self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
                self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,name:Token{text:name,span:expr.span,kind:TokenKind::Name},type_parameters:Vec::new(),fields});
                id
            };
        let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema),span:expr.span},arguments:vec![types.annotation(Type::Record(output),expr.span)?]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,expr.span)?;
        expr.kind=ExprKind::NativeConstructor(std::mem::take(args),Some(Box::new(annotation)));
        Ok(Some(result))
    }
}
