use tsrust_compiler::type_syntax::{ValueAnnotationKind as Kind, parse};

#[test]
fn structured_annotations_preserve_nested_types_and_original_byte_spans() {
    let source = "/* 🦀 */ Result<(User | null)[], Map<string, Box<int>>>[]";
    let value = parse(source).unwrap();
    assert_eq!(
        &source[value.span.start..value.span.end],
        "Result<(User | null)[], Map<string, Box<int>>>[]"
    );
    let Kind::Array(applied) = value.kind else {
        panic!("outer array")
    };
    let Kind::Applied { name, arguments } = applied.kind else {
        panic!("result application")
    };
    assert_eq!(name.text, "Result");
    assert_eq!(&source[name.span.start..name.span.end], "Result");
    assert_eq!(arguments.len(), 2);
    let Kind::Array(group) = &arguments[0].kind else {
        panic!("payload array")
    };
    let Kind::Group(union) = &group.kind else {
        panic!("group")
    };
    let Kind::Union(members) = &union.kind else {
        panic!("nullable union")
    };
    assert_eq!(members.len(), 2);
    assert_eq!(&source[members[1].span.start..members[1].span.end], "null");
    let Kind::Applied { arguments: map, .. } = &arguments[1].kind else {
        panic!("map")
    };
    let Kind::Applied {
        arguments: boxed, ..
    } = &map[1].kind
    else {
        panic!("box")
    };
    assert_eq!(&source[boxed[0].span.start..boxed[0].span.end], "int");
}

#[test]
fn annotation_parser_rejects_incomplete_and_expression_syntax() {
    for source in [
        "",
        "Box<>",
        "Box<int,>",
        "Box<int",
        "Box<int>>",
        "int[",
        "int[3]",
        "int |",
        "a <= b",
        "Box<int>;",
        "A<B<C>>>",
    ] {
        let error = parse(source).unwrap_err();
        assert_eq!(error.code, "syntax", "{source}");
        assert!(error.span.end <= source.len());
        if !matches!(source, "int[" | "int[3]") {
            assert!(error.fixes.is_empty());
        }
    }
    for source in [
        "int",
        "(string)",
        "int[][]",
        "User | null | undefined",
        "Box<Box<int>>",
        "Box<Box<Box<int>>>",
    ] {
        parse(source).unwrap();
    }
}

#[test]
fn type_annotation_nesting_is_bounded_without_a_larger_worker_stack() {
    parse(&format!("int{}", "[]".repeat(128))).unwrap();
    assert_eq!(
        parse(&format!("int{}", "[]".repeat(129))).unwrap_err().code,
        "unsupported"
    );
    for source in [
        format!("{}int{}", "Box<".repeat(140), ">".repeat(140)),
        format!("{}int{}", "(".repeat(140), ")".repeat(140)),
        format!("int{}", "[]".repeat(140)),
        format!(
            "{}int{}{}",
            "Box<".repeat(70),
            ">".repeat(70),
            "[]".repeat(70)
        ),
    ] {
        assert_eq!(parse(&source).unwrap_err().code, "unsupported");
    }
}

#[test]
fn callable_annotations_preserve_modes_nested_types_and_coordinates() {
    use tsrust_compiler::{Mode, type_syntax::CallableOwnership};
    let source = "/* 🦀 */ Once<Function<Result<User, Error> | null, [item: ReadOnly<Box<User>>, update: Mutable<int[]>, next: Owned<Function<Result<int, Error>>>]>>";
    let value = parse(source).unwrap();
    assert_eq!(
        &source[value.span.start..value.span.end],
        source.split("*/ ").nth(1).unwrap()
    );
    let Kind::Callable {
        ownership,
        parameters,
        result,
    } = value.kind
    else {
        panic!("callable")
    };
    assert_eq!(ownership, CallableOwnership::OwnedOnce);
    assert_eq!(parameters.len(), 3);
    // A Function argument states ownership through its own wrapper, so the
    // third argument carries no data access.
    assert!(parameters[2].access.is_none());
    for (parameter, mode) in parameters.iter().zip([Mode::Read, Mode::Mutate]) {
        assert_eq!(parameter.access.as_ref().unwrap().mode, mode);
        assert_eq!(
            &source[parameter.name.span.start..parameter.name.span.end],
            parameter.name.text
        );
    }
    assert_eq!(
        &source[parameters[0].span.start..parameters[0].span.end],
        "item: ReadOnly<Box<User>>"
    );
    assert!(matches!(
        parameters[2].annotation.kind,
        Kind::Callable {
            ownership: CallableOwnership::Owned,
            ..
        }
    ));
    assert!(matches!(result.kind, Kind::Union(_)));
    for (source, expected) in [
        ("()=>int", CallableOwnership::Default),
        ("Once<Function<int>>", CallableOwnership::OwnedOnce),
        ("Function<int>", CallableOwnership::Default),
        ("Owned<Function<int>>", CallableOwnership::Owned),
        ("Mutable<Function<int>>", CallableOwnership::Reusable),
        ("ReadOnly<() => int>", CallableOwnership::Read),
    ] {
        let Kind::Callable { ownership, .. } = parse(source).unwrap().kind else {
            panic!("callable")
        };
        assert_eq!(ownership, expected);
    }
    let array = parse("((path: ReadOnly<string>)=>string)[]").unwrap();
    assert!(matches!(array.kind, Kind::Array(_)));
    let named = parse("(value: read)=>int").unwrap();
    let Kind::Callable { parameters, .. } = named.kind else {
        panic!("callable")
    };
    assert!(parameters[0].access.is_none());
}

#[test]
fn malformed_and_deep_callable_annotations_are_source_errors() {
    for source in [
        "()=>",
        "(x:int)",
        "(x:int,)=>int",
        "(x:)=>int",
        "Once<Function<int>",
        "(x:read)=>",
        "(3:int)=>int",
        "() -> int",
    ] {
        assert_eq!(parse(source).unwrap_err().code, "syntax", "{source}");
    }
    let nested = format!("{}int", "()=>".repeat(140));
    assert_eq!(parse(&nested).unwrap_err().code, "unsupported");
}

#[test]
fn program_annotations_share_parsing_without_changing_shift_expressions() {
    tsrust_compiler::check("type Row={value:(int);};function main(){const left:int=8>>1;const right:int=8>>>1;console.log(left,right);}").unwrap();
    for source in [
        "function main(){const value:Box<Box<int>>=3;}",
        "type Row={value:Box<int>;};function main(){}",
    ] {
        let error = tsrust_compiler::check(source).err().unwrap();
        // Box is not a declared generic record.
        assert_eq!(error.code, "type");
        assert!(error.message.contains("Unknown generic record type"));
        assert!(source[error.span.start..error.span.end].starts_with("Box"));
        assert!(!source[error.span.start..error.span.end].ends_with('='));
    }
    let source = "function main(){const values:int[= []; }";
    let error = tsrust_compiler::check(source).err().unwrap();
    assert_eq!(error.code, "syntax");
    let fix = &error.fixes[0];
    assert_eq!(fix.replacement, "]");
    let mut repaired = source.to_owned();
    repaired.replace_range(fix.span.start..fix.span.end, &fix.replacement);
    tsrust_compiler::check(&repaired).unwrap();
}

#[test]
fn structural_type_resolution_points_to_the_failing_nested_annotation() {
    for (source, expected, code) in [
        (
            "function main(){const value:int|Missing=1;}",
            "Missing",
            "type",
        ),
        (
            "function main(){const value:Box<int>[]= [];}",
            "Box",
            "type",
        ),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap();
        assert_eq!(error.code, code);
        assert_eq!(&source[error.span.start..error.span.end], expected);
    }
}

#[test]
fn explicit_mutable_callable_annotations_preserve_borrowing_and_spans() {
    use tsrust_compiler::type_syntax::{CallableOwnership, ValueAnnotationKind, parse};
    let source = "Mutable<Function<string, [value: ReadOnly<string>]>>";
    let annotation = parse(source).unwrap();
    assert_eq!(annotation.span, tsrust_compiler::Span::new(0, source.len()));
    let ValueAnnotationKind::Callable { ownership, parameters, .. } = annotation.kind else { panic!("callable annotation"); };
    assert_eq!(ownership, CallableOwnership::Reusable);
    assert_eq!(parameters[0].access.as_ref().unwrap().mode, tsrust_compiler::Mode::Read);
    assert_eq!(&source[parameters[0].name.span.start..parameters[0].name.span.end], "value");
}

#[test]
fn literal_annotations_preserve_values_and_compose_with_generic_types() {
    for source in ["'ready'", "\"line\\nend\"", "true", "false", "7", "-9223372036854775808", "9007199254740993", "1.5", "-0.0", "1e3", "0xff", "('ready' | 'done')[]", "Result<'ready', false>"] {
        let annotation=parse(source).unwrap_or_else(|error|panic!("{source}: {error:?}"));
        assert_eq!(annotation.span.start,0);assert_eq!(annotation.span.end,source.len());
    }
}

#[test]
fn literal_annotations_keep_exact_payloads_and_reject_expressions() {
    use tsrust_compiler::type_syntax::LiteralAnnotation as Literal;
    for (source,expected) in [
        ("'ready'",Literal::String("ready".into())),
        ("\"line\\nend\"",Literal::String("line\nend".into())),
        ("9007199254740993",Literal::Int("9007199254740993".into())),
        ("-9223372036854775808",Literal::Int("-9223372036854775808".into())),
        ("-0.0",Literal::Number("-0.0".into())),
        ("true",Literal::Boolean(true)),
        ("false",Literal::Boolean(false)),
    ] {
        let Kind::Literal(actual)=parse(source).unwrap().kind else{panic!("literal type: {source}")};
        assert_eq!(actual,expected);
        assert_eq!(parse(&actual.text()).unwrap().kind,Kind::Literal(actual));
    }
    for source in ["1+2","-'ready'","-true","true()","'ready'.length","1[3]","--1"] {
        assert!(parse(source).is_err(),"expression accepted as type: {source}");
    }
}

#[test]
fn literal_annotation_formatting_preserves_values_without_silently_widening() {
    let source="function wrong(tag:'ready'|'done'):9007199254740993{const fixed:true=false;return 9007199254740992;}";
    let formatted=tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
    assert!(formatted.contains("9007199254740993"));
    assert!(formatted.contains("9007199254740992"));
    assert!(tsrust_compiler::check(&formatted).is_err(),"incompatible literal assignments must not be widened and accepted");
}
