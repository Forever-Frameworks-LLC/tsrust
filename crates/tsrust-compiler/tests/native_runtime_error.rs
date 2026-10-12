mod support;
// This witness includes the shared emitter but exercises only its own helpers.
#[allow(dead_code)]
#[path = "../src/emit/exception_emit.rs"]
mod exception_emit;

#[test]
fn runtime_error_expression_uses_emitted_record_and_string_layout() {
    for ordered in [false,true] {
    let source="type RuntimeError={name:string;code:string;message:string;source:string;start:int;end:int;};function main(){const example:RuntimeError={name:\"\",code:\"\",message:\"\",source:\"\",start:0,end:0};}";
    let source=if ordered {source.replace("};}","};const keys=Object.keys(example);}")}else{source.to_owned()};
    let mut rust=tsrust_compiler::check(&source).unwrap().emit_rust();
    let error=exception_emit::runtime_error_value_with_order(0,ordered,"RangeError","bounds","index unavailable","file\"name.tsr",7,13);
    let verification=format!("let error={error}; assert_eq!(error.field0.as_ref(),\"RangeError\"); assert_eq!(error.field1.as_ref(),\"bounds\"); assert_eq!(error.field2.as_ref(),\"index unavailable\"); assert_eq!(error.field3.as_ref(),\"file\\\"name.tsr\"); assert_eq!(error.field4,7); assert_eq!(error.field5,13);");
    let verification=if ordered {format!("{verification} assert_eq!(error.key_order,[0,1,2,3,4,5]);")}else{verification};
    let failure=exception_emit::runtime_error_value_with_order(0,ordered,"RangeError","overflow","integer addition overflow","input.tsr",7,13);
    let arithmetic=checked_integer_binary("checked_add","{ evaluations.set(evaluations.get()+1); i64::MAX }","{ assert_eq!(evaluations.get(),1); evaluations.set(evaluations.get()+1); 1 }",&failure);
    let verification=format!("{verification} let evaluations=std::cell::Cell::new(0); let result={arithmetic}; assert_eq!(evaluations.get(),2); let completion:TsrCompletion<i64,(),Record0>=tsr_from_result(result); let caught:TsrCompletion<i64,(),Record0>=tsr_catch(completion,|error|{{ assert_eq!(error.field1.as_ref(),\"overflow\"); assert_eq!(error.field4,7); TsrCompletion::Normal(23) }}); assert!(matches!(caught,TsrCompletion::Normal(23)));");
    let index=checked_index("{ order.set(order.get()+1); values.len() }","{ assert_eq!(order.get(),1); order.set(order.get()+1); requested }",&error);
    let verification=format!("{verification} struct NonClone(i64); let values=[NonClone(11),NonClone(23)]; for requested in [-1_i64,2,i64::MAX,1] {{ let order=std::cell::Cell::new(0); let result={index}; assert_eq!(order.get(),2); match result {{ Ok(position)=>{{ assert_eq!(requested,1); let borrowed=&values[position]; assert_eq!(borrowed.0,23); }}, Err(error)=>{{ assert_ne!(requested,1); assert_eq!(error.field1.as_ref(),\"bounds\"); }} }} }}");
    rust.push_str(&exception_emit::typed_error_enum("FunctionError0",&["i64","String","Record0"]));
    let verification=format!("{verification} let errors=[FunctionError0::V0(7),FunctionError0::V1(String::Borrowed(\"text\")),FunctionError0::V2({error})]; let seen=std::cell::Cell::new(0); for error in errors {{ let pending:TsrCompletion<(),(),FunctionError0>=TsrCompletion::Throw(error); let handled:TsrCompletion<(),(),FunctionError0>=tsr_catch(pending,|error|{{ match error {{ FunctionError0::V0(value)=>assert_eq!(value,7), FunctionError0::V1(value)=>assert_eq!(value.as_ref(),\"text\"), FunctionError0::V2(value)=>assert_eq!(value.field1.as_ref(),\"bounds\") }}; seen.set(seen.get()+1); TsrCompletion::Normal(()) }}); assert!(matches!(handled,TsrCompletion::Normal(()))); }} assert_eq!(seen.get(),3);");
    rust.push_str(&exception_emit::typed_error_enum("FunctionError1",&["bool","i64","String","Record0"]));
    let conversion=exception_emit::error_conversion("error","FunctionError0","FunctionError1",&[1,2,3]);
    let verification=format!("{verification} let errors=[FunctionError0::V0(7),FunctionError0::V1(String::Borrowed(\"text\")),FunctionError0::V2({error})]; for error in errors {{ let widened={conversion}; match widened {{ FunctionError1::V0(_)=>panic!(\"wrong error tag\"), FunctionError1::V1(value)=>assert_eq!(value,7), FunctionError1::V2(value)=>assert_eq!(value.as_ref(),\"text\"), FunctionError1::V3(value)=>assert_eq!(value.field1.as_ref(),\"bounds\") }} }}");
    let owned_source=exception_emit::typed_error_enum("OwnedError0",&["Box<NonClone>"]);
    let owned_destination=exception_emit::typed_error_enum("OwnedError1",&["bool","Box<NonClone>"]);
    let owned_conversion=exception_emit::error_conversion("owned","OwnedError0","OwnedError1",&[1]);
    let verification=format!("{verification} {owned_source} {owned_destination} let owned=OwnedError0::V0(Box::new(NonClone(31))); let widened={owned_conversion}; match widened {{ OwnedError1::V1(value)=>assert_eq!(value.0,31), OwnedError1::V0(_)=>panic!(\"wrong owned tag\") }}");
    let call=exception_emit::fallible_call("{ calls.set(calls.get()+1); if succeeds { Ok::<i64,FunctionError0>(23) }else{ Err(FunctionError0::V0(7)) } }","FunctionError0","FunctionError1",&[1,2,3]);
    let verification=format!("{verification} for succeeds in [false,true] {{ let calls=std::cell::Cell::new(0); let pending:TsrCompletion<i64,(),FunctionError1>={call}; assert_eq!(calls.get(),1); let handled:TsrCompletion<i64,(),FunctionError1>=tsr_catch(pending,|error|{{ assert!(!succeeds); match error {{ FunctionError1::V1(value)=>assert_eq!(value,7), _=>panic!(\"wrong propagated tag\") }}; TsrCompletion::Normal(23) }}); assert!(matches!(handled,TsrCompletion::Normal(23))); }}");
    let generic_source=exception_emit::typed_error_enum("GenericError0<P>",&["P"]);
    let generic_destination=exception_emit::typed_error_enum("GenericError1<P>",&["bool","P"]);
    let generic_conversion=exception_emit::error_conversion("source","GenericError0<F>","GenericError1<F>",&[1]);
    let verification=format!(r#"{verification}
{generic_source} {generic_destination}
struct CapturedOwner<'a>(&'a std::cell::Cell<usize>);
impl Drop for CapturedOwner<'_> {{fn drop(&mut self){{self.0.set(self.0.get()+1);}}}}
fn transfer_callable<F:FnOnce()->i64>(source:GenericError0<F>)->GenericError1<F> {{{generic_conversion}}}
let callable_drops=std::cell::Cell::new(0);
let owned_capture=CapturedOwner(&callable_drops);
let callable=move || {{drop(owned_capture);47_i64}};
let widened=transfer_callable(GenericError0::V0(callable));
assert_eq!(callable_drops.get(),0);
match widened {{GenericError1::V1(call)=>assert_eq!(call(),47),GenericError1::V0(_)=>panic!("wrong callable tag")}}
assert_eq!(callable_drops.get(),1);
"#);
    let widening=exception_emit::completion_conversion("{ evaluations.set(evaluations.get()+1); pending }","FunctionError0","FunctionError1",&[1,2,3]);
    let verification=format!(r#"{verification}
for kind in 0..5 {{
 let evaluations=std::cell::Cell::new(0);
 let pending:TsrCompletion<i64,i64,FunctionError0>=match kind {{0=>TsrCompletion::Normal(21),1=>TsrCompletion::Return(22),2=>TsrCompletion::Throw(FunctionError0::V0(23)),3=>TsrCompletion::Break(TsrControlTarget::Loop(24)),_=>TsrCompletion::Continue(TsrControlTarget::Loop(25))}};
 let converted:TsrCompletion<i64,i64,FunctionError1>={widening};
 assert_eq!(evaluations.get(),1);
 match kind {{0=>assert!(matches!(converted,TsrCompletion::Normal(21))),1=>assert!(matches!(converted,TsrCompletion::Return(22))),2=>assert!(matches!(converted,TsrCompletion::Throw(FunctionError1::V1(23)))),3=>assert!(matches!(converted,TsrCompletion::Break(TsrControlTarget::Loop(24)))),_=>assert!(matches!(converted,TsrCompletion::Continue(TsrControlTarget::Loop(25))))}}
}}
"#);
    rust.push_str(include_str!("../src/runtime_completion.rs"));
    let main=rust.find("fn main()").expect("emitted main");
    let opening=main+rust[main..].find('{').expect("main body")+1;
    rust.insert_str(opening,&verification);
    let directory=support::TempDirectory::new();
    let path=directory.0.join("error.rs");
    let binary=directory.0.join("error");
    std::fs::write(&path,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2021").arg(path).arg("-o").arg(&binary),30));
    support::success(&support::run(&mut std::process::Command::new(binary),10));
    }
}

/// Bind operands before checking so failure construction cannot change evaluation.
fn checked_integer_binary(method:&str,left:&str,right:&str,failure:&str)->String {
    format!("{{ let integer_left:i64={left}; let integer_right:i64={right}; tsr_checked(integer_left.{method}(integer_right),||{failure}) }}")
}

/// Validate a position without copying or taking any array element.
fn checked_index(length:&str,index:&str,failure:&str)->String {
    format!("{{ let indexed_length:usize={length}; let indexed_position:i64={index}; tsr_checked(usize::try_from(indexed_position).ok().filter(|position|*position<indexed_length),||{failure}) }}")
}
