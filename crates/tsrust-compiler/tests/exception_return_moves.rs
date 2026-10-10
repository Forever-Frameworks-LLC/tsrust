mod support;

fn runs(source: &str, expected: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
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

/// Known defect: a `finally` must run before the return, so the return travels as a
/// completion value and Rust cannot see that the later use is on another path.
#[test]
#[ignore = "return of an owned value through finally, with a later use, still fails native compilation"]
fn returned_owned_value_through_finally_remains_usable_on_other_paths() {
    runs(
        "function pick(a:string,fail:boolean):string{try{try{if(fail){throw 3;}}catch(error){return a;}}finally{console.log(\"outer\");}return a;}function main(){console.log(pick(\"x\",true));console.log(pick(\"y\",false));}",
        "outer\nx\nouter\ny\n",
    );
}
