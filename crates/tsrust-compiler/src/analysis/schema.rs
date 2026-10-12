//! Schema method lowering preserves ordinary native argument ownership checks.
use super::*;
impl Analyzer {
    pub(super) fn schema_array_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:array"){return Ok(None);}
        if args.len()!=1{return Err(Diagnostic::new("arity","s.array expects one owned element schema.",expr.span));}
        let child=self.expression(context,&mut args[0],Mode::Consume,loans,None)?;
        let Type::Record(id)=child else{return Err(Diagnostic::new("schema-type","s.array requires an element schema.",args[0].span));};
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
    pub(super) fn schema_method_call(&self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
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
