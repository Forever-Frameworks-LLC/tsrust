//! Leaf namespace calls do not invoke source callbacks or change lexical slots.
use super::*;
pub(super) fn native_observer(symbol: &str) -> bool {
    crate::native_bindings::by_symbol(symbol).is_some_and(|(_, binding)| {
        binding.module == "std:env" && binding.name == "get"
            || binding.module == "std:path"
            && matches!(binding.name, "basename" | "dirname" | "extname" | "isAbsolute" | "normalize" | "join" | "relative")
            || binding.module == "std:os"
                && matches!(binding.name, "platform" | "arch" | "endianness")
            || binding.module == "std:bytes"
                && matches!(binding.name, "fromArray" | "length" | "at" | "alloc" | "copy" | "fromUtf8" | "toUtf8")
            || binding.module == "std:testing-runtime"
                && matches!(binding.name, "checkAssertion" | "assertionDetails" | "numericValue" | "lengthValue" | "hasLength" | "containsValue")
    })
}
/// These calls can fail, but cannot invoke source callbacks or change captures.
pub(super) fn observer_call(expression: &Expr, effects: &effect_bodies::Index) -> bool {
    let ExprKind::Call(callee, _) = &expression.kind else { return false; };
    match &callee.kind {
        ExprKind::Name(name) => native_observer(name),
        ExprKind::Field(receiver, method) => matches!(&receiver.kind,
            ExprKind::Name(name) if name == "Object" && effects.binding(name, receiver.span).is_none())
            && method == "is",
        _ => false,
    }
}
pub(super) fn console(expression: &Expr, effects: &effect_bodies::Index) -> bool {
    let ExprKind::Field(receiver, method) = &expression.kind else { return false; };
    matches!(&receiver.kind, ExprKind::Name(name) if name == "console" && effects.binding(name, receiver.span).is_none())
        && matches!(method.as_str(), "log" | "info" | "debug" | "warn" | "error" | "table" | "assert" | "dir" | "dirxml" | "group" | "groupCollapsed" | "groupEnd" | "count" | "countReset" | "time" | "timeLog" | "timeEnd" | "clear" | "trace" | "profile" | "profileEnd" | "timeStamp")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observer_calls_exclude_callback_runners_and_shadowed_namespaces() {
        for (callee, expected) in [
            ("@native:std:testing-runtime#checkAssertion", true),
            ("@native:std:testing-runtime#runTests", false),
            ("@native:std:testing-runtime#registerTest", false),
            ("@native:std:testing-runtime#shareCallback", false),
            ("@native:std:path#join", true),
            ("@native:std:os#platform", true),
            ("unknown", false),
        ] {
            let program=crate::syntax::parse("function main(){probe();}").unwrap();
            let effects=effect_bodies::Index::new(&program).unwrap();
            let mut function=program.functions[0].clone();
            crate::source_walk::function_body(&mut function,&mut |_|Ok(()),&mut |expression| {
                if let ExprKind::Call(target, _) = &mut expression.kind {
                    target.kind=ExprKind::Name(callee.to_string());
                    assert_eq!(observer_call(expression,&effects),expected,"{callee}");
                }
                Ok(())
            }).unwrap();
        }
        for (source,expected) in [
            ("function main(){Object.is(1,1);}",true),
            ("function main(){Object.assign(1,1);}",false),
            ("function main(Object:int){Object.is(1,1);}",false),
        ] {
            let program=crate::syntax::parse(source).unwrap();
            let effects=effect_bodies::Index::new(&program).unwrap();
            let mut function=program.functions[0].clone();
            crate::source_walk::function_body(&mut function,&mut |_|Ok(()),&mut |expression| {
                if matches!(expression.kind,ExprKind::Call(_, _)) {assert_eq!(observer_call(expression,&effects),expected);}
                Ok(())
            }).unwrap();
        }
    }
    #[test]
    fn console_leaf_selection_requires_known_method_and_unshadowed_namespace() {
        for (source, expected) in [
            ("function main(){console.log(1);}",true),
            ("function main(){console.missing(1);}",false),
            ("function action(console:int){console.log(1);}function main(){}",false),
        ] {
            let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
            let mut found=false;
            for function in &program.functions {
                let mut probe=function.clone();
                crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
                    if let ExprKind::Call(callee,_) = &expression.kind { assert_eq!(console(callee,&effects),expected);found=true; } Ok(())
                }).unwrap();
            }
            assert!(found);
        }
    }
}
