//! Source/native controls for nullable recursive ownership.

#[test]
fn nullable_recursive_records_construct_clone_and_consume() {
    let source=r#"
type Chain={value:int;next?:Chain;};
function main(){
 let root:Chain={value:1};
 for(let i=2;i<=3;i++){root={value:i,next:root};}
 const copied=clone(root);copied.next!.value=99;
 let total=0;
 while(root.next!==undefined){total+=root.value;root=root.next!;}
 total+=root.value;
 console.log(total);console.log(copied.value);console.log(copied.next!.value);
}"#;
    let source=crate::format(source).unwrap();
    assert_eq!(crate::format(&source).unwrap(),source);
    let checked=crate::check(&source).unwrap();
    crate::scoped_record_places::tests::native_output(&checked.emit_rust(),"6\n3\n99\n");
}

#[test]
fn nullable_recursive_union_operation_matrix() {
    let mut native=String::new();let mut calls=String::new();let mut expected=String::new();
    for (name,source,output) in [
        ("optional_move",r#"type Chain={label:string;next?:Chain;};function forward(input:Chain):Chain|undefined{return input.next;}function tail(input:Chain):string|undefined{return forward(input)?.label;}function main(){const root:Chain={label:"head",next:{label:"tail"}};console.log(tail(root)??"none");}"#,"tail\n"),
        ("mixed_union",r#"type Leaf={text:string;};type Node={value:int;next:Node|Leaf|null;};function main(){const leaf:Leaf={text:"tail"};const root:Node={value:2,next:{value:1,next:leaf}};const saved=clone(root);root.next=null;console.log(root.next===null);console.log(saved.next===null);console.log(saved.value);}"#,"true\nfalse\n2\n"),
        ("generic_link",r#"type Chain<T>={value:T;next?:Chain<T>;};function main(){const root:Chain<int>={value:2,next:{value:7}};const saved=clone(root);console.log(saved.next!.value);const words:Chain<string>={value:"head",next:{value:"tail"}};const copied=clone(words);console.log(copied.next!.value);}"#,"7\ntail\n"),
        ("mutual",r#"type A={value:int;next?:B;};type B={back:A;};function main(){const root:A={value:3,next:{back:{value:2}}};const copied=clone(root);copied.next!.back.value=99;console.log(root.next!.back.value);const child=root.next!;console.log(child.back.value);console.log(copied.next!.back.value);}"#,"2\n2\n99\n"),
        ("coalescing",r#"type Chain={value:int;next?:Chain;};function choose(input:Chain|undefined):Chain{return input??{value:8};}function widen(input:Chain|undefined):Chain|null|undefined{return input;}function select(input:Chain|null|undefined):Chain{return input??{value:9};}function main(){const root:Chain={value:1};root.next??={value:2};console.log(root.next?.value??0);console.log(choose(root.next).value);const next:Chain|undefined={value:3};console.log(select(widen(next)).value);}"#,"2\n2\n3\n"),
        ("arrays",r#"type Chain={value:int;next?:Chain;};function main(){const rows:(Chain|undefined)[]=[{value:2},undefined];rows.push({value:3});const saved=clone(rows);const child=rows.pop()!;console.log(child.value);console.log(saved[0]!.value);}"#,"3\n2\n"),
        ("borrow",r#"type Chain={value:int;next?:Chain;};function inspect(input:ReadOnly<Chain|undefined>):int{return input?.value??0;}function main(){const root:Chain={value:1,next:{value:4}};console.log(inspect(root.next));console.log(inspect(undefined));}"#,"4\n0\n"),
        ("replace",r#"type Chain={value:int;next?:Chain;};function main(){const root:Chain={value:1,next:{value:2}};root.next = {value:5};console.log(root.next!.value);root.next=undefined;root.next={value:6};const snapshot=clone(root.next);console.log(snapshot!.value);}"#,"5\n6\n"),
    ] {
        let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
        let checked=crate::check(&source).unwrap_or_else(|error|panic!("{name}: {}",error.render(name,&source)));
        native.push_str(&format!("mod {name} {{ {} }}\n",checked.emit_rust().replace("fn main()","pub fn run()").replace("#![recursion_limit = \"512\"]", "")));
        calls.push_str(&format!("{name}::run();"));expected.push_str(output);
    }
    native.push_str(&format!("fn main(){{{calls}}}"));
    crate::scoped_record_places::tests::native_output(&native,&expected);
}
