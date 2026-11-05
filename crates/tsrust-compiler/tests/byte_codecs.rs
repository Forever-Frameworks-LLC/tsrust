mod support;

#[test]
fn codec_project_runs_strict_vectors_views_aliases_and_owned_results() {
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(include_str!("fixtures/byte-codecs/main.tsr")));
    support::success(&support::run(support::cli().arg("run").arg(path), 180));
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
