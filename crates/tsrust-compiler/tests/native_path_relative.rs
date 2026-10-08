mod support;
use tsrust_compiler::native_path_relative::relative_with;

#[test]
fn relative_resolves_context_and_borrows_inputs() {
    for (from, to, expected) in [
        ("/a/b", "/a/c", "../c"),
        ("/a/b", "/a/b", ""),
        ("", "child", "child"),
        ("child", "", ".."),
        ("../a/./b/", "../c//", "../../c"),
        ("/foo", "/foobar", "../foobar"),
        ("/", "/é😊/文", "é😊/文"),
        ("/a/b", "/", "../.."),
    ] {
        assert_eq!(
            relative_with(from, to, false, || Ok("/root/work".into()), |_| None).unwrap(),
            expected,
            "{from:?} {to:?}"
        );
    }
    let from = String::from("/a/é😊");
    let to = String::from("/a/文");
    let mut result = relative_with(
        &from,
        &to,
        false,
        || panic!("absolute inputs do not need cwd"),
        |_| None,
    )
    .unwrap();
    result.push('!');
    assert_eq!(result, "../文!");
    assert_eq!(from, "/a/é😊");
    assert_eq!(to, "/a/文");
    assert_eq!(
        relative_with("", "", false, || Err("missing cwd".into()), |_| None).unwrap(),
        ""
    );
    assert_eq!(
        relative_with("a", "b", false, || Err("missing cwd".into()), |_| None).unwrap_err(),
        "missing cwd"
    );
}

#[test]
fn relative_windows_drives_unc_and_case_are_explicit() {
    for (from, to, expected) in [
        (r"C:\a\b", r"C:\a\c", r"..\c"),
        (r"C:\a", r"D:\b", r"D:\b"),
        (r"C:\Foo", r"c:\foo", ""),
        (r"C:child", r"C:other", r"..\other"),
        (r"D:child", r"D:other", r"..\other"),
        (r"\\server\share\a", r"\\server\share\b", r"..\b"),
        (r"\\server\share\a", r"\\other\share\b", r"\\other\share\b"),
        (r"\\server\share\a", r"\\server\other\b", r"..\..\other\b"),
    ] {
        assert_eq!(
            relative_with(
                from,
                to,
                true,
                || Ok(r"C:\work".into()),
                |drive| if drive.eq_ignore_ascii_case("D:") {
                    Some(r"D:\saved".into())
                } else {
                    None
                }
            )
            .unwrap(),
            expected,
            "{from:?} {to:?}"
        );
    }
}

#[test]
fn imported_relative_preserves_aliases_stored_calls_and_checked_cwd_errors() {
    let temp = support::TempDirectory::new();
    let source = "import {relative as between} from 'std:path';import {strictEqual} from 'std:testing';function main(){const from='/a/b';const to='/a/c';const stored=between;strictEqual(between(from,to),'../c');strictEqual(stored(from,to),'../c');strictEqual(from,'/a/b');strictEqual(to,'/a/c');let caught=false;try{stored('a','b');}catch(error){caught=error.code==='path-relative';}strictEqual(caught,true);console.log('relative accepted');}";
    let formatted = tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let file = temp.source(&formatted);
    let output = support::run(support::cli().arg("emit").arg(file), 30);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let emitted = String::from_utf8(output.stdout).unwrap();
    assert!(emitted.contains("std::env::current_dir()"));
    #[cfg(unix)]
    {
        #[cfg(target_os = "linux")]
        use std::os::unix::ffi::OsStringExt;
        let rust = temp.0.join("relative-host.rs");
        let binary = temp.0.join("relative-host-native");
        std::fs::write(&rust, &emitted).unwrap();
        let output = support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg(&rust)
                .arg("-o")
                .arg(&binary),
            30,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let deleted = temp.0.join("deleted-cwd");
        std::fs::create_dir(&deleted).unwrap();
        let output = support::run(
            std::process::Command::new("sh")
                .arg("-c")
                .arg("rmdir \"$PWD\"; exec \"$1\"")
                .arg("relative-cwd")
                .arg(&binary)
                .current_dir(&deleted),
            10,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"relative accepted\n");
        #[cfg(target_os = "linux")]
        {
            let invalid = temp.0.join(std::ffi::OsString::from_vec(vec![b'c', 0xff]));
            std::fs::create_dir(&invalid).unwrap();
            let output = support::run(
                std::process::Command::new(&binary).current_dir(&invalid),
                10,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"relative accepted\n");
        }
    }
    #[cfg(unix)]
    {
        let invalid=emitted.replace("std::env::current_dir()","Ok::<std::path::PathBuf,std::io::Error>(std::path::PathBuf::from(<std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(vec![b'/',0xff])))");
        let rust = temp.0.join("relative-invalid.rs");
        let binary = temp.0.join("relative-invalid-native");
        std::fs::write(&rust, invalid).unwrap();
        let output = support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg(&rust)
                .arg("-o")
                .arg(&binary),
            30,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = support::run(&mut std::process::Command::new(binary), 10);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"relative accepted\n");
    }
    let emitted=emitted.replace("std::env::current_dir()","Err::<std::path::PathBuf, _>(std::io::Error::new(std::io::ErrorKind::NotFound, \"cwd unavailable\"))");
    let rust = temp.0.join("relative.rs");
    let binary = temp.0.join("relative-native");
    std::fs::write(&rust, emitted).unwrap();
    let output = support::run(
        support::rustc()
            .arg("--edition=2024")
            .arg(&rust)
            .arg("-o")
            .arg(&binary),
        30,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = support::run(&mut std::process::Command::new(binary), 10);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "relative accepted\n"
    );
}

#[test]
fn relative_public_arity_types_and_owned_result_are_checked() {
    for (arguments, code) in [
        ("", "arity"),
        ("'a'", "arity"),
        ("'a','b','c'", "arity"),
        ("1,'b'", "type"),
        ("'a',false", "type"),
    ] {
        let temp = support::TempDirectory::new();
        let source = format!(
            "import {{relative}} from 'std:path';function main(){{relative({arguments});}}"
        );
        let file = temp.source(&source);
        let output = support::run(
            support::cli().arg("check").arg(file).arg("--color=never"),
            30,
        );
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(code), "{error}");
    }
}

#[test]
fn relative_matches_pinned_reference_with_controlled_directory_and_drive_contexts() {
    let mut paths = Vec::new();
    for prefix in [
        "",
        "/",
        "//",
        "///",
        "C:",
        "C:\\",
        "D:",
        "D:\\",
        r"\\host\share\",
        r"\root\",
        r"\\host\other\",
        r"\\other\share\",
    ] {
        for name in [
            "", ".", "..", "child", "foo", "foobar", "é😊", "É😊", "İ", "i̇", "ΣΣ", "σσ",
        ] {
            paths.push(format!("{prefix}{name}"));
            paths.push(format!("{prefix}{name}/a/../b/"));
        }
    }
    let contexts = [
        (false, "/workspace/project"),
        (true, r"C:\work"),
        (true, r"\\host\share\work"),
    ];
    let json = format!(
        "[{}]",
        paths
            .iter()
            .map(|path| tsrust_compiler::json_string(path))
            .collect::<Vec<_>>()
            .join(",")
    );
    let context_json = format!(
        "[{}]",
        contexts
            .iter()
            .map(|(windows, cwd)| format!(
                "{{\"windows\":{windows},\"cwd\":{}}}",
                tsrust_compiler::json_string(cwd)
            ))
            .collect::<Vec<_>>()
            .join(",")
    );
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(include_str!("fixtures/path-relative/reference.ts"))
        .arg(json)
        .arg(context_json)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = String::from_utf8(output.stdout).unwrap();
    let mut rows = expected.lines();
    for (windows, cwd) in contexts {
        for from in &paths {
            for to in &paths {
                let actual = relative_with(
                    from,
                    to,
                    windows,
                    || Ok(cwd.into()),
                    |drive| {
                        if drive.eq_ignore_ascii_case("D:") {
                            Some(r"D:\saved".into())
                        } else {
                            None
                        }
                    },
                )
                .unwrap();
                let hex = actual
                    .bytes()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                assert_eq!(
                    Some(hex.as_str()),
                    rows.next(),
                    "windows={windows} cwd={cwd:?} from={from:?} to={to:?}"
                );
            }
        }
    }
    assert!(rows.next().is_none());
}
