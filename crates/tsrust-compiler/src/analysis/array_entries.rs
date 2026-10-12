//! Owning positional snapshots of a borrowed array receiver.
use super::*;

impl Analyzer {
    pub(super) fn array_entries(&mut self, context:&mut Context, base:&mut Expr, loans:&Loans, expected:Option<Type>, span:Span) -> Result<(Type,usize),Diagnostic> {
        let previous=self.builtin_literals;
        self.builtin_literals=true;
        let source_context=if matches!(base.kind,ExprKind::Array(_)) {
            match expected {
                Some(Type::Array(ArrayElement::Record(pair,0))) if self.program.records[pair].is_tuple() && self.program.records[pair].fields.len()==2 && self.program.records[pair].fields[0].ty==Some(Type::Int) => self.program.records[pair].fields[1].ty.and_then(ArrayElement::from_value_type).map(Type::Array),
                _=>None,
            }
        } else {None};
        let receiver=self.expression(context,base,Mode::Read,loans,source_context);
        self.builtin_literals=previous;
        let Type::Array(element)=receiver? else {
            return Err(Diagnostic::new("unsupported","entries() is available on data arrays.",base.span));
        };
        self.require_cloneable(Type::Array(element),base.span)?;
        let types=crate::generic_inference::Types { callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        let mut annotation=crate::type_syntax::ValueAnnotation {
            kind:crate::type_syntax::ValueAnnotationKind::Tuple(vec![types.annotation(Type::Int,span)?,types.annotation(element.ty(),span)?]),span,
        };
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let Type::Record(pair)=self.resolve_value_annotation(&annotation,false)? else {unreachable!("materialized tuple")};
        let result=Type::Array(ArrayElement::Record(pair,0));
        self.require(result,expected,span)?;
        Ok((result,pair))
    }
}
