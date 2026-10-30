//! Named function declaration witnesses. The grammar steps live in
//! `declaration_work.rs`.
#[cfg(test)]
use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_declarations_retain_name_header_body_and_scope() {
        let source = "function main(){function twice(value:int):int{return value+value;} console.log(twice(3));}";
        let program =
            super::super::parse_raw(source, false, Default::default(), Default::default()).unwrap();
        assert_eq!(program.functions.len(), 1);
        let StmtKind::LocalFunction(function) = &program.functions[0].body[0].kind else {
            panic!("lexical declaration AST");
        };
        assert_eq!(function.name.text, "twice");
        assert_eq!(function.params[0].0.text, "value");
        assert!(function.result.is_some());
        assert_eq!(function.body.len(), 1);
        assert_eq!(
            &source[function.scope_span.start..function.scope_span.end],
            "twice(value:int):int{return value+value;}"
        );
        assert_eq!(program.functions[0].body.len(), 2);
    }
}

#[cfg(test)]
mod depth_tests {
    #[test]
    fn nested_named_declarations_parse_at_any_depth() {
        let source = format!(
            "function main(){{{}{}{}}}",
            "function next():int{".repeat(32),
            "return 1;",
            "}".repeat(32)
        );
        assert!(
            super::super::parse_raw(&source, false, Default::default(), Default::default()).is_ok()
        );
        let source = format!(
            "function main(){{{}{}{}}}",
            "function next():int{".repeat(200),
            "return 1;",
            "}".repeat(200)
        );
        assert!(super::super::parse_raw(&source, false, Default::default(), Default::default()).is_ok());
    }
}
