//! Ordinary checked functions translate backend construction Results into throws.
use crate::{Diagnostic, Span, syntax::*, native_bindings::NativeBinding, native_bindings::NativeType};
pub(crate) fn adapter(program:&mut Program,binding:&NativeBinding)->Result<Function,Diagnostic>{
    let mut function=crate::syntax::parse("function adapter(schema:S,limit:L):S{match(backend(schema,limit)){Ok(value)=>{return value;},Err(error)=>{throw error;}}}")?.functions.remove(0);
    let offset=program.functions.iter().map(|function|function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0)+1;
    let shift=|span:&mut Span|{span.start+=offset;span.end+=offset;};
    function.name.text=format!("@schema:{}",binding.name);shift(&mut function.name.span);shift(&mut function.scope_span);
    for (name,_,_) in &mut function.params {shift(&mut name.span);}
    let NativeType::Structured(signature)=binding.parameters[0].ty else{unreachable!()};
    let annotation=crate::native_type_resolution::project(&mut program.records,signature,function.name.span)?;
    let limit=match binding.parameters[1].ty {
        NativeType::Int=>crate::native_signatures::Signature::Int,
        NativeType::Number=>crate::native_signatures::Signature::Number,
        _=>return Err(Diagnostic::new("schema-limit-type","Schema constraints require an integer or number limit.",function.name.span)),
    };
    let limit=crate::native_type_resolution::project(&mut program.records,&limit,function.name.span)?;
    let replacements=std::collections::HashMap::from([("S".to_owned(),annotation),("L".to_owned(),limit)]);
    crate::source_walk::function_annotations(&mut function,&mut |annotation|crate::type_substitution::apply(annotation,&replacements))?;
    crate::source_walk::function_body(&mut function,&mut |annotation|crate::type_substitution::apply(annotation,&replacements),&mut |expression|{
        shift(&mut expression.span);
        if matches!(&expression.kind,ExprKind::Name(name) if name=="backend"){expression.kind=ExprKind::Name(format!("@native:std:schema#{}",binding.name));}
        Ok(())
    })?;
    Ok(function)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construction_adapter_preserves_declared_limit_type() {
        static PARAMETERS:[crate::native_bindings::NativeParameter;2]=[
            super::super::namespace::MIN_INT.parameters[0],
            crate::native_bindings::NativeParameter{ty:NativeType::Number,..super::super::namespace::MIN_INT.parameters[1]},
        ];
        let binding=NativeBinding{parameters:&PARAMETERS,..super::super::namespace::MIN_INT};
        let mut program=crate::syntax::parse("function main(){}").unwrap();
        let function=adapter(&mut program,&binding).unwrap();
        let crate::type_syntax::ValueAnnotationKind::Named(name)=&function.params[1].1.kind else{panic!("expected scalar limit")};
        assert_eq!(name.text,"number");
    }
}

/// Unwrap a checked validation Result through normal typed exception machinery.
pub(crate) fn parse_adapter(program: &Program) -> Result<Function, Diagnostic> {
    let mut function = crate::syntax::parse("function adapter<R>(result:R){match(result){Ok(value)=>{return value;},Err(error)=>{throw error;}}}")?.functions.remove(0);
    let offset = program.functions.iter().map(|function| function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0) + 1;
    let shift = |span: &mut Span| { span.start += offset; span.end += offset; };
    function.name.text = "@schema:parseResult".into();
    shift(&mut function.name.span);
    shift(&mut function.scope_span);
    for (name, _, _) in &mut function.params { shift(&mut name.span); }
    crate::source_walk::function_body(&mut function, &mut |_| Ok(()), &mut |expression| {
        shift(&mut expression.span);
        Ok(())
    })?;
    Ok(function)
}
