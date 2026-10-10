//! Boundaries of caller-borrowing callback stores: forms that run, and forms
//! that receive a source diagnostic instead of a native failure.
mod support;
use support::{execute, reject};

const HOLDER: &str = "type Holder={run:ReadOnly<Function<int>>|undefined;};function install(h:Mutable<Holder>,s:ReadOnly<int[]>):void{h.run=():int=>s[0];}";

#[test]
fn installers_inside_exception_regions_retain_and_convert_their_inputs() {
    execute(&format!("{HOLDER}function settle(h:Mutable<Holder>,a:ReadOnly<int[]>,b:ReadOnly<int[]>):void{{try{{install(h,a);}}finally{{install(h,b);}}}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};settle(h,a,b);console.log(h.run!());}}"), "10\n", None);
    execute(&format!("{HOLDER}function settle(h:Mutable<Holder>,a:ReadOnly<int[]>,b:ReadOnly<int[]>,f:boolean):void{{try{{if(f)throw \"x\";install(h,a);}}catch(e){{install(h,b);}}}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};settle(h,a,b,true);console.log(h.run!());settle(h,a,b,false);console.log(h.run!());}}"), "10\n3\n", None);
    execute(&format!("{HOLDER}function risky(h:Mutable<Holder>,a:ReadOnly<int[]>,n:int):void{{let i=0;while(i<n){{if(i===1)throw \"stop\";install(h,a);i+=1;}}}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};install(h,b);try{{risky(h,a,3);}}catch(e){{console.log(\"caught\");}}console.log(h.run!());}}"), "caught\n3\n", None);
    // The input stays borrowed after an installer that stores inside a region.
    for region in ["try{install(h,a);}finally{install(h,b);}", "try{install(h,b);}catch(e){install(h,a);}"] {
        reject(&format!("{HOLDER}function settle(h:Mutable<Holder>,a:ReadOnly<int[]>,b:ReadOnly<int[]>):void{{{region}}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};settle(h,a,b);b[0]+=1;console.log(h.run!());}}"), "capture-conflict");
    }
}

#[test]
fn branch_loop_and_early_exit_stores_keep_exact_callbacks() {
    execute(&format!("{HOLDER}function pick(h:Mutable<Holder>,a:ReadOnly<int[]>,b:ReadOnly<int[]>,f:boolean):void{{if(f){{install(h,a);}}else{{install(h,b);}}}}function fill(h:Mutable<Holder>,a:ReadOnly<int[]>,n:int):void{{let i=0;while(i<n){{install(h,a);i+=1;}}}}function maybe(h:Mutable<Holder>,a:ReadOnly<int[]>,f:boolean):void{{if(!f)return;install(h,a);}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};pick(h,a,b,true);console.log(h.run!());pick(h,a,b,false);console.log(h.run!());fill(h,a,2);console.log(h.run!());fill(h,b,0);console.log(h.run!());maybe(h,b,false);console.log(h.run!());maybe(h,b,true);console.log(h.run!());}}"), "3\n10\n3\n3\n3\n10\n", None);
    execute(&format!("{HOLDER}function relay(h:Mutable<Holder>,a:ReadOnly<int[]>,n:int):void{{if(n>0){{relay(h,a,n-1);return;}}install(h,a);}}function even(h:Mutable<Holder>,a:ReadOnly<int[]>,n:int):void{{if(n===0){{install(h,a);return;}}odd(h,a,n-1);}}function odd(h:Mutable<Holder>,a:ReadOnly<int[]>,n:int):void{{even(h,a,n-1);}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};relay(h,a,3);console.log(h.run!());even(h,b,4);console.log(h.run!());}}"), "3\n10\n", None);
    // A store that may not happen keeps the earlier input borrowed.
    reject(&format!("{HOLDER}function maybe(h:Mutable<Holder>,a:ReadOnly<int[]>,f:boolean):void{{if(!f)return;install(h,a);}}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};install(h,b);maybe(h,a,false);b[0]+=1;console.log(h.run!());}}"), "capture-conflict");
}

#[test]
fn installer_values_used_abstractly_have_source_diagnostics() {
    let apply = "function apply(f:Mutable<Function<void,[t:Mutable<Holder>]>>,h:Mutable<Holder>):void{f(h);}";
    for body in [
        // Passed as a callback.
        "const inner=(d:Mutable<Holder>):void=>{install(d,a);};apply(inner,h);",
        // Converted to another callable contract.
        "const inner=(d:Mutable<Holder>):void=>{install(d,a);};const once:Once<Function<void,[t:Mutable<Holder>]>>=inner;once(h);",
        // Passed to a closure that invokes its callback parameter.
        "const inner=(d:Mutable<Holder>):void=>{install(d,a);};const caller=(f:Mutable<Function<void,[t:Mutable<Holder>]>>,t:Mutable<Holder>):void=>{f(t);};caller(inner,h);",
        // The receiving record is captured instead of passed.
        "{const go=():void=>{install(h,a);};go();}",
        "{const go=():void=>{h.run=():int=>a[0];};go();}",
    ] {
        let error = reject(&format!("{HOLDER}{apply}function main(){{const a=[3];const h:Holder={{run:undefined}};{body}console.log(h.run!());}}"), "unsupported");
        assert!(error.message.contains("closure"), "{}", error.message);
    }
    // An alias of an installer closure, and a second alias of a saved named installer.
    reject(&format!("{HOLDER}function main(){{const a=[3];const h:Holder={{run:undefined}};const perform=(t:Mutable<Holder>,s:ReadOnly<int[]>):void=>{{install(t,s);}};const alias=perform;alias(h,a);console.log(h.run!());}}"), "unsupported");
    reject(&format!("{HOLDER}function main(){{const a=[3];const h:Holder={{run:undefined}};const one=install;const two=one;two(h,a);console.log(h.run!());}}"), "unsupported");
    // A saved named installer can be captured and called by a closure.
    execute(&format!("{HOLDER}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};const perform=install;const go=(t:Mutable<Holder>,s:ReadOnly<int[]>):void=>{{perform(t,s);}};go(h,a);go(h,b);a[0]+=1;console.log(h.run!(),a[0]);}}"), "10 4\n", None);
    // A named installer passed as a callback argument.
    reject(&format!("{HOLDER}function relay(f:Mutable<Function<void,[t:Mutable<Holder>,s:ReadOnly<int[]>]>>,h:Mutable<Holder>,a:ReadOnly<int[]>):void{{f(h,a);}}function main(){{const a=[3];const h:Holder={{run:undefined}};relay(install,h,a);console.log(h.run!());}}"), "unsupported");
    // A held installer whose stored callback mutates a captured variable.
    reject("type Holder={run:Mutable<Function<int>>|undefined;};function install(h:Mutable<Holder>,s:Mutable<int[]>):void{h.run=():int=>{s[0]+=1;return s[0];};}function clear(h:Mutable<Holder>):void{h.run=undefined;}function main(){const a=[3];const h:Holder={run:undefined};{const inner=(d:Mutable<Holder>):void=>{install(d,a);};const outer=(t:Mutable<Holder>):void=>{inner(t);};outer(h);console.log(h.run!());clear(h);}a[0]+=1;console.log(a[0]);}", "unsupported");
    // Direct and saved named installers, and owning callback fields, are unaffected.
    execute(&format!("{HOLDER}function main(){{const a=[3];const b=[10];const h:Holder={{run:undefined}};const perform=install;perform(h,a);console.log(h.run!());perform(h,b);a[0]+=1;console.log(h.run!(),a[0]);}}"), "3\n10 4\n", None);
    execute("type Holder={run:Function<int>|undefined;};function main(){const n=3;const h:Holder={run:undefined};{const go=():void=>{h.run=():int=>n;};go();}console.log(h.run!());}", "3\n", None);
}

#[test]
fn recursive_calls_cannot_grow_a_borrowed_callback() {
    let error = reject("function call(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return call((): int => { return c(); }, n - 1); } function main() { let k = 0; console.log(call((): int => { k += 1; return k; }, 3)); }", "callback-escape");
    assert!(error.message.contains("recursive"), "{}", error.message);
    execute("function call(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() { let k = 0; console.log(call((): int => { k += 1; return k; }, 3)); }", "1\n", None);
}
