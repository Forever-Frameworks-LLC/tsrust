//! Singleton schema construction preserves the checked value identity.
use super::*;
impl Analyzer {
    pub(in crate::analysis) fn schema_literal_constructor(&mut self,context:&mut Context,expr:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expr.kind else{return Ok(None)};
        if !matches!(&callee.kind,ExprKind::Name(name) if name=="@schema:literal"){return Ok(None);}
        if args.len()!=1{return Err(Diagnostic::new("arity","s.literal expects one string, integer, number or boolean or null literal.",expr.span));}
        use crate::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,TypeName};
        let kind=match &args[0].kind {ExprKind::Number(_)=>crate::literal_types::LiteralKind::Number,ExprKind::Unary{value,..} if matches!(value.kind,ExprKind::Number(_))=>crate::literal_types::LiteralKind::Number,_=>crate::literal_types::LiteralKind::Int};
        let null_value=matches!(args[0].kind,ExprKind::Null)||self.expression_type_hint(context,&args[0])==Some(Type::Null);
        let literal=if null_value{None}else{Some(if let Some(value)=crate::literal_types::LiteralValue::expression(&args[0],kind){value?}
            else if let Some(Type::Literal(id,_))=self.expression_type_hint(context,&args[0]){self.program.records[id].literal_definition.clone().unwrap()}
            else{return Err(Diagnostic::new("schema-literal","s.literal requires a statically known string, integer, number or boolean value; a broad scalar type does not identify one value.",args[0].span));})};
        let mut annotation=ValueAnnotation{span:expr.span,kind:Kind::Applied{name:TypeName{text:crate::native_type_resolution::linked_name(crate::native_signatures::NativeTypeId::Schema),span:expr.span},arguments:vec![ValueAnnotation{span:args[0].span,kind:literal.map(|literal|Kind::Literal(literal.annotation())).unwrap_or_else(||Kind::Named(TypeName{text:"null".into(),span:args[0].span}))}]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions,&mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;
        let Type::Record(schema)=result else{unreachable!()};let output=self.program.records[schema].application.as_ref().unwrap().resolved_arguments[0];
        self.expression(context,&mut args[0],Mode::Consume,loans,Some(output))?;
        self.require(result,expected,expr.span)?;
        expr.kind=ExprKind::NativeConstructor(std::mem::take(args),Some(Box::new(annotation)));
        Ok(Some(result))
    }
}
