mod support;

#[test]
fn mutable_record_parameter_replaces_callback_with_capture_free_origin() {
    let source=support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder={run:ReadOnly<Function<int>>;};
        function replace(holder:Mutable<Holder>):void {
            holder.run=():int=>11;
        }
        function main(){
            const a=[3];
            {
                let holder:Holder={run:():int=>a[0]};
                replace(holder);
                strictEqual(holder.run(),11);
            }
            a[0]=5;strictEqual(a[0],5);
        }
    "#);
    let directory=support::TempDirectory::new();
    let path=directory.source(&source);
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn capture_free_parameter_replacement_releases_old_capture_before_record_disposal() {
    let source=support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder={run:ReadOnly<Function<int>>;};
        function replace(holder:Mutable<Holder>):void { holder.run=():int=>11; }
        function main(){
            const a=[3];
            let holder:Holder={run:():int=>a[0]};
            replace(holder);
            a[0]=5;
            strictEqual(a[0],5);strictEqual(holder.run(),11);
        }
    "#);
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
    support::success(&output);
    assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn replacement_obligations_cross_relays_and_distinct_caller_environments() {
    let source=support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder={run:ReadOnly<Function<int>>;};
        type Pair={left:Holder;right:Holder;};
        function replace(holder:Mutable<Holder>):void { holder.run=():int=>11; }
        function relay(pair:Mutable<Pair>):void { replace(pair.left); }
        function main(){
            const a=[3];const b=[7];const c=[19];
            {
                let first:Holder={run:():int=>a[0]};
                let pair:Pair={left:{run:():int=>b[0]},right:{run:():int=>c[0]}};
                replace(first);relay(pair);
                strictEqual(first.run(),11);strictEqual(pair.left.run(),11);strictEqual(pair.right.run(),19);
            }
            a[0]=5;b[0]=9;c[0]=23;strictEqual(a[0]+b[0]+c[0],37);
        }
    "#);
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
    support::success(&output);
    assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn both_replacement_branches_and_copied_scalar_environment_release_old_capture() {
    for flag in [false,true]{
        let source=support::format_round_trip(&format!(r#"
            import {{strictEqual}} from "std:testing";
            type Holder={{run:ReadOnly<Function<int>>;}};
            function replace(holder:Mutable<Holder>,flag:boolean,amount:int):void{{
                if(flag){{holder.run=():int=>amount;}}else{{holder.run=():int=>amount+2;}}
            }}
            function main(){{
                const a=[3];let holder:Holder={{run:():int=>a[0]}};
                replace(holder,{flag},11);a[0]=5;
                strictEqual(a[0],5);strictEqual(holder.run(),{});
            }}
        "#,if flag{11}else{13}));
        let directory=support::TempDirectory::new();
        let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
        support::success(&output);
        assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
    }
}

#[test]
fn optional_replacement_and_short_circuit_keep_original_capture_conflict() {
    for body in ["if(flag){holder.run=():int=>11;}","flag && ((holder.run=():int=>11),true);"]{
        let source=support::format_round_trip(&format!(r#"
            type Holder={{run:ReadOnly<Function<int>>;}};
            function replace(holder:Mutable<Holder>,flag:boolean):void{{{body}}}
            function main(){{const a=[3];let holder:Holder={{run:():int=>a[0]}};replace(holder,false);a[0]=5;console.log(holder.run());}}
        "#));
        let error=match tsrust_compiler::check(&source){Ok(_)=>panic!("skipped replacement lost original capture"),Err(error)=>error};
        assert_eq!(error.code,"capture-conflict","{error:?}");
    }
}

#[test]
fn replacement_preserves_separate_callback_owner_and_sibling_conflicts() {
    for extra in [
        "const saved=holder.run;holder={run:():int=>b[0]};replace(holder);a[0]=5;console.log(saved(),holder.run());",
        "const sibling:Holder={run:():int=>a[0]};replace(holder);a[0]=5;console.log(sibling.run(),holder.run());",
    ]{
        let source=support::format_round_trip(&format!(r#"
            type Holder={{run:ReadOnly<Function<int>>;}};
            function replace(holder:Mutable<Holder>):void{{holder.run=():int=>11;}}
            function main(){{const a=[3];const b=[7];let holder:Holder={{run:():int=>a[0]}};{extra}}}
        "#));
        let error=match tsrust_compiler::check(&source){Ok(_)=>panic!("replacement released another owner's capture"),Err(error)=>error};
        assert_eq!(error.code,"capture-conflict","{error:?}");
    }
}

#[test]
fn readonly_record_parameter_cannot_replace_callback() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function replace(holder:ReadOnly<Holder>):void{holder.run=():int=>11;}function main(){}");
    let error=match tsrust_compiler::check(&source){Ok(_)=>panic!("read-only replacement accepted"),Err(error)=>error};
    assert_eq!(error.code,"parameter-access","{error:?}");
}

#[test]
fn nested_replacement_call_releases_capture_before_later_comma_operands() {
    let source=support::format_round_trip(r#"
        import {strictEqual} from "std:testing";
        type Holder={run:ReadOnly<Function<int>>;};
        function replace(holder:Mutable<Holder>):void{holder.run=():int=>11;}
        function main(){const a=[3];const holder:Holder={run:():int=>a[0]};
            const observed=(replace(holder),a[0]=5,holder.run());
            strictEqual(observed,11);strictEqual(a[0],5);
        }
    "#);
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
    support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn reification_keeps_call_result_and_argument_effects_once() {
    let source=support::format_round_trip(r#"
        import {strictEqual} from "std:testing";
        type Holder={run:ReadOnly<Function<int>>;};
        function replace(holder:Mutable<Holder>,seen:Mutable<int[]>):int{
            seen[0]+=1;holder.run=():int=>11;return seen[0];
        }
        function main(){const a=[3];const seen=[0];const holder:Holder={run:():int=>a[0]};
            const observed=replace(holder,seen);a[0]=5;
            strictEqual(observed,1);strictEqual(seen[0],1);strictEqual(a[0],5);strictEqual(holder.run(),11);
        }
    "#);
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
    support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
