use super::*;
#[test]
fn native_view_escaped_carried_capture_keeps_actual_argument_origin() {
    let source = "function save(view:int){return ()=>view;}function relay(first:int,second:int){const ignored=0;const callback=save(second);return callback();}function main(){}";
    let formatted = crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
    let mut parsed = crate::syntax::parse(&formatted).unwrap();
    let annotation = crate::native_type_resolution::project(
        &mut parsed.records,
        &crate::native_signatures::Signature::Named(NativeTypeId::ByteView),
        crate::Span::default(),
    )
    .unwrap();
    for function in &mut parsed.functions[..2] {
        for parameter in &mut function.params {
            parameter.1 = annotation.clone();
        }
    }
    let checked = crate::analysis::analyze(parsed).unwrap();
    let origins = checked.functions[1]
        .native_returns
        .as_ref()
        .expect("returned view must keep its input origin");
    assert_eq!(origins.len(), 1);
    assert_eq!(
        origins[0].root, 1,
        "escaped closure must refer to relay's second argument, not save's binding zero"
    );
}
#[test]
fn native_view_nested_factory_rebinding_keeps_live_receiver_loans() {
    for mutable in [false, true] {
        for (body, expected) in [
            (
                "let reader=wrapped(owner);const value=reader();reader=wrapped(other);",
                Some("native-view-conflict"),
            ),
            (
                "let reader=wrapped(owner);const moved=reader;reader=wrapped(other);const value=moved();",
                None,
            ),
            (
                "const reader=wrapped(owner);return reader();",
                Some("native-view-escape"),
            ),
        ] {
            let tail = format!(
                "function wrapped(owner:int){{const inner=factory(owner);return ()=>inner();}}function caller(owner:int,other:int){{{body}}}"
            );
            let mut parsed = capturing_factory_program(mutable, &tail);
            let owner = parsed.functions[0].params[0].1.clone();
            parsed.functions[1].params[0].1 = owner.clone();
            for parameter in &mut parsed.functions[2].params {
                parameter.1 = owner.clone();
            }
            match (crate::analysis::analyze(parsed), expected) {
                (Err(error), Some(code)) => assert_eq!(error.code, code, "{body}: {error:?}"),
                (Ok(_), None) => (),
                (Err(error), None) => panic!("{body}: {error:?}"),
                (Ok(_), Some(_)) => panic!("{body}: invalid receiver access accepted"),
            }
        }
    }
}
#[test]
fn native_view_nested_factory_preserves_receiver_storage() {
    for mutable in [false, true] {
        let mut parsed = capturing_factory_program(
            mutable,
            "function wrapped(owner:int){const inner=factory(owner);return ()=>inner();}",
        );
        parsed.functions[1].params[0].1 = parsed.functions[0].params[0].1.clone();
        let checked = crate::analysis::analyze(parsed).unwrap();
        assert_eq!(checked.closures.len(), 2);
        assert!(checked.closures[1].function.native_returns.is_some());
        let binding = if mutable {
            &super::super::bindings::VIEW_MUT
        } else {
            &super::super::bindings::VIEW
        };
        let bytes = checked
            .records
            .iter()
            .position(|record| record.native_type == Some(NativeTypeId::Bytes))
            .unwrap();
        let NativeType::Structured(signature) = binding.result else {
            panic!("Result")
        };
        let Type::Union(result) =
            crate::native_binding_types::resolve(&checked.records, signature).unwrap()
        else {
            panic!("union")
        };
        let members = &checked.unions[result.index()].members;
        let [ok, _] = crate::result_identity::variants(&checked.records, members).unwrap();
        let tag = members
            .iter()
            .position(|member| *member == Type::Record(ok))
            .unwrap();
        let mutation = if mutable {
            "value.field0.bytes[0]=8;"
        } else {
            ""
        };
        let expected = if mutable { 8 } else { 4 };
        let probe = format!(
            "fn nested_probe(){{let owner=Record{bytes}{{bytes:vec![4,5,6]}};let pointer=owner.bytes.as_ptr();let mut reader=function1(owner);match reader.call(){{Union{result_id}::V{tag}(value)=>{{assert_eq!(value.field0.bytes.as_ptr(),pointer);assert_eq!(value.field0.bytes,&[4,5]);{mutation}}},_=>panic!(\"nested view\")}}match reader.call(){{Union{result_id}::V{tag}(value)=>assert_eq!(value.field0.bytes[0],{expected}),_=>panic!(\"later view\")}}}}",
            result_id = result.index()
        );
        let mut generated = crate::emit::emit(&checked);
        let main = generated.find("fn main()").unwrap();
        let opening = main + generated[main..].find('{').unwrap() + 1;
        generated.insert_str(opening, "nested_probe();");
        let directory =
            std::env::temp_dir().join(format!("tsrust-nested-view-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let input = directory.join("nested.rs");
        std::fs::write(&input, format!("{generated}\n{probe}")).unwrap();
        for level in ["0", "3"] {
            let output = directory.join(format!("nested-{level}"));
            let compiled = std::process::Command::new("rustc")
                .arg("--edition=2024")
                .arg("-C")
                .arg(format!("opt-level={level}"))
                .arg(&input)
                .arg("-o")
                .arg(&output)
                .output()
                .unwrap();
            assert!(
                compiled.status.success(),
                "{}",
                String::from_utf8_lossy(&compiled.stderr)
            );
            assert!(
                std::process::Command::new(output)
                    .status()
                    .unwrap()
                    .success()
            );
        }
    }
}
#[test]
fn native_view_capturing_factory_moves_preserve_source_loans() {
    for mutable in [false, true] {
        for (body, expected) in [
            (
                "const reader=factory(owner);const value=reader();const moved=reader;",
                Some("native-view-conflict"),
            ),
            (
                "const reader=factory(owner);const moved=reader;const value=moved();",
                None,
            ),
        ] {
            let mut parsed = capturing_factory_program(
                mutable,
                &format!("function useFactory(owner:int){{{body}}}"),
            );
            parsed.functions[1].params[0].1 = parsed.functions[0].params[0].1.clone();
            match (crate::analysis::analyze(parsed), expected) {
                (Err(error), Some(code)) => assert_eq!(error.code, code, "{body}: {error:?}"),
                (Ok(_), None) => (),
                (Err(error), None) => panic!("{body}: {error:?}"),
                (Ok(_), Some(_)) => panic!("{body}: conflicting receiver move accepted"),
            }
        }
    }
}
#[test]
fn native_view_capturing_factory_rejects_source_receiver_escape() {
    for mutable in [false, true] {
        let mut parsed = capturing_factory_program(
            mutable,
            "function invalid(owner:int){const reader=factory(owner);return reader();}",
        );
        parsed.functions[1].params[0].1 = parsed.functions[0].params[0].1.clone();
        let error = crate::analysis::analyze(parsed)
            .err()
            .expect("local receiver cannot supply an escaping view");
        assert_eq!(error.code, "native-view-escape", "{error:?}");
    }
}
fn capturing_factory_program(mutable: bool, tail: &str) -> crate::syntax::Program {
    let source = format!(
        "function factory(owner:int){{return ()=>nativeView(owner,0,2);}}{tail}function main(){{}}"
    );
    let formatted = crate::formatting::format(&source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
    let mut parsed = crate::syntax::parse(&formatted).unwrap();
    let binding = if mutable {
        &super::super::bindings::VIEW_MUT
    } else {
        &super::super::bindings::VIEW
    };
    crate::native_binding_types::prepare_private_view_test(&mut parsed.records, binding).unwrap();
    parsed.functions[0].params[0].1 = crate::native_type_resolution::project(
        &mut parsed.records,
        &crate::native_signatures::Signature::Named(NativeTypeId::Bytes),
        crate::Span::default(),
    )
    .unwrap();
    let crate::syntax::StmtKind::Return(Some(value)) = &mut parsed.functions[0].body[0].kind else {
        panic!("factory return")
    };
    let crate::syntax::ExprKind::Closure { body, .. } = &mut value.kind else {
        panic!("closure")
    };
    let crate::syntax::StmtKind::Return(Some(value)) = &mut body[0].kind else {
        panic!("view return")
    };
    let crate::syntax::ExprKind::Call(callee, _) = &mut value.kind else {
        panic!("factory call")
    };
    callee.kind = crate::syntax::ExprKind::Name(format!("@native:std:bytes#{}", binding.name));
    parsed
}
#[test]
fn native_view_capturing_factory_emits_complete_owned_body() {
    for mutable in [false, true] {
        let binding = if mutable {
            &super::super::bindings::VIEW_MUT
        } else {
            &super::super::bindings::VIEW
        };
        let parsed = capturing_factory_program(mutable, "");
        let checked = crate::analysis::analyze(parsed).unwrap();
        let closure = &checked.closures[0];
        assert_eq!(closure.captures[0].storage, crate::Mode::Consume);
        assert_eq!(
            closure.invocation,
            if mutable {
                crate::Mode::Mutate
            } else {
                crate::Mode::Read
            }
        );
        assert!(closure.function.native_returns.is_some());
        let bytes = checked
            .records
            .iter()
            .position(|record| record.native_type == Some(NativeTypeId::Bytes))
            .unwrap();
        let NativeType::Structured(signature) = binding.result else {
            panic!("result")
        };
        let Type::Union(result) =
            crate::native_binding_types::resolve(&checked.records, signature).unwrap()
        else {
            panic!("union")
        };
        let members = &checked.unions[result.index()].members;
        let [ok, _] = crate::result_identity::variants(&checked.records, members).unwrap();
        let tag = members
            .iter()
            .position(|member| *member == Type::Record(ok))
            .unwrap();
        let mut generated = crate::emit::emit(&checked);
        let main = generated.find("fn main()").unwrap();
        let opening = main + generated[main..].find('{').unwrap() + 1;
        generated.insert_str(opening, "capturing_view_probe();");
        let mutation = if mutable {
            "value.field0.bytes[0]=7;"
        } else {
            ""
        };
        let expected = if mutable { 7 } else { 4 };
        let probe = format!(
            "fn capturing_view_probe(){{let owner=Record{bytes}{{bytes:vec![4,5,6]}};let pointer=owner.bytes.as_ptr();let mut reader=function0(owner);match reader.call(){{Union{result_id}::V{tag}(value)=>{{assert_eq!(value.field0.bytes,&[4,5]);assert_eq!(value.field0.bytes.as_ptr(),pointer);{mutation}}},_=>panic!(\"view\")}}match reader.call(){{Union{result_id}::V{tag}(value)=>assert_eq!(value.field0.bytes[0],{expected}),_=>panic!(\"later view\")}}}}",
            result_id = result.index()
        );
        let directory =
            std::env::temp_dir().join(format!("tsrust-capturing-view-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let input = directory.join("capture.rs");
        std::fs::write(&input, format!("{generated}\n{probe}")).unwrap();
        for level in ["0", "3"] {
            let output = directory.join(format!("capture-{level}"));
            let compile = std::process::Command::new("rustc")
                .arg("--edition=2024")
                .arg("-C")
                .arg(format!("opt-level={level}"))
                .arg(&input)
                .arg("-o")
                .arg(&output)
                .output()
                .unwrap();
            assert!(
                compile.status.success(),
                "{}",
                String::from_utf8_lossy(&compile.stderr)
            );
            assert!(
                std::process::Command::new(output)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let escaped = format!(
            "{generated}\n{probe}\nfn receiver_escape(){{let escaped;{{let mut reader=function0(Record{bytes}{{bytes:vec![4,5,6]}});escaped=reader.call();}}match escaped{{Union{}::V{tag}(value)=>assert_eq!(value.field0.bytes.len(),2),_=>()}}}}",
            result.index()
        );
        std::fs::write(&input, escaped).unwrap();
        let rejected = std::process::Command::new("rustc")
            .arg("--edition=2024")
            .arg(&input)
            .arg("-o")
            .arg(directory.join("escape"))
            .output()
            .unwrap();
        assert!(
            !rejected.status.success(),
            "view escaped its owning closure"
        );
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not live long enough"));
    }
}
#[test]
fn native_view_factory_bridges_name_the_source_lifetime() {
    let source = "function read(bytes:ReadOnly<int>,start:int,end:int):int{return nativeView(bytes,start,end);}function write(bytes:Mutable<int>,start:int,end:int):int{return nativeView(bytes,start,end);}function main(){}";
    let formatted = crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
    let mut parsed = crate::syntax::parse(&formatted).unwrap();
    for (index, binding) in [
        &super::super::bindings::VIEW,
        &super::super::bindings::VIEW_MUT,
    ]
    .into_iter()
    .enumerate()
    {
        crate::native_binding_types::prepare_private_view_test(&mut parsed.records, binding)
            .unwrap();
        let span = parsed.functions[index].name.span;
        let parameter = crate::native_type_resolution::project(
            &mut parsed.records,
            &crate::native_signatures::Signature::Named(NativeTypeId::Bytes),
            span,
        )
        .unwrap();
        let result = crate::native_type_resolution::project(
            &mut parsed.records,
            &binding.result.signature(),
            span,
        )
        .unwrap();
        let function = &mut parsed.functions[index];
        function.params[0].1 = parameter;
        function.result = Some(crate::syntax::ResultAnnotation::Named(Box::new(result)));
        let crate::syntax::StmtKind::Return(Some(value)) = &mut function.body[0].kind else {
            panic!("return")
        };
        let crate::syntax::ExprKind::Call(callee, _) = &mut value.kind else {
            panic!("call")
        };
        callee.kind = crate::syntax::ExprKind::Name(format!("@native:std:bytes#{}", binding.name));
    }
    let checked = crate::analysis::analyze(parsed).unwrap();
    for function in &checked.functions[..2] {
        let sources = function
            .native_returns
            .as_ref()
            .expect("factory body must infer source provenance");
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].root, 0);
        assert_eq!(
            sources[0].source,
            crate::analysis::native_view_loans::LifetimeSource::Storage
        );
    }
    let emitted = bridges(
        &checked,
        &[
            &super::super::bindings::VIEW,
            &super::super::bindings::VIEW_MUT,
        ],
    );
    assert!(emitted.contains("fn tsrust_std_bytes_view<'a>(bytes: &'a Record"));
    assert!(emitted.contains("fn tsrust_std_bytes_view_mut<'a>(bytes: &'a mut Record"));
    assert!(emitted.contains("views_runtime") || emitted.contains("pub fn view_mut("));
    let bytes = checked
        .records
        .iter()
        .position(|record| record.native_type == Some(NativeTypeId::Bytes))
        .unwrap();
    let variant = |binding: &NativeBinding| {
        let NativeType::Structured(signature) = binding.result else {
            panic!("Result signature")
        };
        let Type::Union(id) =
            crate::native_binding_types::resolve(&checked.records, signature).unwrap()
        else {
            panic!("Result type")
        };
        let members = &checked.unions[id.index()].members;
        let [ok, err] = crate::result_identity::variants(&checked.records, members).unwrap();
        (
            id.index(),
            members
                .iter()
                .position(|ty| *ty == Type::Record(ok))
                .unwrap(),
            members
                .iter()
                .position(|ty| *ty == Type::Record(err))
                .unwrap(),
        )
    };
    let (read, read_ok, read_err) = variant(&super::super::bindings::VIEW);
    let (write, write_ok, _) = variant(&super::super::bindings::VIEW_MUT);
    let probe = format!(
        r#"
fn native_view_bridge_probe() {{
 let mut owner=Record{bytes}{{bytes:vec![1,2,3]}};
 let pointer=owner.bytes.as_ptr();
 match function0(&owner,1,3) {{Union{read}::V{read_ok}(value)=>{{assert_eq!(value.field0.bytes,&[2,3]);assert_eq!(value.field0.bytes.as_ptr(),pointer.wrapping_add(1));}},_=>panic!("valid view")}}
 match function0(&owner,3,3) {{Union{read}::V{read_ok}(value)=>assert!(value.field0.bytes.is_empty()),_=>panic!("empty view")}}
 match function0(&owner,3,2) {{Union{read}::V{read_err}(_)=>(),_=>panic!("range error")}}
 match function1(&mut owner,1,2) {{Union{write}::V{write_ok}(value)=>{{value.field0.bytes[0]=9;}},_=>panic!("valid mutable view")}}
 assert_eq!(owner.bytes,[1,9,3]);
}}
"#
    );
    // The bridge uses actual generated records and tagged Result layouts.
    let mut generated = crate::emit::emit(&checked);
    let main = generated.find("fn main()").unwrap();
    let opening = main + generated[main..].find('{').unwrap() + 1;
    generated.insert_str(opening, "native_view_bridge_probe();");
    let directory =
        std::env::temp_dir().join(format!("tsrust-view-factory-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    let input = directory.join("factory.rs");
    std::fs::write(&input, format!("{generated}\n{probe}")).unwrap();
    for level in ["0", "3"] {
        let output = directory.join(format!("factory-{level}"));
        let compile = std::process::Command::new("rustc")
            .arg("--edition=2024")
            .arg("-C")
            .arg(format!("opt-level={level}"))
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        assert!(
            std::process::Command::new(output)
                .status()
                .unwrap()
                .success()
        );
    }
    let escaped = format!(
        "{generated}\n{probe}\nfn escape_probe() {{let escaped;{{let owner=Record{bytes}{{bytes:vec![1,2,3]}};escaped=function0(&owner,0,1);}}match escaped {{Union{read}::V{read_ok}(value)=>assert_eq!(value.field0.bytes.len(),1),_=>()}}}}\n"
    );
    std::fs::write(&input, escaped).unwrap();
    let rejected = std::process::Command::new("rustc")
        .arg("--edition=2024")
        .arg(&input)
        .arg("-o")
        .arg(directory.join("escape"))
        .output()
        .unwrap();
    assert!(
        !rejected.status.success(),
        "forwarded view escaped its source owner"
    );
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not live long enough"));
}
