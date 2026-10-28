mod support;
// Exercise the exact bundled HTTP source without unrelated platform-only lib tests.
#[allow(dead_code)]
mod runtime {
    include!("../src/stdlib/futures/runtime.rs");
    include!("../src/stdlib/http/runtime.rs");

    mod regressions {
        use super::*;
        include!("../src/stdlib/http/runtime_tests.rs");
    }


}

#[test]
fn static_mount_signatures_emission_and_ownership() {
    assert!(tsrust_compiler::native_bindings::lookup("std:http", "__routerStatic").is_ok());
    let source = support::format_round_trip("import {Router} from 'std:http';function configure(app:Mutable<Router>,root:string){app.serveStatic('/web',root,'index.html');app.serveStatic('/public',root);}function main(){const app=new Router();configure(app,'dist');}");
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("emit").arg(directory.source(&source)), 30);
    support::success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains(".serve_static("));
    for invalid in [
        "import {Router} from 'std:http';function main(){const app=new Router();app.serveStatic('/web');}",
        "import {Router} from 'std:http';function main(){const app=new Router();app.serveStatic('/web',1);}",
        "import {Router} from 'std:http';function configure(app:ReadOnly<Router>){app.serveStatic('/web','dist');}function main(){}",
    ] {
        let directory = support::TempDirectory::new();
        let output = support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(invalid))), 30);
        assert!(!output.status.success(), "accepted {invalid}");
    }
}
