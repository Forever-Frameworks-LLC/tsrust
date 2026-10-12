mod support;

#[test]
fn checked_literal_types_preserve_exact_values_and_widen_safely() {
    let source=r#"
    function ready(value:'ready'):string{return value;}
    function exact(value:9007199254740993):int{return value;}
    function flag(value:false):boolean{return value;}
    function main(){const tag:'ready'='ready';const value:9007199254740993=9007199254740993;console.log(ready(tag));console.log(exact(value));console.log(flag(false));}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready\n9007199254740993\nfalse\n");
}

#[test]
fn checked_literal_types_reject_unproven_values() {
    for source in [
      "function main(){const value:'ready'='wrong';}",
      "function main(){const value:9007199254740993=9007199254740992;}",
      "function narrow(value:string):'ready'{return value;}function main(){}",
      "function main(){const value:false=true;}",
    ] {assert!(tsrust_compiler::check(source).is_err(),"accepted unproven literal: {source}");}
}

#[test]
fn literal_types_compose_with_aliases_records_arrays_and_generics() {
    let source=r#"
    type Tag='ready';type Row={tag:Tag;count:9007199254740993;};
    function identity<T>(value:T):T{return value;}
    function main(){const row:Row={tag:'ready',count:9007199254740993};console.log(row.tag,row.count);const values:Tag[]=['ready','ready'];values.push('ready');for(const value of values){console.log(value);}const chosen=identity<Tag>('ready');console.log(chosen);const zero:0.0=0;const minimum:-9223372036854775808=-9223372036854775808;console.log(zero,minimum);}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready 9007199254740993\nready\nready\nready\nready\n0 -9223372036854775808\n");
}

#[test]
fn literal_arrays_and_records_reject_incompatible_mutation() {
    for source in [
      "function main(){const values:'ready'[]=['ready'];values.push('wrong');}",
      "type Row={tag:'ready';};function main(){const row:Row={tag:'ready'};row.tag='wrong';}",
      "function main(){const values:'ready'[]=['ready'];const broad:string[]=values;broad.push('wrong');}",
    ] {assert!(tsrust_compiler::check(source).is_err(),"literal mutation accepted: {source}");}
}

#[test]
fn literal_unions_inject_values_and_preserve_owned_strings() {
    let source=r#"
    type State='ready'|'done';
    function pass(value:State):State{return value;}
    function main(){const first:State='ready';const second=pass('done');console.log(first,second);const values:State[]=['ready','done'];for(const value of values){console.log(value);}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready done\nready\ndone\n");
}

#[test]
fn literal_operators_return_scalar_results() {
    let source=r#"
    function main(){const label:'ready'='ready';const n:7=7;const decimal:2.5=2.5;console.log(label.length);console.log(label+'!');console.log(label==='ready');console.log(n+1,n<8,decimal+0.5);}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"5\nready!\ntrue\n8 true 3\n");
}

#[test]
fn literal_updates_cannot_break_the_declared_value() {
    for source in [
      "function main(){let value:7=7;value++;}",
      "function main(){let value:7=7;value+=1;}",
      "function main(){let value:'ready'='ready';value+='!';}",
    ] {assert!(tsrust_compiler::check(source).is_err(),"literal update accepted: {source}");}
}

#[test]
fn literal_union_equality_narrows_both_branches() {
    let source=r#"
    type State='ready'|'done';
    function classify(value:State):int{if(value==='ready'){const ready:'ready'=clone(value);return ready.length;}else{const done:'done'=clone(value);return done.length;}}
    function main(){console.log(classify('ready'),classify('done'));}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"5 4\n");
}

#[test]
fn literal_methods_use_scalar_contracts_without_losing_declared_identity() {
    let source=r#"
    function main(){const label:'ready'='ready';const upper=label.toUpperCase();console.log(upper,label.includes('ad'));const value:2.5=2.5;console.log(value.toFixed(1));const zero:0.0=-0.0;console.log(Object.is(zero,-0.0),Object.is(label,'ready'));}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"READY true\n2.5\ntrue true\n");
}

#[test]
fn dynamic_comparisons_do_not_prove_a_particular_literal() {
    for source in [
        "type State='ready'|'done';function f(value:State,key:string):'ready'{if(value===key){return clone(value);}return 'ready';}function main(){}",
        "type State='ready'|'done';function f(value:State,key:string):'done'{if(value!==key){return clone(value);}return 'done';}function main(){}",
    ] { assert!(tsrust_compiler::check(source).is_err(),"dynamic guard unsafely narrowed: {source}"); }
}

#[test]
fn literal_conversions_observe_underlying_values() {
    let source=r#"
    function main(){const empty:''='';const zero:0=0;const no:false=false;const numeric:'42'='42';const choice:''|'yes'='';console.log(Boolean(empty),Boolean(zero),Boolean(no),Boolean(choice));console.log(String(empty),String(zero),String(no),String(choice));console.log(Number(numeric),Number(zero),Number(no));}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"false false false false\n 0 false \n42 0 0\n");
}

#[test]
fn literal_strings_support_indexing_and_iteration() {
    let source=r#"
    function main(){const text:'ab'='ab';console.log(text[0]);for(const letter of text){console.log(letter);}console.log(text);}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"a\na\nb\nab\n");
}

#[test]
fn literal_union_typeof_reports_scalar_kinds() {
    let source=r#"
    function inspect(value:'ready'|7|false):string{return typeof value;}
    function main(){console.log(inspect('ready'),inspect(7),inspect(false));}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"string number boolean\n");
}

#[test]
fn literal_values_and_unions_widen_into_scalar_union_members() {
    let source=r#"
    function show(value:string|null):void{console.log(value);}
    function main(){const tag:'ready'='ready';show(tag);const state:'done'|'ready'='done';show(state);const n:7|false=7;const broad:int|boolean=n;console.log(broad);}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready\ndone\n7\n");
}

#[test]
fn owned_union_widening_preserves_transfer() {
    support::reject("function main(){const value:'ready'='ready';const tagged:string|null=value;console.log(value,tagged);}", "transferred");
}
