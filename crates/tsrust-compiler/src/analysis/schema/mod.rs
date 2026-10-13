//! Schema method lowering preserves ordinary native argument ownership checks.
use super::*;
impl Analyzer {
    /// Keep input consumption, callback effects and validation identical to safeParse.
    pub(in crate::analysis) fn schema_parse_call(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, _) = &expr.kind else { return Ok(None); };
        let ExprKind::Field(base, method) = &callee.kind else { return Ok(None); };
        if method != "parse" { return Ok(None); }
        let known = matches!(self.expression_type_hint(context, base), Some(Type::Record(id))
            if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::Schema)
                || self.schema_pipelines.types(id).is_some());
        let mut root = &**base;
        while let ExprKind::Call(callee, _) = &root.kind {
            match &callee.kind {
                ExprKind::Field(receiver, _) => root = receiver,
                _ => { root = callee; break; }
            }
        }
        let constructing = matches!(&root.kind, ExprKind::Name(name)
            if name.starts_with("@schema:") || name.starts_with("@native:std:schema#"));
        if !known && !constructing { return Ok(None); }
        let mut validation = expr.clone();
        let ExprKind::Call(callee, args) = &mut validation.kind else { unreachable!() };
        if args.len() != 1 { return Err(Diagnostic::new("arity", "parse expects one input value.", expr.span)); }
        let ExprKind::Field(_, method) = &mut callee.kind else { unreachable!() };
        *method = "safeParse".into();
        // Check the original receiver and input exactly once, including effects.
        let result = self.expression(context, &mut validation, Mode::Consume, loans, None)?;
        let arguments = vec![self.checked_value_annotation(result, expr.span)?];
        let name = self.generic_functions.request_explicit_value("@schema:parseResult", arguments, Vec::new(), expr.span, self.function_stack.len())?;
        let name = self.publish_generic_call(context, name)?;
        expr.kind = ExprKind::Call(Box::new(Expr::new(ExprKind::Name(name), expr.span)), vec![validation]);
        let (index, captures, params) = self.declared_call_contract(context, expr)?;
        // The one argument is an already checked owned Result. Rechecking its
        // source would consume input twice and duplicate callback access effects.
        self.require(result, Some(params[0].ty), expr.span)?;
        let output = self.finish_declared_call(context, expr, index, &captures, &params, loans)?;
        self.require(output, expected, expr.span)?;
        Ok(Some(output))
    }

    // A normalized scalar union can reuse an existing scalar schema identity.
    // Establish this hint without analyzing (and therefore consuming) any branch.
    // Callback pipelines retain separate identities and must not enter this path.
    pub(in crate::analysis) fn schema_union_type_hint(&self,context:&Context,expr:&Expr)->Option<Type>{
        let ExprKind::Call(callee,args)=&expr.kind else{return None};
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:union"){return None;}
        let [argument]=args.as_slice() else{return None};
        let ExprKind::Array(branches)=&argument.kind else{return None};
        let schemas=branches.iter().map(|branch|{
            let Type::Record(id)=self.expression_type_hint(context,branch)? else{return None};
            let record=&self.program.records[id];
            if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return None;}
            Some((id,*record.application.as_ref()?.resolved_arguments.first()?))
        }).collect::<Option<Vec<_>>>()?;
        // Do not widen singleton, nominal or distinct scalar alternatives here.
        // The hint must agree with the union constructor's actual normalization.
        let (id,output)=schemas.first()?;
        (matches!(output,Type::String|Type::Int|Type::Number)&&schemas.iter().all(|(_,branch)|output==branch)).then_some(Type::Record(*id))
    }

    pub(in crate::analysis) fn schema_array_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:array"){return Ok(None);}
        if args.len()!=1{return Err(Diagnostic::new("arity","s.array expects one owned element schema.",expr.span));}
        let child=self.expression(context,&mut args[0],Mode::Consume,loans,None)?;
        let Type::Record(id)=child else{return Err(Diagnostic::new("schema-type","s.array requires an element schema.",args[0].span));};
        if self.schema_pipelines.types(id).is_some() {
            let child=args.remove(0);
            return self.schema_pipeline_array(expr,id,child,expected).map(Some);
        }
        let record=&self.program.records[id];
        if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return Err(Diagnostic::new("schema-type","s.array requires an element schema.",args[0].span));}
        let element=record.application.as_ref().unwrap().resolved_arguments[0];
        let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema),span:expr.span},arguments:vec![ValueAnnotation{span:expr.span,kind:Kind::Array(Box::new(types.annotation(element,expr.span)?))}]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,expr.span)?;
        expr.kind=ExprKind::NativeConstructor(std::mem::take(args),Some(Box::new(annotation)));
        Ok(Some(result))
    }
    pub(in crate::analysis) fn schema_composite_validation(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        let ExprKind::Field(base,member)=&callee.kind else{return Ok(None)};
        if member!="safeParse" {return Ok(None);}
        let constructing=matches!(&base.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(name) if (name.starts_with("@schema:")||name.starts_with("@native:std:schema#"))) || matches!(&callee.kind,ExprKind::Field(_,method) if matches!(method.as_str(),"strip"|"retain"|"optional"|"nullable"|"default"|"min"|"max"|"gt"|"lt")));
        let mut receiver=(**base).clone();
        let receiver_type=if constructing {Some(self.expression(context,&mut receiver,Mode::Read,loans,None)?)}else{self.expression_type_hint(context,base)};
        let Some(Type::Record(id))=receiver_type else{return Ok(None)};
        if constructing && self.schema_pipelines.types(id).is_some() {
            return self.schema_pipeline_validation_receiver(context,expr,receiver,loans,expected);
        }
        let record=&self.program.records[id];
        if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return Ok(None);}
        let output=record.application.as_ref().unwrap().resolved_arguments[0];
        if !matches!(output,Type::Array(_)|Type::Record(_)|Type::Union(_)|Type::Literal(_,_)|Type::Enum(_,_)|Type::Null|Type::String|Type::Int|Type::Number|Type::Boolean){return Ok(None);}
        let validation_output=if let Type::Record(object)=output && self.program.records[object].name.text.starts_with("@schema:retained:"){self.program.records[object].fields[0].ty.unwrap()}else{output};
        let mut pending=vec![validation_output];let mut seen=HashSet::new();
        while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty{
            Type::Array(element)=>pending.push(element.ty()),
            Type::Union(id)=>pending.extend(self.unions.members(id)),
            Type::Record(id) if self.program.records[id].name.text.starts_with("@schema:retained:")=>pending.push(self.program.records[id].fields[0].ty.unwrap()),
            Type::Record(id) if self.program.records[id].native_type.is_none()&&!self.program.records[id].is_tuple()=>pending.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty)),
            Type::Literal(_,_)|Type::Enum(_,_)|Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined=>{},
            _=>return Err(Diagnostic::new("schema-type","Composite validation requires supported scalar, array or object fields.",callee.span)),
        }}
        if args.len()!=1{return Err(Diagnostic::new("arity","safeParse expects one input value.",expr.span));}
        if !constructing {self.expression(context,&mut receiver,Mode::Read,loans,Some(Type::Record(id)))?;}
        let hint=self.expression_type_hint(context,&args[0]);
        let json=matches!(hint,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonValue));
        if output==Type::Null && !json && hint!=Some(Type::Null){return Err(Diagnostic::new("schema-input","A null schema accepts null or an owned JsonValue.",args[0].span));}
        let conversion=hint.is_some_and(|input|{
            let (mut from,mut to)=(input,output);
            while let (Type::Array(a),Type::Array(b))=(from,to){from=a.ty();to=b.ty();}
            from.scalar_base()==Type::Int && (to.scalar_base()==Type::Number || matches!(to,Type::Union(id) if self.unions.members(id).iter().any(|ty|ty.scalar_base()==Type::Number) && self.unions.members(id).iter().all(|ty|matches!(ty.scalar_base(),Type::Number|Type::Null|Type::Undefined))))
        });
        if !json && !conversion && matches!(validation_output, Type::String | Type::Boolean | Type::Int | Type::Number)
            && hint.is_some_and(|input| input.scalar_base() != validation_output) {
            return Err(Diagnostic::new("schema-input", "safeParse requires a matching concrete scalar or owned JsonValue input.", args[0].span));
        }
        let owned_input=if json||conversion {validation_output}else{self.schema_default_input(context,validation_output,&args[0])?};
        let expected_input=if json||conversion {hint}else{Some(owned_input)};
        let actual=self.expression(context,&mut args[0],Mode::Consume,loans,expected_input)?;
        self.require(actual,expected_input,args[0].span)?;
        let error=self.program.records.iter().position(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::ValidationError)).unwrap();
        let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:"Result".into(),span:expr.span},arguments:vec![types.annotation(output,expr.span)?,types.annotation(Type::Record(error),expr.span)?]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;
        self.require(result,expected,expr.span)?;
        args.insert(0,receiver);
        callee.kind=ExprKind::Name("@schema:validate".into());
        let mut pending=vec![output];let mut visited=HashSet::new();let mut factories=Vec::new();
        while let Some(ty)=pending.pop(){
            if !visited.insert(ty){continue;}
            // Normalizing output unions can erase an intermediate branch union.
            // Its default factory still executes and its exception must propagate.
            pending.extend(self.schema_choices.for_output(ty).flat_map(|(_,choice)|choice.branches.iter().copied()));
            if let Some(&family)=self.schema_factories.get(&ty){factories.push(family);}
            factories.extend(crate::stdlib::schema::defaults::inherited_factories(ty,&self.schema_factories,|id|self.unions.members(id).to_vec()).into_iter().map(|(_,family)|family));
            match ty {
                Type::Array(element)=>pending.push(element.ty()),
                Type::Union(id)=>pending.extend(self.unions.members(id)),
                Type::Record(id) if self.program.records[id].name.text.starts_with("@schema:retained:")=>pending.push(self.program.records[id].fields[0].ty.unwrap()),
                Type::Record(id)=>pending.extend(self.program.records[id].fields.iter().filter_map(|field|field.ty)),
                _=>{}
            }
        }
        factories.sort_unstable();factories.dedup();
        expr.schema_invocations=(!factories.is_empty()).then(||factories.into_boxed_slice());
        expr.target=Some(CallTarget::SchemaValidate);
        Ok(Some(result))
    }
    pub(in crate::analysis) fn schema_method_call(&self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()); };
        let ExprKind::Field(base, member) = &callee.kind else { return Ok(()); };
        let Some(Type::Record(id)) = self.expression_type_hint(context, base) else { return Ok(()); };
        let record = &self.program.records[id];
        if record.native_type != Some(crate::native_signatures::NativeTypeId::Schema) { return Ok(()); }
        if matches!((member.as_str(),record.application.as_ref().and_then(|application|application.resolved_arguments.first())), ("min"|"max",Some(Type::String|Type::Int|Type::Number)) | ("gt"|"lt",Some(Type::Int|Type::Number))) {
            if args.len()!=1 {return Err(Diagnostic::new("arity","Constraints expect one numeric limit.",expr.span));}
            args.insert(0,(**base).clone());
            let kind=match record.application.as_ref().unwrap().resolved_arguments[0] {Type::Int=>"Int",Type::Number=>"Number",_=>"String"};
            callee.kind=ExprKind::Name(format!("@schema:__{}{kind}",member));
            return Ok(());
        }
        if member != "safeParse" { return Err(Diagnostic::new("schema-member", "Unsupported schema method.", callee.span)); }
        if args.len() != 1 { return Err(Diagnostic::new("arity", "safeParse expects one input value.", expr.span)); }
        let output = record.application.as_ref().and_then(|application| application.resolved_arguments.first()).copied();
        let input = self.expression_type_hint(context, &args[0]);
        let json = matches!(input, Some(Type::Record(input)) if self.program.records[input].native_type == Some(crate::native_signatures::NativeTypeId::JsonValue));
        let symbol = match (output, input, json) {
            (Some(Type::String), Some(Type::String), _) => "@native:std:schema#__safeParseOwnedString",
            (Some(Type::String), _, true) => "@native:std:schema#__safeParseString",
            (Some(Type::Boolean), Some(Type::Boolean), _) => "@native:std:schema#__safeParseBooleanValue",
            (Some(Type::Boolean), _, true) => "@native:std:schema#__safeParseBoolean",
            (Some(Type::Int), Some(Type::Int), _) => "@native:std:schema#__safeParseIntValue",
            (Some(Type::Int), _, true) => "@native:std:schema#__safeParseInt",
            (Some(Type::Number), Some(Type::Number), _) => "@native:std:schema#__safeParseNumberValue",
            (Some(Type::Number), Some(Type::Int), _) => "@native:std:schema#__safeParseNumberInt",
            (Some(Type::Number), _, true) => "@native:std:schema#__safeParseNumber",
            (Some(Type::Null), Some(Type::Null), _) => "@native:std:schema#__safeParseNullValue",
            (Some(Type::Null), _, true) => "@native:std:schema#__safeParseNull",
            (Some(Type::String | Type::Boolean | Type::Int | Type::Number | Type::Null), _, _) => return Err(Diagnostic::new("schema-input", "safeParse requires a matching concrete scalar or owned JsonValue input.", args[0].span)),
            _ => return Err(Diagnostic::new("schema-type", "safeParse currently supports string, boolean, integer, number and null schemas.", callee.span)),
        };
        args.insert(0, (**base).clone());
        callee.kind = ExprKind::Name(symbol.into());
        Ok(())
    }
}

#[path = "collections.rs"]
mod schema_collections;
#[path = "context.rs"]
mod schema_context;
#[path = "decode.rs"]
mod schema_decode;
#[path = "defaults.rs"]
mod schema_defaults;
#[path = "diagnostic_options.rs"]
mod schema_diagnostic_options;
#[path = "diagnostic_paths.rs"]
mod schema_diagnostic_paths;
#[path = "enums.rs"]
mod schema_enums;
#[path = "export.rs"]
mod schema_export;
#[path = "literals.rs"]
mod schema_literals;
#[path = "objects.rs"]
mod schema_objects;
#[path = "pipeline_inputs.rs"]
mod schema_pipeline_inputs;
#[path = "pipeline_objects.rs"]
mod schema_pipeline_objects;
#[path = "pipeline_unions.rs"]
mod schema_pipeline_unions;
#[path = "pipeline_wrappers.rs"]
mod schema_pipeline_wrappers;
#[path = "pipelines.rs"]
mod schema_pipelines;
#[path = "presence.rs"]
mod schema_presence;
#[path = "recursive.rs"]
mod schema_recursive;
#[path = "unions.rs"]
mod schema_unions;
