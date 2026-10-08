mod support;

const HOLDER: &str = "import { strictEqual } from \"std:testing\"; type Holder={run:ReadOnly<Function<int>>;};";

fn native(body: &str) {
    let source = support::format_round_trip(&format!("{HOLDER}{body}"));
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

fn rejected(body: &str, code: &str) {
    // The single-file check has no module loader; these sources need no import.
    let source = support::format_round_trip(&format!("type Holder={{run:ReadOnly<Function<int>>;}};{body}"));
    let error = match tsrust_compiler::check(&source) {
        Ok(_) => panic!("must be rejected: {body}"),
        Err(error) => error,
    };
    assert_eq!(error.code, code, "{body}: {}", error.message);
}

#[test]
fn callbacks_stored_through_a_record_parameter_cannot_borrow_the_callee() {
    for body in [
        // A borrowed parameter of the callee.
        "function replace(holder:Mutable<Holder>,next:ReadOnly<int[]>):void{holder.run=():int=>next[0];} function main(){const a=[3];const b=[7];let holder:Holder={run:():int=>a[0]};replace(holder,b);}",
        // A local of the callee.
        "function replace(holder:Mutable<Holder>):void{const local=[4];holder.run=():int=>local[0];} function main(){const a=[3];let holder:Holder={run:():int=>a[0]};replace(holder);}",
        // A callback parameter lent for the call.
        "function set(holder:Mutable<Holder>,f:ReadOnly<Function<int>>):void{holder.run=f;} function main(){const a=[3];let holder:Holder={run:():int=>a[0]};set(holder,():int=>9);}",
    ] {
        rejected(body, "callback-escape");
    }
    // A copied number is not a borrow.
    native("function replace(holder:Mutable<Holder>,amount:int):void{holder.run=():int=>amount;} function main(){const a=[3];let holder:Holder={run:():int=>a[0]};replace(holder,12);a[0]=5;strictEqual(holder.run(),12);}");
}

#[test]
fn containers_and_unions_do_not_hold_borrowing_records_yet() {
    for body in [
        "function main(){const a=[3];const holders:Holder[]=[{run:():int=>a[0]}];}",
        "function main(){const a=[3];const m=new Map<string,Holder>();m.set(\"k\",{run:():int=>a[0]});}",
        "function call(extra:int,holder:ReadOnly<Holder>|undefined=undefined):int{return extra;} function main(){call(1);}",
    ] {
        rejected(body, "unsupported");
    }
    // A tuple is a record; its fields keep separate layouts.
    native("function main(){const a=[3];const pair:[Holder,int]=[{run:():int=>a[0]},1];strictEqual(pair[0].run()+pair[1],4);}");
}

#[test]
fn nullable_borrowing_callback_fields_are_read_but_not_replaced() {
    let nullable = "type Maybe={run:ReadOnly<Function<int>>|null;};";
    native(&format!("{nullable} function main(){{const a=[3];const maybe:Maybe={{run:():int=>a[0]}};if(maybe.run!==null){{strictEqual(maybe.run(),3);}}const none:Maybe={{run:null}};strictEqual(none.run===null,true);}}"));
    for body in [
        "function main(){const a=[3];let maybe:Maybe={run:null};maybe.run=():int=>a[0];}",
        "function main(){const a=[3];let maybe:Maybe={run:():int=>a[0]};maybe.run=null;}",
    ] {
        rejected(&format!("{nullable}{body}"), "unsupported");
    }
}

#[test]
fn replaced_fields_keep_their_error_layout_and_moved_carrier() {
    // Calls and stores inside exception regions.
    native("function main(){const a=[3];const b=[4];let holder:Holder={run:():int=>a[0]};holder.run=():int=>b[0];try{strictEqual(holder.run(),4);}finally{strictEqual(b[0],4);}}");
    native("function main(){const a=[3];const b=[4];let holder:Holder={run:():int=>a[0]};try{holder.run=():int=>b[0];}finally{strictEqual(holder.run(),4);} strictEqual(holder.run(),4);}");
    native("function main(){const a=[3];const b=[4];let holder:Holder={run:():int=>a[0]};let caught=0;try{holder.run=():int=>b[0];}catch(error){caught=1;} strictEqual(holder.run()+caught,4);}");
    // A field moved out after a replacement, by a callee or locally.
    native("function replace(holder:Mutable<Holder>,amount:int):void{holder.run=():int=>amount;} function main(){const base=[1,2];let holder:Holder={run:():int=>base[0]};replace(holder,9);const run=holder.run;strictEqual(run(),9);}");
    native("function main(){const base=[1,2];let holder:Holder={run:():int=>base[0]};holder.run=():int=>base[1]*2;const run=holder.run;strictEqual(run(),4);strictEqual(run(),4);}");
}

#[test]
fn loop_exits_release_captures_of_iteration_locals() {
    native("function main(){let n=0;for(const k of [1,2]){const advance=():void=>{n+=k;};advance();if(k>0){continue;}}strictEqual(n,3);const a=[0];while(a[0]<3){const step=():void=>{a[0]+=1;};step();if(true){continue;}}strictEqual(a[0],3);let m=0;for(const k of [1,2]){const once=():void=>{m+=k;};once();break;}strictEqual(m,1);}");
    // A callback declared before the loop still holds its capture after it.
    rejected("function main(){let n=0;const outer=():void=>{n+=1;};for(const k of [1,2]){outer();if(k>0){break;}}console.log(n);outer();}", "capture-conflict");
}

#[test]
fn statement_keywords_are_not_values() {
    for body in ["const x = return;", "for (;true;break) {}", "console.log(const);"] {
        let error = tsrust_compiler::check(&format!("function main() {{ {body} }}")).err().expect("keyword value");
        assert_eq!(error.code, "syntax", "{body}: {}", error.message);
    }
}

#[test]
fn handler_of_a_try_body_that_cannot_fail_is_checked_and_never_runs() {
    native("function main(){let caught=0;let y=0;try{y=2;}catch(error){caught=1;console.log(error);}finally{caught+=2;}strictEqual(caught+y,4);const a=[3];const b=[4];let holder:Holder={run:():int=>a[0]};try{holder.run=():int=>b[0];}catch(error){caught=9;}strictEqual(holder.run()+caught,6);}");
}
