mod support;

#[test]
fn typed_arrow_missing_parenthesis_explains_and_repairs_the_source() {
    let source="function main(){const users=['Alice'];const values=users.map(user: string): string => {return user.toUpperCase();});console.log(values.join('|'));}";
    let error=tsrust_compiler::check(source).err().unwrap();
    assert_eq!(error.code,"syntax");
    assert!(error.message.contains("opening `(` is missing"));
    assert_eq!(&source[error.span.start..error.span.end],":");
    assert_eq!(error.fixes.len(),1);
    let fix=&error.fixes[0];
    let repaired=format!("{}{}{}",&source[..fix.span.start],fix.replacement,&source[fix.span.end..]);
    support::execute(&repaired,"ALICE\n",None);
    let inference=tsrust_compiler::check("function main(){const action=user=>user.toUpperCase();}").err().unwrap();
    assert!(inference.message.contains("Cannot infer the type"));
    assert!(inference.message.contains("Add a type annotation"));
}

#[test]
fn inferred_arrows_map_strings_and_preserve_expression_bodies() {
    for arrow in ["user => user.toUpperCase()", "(user) => user.toUpperCase()", "(user: string) => user.toUpperCase()", "(user) => { return user.toUpperCase(); }"] {
        support::execute(&format!("function main(){{const users=['Alice','Bob'];const values=users.map({arrow});console.log(values.join('|'));}}"), "ALICE|BOB\n", None);
    }
    support::execute("function main(){const add=(value:int)=>value+1;console.log(add(4));const values=[1,2,3];console.log(values.map((value,index)=>value+index).join('|'));console.log(values.filter(value=>value>1).join('|'));}", "5\n1|3|5\n2|3\n", None);
}

#[test]
fn arrows_preserve_context_capture_object_returns_and_missing_context_errors() {
    support::execute("function apply(action:(input:ReadOnly<string>)=>string):string{return action('Pat');}type Row={value:int;};function main(){console.log(apply(value=>value.toUpperCase()));const make=():Row=>({value:7});console.log(make().value);let offset=2;const add=(value:int)=>value+offset;console.log(add(3));const twice=(value:int)=>(next:int)=>value+next;const next=twice(4);console.log(next(5));}", "PAT\n7\n5\n9\n", None);
    let error=tsrust_compiler::check("function main(){const action=value=>value+1;}").err().unwrap();
    assert_eq!(error.code,"parameter-inference");
}

#[test]
fn contextual_arrows_cover_comparators_reductions_and_format_roundtrips() {
    let source="function main(){const values=[3,1,2];values.sort((left,right)=>left-right);console.log(values.join('|'));console.log(values.reduce((sum,value)=>sum+value,0));values.forEach(value=>console.log(value));}";
    support::execute(source,"1|2|3\n6\n1\n2\n3\n",None);
    let formatted=tsrust_compiler::format(source).unwrap();
    support::execute(&formatted,"1|2|3\n6\n1\n2\n3\n",None);
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
    support::execute("function main(){const actions:(()=>int)[]=[():int=>7,():int=>9];const results=actions.map((action:Mutable<Function<int>>)=>action());console.log(results.join('|'),actions[0]());}","7|9 7\n",None);
}
