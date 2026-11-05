mod support;

#[test]
fn url_query_and_component_package_builds_safely_offline_after_relocation_at_o0_and_o3() {
    let directory=support::TempDirectory::new();
    let source=r#"
import {parseUrl,resolveUrl,formatUrl,searchParams,setSearchParams,formatQuery,encodeURIComponent,decodeURIComponent} from 'std:url';
function main(){
  const address=unwrap(parseUrl('https://EXAMPLE.com/a?tag=one&tag=two'));
  const params=searchParams(address);params.append('q','é +');params.sort();
  setSearchParams(address,params);
  console.log(formatUrl(address));console.log(formatQuery(params));
  const next=unwrap(resolveUrl(address,'../next'));console.log(formatUrl(next));
  console.log(encodeURIComponent('é +'),unwrap(decodeURIComponent('%C3%A9%20%2B')));
  try{address.hostname='';}catch(error){console.log(error.code);}
  console.log(address.hostname);
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
    let package=directory.0.join("package");
    support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));
    std::fs::remove_file(path).unwrap();
    let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
    let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();
    let document=manifest.parse::<toml_edit::DocumentMut>().unwrap();
    assert!(document["features"]["default"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("url")));
    assert_eq!(document["patch"]["crates-io"]["url"]["path"].as_str(),Some("patched-dependencies/url"));
    assert!(relocated.join("patched-dependencies/url/LICENSE-MIT").is_file());
    assert!(relocated.join("patched-dependencies/url/TSRUST-PATCH.md").is_file());
    assert!(relocated.join("vendor/percent-encoding-2.3.2/LICENSE-MIT").is_file());
    tsrust_compiler::native_url_patch::visit(&relocated,&manifest,|_,_|Ok(())).unwrap();
    let source=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&source).unwrap();
    assert!(emitted.contains("mod tsrust_native_url"));assert!(!emitted.contains("unsafe"));
    std::fs::write(source,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
    let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    assert!(target.is_absolute());
    let home=directory.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();
    for release in [false,true] {
        let mut build=std::process::Command::new("rustup");
        build.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated)
            .env("CARGO_HOME",&home).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");
        if release {build.arg("--release");}
        support::success(&support::run(&mut build,180));
        let executable=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));
        let output=support::run(std::process::Command::new(executable).current_dir(&relocated),30);support::success(&output);
        assert_eq!(output.stdout,"https://example.com/a?q=%C3%A9+%2B&tag=one&tag=two\nq=%C3%A9+%2B&tag=one&tag=two\nhttps://example.com/next\n%C3%A9%20%2B é +\nparse\nexample.com\n".as_bytes(),"release={release}");
    }
}
