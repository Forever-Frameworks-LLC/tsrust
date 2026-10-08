//! Native expressions used by typed exception lowering.
pub(crate) fn runtime_error_value_with_order(
    record:usize, ordered:bool, name:&str, code:&str, message:&str,
    source:&str, start:usize, end:usize,
) -> String {
    let order=if ordered {", key_order: [0,1,2,3,4,5]"}else{""};
    format!("Record{record} {{ field0: String::Borrowed({name:?}), field1: String::Borrowed({code:?}), field2: String::Borrowed({message:?}), field3: String::Borrowed({source:?}), field4: {start}_i64, field5: {end}_i64{order} }}")
}

/// Native payload types are resolved by the caller from the checked error contract.
pub(crate) fn typed_error_enum(name:&str,payloads:&[&str])->String {
    let variants=payloads.iter().enumerate().map(|(index,payload)|format!("V{index}({payload})")).collect::<Vec<_>>().join(",");
    format!("\nenum {name} {{ {variants} }}\n")
}

/// Native type arguments need turbofish syntax in constructor and pattern paths.
pub(crate) fn constructor_path(name:&str)->String {
    name.split_once('<').map_or_else(||name.into(),|(head,tail)|format!("{head}::<{tail}"))
}

/// The checked contract supplies a total map; matching consumes each payload once.
pub(crate) fn error_conversion(value:&str,source:&str,destination:&str,mapping:&[usize])->String {
    let source=constructor_path(source);let destination=constructor_path(destination);
    let arms=mapping.iter().enumerate().map(|(index,target)|format!("{source}::V{index}(payload)=>{destination}::V{target}(payload)")).collect::<Vec<_>>().join(",");
    format!("match {value} {{ {arms} }}")
}

/// Widen only a throw while retaining every other completion payload.
pub(crate) fn completion_conversion(pending:&str,source:&str,destination:&str,mapping:&[usize])->String {
    let conversion=error_conversion("error",source,destination,mapping);
    format!("tsr_catch(({pending}),|error|TsrCompletion::Throw({conversion}))")
}

/// Evaluate the call once and widen only its error payload before routing it.
pub(crate) fn fallible_call(call:&str,source:&str,destination:&str,mapping:&[usize])->String {
    let conversion=error_conversion("error",source,destination,mapping);
    format!("tsr_from_result(({call}).map_err(|error|{conversion}))")
}

/// A labeled block transports source exits without a Rust closure boundary.
pub(crate) fn completion_region(id:usize,body:&str)->String {
    format!("'exception_region{id}: {{ {body} TsrCompletion::Normal(()) }}")
}

/// Cleanup executes in the surrounding scope and may replace the pending exit.
pub(crate) fn finally_region(pending:&str,cleanup:&str)->String {
    format!("{{ let pending_completion={pending}; match {cleanup} {{ TsrCompletion::Normal(())=>pending_completion, TsrCompletion::Return(value)=>TsrCompletion::Return(value), TsrCompletion::Throw(error)=>TsrCompletion::Throw(error), TsrCompletion::Break(target)=>TsrCompletion::Break(target), TsrCompletion::Continue(target)=>TsrCompletion::Continue(target) }} }}")
}

/// A handled throw binds its owned payload; other exits bypass the handler.
pub(crate) fn catch_region(pending:&str,binding:&str,handler:&str)->String {
    format!("match {pending} {{ TsrCompletion::Throw({binding})=>{handler}, TsrCompletion::Normal(value)=>TsrCompletion::Normal(value), TsrCompletion::Return(value)=>TsrCompletion::Return(value), TsrCompletion::Break(target)=>TsrCompletion::Break(target), TsrCompletion::Continue(target)=>TsrCompletion::Continue(target) }}")
}

/// Continue with a normal expression value or exit the enclosing source region.
pub(crate) fn propagate_completion(id:usize,completion:&str)->String {
    format!("match {completion} {{ TsrCompletion::Normal(value)=>value, TsrCompletion::Return(value)=>break 'exception_region{id} TsrCompletion::Return(value), TsrCompletion::Throw(error)=>break 'exception_region{id} TsrCompletion::Throw(error), TsrCompletion::Break(target)=>break 'exception_region{id} TsrCompletion::Break(target), TsrCompletion::Continue(target)=>break 'exception_region{id} TsrCompletion::Continue(target) }}")
}
