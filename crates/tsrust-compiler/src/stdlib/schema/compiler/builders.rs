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
    source_adapter(program, "@schema:parseResult", crate::syntax::parse("function adapter<R>(result:R){match(result){Ok(value)=>{return value;},Err(error)=>{throw error;}}}")?.functions.remove(0))
}

/// Public names of the schema outcome alternatives.
pub(crate) const SUCCESS: &str = "SafeParseSuccess";
pub(crate) const FAILURE: &str = "SafeParseError";

/// Present a checked validation Result as a success-discriminated outcome.
/// The match moves the validated output or error once; nothing is copied.
/// The alternatives are generic records with boolean literal discriminants.
pub(crate) fn safe_parse_adapter(program: &mut Program) -> Result<(), Diagnostic> {
    let source = "type SafeParseSuccess<T>={success:true;data:T;};type SafeParseError<E>={success:false;error:E;};function adapter<R,T,E>(result:R):SafeParseSuccess<T>|SafeParseError<E>{match(result){Ok(value)=>{return {success:true,data:value};},Err(error)=>{return {success:false,error:error};}}}";
    let mut parsed = crate::syntax::parse(source)?;
    let function = source_adapter(program, "@schema:safeParseOutcome", parsed.functions.remove(0))?;
    // A program declaring these names keeps its own types; the adapter fails clearly then.
    if program.records.iter().any(|record| record.name.text == SUCCESS || record.name.text == FAILURE) {
        return Err(Diagnostic::new("schema-type", "SafeParseSuccess and SafeParseError are reserved by std:schema safeParse results.", function.name.span));
    }
    program.records.extend(parsed.records.drain(..).filter(|record| record.name.text == SUCCESS || record.name.text == FAILURE));
    program.functions.push(function);
    Ok(())
}

fn source_adapter(program: &Program, name: &str, mut function: Function) -> Result<Function, Diagnostic> {
    let offset = program.functions.iter().map(|function| function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0) + 1;
    let shift = |span: &mut Span| { span.start += offset; span.end += offset; };
    function.name.text = name.into();
    shift(&mut function.name.span);
    shift(&mut function.scope_span);
    for (name, _, _) in &mut function.params { shift(&mut name.span); }
    crate::source_walk::function_annotations(&mut function, &mut |annotation| { shift(&mut annotation.span); Ok(()) })?;
    crate::source_walk::function_body(&mut function, &mut |_| Ok(()), &mut |expression| {
        shift(&mut expression.span);
        Ok(())
    })?;
    Ok(function)
}
