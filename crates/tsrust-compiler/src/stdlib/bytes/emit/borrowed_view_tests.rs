use super::*;

fn program(mutable: bool) -> crate::syntax::Program {
    let access = if mutable { "Mutable" } else { "ReadOnly" };
    let source = format!(
        "function observe(owner:{access}<int>){{const reader=()=>nativeView(owner,0,2);return reader();}}function main(){{}}"
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
    let crate::syntax::StmtKind::Let { value, .. } = &mut parsed.functions[0].body[0].kind else {
        panic!("reader")
    };
    let crate::syntax::ExprKind::Closure { body, .. } = &mut value.kind else {
        panic!("closure")
    };
    let crate::syntax::StmtKind::Return(Some(value)) = &mut body[0].kind else {
        panic!("return")
    };
    let crate::syntax::ExprKind::Call(callee, _) = &mut value.kind else {
        panic!("call")
    };
    callee.kind = crate::syntax::ExprKind::Name(format!("@native:std:bytes#{}", binding.name));
    parsed
}

#[test]
fn native_view_shared_capture_body_returns_external_owner_lifetime() {
    let checked = crate::analysis::analyze(program(false)).unwrap();
    assert_eq!(checked.closures[0].captures[0].storage, crate::Mode::Read);
    let bytes = checked
        .records
        .iter()
        .position(|record| record.native_type == Some(NativeTypeId::Bytes))
        .unwrap();
    let NativeType::Structured(signature) = super::super::bindings::VIEW.result else {
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
    let probe = format!(
        "fn borrowed_view_probe(){{let owner=Record{bytes}{{bytes:vec![4,5,6]}};let pointer=owner.bytes.as_ptr();let returned=function0(&owner);match returned{{Union{}::V{tag}(value)=>{{assert_eq!(value.field0.bytes,&[4,5]);assert_eq!(value.field0.bytes.as_ptr(),pointer);}},_=>panic!(\"view\")}}}}",
        result.index()
    );
    let negative = format!(
        "fn invalid_escape(){{let escaped;{{let owner=Record{bytes}{{bytes:vec![4,5,6]}};escaped=function0(&owner);}}match escaped{{Union{}::V{tag}(value)=>assert_eq!(value.field0.bytes.len(),2),_=>()}}}}",
        result.index()
    );
    execute(&checked, &probe, &negative, "shared");
}

#[test]
fn native_view_mutable_capture_return_keeps_receiver_alive() {
    let error = crate::analysis::analyze(program(true))
        .err()
        .expect("local mutable receiver cannot supply an escaping view");
    assert_eq!(error.code, "native-view-escape", "{error:?}");
}

#[test]
fn native_view_mutable_capture_body_writes_external_storage() {
    let mut parsed = program(true);
    let crate::syntax::StmtKind::Return(Some(value)) = &parsed.functions[0].body[1].kind else {
        panic!("return")
    };
    parsed.functions[0].body[1].kind = crate::syntax::StmtKind::Expr(value.clone());
    let checked = crate::analysis::analyze(parsed).unwrap();
    let capture = checked.closures[0].captures[0].inner;
    assert_eq!(checked.closures[0].captures[0].storage, crate::Mode::Mutate);
    assert_eq!(checked.closures[0].invocation, crate::Mode::Mutate);
    let bytes = checked
        .records
        .iter()
        .position(|record| record.native_type == Some(NativeTypeId::Bytes))
        .unwrap();
    let NativeType::Structured(signature) = super::super::bindings::VIEW_MUT.result else {
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
    let probe = format!(
        "fn borrowed_view_probe(){{let mut owner=Record{bytes}{{bytes:vec![4,5,6]}};function0(&mut owner);let pointer=owner.bytes.as_ptr();{{let mut reader=Closure0{{capture{capture}:&mut owner}};match reader.call(){{Union{}::V{tag}(value)=>{{assert_eq!(value.field0.bytes.as_ptr(),pointer);value.field0.bytes[0]=7;}},_=>panic!(\"view\")}}}}assert_eq!(owner.bytes,[7,5,6]);}}",
        result.index()
    );
    let negative = format!(
        "fn invalid_escape(){{let mut owner=Record{bytes}{{bytes:vec![4,5,6]}};let escaped;{{let mut reader=Closure0{{capture{capture}:&mut owner}};escaped=reader.call();}}match escaped{{Union{}::V{tag}(value)=>assert_eq!(value.field0.bytes.len(),2),_=>()}}}}",
        result.index()
    );
    execute(&checked, &probe, &negative, "mutable");
}

fn execute(checked: &CheckedProgram, probe: &str, negative: &str, name: &str) {
    let mut generated = crate::emit::emit(checked);
    let main = generated.find("fn main()").unwrap();
    let opening = main + generated[main..].find('{').unwrap() + 1;
    generated.insert_str(opening, "borrowed_view_probe();");
    let directory = std::env::temp_dir().join(format!(
        "tsrust-borrowed-view-body-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    let input = directory.join("borrow.rs");
    std::fs::write(&input, format!("{generated}\n{probe}")).unwrap();
    for level in ["0", "3"] {
        let output = directory.join(format!("borrow-{level}"));
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
    std::fs::write(&input, format!("{generated}\n{probe}\n{negative}")).unwrap();
    let rejected = std::process::Command::new("rustc")
        .arg("--edition=2024")
        .arg(&input)
        .arg("-o")
        .arg(directory.join("rejected"))
        .output()
        .unwrap();
    assert!(
        !rejected.status.success(),
        "borrowed view escaped its required lifetime"
    );
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not live long enough"));
}
