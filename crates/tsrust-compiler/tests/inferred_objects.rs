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
