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

    #[test]
    fn static_configuration_is_atomic_and_streamed_routes_preserve_policy() {
        let directory = super::support::TempDirectory::new();
        std::fs::write(directory.0.join("index.html"), "<h1>web</h1>").unwrap();
        std::fs::write(directory.0.join("file.txt"), "native file").unwrap();
        let root = directory.0.to_str().unwrap();
        let mut app = tsrust_native_http::Router::new();
        app.use_middleware(|context,next|Box::pin(async move{
            context.header("x-before","entry").unwrap();
            context.header("x-content-type-options","nosniff").unwrap();
            next.run(context).await;
            context.header("x-after","exit").unwrap();
            None
        }));
        app.route("GET","/raw",|_|Box::pin(async{Some(tsrust_native_http::Response::text("raw response".into(),200).unwrap())})).unwrap();
        for (prefix, fallback) in [("/", ""), ("/bad/", ""), ("/web", "../secret.html"), ("/web", "missing.html"), ("/web", "file.txt")] {
            assert_eq!(app.serve_static(prefix, root, fallback).unwrap_err().code, "static");
        }
        app.serve_static("/web", root, "index.html").unwrap();
        assert!(app.serve_static("/web/nested", root, "").is_err());
        assert!(app.serve_static("/web", root, "").is_err());
        app.serve_static("/second", root, "").unwrap();
        tsrust_native_futures::block_on(async {
            let server = app.listen("127.0.0.1:0").await.unwrap();
            let client = tsrust_native_http::Client::new().unwrap();
            for (path, accept, status, body) in [
                ("/web/file.txt", "*/*", 200, "native file"),
                ("/raw", "*/*", 200, "raw response"),
                ("/web/projects", "text/html", 200, "<h1>web</h1>"),
                ("/web/projects", "text/html;q=0", 404, "Not Found"),
                ("/second/projects", "text/html", 404, "Not Found"),
                ("/web/missing.js", "text/html", 404, "Not Found"),
                ("/api", "text/html", 404, "Not Found"),
            ] {
                let mut request = tsrust_native_http::Request::new("GET", &format!("http://{}{path}", server.address()), String::new()).unwrap();
                request.set_header("accept", accept).unwrap();
                let response = client.send(request).await.unwrap();
                assert_eq!(response.status(), status, "{path}");
                assert_eq!(response.header("x-before").unwrap().as_deref(),Some("entry"),"{path}");
                assert_eq!(response.header("x-after").unwrap().as_deref(),Some("exit"),"{path}");
                assert_eq!(response.header("x-content-type-options").unwrap().as_deref(),Some("nosniff"),"{path}");
                assert_eq!(response.text_body().await.unwrap(), body, "{path}");
            }
            server.shutdown().await;
        });
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
