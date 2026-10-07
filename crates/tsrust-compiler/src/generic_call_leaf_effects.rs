//! Leaf namespace calls do not invoke source callbacks or change lexical slots.
use super::*;
pub(super) fn console(expression: &Expr, effects: &effect_bodies::Index) -> bool {
    let ExprKind::Field(receiver, method) = &expression.kind else { return false; };
    matches!(&receiver.kind, ExprKind::Name(name) if name == "console" && effects.binding(name, receiver.span).is_none())
        && matches!(method.as_str(), "log" | "info" | "debug" | "warn" | "error" | "table" | "assert" | "dir" | "dirxml" | "group" | "groupCollapsed" | "groupEnd" | "count" | "countReset" | "time" | "timeLog" | "timeEnd" | "clear" | "trace" | "profile" | "profileEnd" | "timeStamp")
}

#[cfg(test)]
mod tests {
    use super::*;
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
