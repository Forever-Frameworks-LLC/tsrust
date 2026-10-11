mod support;

fn runs(source: &str, expected: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 120);
    support::success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected,
        "{source}"
    );
}

/// A `return` inside a `try` or `catch` body leaves the function. An owned value that
/// it returns is still available on the paths that did not return.
#[test]
fn returned_owned_value_in_try_or_catch_remains_usable_on_other_paths() {
    runs(
        "function pick(a:string,fail:boolean):string{try{if(fail){throw 3;}}catch(error){return a;}return a;}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}",
        "x y\n",
    );
    runs(
        "function pick(a:string,early:boolean):string{try{if(early){return a;}}catch(error){}return a+\"!\";}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}",
        "x y!\n",
    );
    runs(
        "type Row={name:string;};function pick(row:Row,fail:boolean):Row{try{if(fail){throw 3;}}catch(error){return row;}return row;}function main(){console.log(pick({name:\"a\"},true).name,pick({name:\"b\"},false).name);}",
        "a b\n",
    );
    runs(
        "function pick(values:int[],fail:boolean):int[]{try{try{if(fail){throw 3;}}catch(error){return values;}}catch(outer){}return values;}function main(){console.log(pick([1],true).length,pick([1,2],false).length);}",
        "1 2\n",
    );
    runs(
        "function pick(a:string,fail:boolean):string{for(let i=0;i<2;i+=1){try{if(fail){throw 3;}}catch(error){return a;}}return a;}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}",
        "x y\n",
    );
}

/// `finally` still runs before a return that passes through it.
#[test]
fn return_through_finally_runs_cleanup_first() {
    runs(
        "function pick(a:string,fail:boolean):string{try{if(fail){throw 3;}return a;}catch(error){return \"caught\";}finally{console.log(\"cleanup\");}}function main(){console.log(pick(\"x\",true));console.log(pick(\"y\",false));}",
        "cleanup\ncaught\ncleanup\ny\n",
    );
}

/// A `finally` that returns replaces a pending return, and a closure body keeps
/// its own return target.
#[test]
fn replaced_and_nested_returns_keep_their_targets() {
    runs(
        "function pick(a:string):string{try{return a;}finally{return \"final\";}}function main(){console.log(pick(\"x\"));}",
        "final\n",
    );
    runs(
        "function pick(a:string,fail:boolean):string{const inner=():string=>{try{if(fail){throw 3;}}catch(error){return \"inner\";}return \"plain\";};try{if(fail){throw 4;}}catch(error){return inner()+a;}return inner()+a;}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}",
        "innerx plainy\n",
    );
    runs(
        "function pick(a:string,fail:boolean):string{outer:for(let i=0;i<3;i+=1){try{if(i===0){continue outer;}if(fail){throw 3;}break outer;}catch(error){return a+String(i);}}return a;}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}",
        "x1 y\n",
    );
}

/// Ownership remains available on paths that do not return through cleanup.
#[test]
fn returned_owned_value_through_finally_remains_usable_on_other_paths() {
    runs(
        "function pick(a:string,fail:boolean):string{try{try{if(fail){throw 3;}}catch(error){return a;}}finally{console.log(\"outer\");}return a;}function main(){console.log(pick(\"x\",true));console.log(pick(\"y\",false));}",
        "outer\nx\nouter\ny\n",
    );
}

#[test]
fn finally_returns_preserve_owned_containers_locals_and_replacements() {
    for (source,expected) in [
        ("type Row={name:string;};function pick(a:Row,early:boolean):Row{try{if(early){return a;}}finally{console.log(\"cleanup\");}console.log(a.name);return a;}function main(){console.log(pick({name:\"x\"},true).name,pick({name:\"y\"},false).name);}","cleanup\ncleanup\ny\nx y\n"),
        ("function pick(a:int[],early:boolean):int[]{try{if(early){return a;}}finally{console.log(\"cleanup\");}a.push(2);return a;}function main(){console.log(pick([1],true).length,pick([1],false).length);}","cleanup\ncleanup\n1 2\n"),
        ("function pick(early:boolean):string{let a=\"old\";try{if(early){return a;}}finally{a=\"new\";}return a;}function main(){console.log(pick(true),pick(false));}","old new\n"),
        ("function pick(early:boolean):string{let a:string;a=\"ready\";try{if(early){return a;}}finally{console.log(\"cleanup\");}return a;}function main(){console.log(pick(true),pick(false));}","cleanup\ncleanup\nready ready\n"),
        ("function pick(a:string,early:boolean):string{try{if(early){return a;}}finally{if(early){return \"override\";}}return a;}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}","override y\n"),
        ("function pick(a:string,early:boolean):string{try{if(early){return a;}}finally{if(early){throw 7;}}return a;}function main(){try{console.log(pick(\"x\",true));}catch(error){console.log(error);}console.log(pick(\"y\",false));}","7\ny\n"),
    ] {runs(source,expected);}
}

#[test]
fn finally_returns_keep_owners_moved_inside_result_expressions() {
    runs("function pass(a:string):string{return a;}function pick(a:string,early:boolean):string{try{if(early){return pass(a);}}finally{console.log(\"cleanup\");}return a;}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}","cleanup\ncleanup\nx y\n");
    runs("type Row={name:string;};function pick(a:string,early:boolean):Row{try{if(early){return {name:a};}}finally{console.log(\"cleanup\");}return {name:a};}function main(){console.log(pick(\"x\",true).name,pick(\"y\",false).name);}","cleanup\ncleanup\nx y\n");
}

#[test]
fn finally_returns_keep_nested_and_closure_owner_paths() {
    runs("function pick(a:string,early:boolean):string{try{try{if(early){return a;}}finally{console.log(1);}return a;}finally{console.log(2);}}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}","1\n2\n1\n2\nx y\n");
    runs("function main(){const pick=(a:string,early:boolean):string=>{try{if(early){return a;}}finally{console.log(1);}return a;};console.log(pick(\"x\",true),pick(\"y\",false));}","1\n1\nx y\n");
    runs("function pick(early:boolean):string{const a=\"x\";const run=():string=>{try{if(early){return a;}}finally{console.log(1);}return a;};return run();}function main(){console.log(pick(true),pick(false));}","1\n1\nx x\n");
}

#[test]
fn finally_return_borrow_does_not_transfer_its_argument() {
    runs("type Row={name:string;};function read(a:ReadOnly<Row>):string{return String(a.name.length);}function pick(a:Row,early:boolean):string{try{if(early){return read(a);}}finally{console.log(1);}return a.name;}function main(){console.log(pick({name:\"xx\"},true),pick({name:\"y\"},false));}","1\n1\n2 y\n");
}

#[test]
fn finally_normal_path_can_capture_returned_owner() {
    runs("function pick(a:string,early:boolean):string{try{if(early){return a;}}finally{console.log(1);}const run=():string=>a;return run();}function main(){console.log(pick(\"x\",true),pick(\"y\",false));}","1\n1\nx y\n");
}

#[test]
fn partial_field_return_through_finally_preserves_siblings() {
    runs("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{if(early){return a.name;}}finally{console.log(a.other);}return a.name;}function main(){console.log(pick({name:\"x\",other:\"one\"},true),pick({name:\"y\",other:\"two\"},false));}","one\ntwo\nx y\n");
}

#[test]
fn partial_returns_route_nested_cleanup_catch_errors_and_replacement() {
 for (source,expected) in [
  ("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{if(early){throw 3;}}catch(error){return a.name;}finally{console.log(a.other);}return a.name;}function main(){console.log(pick({name:\"x\",other:\"one\"},true),pick({name:\"y\",other:\"two\"},false));}","one\ntwo\nx y\n"),
  ("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{try{if(early){return a.name;}}finally{console.log(a.other);}}finally{console.log(2);}return a.name;}function main(){console.log(pick({name:\"x\",other:\"one\"},true),pick({name:\"y\",other:\"two\"},false));}","one\n2\ntwo\n2\nx y\n"),
  ("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{if(early){return a.name;}}catch(error){return \"wrong catch\";}finally{console.log(a.other);if(early){throw 7;}}return a.name;}function main(){try{console.log(pick({name:\"x\",other:\"one\"},true));}catch(error){console.log(error);}console.log(pick({name:\"y\",other:\"two\"},false));}","one\n7\ntwo\ny\n"),
  ("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{if(early){return a.name;}}finally{if(early){return a.other;}}return a.name;}function main(){console.log(pick({name:\"x\",other:\"one\"},true),pick({name:\"y\",other:\"two\"},false));}","one y\n"),
  ("type Child={name:string;};type Row={child:Child;other:string;};function pick(a:Row,early:boolean):string{for(let i=0;i<2;i+=1){try{if(early){return a.child.name;}}finally{console.log(a.other);}}return a.child.name;}function main(){console.log(pick({child:{name:\"x\"},other:\"one\"},true),pick({child:{name:\"y\"},other:\"two\"},false));}","one\ntwo\ntwo\nx y\n"),
 ] {runs(source,expected);}
}

#[test]
fn partial_return_cleanup_restoration_and_control_keep_owner_state() {
 runs("type Row={name:string;other:string;};function pick(a:Row):string{for(let i=0;i<2;i+=1){try{if(i===0){return a.name;}}finally{a.name=\"new\";if(i===0){continue;}}}return a.name;}function main(){console.log(pick({name:\"old\",other:\"sibling\"}));}","new\n");
 runs("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{if(early){return a.name;}}finally{a.name=\"new\";}return a.name;}function main(){console.log(pick({name:\"old\",other:\"sibling\"},true),pick({name:\"old\",other:\"sibling\"},false));}","old new\n");
 runs("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{outer:for(let i=0;i<2;i+=1){try{if(early){return a.name;}}finally{if(early){break outer;}}}return a.other;}function main(){console.log(pick({name:\"x\",other:\"one\"},true),pick({name:\"y\",other:\"two\"},false));}","one two\n");
 runs("type Row={name:string;other:string;};function pick(a:Row,early:boolean):string{try{if(early){return a.name;}}finally{for(let i=0;i<2;i+=1){if(i===1){break;}console.log(a.other);}}const run=():string=>a.name;return run();}function main(){console.log(pick({name:\"x\",other:\"one\"},true),pick({name:\"y\",other:\"two\"},false));}","one\ntwo\nx y\n");
}
