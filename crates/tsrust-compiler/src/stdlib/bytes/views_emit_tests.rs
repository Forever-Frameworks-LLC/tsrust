use super::*;
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
