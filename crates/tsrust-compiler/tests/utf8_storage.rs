mod support;

#[test]
fn source_slices_own_independent_utf8_buffers_and_survive_parent_disposal() {
    let source = r#"
        function slice(text: string, start: int, end: int): string {
            return text.slice(start, end);
        }
        function main() {}
    "#;
    let emitted = tsrust_compiler::check(source).unwrap().emit_rust();
    let rust = emitted.replace("fn main()", "fn source_main()")
        + r#"
fn source_slice(value: &String, start: usize, end: usize) -> String {
    function0(value, start as i64, end as i64)
}
fn main() {
    fn transferable<T: Send + Sync>() {} transferable::<String>();
    assert_eq!(std::mem::size_of::<String>(), std::mem::size_of::<std::borrow::Cow<'static, str>>());
    let mut buffer = std::string::String::with_capacity(16384); buffer.push_str(&"A".repeat(2048));
    let mut original = String::Owned(buffer);
    let first = source_slice(&original, 1, 2000);
    let nested = source_slice(&first, 1, 1998);
    let tiny = source_slice(&original, 10, 30);
    assert_eq!(nested.as_ref(), "A".repeat(1997)); assert_eq!(tiny.as_ref(), "A".repeat(20));
    assert!(matches!(first, String::Owned(_))); assert!(matches!(tiny, String::Owned(_)));
    assert_ne!(original.as_ptr(), first.as_ptr()); assert_ne!(first.as_ptr(), nested.as_ptr());
    if let String::Owned(text) = &tiny { assert!(text.capacity() < 16384); }
    original.to_mut().replace_range(.., "changed"); drop(original); drop(first);
    assert_eq!(nested.as_ref(), "A".repeat(1997)); assert_eq!(tiny.as_ref(), "A".repeat(20));
    let text = "a😀é中\0".repeat(200);
    let original = String::Owned(text);
    for (start, end) in [(1, 606), (5, 610), (7, 612), (0, 1100)] {
        let result = source_slice(&original, start, end);
        assert_eq!(result.as_ref(), &original[start..end]);
    }
    let aligned = source_slice(&original, 1, 606);
    let nested = source_slice(&aligned, 4, 604);
    assert_eq!(nested.as_ref(), &original[5..605]);
    let mut cloned = nested.clone(); cloned.to_mut().push('!');
    assert_eq!(nested.as_ref(), &original[5..605]);
    drop(original); drop(aligned);
    let expected = nested.to_string(); let threaded = nested.clone();
    assert_eq!(std::thread::spawn(move || threaded.into_owned()).join().unwrap(), expected);
    let result = std::panic::catch_unwind(move || { assert_eq!(nested.as_ref(), expected); panic!("release owner"); });
    assert!(result.is_err()); assert!(cloned.ends_with('!'));
    let literal = String::Borrowed("😀é"); let copied = literal.clone();
    assert!(matches!(copied, String::Borrowed(_))); assert_eq!(source_slice(&copied, 0, 4).as_ref(), "😀");
}

"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("native-owners.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("native-owners-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        support::success(&support::run(
            &mut std::process::Command::new(&executable),
            10,
        ));
    }
}
