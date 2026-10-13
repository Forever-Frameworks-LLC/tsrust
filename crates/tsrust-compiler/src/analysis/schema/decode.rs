//! Infer fused JSON result types without constructing a dynamic JSON input.
use super::*;

impl Analyzer {
    pub(in crate::analysis) fn schema_text_decode(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Field(base,method)=&callee.kind else{return Ok(None)};
        if method!="safeParseJson"{return Ok(None);}
        self.schema_native_dependency(crate::native_signatures::NativeTypeId::JsonValue,expr.span)?;
        let mut receiver=(**base).clone();
        let ty=self.expression(context,&mut receiver,Mode::Read,loans,None)?;
        let Type::Record(root)=ty else{return Err(Diagnostic::new("schema-type","safeParseJson requires a schema receiver.",callee.span));};
        let output=if let Some((_,output,_))=self.schema_pipelines.types(root){output}
            else if self.program.records[root].native_type==Some(crate::native_signatures::NativeTypeId::Schema){self.program.records[root].application.as_ref().unwrap().resolved_arguments[0]}
            else{return Err(Diagnostic::new("schema-type","safeParseJson requires a schema receiver.",callee.span));};
        if !self.schema_pipelines.accepts_text(root){
            return Err(Diagnostic::new("schema-json-input","Native Map/Set schemas have no implicit JSON encoding. Decode a JSON array or record and convert explicitly.",expr.span));
        }
        self.schema_decode_inputs(root,expr.span)?;
        if args.len()!=1{return Err(Diagnostic::new("arity","safeParseJson expects one borrowed JSON string.",expr.span));}
        let actual=self.expression(context,&mut args[0],Mode::Read,loans,Some(Type::String))?;
        self.require(actual,Some(Type::String),args[0].span)?;
        let failure=self.schema_decode_error(expr.span)?;
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:"Result".into(),span:expr.span},arguments:vec![self.checked_value_annotation(output,expr.span)?,self.checked_value_annotation(failure,expr.span)?]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,expr.span)?;
        args.insert(0,receiver);
        callee.kind=ExprKind::Name("@schema:decode".into());
        // The final effect pass follows the exact plan to include nested default
        // factories, refinements and transforms, independently of output identity.
        expr.target=Some(CallTarget::SchemaValidate);
        Ok(Some(result))
    }

    fn schema_decode_inputs(&self,root:usize,span:Span)->Result<(),Diagnostic>{
        let mut pending=self.schema_pipelines.coercion_inputs(root).into_iter().map(|ty|(ty,Vec::new(),false)).collect::<Vec<_>>();
        while let Some((ty,mut ancestors,optional))=pending.pop(){
            if ancestors.contains(&ty){return Err(Diagnostic::new("schema-json-input","Recursive coercion input requires recursive schema decoding, which is not implemented yet.",span));}
            ancestors.push(ty);
            let children=match ty{
                Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Literal(_,_)|Type::Enum(_,_)=>Vec::new(),
                Type::Undefined if optional=>Vec::new(),
                Type::Array(element)=>vec![(element.ty(),false)],
                Type::Union(id)=>self.unions.members(id).into_iter().map(|ty|(ty,true)).collect(),
                Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue)=>Vec::new(),
                Type::Record(id) if self.program.records[id].native_type.is_none()&&!self.program.records[id].is_tuple()&&self.program.records[id].map_definition.is_none()&&self.program.records[id].set_definition.is_none()=>self.program.records[id].fields.iter().filter_map(|field|field.ty.map(|ty|(ty,false))).collect(),
                _=>return Err(Diagnostic::new("schema-json-input",format!("JSON text cannot provide coercion input {}. Use an owned JSON data shape or JsonValue.",self.type_name(ty)),span)),
            };
            for (child,optional)in children{pending.push((child,ancestors.clone(),optional));}
        }
        Ok(())
    }

    pub(in crate::analysis) fn schema_native_dependency(&mut self,identity:crate::native_signatures::NativeTypeId,span:Span)->Result<usize,Diagnostic>{
        let start=self.program.records.len();
        let id=crate::native_type_resolution::intern(&mut self.program.records,identity,span)?;
        for record in start..self.program.records.len(){self.record_names.insert(self.program.records[record].name.text.clone(),record);}
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        Ok(id)
    }

    fn schema_decode_error(&mut self,span:Span)->Result<Type,Diagnostic>{
        use crate::native_signatures::NativeTypeId;
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind};
        let mut alternatives=Vec::new();
        for(tag,identity)in [("json",NativeTypeId::JsonError),("validation",NativeTypeId::ValidationError)]{
            let error=self.schema_native_dependency(identity,span)?;
            let literal=crate::literal_types::LiteralValue::String(tag.into());
            let mut annotation=ValueAnnotation{span,kind:ValueAnnotationKind::Literal(literal.annotation())};
            self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
            append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
            let literal=self.resolve_value_annotation(&annotation,false)?;
            let fields=[("kind",literal),("error",Type::Record(error))].into_iter().map(|(name,ty)|Ok(RecordField{
                name:Token{text:name.into(),kind:TokenKind::Name,span},annotation:self.checked_value_annotation(ty,span)?,ty:Some(ty),callable_declaration:None,optional:false,
            })).collect::<Result<Vec<_>,Diagnostic>>()?;
            let record=self.schema_pipeline_shape(fields,&format!("decode-error-{tag}"),span);
            alternatives.push(Type::Record(record));
        }
        self.unions.intern(alternatives,span)
    }
}
