//! Recursive definitions reserve a type knot and a non-owning plan marker.
use super::*;
use crate::stdlib::schema::pipelines::{RecursivePlan,RecursiveReference};

impl Analyzer {
    /// Unfold a checked recursive array receiver through its real enum payload.
    /// Keep the projection in the tree so storage access and native borrowing
    /// retain the original owner; changing only the type would erase the tag.
    pub(in crate::analysis) fn recursive_array_receiver(&self, context: &mut Context, base: &mut Expr, ty: Type) -> Type {
        let Type::Union(union) = ty else { return ty; };
        if !self.unions.is_recursive(union) { return ty; }
        let members = self.unions.members(union);
        let [payload @ Type::Array(_)] = members.as_slice() else { return ty; };
        let payload = *payload;
        context.unconditional_projections.insert((union, payload));
        let span = base.span;
        let binding = base.binding;
        let source = std::mem::replace(base, Expr::new(ExprKind::Undefined, span));
        *base = Expr::new(ExprKind::Project { value: Box::new(source), union, narrowed: payload }, span);
        base.ty = Some(payload);
        base.binding = binding;
        payload
    }

    fn recursive_record(&mut self,name:&str,span:Span)->usize {
        let id=self.program.records.len();
        let name=format!("@schema:{name}:{id}");
        self.record_names.insert(name.clone(),id);
        self.fields.push(Default::default());
        self.program.records.push(Record{native_type:None,map_definition:None,set_definition:None,enum_definition:None,literal_definition:None,application:None,
            name:Token{text:name,kind:TokenKind::Name,span},type_parameters:vec![],fields:vec![]});
        id
    }

    pub(in crate::analysis) fn schema_recursive_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
        let ExprKind::Call(callee,args)=&expr.kind else{return Ok(None)};
        let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
        if let Some(identity)=name.strip_prefix("@schema:self:") {
            let id=identity.parse::<usize>().ok().filter(|id|self.schema_pipelines.reference(*id).is_some())
                .ok_or_else(||Diagnostic::new("schema-recursion","Unresolved recursive schema reference.",expr.span))?;
            if !args.is_empty(){return Err(Diagnostic::new("arity","A recursive schema reference takes no arguments.",expr.span));}
            self.require(Type::Record(id),expected,expr.span)?;
            expr.kind=ExprKind::Record(vec![]);
            return Ok(Some(Type::Record(id)));
        }
        if name!="@schema:recursive" {return Ok(None);}
        let [builder]=args.as_slice() else{return Err(Diagnostic::new("arity","s.recursive expects one definition builder.",expr.span));};
        let mut builder=builder.clone();
        let ExprKind::Closure{params,body,..}=&mut builder.kind else {
            return Err(Diagnostic::new("schema-recursion","Recursive definition inference requires a builder callback.",builder.span));
        };
        if params.len()!=1 || params[0].rest {return Err(Diagnostic::new("schema-recursion","A recursive definition builder expects one self parameter.",builder.span));}
        let parameter=params[0].name.text.clone();
        let explicit=if let Some(mut arguments)=self.generic_functions.value_arguments(&context.function_name,expr.span) {
            if arguments.len()!=1 { return Err(Diagnostic::new("arity","s.recursive accepts one output type argument.",expr.span)); }
            let annotation=&mut arguments[0];
            self.generic_records.materialize(annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
            append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
            Some(self.resolve_value_annotation(annotation,false)?)
        }else{None};
        let declaration=self.recursive_record("recursive-output",expr.span);
        let input_declaration=self.recursive_record("recursive-input",expr.span);
        let input=self.unions.reserve_recursive(expr.span)?;
        let Type::Union(input_knot)=input else{unreachable!()};
        self.unions.name_recursive(input_knot,input_declaration,expr.span)?;
        let output=self.unions.reserve_recursive(expr.span)?;
        let Type::Union(knot)=output else{unreachable!()};
        self.unions.name_recursive(knot,declaration,expr.span)?;
        let root=self.recursive_record("recursive-plan",expr.span);
        let reference=self.recursive_record("recursive-reference",expr.span);
        self.schema_pipelines.insert_reference(reference,RecursiveReference{root,input,output:explicit.unwrap_or(output),json_input:true});
        replace_self(body,&parameter,reference)?;
        params.clear();
        // An ordinary immediate invocation runs the definition body once and
        // preserves its capture access, side effects and thrown exceptions.
        let mut invocation=Expr::new(ExprKind::Call(Box::new(builder),vec![]),expr.span);
        let body_type=self.expression(context,&mut invocation,Mode::Consume,loans,None)?;
        let Type::Record(body_id)=body_type else{return Err(Diagnostic::new("schema-type","A recursive definition must return a schema.",expr.span));};
        let (body_input,body_output,json_input)=self.schema_pipelines.types(body_id).or_else(||{
            let record=&self.program.records[body_id];
            (record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)).then(||{
                let output=record.application.as_ref().unwrap().resolved_arguments[0];(output,output,true)
            })
        }).ok_or_else(||Diagnostic::new("schema-type","A recursive definition must return a schema.",expr.span))?;
        self.schema_pipelines.validate_recursive_body(body_id,root).map_err(|_|Diagnostic::new(
            "schema-recursion","Recursive schemas must descend into a container before self and admit a finite value.",expr.span))?;
        // Input and output are separate fixed points: transforms can change
        // each node's shape without changing the next node's accepted input.
        self.unions.bind_recursive(input_knot,vec![body_input],expr.span)?;
        self.unions.bind_recursive(knot,vec![body_output],expr.span)?;
        if let Some(target)=explicit {
            if !crate::stdlib::schema::type_contract::equivalent(body_output,target,&self.program.records,&|id|self.unions.members(id)) {
                return Err(Diagnostic::new("schema-type","The recursive schema output does not match its explicit type argument.",expr.span));
            }
        }
        self.schema_pipelines.insert_recursive(root,RecursivePlan{body:body_id,input:body_input,output:explicit.unwrap_or(body_output),json_input});
        let field=RecordField{name:Token{text:"@schema".into(),kind:TokenKind::Name,span:expr.span},annotation:self.checked_value_annotation(body_type,expr.span)?,ty:Some(body_type),callable_declaration:None,optional:false};
        self.fields[root].insert(field.name.text.clone(),body_type);
        self.program.records[root].fields.push(field.clone());
        self.require(Type::Record(root),expected,expr.span)?;
        expr.kind=ExprKind::Record(vec![(field.name,invocation)]);
        Ok(Some(Type::Record(root)))
    }
}

fn replace_self(body:&mut [Stmt],name:&str,reference:usize)->Result<(),Diagnostic> {
    let mut nested=Vec::new();
    let mut shadows=Vec::new();
    crate::source_walk::body(body,&mut |_|Ok(()),&mut |expression|{
        if let ExprKind::Closure{params,..}=&expression.kind {
            nested.push(expression.span);
            if params.iter().any(|parameter|parameter.name.text==name){shadows.push(expression.span);}
        }
        Ok(())
    })?;
    let inside=|span:Span,regions:&[Span]|regions.iter().any(|region|span.start>=region.start&&span.end<=region.end);
    crate::source_walk::body(body,&mut |_|Ok(()),&mut |expression|{
        if matches!(&expression.kind,ExprKind::Name(found) if found==name)&&!inside(expression.span,&shadows)&&inside(expression.span,&nested) {
            return Err(Diagnostic::new("schema-recursion","The recursive self reference cannot escape into a captured callback.",expression.span));
        }
        Ok(())
    })?;
    let replace=|value:&mut Expr|{
        if matches!(&value.kind,ExprKind::Name(found) if found==name) {
            value.kind=ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@schema:self:{reference}")),value.span)),vec![]);
        }
    };
    crate::source_walk::body(body,&mut |_|Ok(()),&mut |expression|{
        if inside(expression.span,&shadows){return Ok(());}
        if let ExprKind::Call(callee,args)=&mut expression.kind {
            match &mut callee.kind {
                ExprKind::Name(method) if matches!(method.as_str(),"@schema:array"|"@schema:record"|"@schema:map"|"@schema:set")=>{for arg in args{replace(arg);}},
                ExprKind::Name(method) if method=="@schema:object"=>{if let Some(Expr{kind:ExprKind::Record(fields),..})=args.first_mut(){for(_,value)in fields{replace(value);}}},
                ExprKind::Name(method) if method=="@schema:union"=>{if let Some(Expr{kind:ExprKind::Array(branches),..})=args.first_mut(){for value in branches{replace(value);}}},
                ExprKind::Field(base,method) if matches!(method.as_str(),"nullable"|"optional"|"refine"|"superRefine"|"transform"|"default")=>replace(base),
                _=>{},
            }
        }
        if matches!(&expression.kind,ExprKind::Name(found) if found==name) {
            return Err(Diagnostic::new("schema-recursion","Use self inside a schema constructor; it cannot be stored or returned as an ordinary value.",expression.span));
        }
        Ok(())
    })
}
