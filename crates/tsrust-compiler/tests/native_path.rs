use tsrust_compiler::native_path::basename;
#[test]
fn basename_boundaries_are_lexical_and_borrow_inputs() {
    for (path, suffix, expected) in [
        ("/tmp/report.txt", None, "report.txt"),
        ("/tmp/report.txt/", Some(".txt"), "report"),
        ("", None, ""),
        ("///", None, ""),
        ("///", Some("x"), "///"),
        ("/a", Some("a"), "a"),
        ("a", Some("a"), ""),
        ("a/", Some("a"), "a"),
        ("/foo/a/", Some("ba"), "a/"),
        ("/foo/..", None, ".."),
        ("/foo/.", None, "."),
        ("/tmp/é😊.txt", Some(".txt"), "é😊"),
        ("/tmp/é😊", Some("😊"), "é"),
        ("report.TXT", Some(".txt"), "report.TXT"),
    ] {
        assert_eq!(
            basename(path, suffix, false),
            expected,
            "{path:?} {suffix:?}"
        );
    }
    assert_eq!(basename("C:\\tmp\\report.txt", None, true), "report.txt");
    assert_eq!(basename("C:report.txt", Some(".txt"), true), "report");
    assert_eq!(basename("C:\\", None, true), "");
    let input = String::from("/tmp/report.txt");
    let mut result = basename(&input, None, false);
    result.push('!');
    assert_eq!(input, "/tmp/report.txt");
}

#[test]
fn basename_matches_reference_for_utf8_paths_and_both_separator_conventions() {
    use std::process::Command;
    let mut paths = vec![
        String::new(),
        "/".into(),
        "///".into(),
        "C:".into(),
        "C:\\".into(),
        "\\\\server\\share\\".into(),
    ];
    for prefix in ["", "/tmp/", "C:\\tmp\\", "C:", "//host/"] {
        for component in ["a", "a.txt", "a.TXT", ".", "..", "é😊", ".hidden", "a:b"] {
            for trailing in ["", "/", "\\", "//"] {
                paths.push(format!("{prefix}{component}{trailing}"));
            }
        }
    }
    let suffixes = [
        None,
        Some(""),
        Some("a"),
        Some(".txt"),
        Some(".TXT"),
        Some("ba"),
        Some("a/"),
        Some("."),
        Some(".."),
        Some("😊"),
        Some("é😊"),
        Some("/"),
        Some("longer-than-path"),
    ];
    let paths_json = paths
        .iter()
        .map(|path| tsrust_compiler::json_string(path))
        .collect::<Vec<_>>()
        .join(",");
    let suffix_json = suffixes
        .iter()
        .map(|suffix| {
            suffix
                .map(tsrust_compiler::json_string)
                .unwrap_or_else(|| "null".into())
        })
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "const p=require('node:path');for(const windows of [false,true])for(const path of [{paths_json}])for(const suffix of [{suffix_json}])console.log(Buffer.from((windows?p.win32:p.posix).basename(path,suffix===null?undefined:suffix)).toString('hex'));"
    );
    let reference = Command::new("node")
        .arg("-e")
        .arg(script)
        .output()
        .expect("Node is required for the focused path reference check");
    assert!(
        reference.status.success(),
        "{}",
        String::from_utf8_lossy(&reference.stderr)
    );
    let expected = String::from_utf8(reference.stdout).unwrap();
    let mut rows = expected.lines();
    for windows in [false, true] {
        for path in &paths {
            for suffix in suffixes {
                let actual = basename(path, suffix, windows);
                let hex = actual
                    .bytes()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                assert_eq!(
                    Some(hex.as_str()),
                    rows.next(),
                    "windows={windows} path={path:?} suffix={suffix:?}"
                );
            }
        }
    }
    assert!(rows.next().is_none());
}
