mod support;
use std::fs;

#[test]
fn new_application_generates_checked_editable_backend_contracts() {
    let directory = support::TempDirectory::new();
    let created = support::run(support::cli().current_dir(&directory.0).args(["new", "hello-app", "--app"]), 40);
    support::success(&created);
    let root = directory.0.join("hello-app");
    for path in ["app/main.tsr", "app/lib/greeting.tsr", "app/tests/greet.test.tsr", "AGENTS.md"] {
        assert!(root.join(path).is_file(), "Missing {path}");
    }
    for path in ["generated", "target", "js-client", "web", "native", "mobile", "node_modules", "package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml"] { assert!(!root.join(path).exists(), "Unexpected frontend output {path}"); }
    assert!(!root.join("src").exists(), "No legacy renderer scaffold");
    assert!(!root.join("packages").exists());
    assert!(!root.join("tests").exists());
    assert!(!root.join("tsrust.api.json").exists());
    assert!(!root.join("tsrust.integration.json").exists());
    assert!(!root.join("native").exists());
    assert!(!root.join("mobile").exists());
    assert!(!root.join("web/node_modules").exists(), "No implicit package installation");
    let main=fs::read_to_string(root.join("app/main.tsr")).unwrap();
    assert!(main.contains("app.use(")&&main.contains("c.json("));assert!(!main.contains("api:server"));
    let config:serde_json::Value=serde_json::from_slice(&fs::read(root.join("tsrust.config.json")).unwrap()).unwrap();assert!(config.get("api").is_none());
    let original_main=fs::read(root.join("app/main.tsr")).unwrap();
    for path in ["app/main.tsr", "app/lib/greeting.tsr", "app/tests/greet.test.tsr"] {
        let source = fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(source, support::format_round_trip(&source));
    }
    support::success(&support::run(support::cli().current_dir(&root).arg("check"), 40));
    support::success(&support::run(support::cli().current_dir(&root).args(["api", "generate", "--out", "js-client"]), 40));
    assert_eq!(fs::read(root.join("app/main.tsr")).unwrap(),original_main);
    support::success(&support::run(support::cli().current_dir(&root).args(["api","generate","--language","rust","--out","rust-client"]),40));
    support::success(&support::run(support::cli().current_dir(&root).args(["api","generate","app/main.tsr","--out","js-client","--check"]),40));
    assert!(!root.join("generated").exists());
    assert!(!root.join("target").exists(),"Client generation must not write a target source");
    // An external consumer has only the saved description, no backend sources.
    let external = directory.0.join("external");
    fs::create_dir(&external).unwrap();
    fs::copy(root.join("js-client/http-api.json"), external.join("api.json")).unwrap();
    support::success(&support::run(support::cli().current_dir(&external).args(["api", "generate", "--description", "api.json", "--out", "sdk"]), 40));
    support::success(&support::run(support::cli().current_dir(&external).args(["api", "generate", "--description", "api.json", "--out", "sdk", "--check"]), 40));
    assert_eq!(fs::read(root.join("js-client/index.js")).unwrap(), fs::read(external.join("sdk/index.js")).unwrap());
    support::success(&support::run(support::cli().current_dir(&external).args(["api", "generate", "--description", "api.json", "--language", "rust", "--out", "rust-sdk"]), 40));
    assert!(external.join("rust-sdk/src/lib.rs").is_file());
    assert!(!external.join("rust-sdk/package.json").exists());
    support::success(&support::run(support::cli().current_dir(&external).args(["api", "generate", "--description", "api.json", "--language", "rust", "--out", "rust-sdk", "--check"]), 40));
    let rust_source = external.join("rust-sdk/src/lib.rs");
    let customized_rust = format!("{}\n// Owner customization\n", fs::read_to_string(&rust_source).unwrap());
    fs::write(&rust_source, &customized_rust).unwrap();
    assert!(!support::run(support::cli().current_dir(&external).args(["api", "generate", "--description", "api.json", "--language", "rust", "--out", "rust-sdk"]), 40).status.success());
    assert_eq!(fs::read_to_string(&rust_source).unwrap(), customized_rust);
    for args in [vec!["api","generate","--language","python"],vec!["api","generate","--header-env","Authorization=TOKEN"],vec!["api","generate","--url","http://example.com/api","--description","api.json","--out","bad"]] {
        assert!(!support::run(support::cli().current_dir(&external).args(args), 40).status.success());
        assert!(!external.join("bad").exists());
    }
    let bootstrap = root.join("app/lib/greeting.tsr");
    let customized = format!("{}\n// Owner customization\n", fs::read_to_string(&bootstrap).unwrap());
    fs::write(&bootstrap, &customized).unwrap();
    support::success(&support::run(support::cli().current_dir(&root).args(["api", "generate", "--out", "js-client"]), 40));
    assert_eq!(fs::read_to_string(&bootstrap).unwrap(), customized);
    let denied = support::run(support::cli().current_dir(&directory.0).args(["new", "hello-app", "--app"]), 20);
    assert!(!denied.status.success());
    assert_eq!(fs::read_to_string(&bootstrap).unwrap(), customized);
}

#[test]
fn application_choices_reject_before_writes_and_backend_has_no_frontend() {
    let directory = support::TempDirectory::new();
    for args in [vec!["new", "bad", "--app", "--ui"], vec!["new", "bad", "--app", "--app"], vec!["new", "bad", "--app", "--target", "desktop"], vec!["new", "bad", "--app", "--target", "web"], vec!["new", "bad", "--app", "--target", "web", "--target", "backend"], vec!["new", "bad", "--app", "--target"]] {
        assert!(!support::run(support::cli().current_dir(&directory.0).args(args), 20).status.success());
        assert!(fs::read_dir(&directory.0).unwrap().next().is_none());
    }
    support::success(&support::run(support::cli().current_dir(&directory.0).args(["new", "service", "--app", "--target", "backend"]), 40));
    let root = directory.0.join("service");
    assert!(!root.join("web").exists());
    assert!(!root.join("package.json").exists());
    assert!(!root.join("js-client").exists());
    assert!(!root.join("generated").exists());
    support::success(&support::run(support::cli().current_dir(&root).arg("check"), 40));
}

#[test]
fn consolidated_api_config_preserves_standalone_generation_and_rejects_ambiguity() {
    let directory = support::TempDirectory::new();
    support::success(&support::run(support::cli().current_dir(&directory.0).args(["new", "service", "--app", "--target", "backend"]),40));
    let root=directory.0.join("service");
    fs::create_dir_all(root.join("app/api")).unwrap();
    for (path,text) in [
      ("app/main.tsr",include_str!("../templates/application/app/main.tsr.template")),
      ("app/api/operations.tsr",include_str!("../templates/application/app/api/operations.tsr.template")),
      ("app/api/context.tsr",include_str!("../templates/application/app/api/context.tsr.template")),
      ("app/api/access.tsr",include_str!("../templates/application/app/api/access.tsr.template")),
    ]{fs::write(root.join(path),support::format_round_trip(text)).unwrap();}
    let config_path=root.join("tsrust.config.json");
    let mut config:serde_json::Value=serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    config["api"]=serde_json::json!({"entry":"app/api/operations.tsr","package":{"name":"@app/api","version":"0.1.0"},"server":{"context":{"module":"app/api/context.tsr","export":"AppContext"},"authorize":{"module":"app/api/access.tsr","export":"authorize"}},"operations":[{"name":"greet","module":"app/api/operations.tsr","export":"greet","kind":"mutation"}]});
    fs::write(&config_path,serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    let mut standalone=config["api"].clone();
    standalone["schemaVersion"]=1.into();standalone["output"]="legacy/client".into();standalone["server"]["output"]="legacy/server".into();
    fs::write(root.join("tsrust.api.json"),serde_json::to_vec_pretty(&standalone).unwrap()).unwrap();
    let ambiguous=support::run(support::cli().current_dir(&root).args(["api","generate"]),40);
    assert!(!ambiguous.status.success());assert!(String::from_utf8_lossy(&ambiguous.stderr).contains("Both"));assert!(!root.join("legacy").exists());
    support::success(&support::run(support::cli().current_dir(&root).args(["api","generate","--config","tsrust.api.json"]),40));
    assert!(root.join("legacy/client/index.js").is_file());
    support::success(&support::run(support::cli().current_dir(&root).args(["api","generate","--config","tsrust.config.json","--out","js-client"]),40));
    config.as_object_mut().unwrap().remove("api");fs::write(&config_path,serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    support::success(&support::run(support::cli().current_dir(&root).args(["api","generate","--check"]),40));
    config["api"]=serde_json::json!({"entry":"../escape.tsr"});fs::write(&config_path,serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    assert!(!support::run(support::cli().current_dir(&root).arg("check"),40).status.success());
}

#[test]
fn source_client_generation_stays_inside_the_nearest_plain_project(){
 let directory=support::TempDirectory::new();fs::write(directory.0.join("tsrust.api.json"),"parent RPC config must not be read").unwrap();
 support::success(&support::run(support::cli().current_dir(&directory.0).args(["new","child","--app"]),40));let child=directory.0.join("child");
 support::success(&support::run(support::cli().current_dir(&child).args(["api","generate","--out","client"]),40));
 let value:serde_json::Value=serde_json::from_slice(&fs::read(child.join("client/http-api.json")).unwrap()).unwrap();assert!(value.get("routes").is_some());assert!(value.get("history").is_none());
}

#[test]
fn desktop_ui_template_checks_against_the_current_component_api() {
    let directory = support::TempDirectory::new();
    support::success(&support::run(support::cli().current_dir(&directory.0).args(["new", "desk", "--ui"]), 40));
    support::success(&support::run(support::cli().current_dir(directory.0.join("desk")).arg("check"), 120));
}
