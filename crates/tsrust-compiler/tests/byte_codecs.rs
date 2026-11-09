mod support;

#[test]
fn codec_project_runs_strict_vectors_views_aliases_and_owned_results() {
    let directory = support::TempDirectory::new();
    let fixture = include_str!("fixtures/byte-codecs/main.tsr");
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut examples = String::new();
    let mut expected = String::new();
    for name in ["toHex", "fromHex", "toBase64", "fromBase64", "toBase64Url", "fromBase64Url", "toUtf8Lossy", "fromUtf16", "toUtf16"] {
        let entry = catalog["std:bytes"].as_array().unwrap().iter().find(|entry| entry["name"]==name).unwrap();
        let example = &entry["examples"][0];
        let body = example["source"].as_str().unwrap().split_once("function main() {").unwrap().1.trim().strip_suffix('}').unwrap();
        examples.push_str(&format!("{{{body}}}\n"));
        expected.push_str(example["output"].as_str().unwrap());
    }
    let end = fixture.rfind('}').unwrap();
    let source = format!("import {{toBase64}} from 'std:bytes';\n{}{examples}}}", &fixture[..end]);
    let path = directory.source(&support::format_round_trip(&source));
    for release in [false, true] {
        let binary = directory.0.join(format!("codecs{}", std::env::consts::EXE_SUFFIX));
        let mut build = support::cli(); build.arg("build").arg(&path).arg("-o").arg(&binary);
        if release { build.arg("--release"); }
        support::success(&support::run(&mut build, 180));
        let output = support::run(&mut std::process::Command::new(&binary), 30);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
    }
}

#[test]
fn codec_imports_keep_arity_nominal_inputs_and_view_loans() {
    for source in [
        "import {toHex} from 'std:bytes';function main(){toHex([0,1]);}",
        "import {fromHex} from 'std:bytes';function main(){fromHex();}",
        "import {fromUtf16} from 'std:bytes';function main(){fromUtf16('x');}",
        "import {fromUtf16} from 'std:bytes';function main(){fromUtf16('x',1);}",
        "import {fromUtf16} from 'std:bytes';function main(){fromUtf16('x','native');}",
        "import {fromArray,view,toHex,resize,length} from 'std:bytes';function main(){const b=unwrap(fromArray([1]));const v=unwrap(view(b,0,1));toHex(v);resize(b,2);length(v);}",
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "accepted invalid codec contract: {source}");
    }
}
