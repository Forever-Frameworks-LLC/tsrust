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

#[test]
fn lexical_directory_and_extension_boundaries() {
    use tsrust_compiler::native_path::{dirname, extname};
    for (path, directory, extension) in [
        ("", ".", ""), ("/", "/", ""), ("//a", "//", ""),
        ("/tmp/report.txt/", "/tmp", ".txt"), (".hidden", ".", ""),
        ("..", ".", ""), ("...", ".", "."), ("a.", ".", "."),
        ("/é😊/a.tar.gz", "/é😊", ".gz"), ("a//b", "a/", ""),
    ] {
        assert_eq!(dirname(path, false), directory, "{path:?}");
        assert_eq!(extname(path, false), extension, "{path:?}");
    }
    for (path, directory, extension) in [
        ("C:", "C:", ""), ("C:report.txt", "C:", ".txt"),
        ("C:\\report.txt", "C:\\", ".txt"),
        ("\\\\server\\share", "\\\\server\\share", ""),
        ("\\\\server\\share\\file.txt", "\\\\server\\share\\", ".txt"),
    ] {
        assert_eq!(dirname(path, true), directory, "{path:?}");
        assert_eq!(extname(path, true), extension, "{path:?}");
    }
}

#[test]
fn directory_and_extension_match_independent_node_reference() {
    use tsrust_compiler::native_path::{dirname, extname};
    let mut paths = vec!["".to_owned(), "/".into(), "//".into(), "C:".into(), "\\\\host\\share".into()];
    for prefix in ["", "/", "//", "a//", "C:", "C:\\", "\\\\host\\share\\", "é😊/"] {
        for component in ["a", ".", "..", "...", ".hidden", ".hidden.txt", "a.tar.gz", "a.", "é😊.txt"] {
            for trailing in ["", "/", "//", "\\"] { paths.push(format!("{prefix}{component}{trailing}")); }
        }
    }
    let json = paths.iter().map(|p| tsrust_compiler::json_string(p)).collect::<Vec<_>>().join(",");
    let reference=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lexical-path/reference.ts");
    let output = std::process::Command::new("node").arg(reference).args(["parent-extension", &format!("[{json}]")]).output().unwrap();
    assert!(output.status.success());
    let expected = String::from_utf8(output.stdout).unwrap();
    let mut lines = expected.lines();
    for windows in [false,true] { for path in &paths {
        for actual in [dirname(path,windows), extname(path,windows)] {
            let hex=actual.bytes().map(|b|format!("{b:02x}")).collect::<String>();
            assert_eq!(Some(hex.as_str()),lines.next(),"windows={windows} path={path:?}");
        }
    }}
    assert!(lines.next().is_none());
}

#[test]
fn absolute_and_normalize_boundaries_and_idempotence() {
    use tsrust_compiler::native_path::{is_absolute,normalize};
    for (path,absolute,expected) in [
        ("",false,"."),("/",true,"/"),("///",true,"/"),
        ("a//b/../c/",false,"a/c/"),("/../../a",true,"/a"),
        ("../../a/../",false,"../../"),("./",false,"./"),
        ("é😊/../文/",false,"文/"),
    ] { assert_eq!(is_absolute(path,false),absolute);assert_eq!(normalize(path,false),expected);assert_eq!(normalize(expected,false),expected); }
    for (path,absolute,expected) in [
        ("C:a\\..\\b",false,"C:b"),("C:",false,"C:."),
        ("C:\\..\\a\\",true,"C:\\a\\"),
        ("\\\\host\\share\\..\\a",true,"\\\\host\\share\\a"),
        ("\\\\host\\share",true,"\\\\host\\share\\"),
        ("a\\..\\C:foo",false,".\\C:foo"),
    ] { assert_eq!(is_absolute(path,true),absolute);assert_eq!(normalize(path,true),expected);assert_eq!(normalize(expected,true),expected); }
}

#[test]
fn normalize_and_absolute_match_both_node_conventions() {
    use tsrust_compiler::native_path::{normalize,is_absolute};
    let mut paths=vec!["".to_owned(),"/".into(),"\\".into(),"C:".into(),"\\\\host\\share".into(),"CON:".into(),"COM1:foo".into(),"\\\\?\\COM1:".into(),"\\\\.\\PHYSICALDRIVE0".into()];
    for prefix in ["", "/", "//", "///", "C:", "C:/", "\\\\host\\share\\", "\\\\?\\C:\\", "é😊/"] {
        for middle in ["", "a/", "./", "../", "a/../../", "a//b/../", "a/../"] {
            for component in ["a", ".", "..", "...", "é😊.txt", "C:foo", "a:", "a:/", "CON:foo"] {
                for trailing in ["", "/", "\\"] {paths.push(format!("{prefix}{middle}{component}{trailing}"));}
            }
        }
    }
    let json=paths.iter().map(|p|tsrust_compiler::json_string(p)).collect::<Vec<_>>().join(",");
    let reference=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lexical-path/reference.ts");
    let output=std::process::Command::new("node").arg(reference).args(["absolute-normalize",&format!("[{json}]")]).output().unwrap();assert!(output.status.success());
    let text=String::from_utf8(output.stdout).unwrap();let mut lines=text.lines();
    for windows in [false,true] {for path in &paths {
        let normalized=normalize(path,windows);
        let second=normalize(&normalized,windows);
        let actual=format!("[{},{},{}]",tsrust_compiler::json_string(&normalized),is_absolute(path,windows),tsrust_compiler::json_string(&second));
        assert_eq!(Some(actual.as_str()),lines.next(),"windows={windows} path={path:?}");
        // Pin idempotence where the reference is stable; reserved device safeguards can change on a second pass.
        if !windows || (!path.contains(':') && !normalized.starts_with(r"\\.\") && !normalized.starts_with(r"\\?\")) {assert_eq!(second,normalized,"{path:?}");}
    }}
    assert!(lines.next().is_none());
}

#[test]
fn join_matches_independent_posix_and_windows_references() {
    use std::io::Write;
    use std::process::{Command,Stdio};
    let mut cases:Vec<Vec<String>>=vec![vec![],vec!["".into()],vec!["".into(),"".into()]];
    for first in ["","/","//","///","//server","\\\\server","C:","C:\\","\\\\?\\C:\\","\\\\.\\pipe","é😊","CON:","LPT²:"] {
        for middle in ["","a","..",".","/root","\\tail","share","AUX:","x/PRN:"] {
            for last in ["","/","\\","..","文.txt","x//y/"] {
                cases.push(vec![first.into(),middle.into(),last.into()]);
            }
        }
    }
    cases.push((0..1024).map(|i|["a","..","é😊",""][i%4].into()).collect());
    cases.push(vec!["é😊".repeat(8192),"..".into(),"report".into()]);
    let mut node=Command::new("node")
        .args(["-e","const p=require('node:path');const cases=JSON.parse(require('node:fs').readFileSync(0,'utf8'));console.log(cases.flatMap(parts=>[p.posix.join(...parts),p.win32.join(...parts)]).map(value=>Buffer.from(value).toString('hex')).join('\\n'));"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    node.stdin.take().unwrap().write_all(format!("{cases:?}").as_bytes()).unwrap();
    let output=node.wait_with_output().unwrap();assert!(output.status.success());
    let text=String::from_utf8(output.stdout).unwrap();
    let expected=text.lines().collect::<Vec<_>>();
    assert_eq!(expected.len(),cases.len()*2);
    for (paths,reference) in cases.iter().zip(expected.chunks_exact(2)) {
        let borrowed=paths.iter().map(String::as_str).collect::<Vec<_>>();
        for (platform,expected) in reference.iter().enumerate() {
            let actual=tsrust_compiler::native_path::join(&borrowed,platform==1).bytes().map(|byte|format!("{byte:02x}")).collect::<String>();
            assert_eq!(actual,*expected,"platform={platform} paths={paths:?}");
        }
    }
}
