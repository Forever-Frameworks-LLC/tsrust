//! Owned object-schema construction uses ordinary typed record specialization.
use super::*;
impl Analyzer {
    pub(super) fn schema_object_policy(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Field(base,method)=&callee.kind else{return Ok(None)};
        if method!="strip"{return Ok(None);}
        let hinted=self.expression_type_hint(context,base);
        let constructing=matches!(&base.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:object"));
        if !constructing&&!matches!(hinted,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)){return Ok(None);}
        if !args.is_empty(){return Err(Diagnostic::new("arity","strip expects no arguments.",expr.span));}
        let mut receiver=(**base).clone();
        let ty=self.expression(context,&mut receiver,Mode::Consume,loans,None)?;
        let Type::Record(id)=ty else{unreachable!()};
        if !matches!(self.program.records[id].application.as_ref().unwrap().resolved_arguments[0],Type::Record(_)){return Err(Diagnostic::new("schema-member","strip requires an object schema.",expr.span));}
        self.require(ty,expected,expr.span)?;
        args.push(receiver);callee.kind=ExprKind::Name("@schema:strip".into());expr.target=Some(CallTarget::SchemaStrip);
        Ok(Some(ty))
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
        let mut fields=self.program.records[shape].fields.clone();
        for field in &mut fields {
            let Some(Type::Record(child))=field.ty else{return Err(Diagnostic::new("schema-type","Every object field must contain a schema.",field.name.span));};
            let child=&self.program.records[child];
            if child.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return Err(Diagnostic::new("schema-type","Every object field must contain a schema.",field.name.span));}
            field.ty=Some(child.application.as_ref().unwrap().resolved_arguments[0]);
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
            }else{
                self.record_names.insert(name.clone(),id);
                self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
                self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,application:None,name:Token{text:name,span:expr.span,kind:TokenKind::Name},type_parameters:Vec::new(),fields});
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
