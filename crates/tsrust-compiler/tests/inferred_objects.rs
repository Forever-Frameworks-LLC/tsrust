mod support;

#[test]
fn ordinary_objects_infer_nested_fields_arrays_and_factory_results() {
    support::execute(r#"
        function make(n:int){return {count:n,label:"row",nested:{ready:true}};}
        function main(){
            const first=make(1);let current=make(2);current=make(3);
            const rows=[{count:4,label:"row",nested:{ready:false}},{count:5,label:"row",nested:{ready:true}}];
            const callable={double:(n:int):int=>{return n*2;}};
            console.log(first.count,current.count,rows[1].nested.ready,callable.double(3));
        }
    "#, "1 3 true 6\n", None);
}

#[test]
fn inferred_objects_keep_duplicate_and_ownership_checks() {
    for source in [
        "function main(){const value={x:1,x:2};}",
        "function main(){const values=[1];const object={values:values};console.log(values.length,object.values.length);}",
    ] {
        assert!(tsrust_compiler::check(source).is_err(),"accepted {source}");
    }
}

#[test]
fn generic_calls_infer_objects_without_losing_named_context_or_ownership(){
    let source=r#"
        type Named={count:int;};
        function echo<T>(value:T):T{return value;}
        function second<T>(first:T,last:T):T{return last;}
        function main(){
            const named:Named={count:3};
            const inferred=echo({count:1,nested:{ready:true}});
            const contextual=second({count:2},named);
            console.log(inferred.count,inferred.nested.ready,contextual.count,JSON.stringify({x:1}));
        }
    "#;
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().arg("run").arg(directory.source(&support::format_round_trip(source))),300);
    support::success(&output);assert_eq!(output.stdout,b"1 true 3 {\"x\":1}\n");
    for source in [
      "function echo<T>(value:T):T{return value;}function main(){const values=[1];const result=echo({values:values});console.log(values.length,result.values.length);}",
      "function echo<T>(value:T):T{return value;}function main(){const result=echo({x:1,x:2});}",
    ]{assert!(tsrust_compiler::check(source).is_err(),"accepted {source}");}
}
