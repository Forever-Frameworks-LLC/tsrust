use tsrust_compiler::check;

#[test]
fn unused_generic_defaults_reject_names_outside_initialized_parameter_scope() {
    for (source, name) in [
        (
            "function bad<T>(first:T=second,second:T):T{return first;}function main(){}",
            "second",
        ),
        (
            "function bad<T>(value:T=value):T{return value;}function main(){}",
            "value",
        ),
        (
            "function bad<T>(value:T=local):T{const local=7;return value;}function main(){}",
            "local",
        ),
        (
            "function bad<T>(value:T=missing):T{return value;}function main(){}",
            "missing",
        ),
        (
            "function bad<T>(fallback:T,action:ReadOnly<Function<int>>=():int=>{return missing;}):T{return fallback;}function main(){}",
            "missing",
        ),
        (
            "function bad<T>(fallback:T,action:ReadOnly<Function<int, [first:int,second:int]>>=(first:int=second,second:int=7):int=>{return first;}):T{return fallback;}function main(){}",
            "second",
        ),
        (
            "function bad<T>(fallback:T,action:ReadOnly<Function<int>>=():int=>{const local=missing;return local;}):T{return fallback;}function main(){}",
            "missing",
        ),
        (
            "function bad<T>(fallback:T,action:ReadOnly<Function<int>>=():int=>{if(true){const inner=7;}return inner;}):T{return fallback;}function main(){}",
            "inner",
        ),
    ] {
        let error = match check(source) {
            Ok(_) => panic!("uninitialized or unavailable name accepted: {source}"),
            Err(error) => error,
        };
        assert_eq!(error.code, "name", "{}", error.render("scope.tsr", source));
        assert_eq!(error.message, format!("Unknown value {name:?}."));
        assert_eq!(&source[error.span.start..error.span.end], name);
    }
}

#[test]
fn generic_defaults_keep_earlier_parameters_globals_and_nested_local_scopes() {
    for source in [
        "function relay<T>(fallback:T,value:T=fallback):T{return value;}function main(){console.log(relay<int>(7));}",
        "function source<T>(value:T):T{return value;}function relay<T>(fallback:T,value:T=source<T>(fallback)):T{return value;}function main(){console.log(relay<int>(7));}",
        "function maybe<T>(value:T|undefined=undefined):T|undefined{return value;}function main(){const value=maybe<int>();}",
        "function values<T>(value:T[]=Array.of<T>()):T[]{return value;}function main(){const value=values<int>();}",
        "function local<T>(fallback:T,action:ReadOnly<Function<int>>=():int=>{let total=0;for(const item of [1,2]){total=total+item;}return total;}):T{return fallback;}function main(){}",
        "function local<T>(fallback:T,action:ReadOnly<Function<int, [first:int,second:int]>>=(first:int=1,second:int=first):int=>{const sum=first+second;return sum;}):T{return fallback;}function main(){}",
        "function local<T>(fallback:T,action:ReadOnly<Function<int>>=():int=>{let total=0;for(let index=0;index<3;index=index+1){total=total+index;}return total;}):T{return fallback;}function main(){}",
        "function local<T>(fallback:T,action:ReadOnly<Function<int>>=():int=>{let total=0;while(total<2){total=total+1;}if(total===2){const inside=1;total=total+inside;}return total;}):T{return fallback;}function main(){}",
    ] {
        let checked =
            check(source).unwrap_or_else(|error| panic!("{}", error.render("scope.tsr", source)));
        assert!(!checked.emit_rust().contains("unsafe"));
    }
}

#[test]
fn generic_default_scope_keeps_the_existing_source_nesting_limit() {
    let depth = 100;
    let annotation = format!("T{}", "[]".repeat(depth));
    let initializer = format!("{}fallback{}", "[".repeat(depth), "]".repeat(depth));
    let source = format!(
        "function deep<T>(fallback:T,value:{annotation}={initializer}):{annotation}{{return value;}}function main(){{}}"
    );
    check(&source).unwrap_or_else(|error| panic!("{}", error.render("scope.tsr", &source)));
}
