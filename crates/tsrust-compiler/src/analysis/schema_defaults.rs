//! Default factories are retained as uniquely owned readonly environments.
use super::*;
impl Analyzer {
    pub(super) fn schema_may_default(&self,output:Type)->bool {
        if let Type::Record(id)=output && self.program.records[id].name.text.starts_with("@schema:retained:") && self.schema_may_default(self.program.records[id].fields[0].ty.unwrap()){return true;}
        self.schema_factories.contains_key(&output) || !crate::stdlib::schema::defaults::inherited_factories(output,&self.schema_factories,|id|self.unions.members(id).to_vec()).is_empty() || matches!(output,Type::Union(id) if self.unions.members(id).iter().any(|ty|self.schema_factories.contains_key(ty)))
    }

    pub(super) fn schema_owned_input_compatible(&self,output:Type,input:Type)->bool {
        self.schema_owned_input_compatible_at(output,input,&mut HashSet::new())
    }

    /// Compare recursive shapes coinductively, keeping alternative branches
    /// independent so a failed branch cannot prove another branch compatible.
    fn schema_owned_input_compatible_at(&self,output:Type,input:Type,active:&mut HashSet<(Type,Type)>)->bool {
        if output==input || matches!(output,Type::Literal(_,_))&&output.scalar_base()==input.scalar_base(){return true;}
        if matches!(output,Type::Union(_)) && self.schema_scalar_input(output,Span::default()).is_ok_and(|wide|wide.accepts_scalar_value(input)){return true;}
        if !active.insert((output,input)) { return true; }
        let compatible = match (output,input) {
            (Type::Enum(_,_),Type::Union(input))=>self.unions.members(input).iter().all(|input|self.schema_owned_input_compatible_at(output,*input,active)),
            (Type::Enum(_,crate::syntax::EnumKind::String),_)=>input.scalar_base()==Type::String,
            (Type::Enum(_,crate::syntax::EnumKind::Number),_)=>matches!(input.scalar_base(),Type::Int|Type::Number),
            (Type::Union(output),Type::Union(input))=>self.unions.members(input).iter().all(|input|self.unions.members(output).iter().any(|output|self.schema_owned_input_compatible_at(*output,*input,active))),
            (Type::Union(output),_)=>self.unions.members(output).iter().any(|output|self.schema_owned_input_compatible_at(*output,input,active)),
            (output,Type::Union(input)) if self.unions.is_recursive(input)=>self.unions.members(input).iter().all(|input|self.schema_owned_input_compatible_at(output,*input,active)),
            (Type::Array(output),Type::Array(input))=>self.schema_owned_input_compatible_at(output.ty(),input.ty(),active),
            (Type::Map(output),Type::Map(input)) if self.program.records[output].native_type==self.program.records[input].native_type=>{
                let expected=self.program.records[output].map_definition.as_ref().unwrap();
                let actual=self.program.records[input].map_definition.as_ref().unwrap();
                self.schema_owned_input_compatible_at(expected.key,actual.key,active)
                    &&self.schema_owned_input_compatible_at(expected.value,actual.value,active)
            },
            (Type::Set(output),Type::Set(input))=>self.schema_owned_input_compatible_at(
                self.program.records[output].set_definition.as_ref().unwrap().key,
                self.program.records[input].set_definition.as_ref().unwrap().key,active),
            (Type::Record(output),Type::Record(input)) if self.program.records[output].native_type.is_none()&&self.program.records[input].native_type.is_none()=>{
                let expected=&self.program.records[output].fields;let actual=&self.program.records[input].fields;
                actual.iter().all(|field|expected.iter().any(|target|target.name.text==field.name.text&&self.schema_owned_input_compatible_at(target.ty.unwrap(),field.ty.unwrap(),active)))
                    && expected.iter().all(|field|actual.iter().any(|actual|actual.name.text==field.name.text)||self.schema_may_default(field.ty.unwrap()))
            },
            _=>false,
        };
        active.remove(&(output,input));
        compatible
    }
    fn schema_scalar_input(&self,output:Type,span:Span)->Result<Type,Diagnostic> {
        Ok(match output {
            Type::Literal(_,_)=>output.scalar_base(),
            Type::Enum(_,crate::syntax::EnumKind::String)=>Type::String,
            Type::Enum(_,crate::syntax::EnumKind::Number)=>Type::Number,
            Type::Array(element)=>Type::Array(ArrayElement::from_value_type(self.schema_scalar_input(element.ty(),span)?).expect("schema input array")),
            Type::Union(id) if self.unions.is_recursive(id)=>output,
            Type::Union(id)=>self.unions.intern(self.unions.members(id).into_iter().map(|ty|self.schema_scalar_input(ty,span)).collect::<Result<Vec<_>,_>>()?,span)?,
            _=>output,
        })
    }

    pub(super) fn schema_default_input(&mut self,context:&Context,output:Type,input:&Expr)->Result<Type,Diagnostic>{
        if let Some(input_type)=self.expression_type_hint(context,input) && self.schema_owned_input_compatible(output,input_type){return Ok(input_type);}
        if matches!(output,Type::Literal(_,_)|Type::Enum(_,_)|Type::Array(_)|Type::Union(_)){return self.schema_scalar_input(output,input.span);}
        let Type::Record(object)=output else{return Ok(output)};
        let original=self.program.records[object].fields.clone();
        let provided=if let ExprKind::Record(values)=&input.kind {values.iter().map(|(name,_)|name.text.clone()).collect::<HashSet<_>>()}else if let Some(Type::Record(actual))=self.expression_type_hint(context,input) {
            let fields=&self.program.records[actual].fields;
            if fields.iter().any(|field|!original.iter().any(|target|target.name.text==field.name.text&&target.ty==field.ty)){return Ok(output);}
            let names=fields.iter().map(|field|field.name.text.clone()).collect::<HashSet<_>>();
            if original.iter().any(|field|!names.contains(&field.name.text))&&original.iter().filter(|field|!names.contains(&field.name.text)).all(|field|self.schema_may_default(field.ty.unwrap())) {return Ok(Type::Record(actual));}
            return Ok(output);
        }else{return Ok(output)};
        if provided.iter().any(|name|!original.iter().any(|field|field.name.text==*name)) {return Ok(output);}
        if original.iter().filter(|field|!provided.contains(&field.name.text)).any(|field|!field.optional&&!self.schema_may_default(field.ty.unwrap())) {return Ok(output);}
        let mut fields=original.clone();fields.retain(|field|provided.contains(&field.name.text)||(field.optional&&!self.schema_may_default(field.ty.unwrap())));
        if let ExprKind::Record(values)=&input.kind {
            for field in &mut fields {
                if let Some((_,value))=values.iter().find(|(name,_)|name.text==field.name.text) {
                    field.ty=Some(self.schema_default_input(context,field.ty.unwrap(),value)?);
                }
            }
        }
        if fields.len()==original.len()&&fields.iter().zip(&original).all(|(a,b)|a.ty==b.ty){return Ok(output);}
        let id=self.program.records.len();let name=format!("@schema:input:{id}");
        self.record_names.insert(name.clone(),id);self.fields.push(fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());
        self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,name:Token{text:name,span:input.span,kind:TokenKind::Name},type_parameters:Vec::new(),fields});
        Ok(Type::Record(id))
    }

    pub(super) fn schema_default_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Field(base,method)=&callee.kind else{return Ok(None)};
        if method!="default" {return Ok(None);}
        let hint=self.expression_type_hint(context,base);
        let mut root=&**base;
        while let ExprKind::Call(callee,_)=&root.kind {
            match &callee.kind {ExprKind::Field(receiver,_)=>root=receiver,_=>{root=callee;break;}}
        }
        let constructing=matches!(&root.kind,ExprKind::Name(name) if name.starts_with("@schema:")||name.starts_with("@native:std:schema#"));
        if !constructing&&!matches!(hint,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema)||self.schema_pipelines.types(id).is_some()){return Ok(None);}
        if args.len()!=1 {return Err(Diagnostic::new("arity","Schema defaults expect one reusable zero-argument factory.",expr.span));}
        let mut receiver=(**base).clone();
        let ty=self.expression(context,&mut receiver,Mode::Consume,loans,None)?;
        let Type::Record(id)=ty else{return Ok(None)};
        let pipeline=self.schema_pipelines.types(id);
        if self.program.records[id].native_type!=Some(crate::native_signatures::NativeTypeId::Schema)&&pipeline.is_none(){return Ok(None);}
        let output=pipeline.map(|(input,_,_)|input).unwrap_or_else(||self.program.records[id].application.as_ref().unwrap().resolved_arguments[0]);
        if pipeline.is_none(){self.require(ty,expected,expr.span)?;}
        let structural=self.callables.shapes.intern_rest(vec![],output.into(),Some(CallbackKind::Owned),0,false);
        let declaration=self.callables.callbacks.len();
        self.callables.callbacks.push(CallbackContract{structural,parameter_names:vec![],params:vec![],result:output,kind:CallbackKind::Owned});
        let factory=&mut args[0];
        if let ExprKind::Closure{result,..}=&mut factory.kind && matches!(result,ResultAnnotation::Inferred(_)) {*result=ResultAnnotation::Named(Box::new(self.checked_value_annotation(output,factory.span)?));}
        if let ExprKind::Closure{id,..}=&factory.kind {self.escaping.insert(*id);}
        let actual=self.expression(context,factory,Mode::Consume,loans,Some(Type::Callback(declaration)))?;
        let Type::Closure(origin)=actual else{return Err(Diagnostic::new("schema-default-factory","A default requires a concrete reusable factory environment.",factory.span));};
        let structural=self.callables.shapes.intern_rest(vec![],output.into(),Some(CallbackKind::Read),0,false);
        let declaration=self.callables.callbacks.len();
        let callback=CallbackContract{structural,parameter_names:vec![],params:vec![],result:output,kind:CallbackKind::Read};
        let Type::StoredCallable(family)=self.callables.storage.intern_owned_read(&self.callables.shapes,&callback,declaration).unwrap() else{unreachable!()};
        self.callables.callbacks.push(callback);
        if !context.template&&!self.discovering_results&&!self.effects_only {
            self.callables.storage.register_origin(family,origin,&self.callables.callbacks,&self.callables.shapes,&self.closures,factory.span)?;
        }else if self.speculative_access_run&&!context.template&&!self.discovering_results {
            self.callables.storage.note_speculative_origin(family,origin);
        }
        factory.wrap_storage(family);
        if pipeline.is_some() {
            let factory=args.remove(0);
            return self.schema_pipeline_wrapper(expr,id,receiver,crate::stdlib::schema::pipelines::WrapperKind::Default(family),Some(factory),expected).map(Some);
        }
        self.schema_factories.insert(output,family);
        args.insert(0,receiver);
        callee.kind=ExprKind::Name("@schema:default".into());
        expr.target=Some(CallTarget::SchemaDefault);
        Ok(Some(ty))
    }
}
