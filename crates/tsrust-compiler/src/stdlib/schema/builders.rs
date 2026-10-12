//! Ordinary checked functions translate backend construction Results into throws.
use crate::{Diagnostic, Span, syntax::*, native_bindings::NativeBinding, native_signatures::{Signature,NativeTypeId}};
pub(crate) fn adapter(program:&mut Program,binding:&NativeBinding)->Result<Function,Diagnostic>{
    let mut function=crate::syntax::parse("function adapter(schema:S,limit:int):S{match(backend(schema,limit)){Ok(value)=>{return value;},Err(error)=>{throw error;}}}")?.functions.remove(0);
    let offset=program.functions.iter().map(|function|function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0)+1;
    let shift=|span:&mut Span|{span.start+=offset;span.end+=offset;};
    function.name.text=format!("@schema:{}",binding.name);shift(&mut function.name.span);shift(&mut function.scope_span);
    for (name,_,_) in &mut function.params {shift(&mut name.span);}
    let annotation=crate::native_type_resolution::project(&mut program.records,&Signature::Applied(NativeTypeId::Schema,&[Signature::String]),function.name.span)?;
    let replacements=std::collections::HashMap::from([("S".to_owned(),annotation)]);
    crate::source_walk::function_annotations(&mut function,&mut |annotation|crate::type_substitution::apply(annotation,&replacements))?;
    crate::source_walk::function_body(&mut function,&mut |annotation|crate::type_substitution::apply(annotation,&replacements),&mut |expression|{
        shift(&mut expression.span);
        if matches!(&expression.kind,ExprKind::Name(name) if name=="backend"){expression.kind=ExprKind::Name(format!("@native:std:schema#{}",binding.name));}
        Ok(())
    })?;
    Ok(function)
}
