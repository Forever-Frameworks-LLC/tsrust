//! String enum plans retain one owned literal schema per alternative.
use super::*;
impl Analyzer {
    pub(in crate::analysis) fn schema_enum_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:enum"){return Ok(None);}
        if args.len()!=1{return Err(Diagnostic::new("arity","s.enum expects one nonempty array literal of distinct known strings.",expr.span));}
        if let ExprKind::Name(name)=&args[0].kind && !context.names.contains_key(name) && let Some(&id)=self.record_names.get(name) && let Some(definition)=&self.program.records[id].enum_definition {
            if definition.members.is_empty()||definition.members.iter().any(|member|matches!(member.value,crate::syntax::EnumValue::Number(value) if !value.is_finite())) {
                return Err(Diagnostic::new("schema-enum","An enum schema requires a nonempty declaration with finite numeric or string values.",args[0].span));
            }
            let output=Type::Enum(id,definition.kind);
            return self.schema_enum_plan(expr,output,Vec::new(),expected).map(Some);
        }
        let ExprKind::Array(values)=&mut args[0].kind else{return Err(Diagnostic::new("schema-enum","s.enum requires an array literal so every alternative is statically known.",args[0].span));};
        if values.is_empty(){return Err(Diagnostic::new("schema-enum","An enum schema requires at least one string alternative.",args[0].span));}
        let mut plans=Vec::new();let mut outputs=Vec::new();let mut names=HashSet::new();
        for value in values.iter_mut(){
            let span=value.span;
            let mut plan=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@schema:literal".into()),span)),vec![value.clone()]),span);
            let schema=self.expression(context,&mut plan,Mode::Consume,loans,None)?;
            let Type::Record(schema)=schema else{unreachable!()};let output=self.program.records[schema].application.as_ref().unwrap().resolved_arguments[0];
            if !matches!(output,Type::Literal(_,crate::literal_types::LiteralKind::String)){return Err(Diagnostic::new("schema-enum","Enum alternatives must be statically known strings. Use literal schemas for other scalar kinds.",span));}
            if !names.insert(output){return Err(Diagnostic::new("schema-enum","Enum alternatives must be distinct.",span));}
            outputs.push(output);plans.push(plan);
        }
        let output=self.unions.intern(outputs,expr.span)?;
        if plans.len()==1 {let plan=plans.pop().unwrap();let result=plan.ty.unwrap();self.require(result,expected,expr.span)?;*expr=plan;return Ok(Some(result));}
        self.schema_enum_plan(expr,output,plans,expected).map(Some)
    }
    fn schema_enum_plan(&mut self,expr:&mut Expr,output:Type,plans:Vec<Expr>,expected:Option<Type>)->Result<Type,Diagnostic>{
        let types=crate::generic_inference::Types{callable_annotations:None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema),span:expr.span},arguments:vec![types.annotation(output,expr.span)?]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;self.require(result,expected,expr.span)?;
        expr.kind=ExprKind::NativeConstructor(plans,Some(Box::new(annotation)));Ok(result)
    }
}
