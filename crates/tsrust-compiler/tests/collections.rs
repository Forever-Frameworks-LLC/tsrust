mod support;
#[path = "../src/runtime/sets.rs"] mod runtime_sets;
#[path = "../src/runtime/maps.rs"] mod runtime_maps;

#[test]
fn deque_clone_is_deep_and_requires_cloneable_payloads() {
    let directory = support::TempDirectory::new();


    for declaration in ["import {ByteView} from 'std:bytes';type Item=ByteView;", "type Item={read:ReadOnly<Function<int>>;};", "import {ByteView} from 'std:bytes';type Item={next?:Item;data:ByteView;};"] {
        let source=format!("import {{Deque}} from 'std:collections';{declaration}function main(){{const values=new Deque<Item>();const copied=clone(values);}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(), "non-cloneable deque accepted");
        assert!(String::from_utf8_lossy(&output.stderr).contains("clone"), "{}",String::from_utf8_lossy(&output.stderr));
    }
}


#[test]
fn deque_callback_records_preserve_borrowed_captures() {
    let source = r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
type Reader={read:ReadOnly<Function<int>>;};
function main(){const text='borrowed';const values=new Deque<Reader>();values.pushBack({read:()=>text.length});for(const item of values){strictEqual(item.read(),8);}const item=values.popFront()!;strictEqual(item.read(),8);}
"#;
    let directory = support::TempDirectory::new();


    for source in [
        source.replace("for(const item of values)", "const moved=text;for(const item of values)"),
        source.replace("const item=values.popFront()!;strictEqual", "const item=values.popFront()!;const moved=text;strictEqual"),
    ] {
        let path = directory.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "captured owner transferred while callback remained live");
        assert!(String::from_utf8_lossy(&output.stderr).contains("capture-conflict"), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn returned_deque_callback_records_keep_parameter_owners() {
    let helpers = include_str!("fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
    let source = format!(r#"import {{Deque}} from 'std:collections';{helpers}
type Reader={{read:Once<Function<ByteView>>;}};
function collect(owner:ReadOnly<Bytes>):Deque<Reader>{{const part=window(owner);const values=new Deque<Reader>();values.pushBack({{read:()=>part}});return values;}}
function main(){{const owner=bytes('ab');const values=collect(owner);const item=values.popFront()!;strictEqual(length(item.read()),2);}}
"#);
    let directory = support::TempDirectory::new();


    let invalid = source.replace("const values=collect(owner);const item", "const values=collect(owner);const moved=owner;const item");
    let path = directory.source(&support::format_round_trip(&invalid));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success(), "returned callback deque lost its byte owner");
    assert!(String::from_utf8_lossy(&output.stderr).contains("borrow"), "{}", String::from_utf8_lossy(&output.stderr));
    let invalid = source.replace("Once<Function<ByteView>>", "ReadOnly<Function<int>>").replace("read:()=>part", "read:()=>length(part)").replace("length(item.read())", "item.read()");
    let path = directory.source(&support::format_round_trip(&invalid));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("callback-escape"), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn deque_iteration_borrows_in_front_to_back_order() {
    let source = r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
function main(){const values=new Deque<string>();values.pushBack('b');values.pushFront('a');values.pushBack('c');let joined='';for(const value of values){joined+=value;}strictEqual(joined,'abc');strictEqual(values.size,3);values.clear();for(const value of values){throw 'empty queue visited';}}
"#;
    let directory = support::TempDirectory::new();


    let source = source.replace("joined+=value;", "values.clear();joined+=value;");
    let path = directory.source(&support::format_round_trip(&source));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("iteration-conflict"));
}

#[test]
fn deque_view_transport_keeps_owners_and_rejects_mutation_conflicts() {
    let helpers = r#"import {Deque} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length,set} from 'std:bytes';import {strictEqual} from 'std:testing';
function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
function transfer(input:ByteView):ByteView{const values=new Deque<ByteView>();values.pushBack(input);return values.popFront()!;}
"#;
    let directory = support::TempDirectory::new();
    for body in [
        "const owner=bytes();const values=new Deque<ByteView>();values.pushBack(window(owner));set(owner,0,9);console.log(values.size);",
        "const owner=bytes();const values=new Deque<ByteView>();values.pushBack(window(owner));const first=values.front();values.clear();if(first!==undefined){console.log(length(first));}",
        "const values=new Deque<string>();const text='owned';values.pushBack(text);console.log(text);",
    ] {
        let source = format!("{helpers}function main(){{{body}}}");
        let path = directory.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "invalid ownership accepted: {body}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("borrow") || error.contains("native-view") || error.contains("error[transferred]"), "{error}");
    }
}

#[test]
fn deque_methods_reject_invalid_access_and_arguments() {
    for (body, code) in [
        ("function f(values:ReadOnly<Deque<int>>){values.clear();}", "parameter-access"),
        ("function f(){const values=new Deque<int>();values.size=2;}", "readonly"),
        ("function f(){const values=new Deque<int>();values.pushBack();}", "arity"),
        ("function f(){const values=new Deque<int>();values.popFront(1);}", "arity"),
        ("function f(){const values=new Deque<int>();values.at('x');}", "type"),
        ("function f(){const values=new Deque<int>();values.pushFront('x');}", "type"),
    ] {
        let source = format!("import {{Deque}} from 'std:collections';{body}function main(){{}}");
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success() && error.contains(code), "{body}: {error}");
    }
}

#[test]
fn deque_constructor_rejects_arguments_and_unregistered_type_identities() {
    for (source, code) in [
        ("import {Deque} from 'std:collections';function main(){const values=new Deque<int>(1);}", "arity"),
        ("type Deque<T>={};function main(){const values=new Deque<int>();}", "native-constructor"),
        ("import {Bytes} from 'std:bytes';function main(){const values=new Bytes();}", "native-constructor"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(code), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn collection_storage_keeps_view_owners_and_iteration_loans_live() {
    let fixture = include_str!("fixtures/collections/borrowed-values.tsr");
    let helpers = fixture.split("function main()").next().unwrap();
    for (body, diagnostic) in [
        ("const owner=bytes(\"x\");const values=new Map<int,ByteView>();values.set(1,window(owner));const copied=clone(values);", "clone"),
        ("const owner=bytes(\"x\");const values=new Map<int,ByteView>();values.set(1,window(owner));const moved=owner;for(const [key,value] of values){console.log(text(value));}", "borrow"),
        ("const values=new Map<int,int>();values.set(1,2);for(const entry of values){values.clear();}", "borrow"),
        ("const values=new Set<int>([1]);for(const value of values){values.delete(value);}", "borrow"),
    ] {
        let directory = support::TempDirectory::new();
        let source = support::format_round_trip(&format!("{helpers}function main(){{{body}}}"));
        let path = directory.source(&source);
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "unsafe collection accepted: {body}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.to_lowercase().contains(diagnostic), "{stderr}");
    }
}

#[test]
fn map_view_transport_rejects_short_owner_escape_and_mutated_parameter_conflicts() {
    let fixture = include_str!("fixtures/collections/returned-views.tsr");
    let helpers = fixture.split("function main()").next().unwrap();
    for source in [
        format!("{helpers}function leak():Map<int,ByteView>{{const owner=bytes('short');return collect(owner);}}function main(){{}}"),
        format!("{helpers}function main(){{const owner=bytes('short');const values=new Map<int,ByteView>();fill(values,owner);const moved=owner;console.log(inspect(values));}}"),
        format!("{helpers}function leak():Map<int,ByteView>{{const owner=bytes('short');const saved=save(collect(owner));return saved();}}function main(){{}}"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path), 30);
        assert!(!output.status.success(), "short owner escaped: {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("native-view"), "{stderr}");
    }
}

#[test]
fn projected_views_and_recursive_map_children_cannot_escape_short_owners() {
    let fixture = include_str!("fixtures/collections/projected-views.tsr");
    let helpers = fixture.split("function main()").next().unwrap();
    let source = format!("{helpers}function leak():ByteView{{const owner=bytes('short');return expose(owner);}}function main(){{}}");
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(&source));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("native-view-escape"));
    let fixture = include_str!("fixtures/collections/recursive-map-views.tsr");
    let helpers = fixture.split("function main()").next().unwrap();
    let source = format!("{helpers}function leak(owner:ReadOnly<Bytes>):Chunk{{const short=bytes('short');const root=leaf(window(owner));root.children.set(1,leaf(window(short)));return root;}}function main(){{}}");
    let path = directory.source(&support::format_round_trip(&source));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("native-view-escape"));
}

#[test]
fn readonly_reborrow_of_local_exclusive_deque_view_cannot_escape_its_storage() {
    let source = r#"import {Deque} from 'std:collections';import {Bytes,ByteView,ByteViewMut,view,viewMut} from 'std:bytes';
function leak(owner:Mutable<Bytes>):ByteView {
 const values=new Deque<ByteViewMut>();
 match(viewMut(owner,0,1)){Ok(part)=>{values.pushBack(part);},Err(error)=>{throw error.message;}}
 const selected=values.front();
 if(selected!==undefined){match(view(selected,0,1)){Ok(part)=>{return part;},Err(error)=>{throw error.message;}}}
 throw "missing";
}function main(){}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success(), "local deque reborrow escaped");
    assert!(String::from_utf8_lossy(&output.stderr).contains("native-view-escape"), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn readonly_reborrow_of_local_exclusive_map_view_cannot_escape_its_storage() {
    let source = r#"import {Bytes,ByteView,ByteViewMut,view,viewMut} from 'std:bytes';
function leak(owner:Mutable<Bytes>):ByteView {
 const values=new Map<int,ByteViewMut>();
 match(viewMut(owner,0,1)){Ok(part)=>{values.set(1,part);},Err(error)=>{throw error.message;}}
 const item=values.get(1);
 if(item!==undefined){match(view(item,0,1)){Ok(part)=>{return part;},Err(error)=>{throw error.message;}}}
 throw 'missing';
}
function main() {}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("native-view-escape"));
}

#[test]
fn deque_backend_matches_sequence_model_and_preserves_owned_payloads() {
    let directory = support::TempDirectory::new();
    let source = format!("{}\n{}", include_str!("../src/stdlib/collections/deque_runtime.rs"), include_str!("fixtures/collections/deque-backend.rs"));
    let path = directory.0.join("deque.rs");
    std::fs::write(&path, source).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("deque-{level}"));
        support::success(&support::run(support::rustc().arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 30));
    }
}

#[test]
fn deque_backend_rejects_borrow_invalidation_view_clone_and_owner_escape() {
    for (body, code) in [
        ("fn main(){let mut values=TsrDeque::new();values.push_back(1);let first=values.front().unwrap();values.pop_front();println!(\"{}\",first);}", "E0502"),
        ("fn main(){let mut values=TsrDeque::new();values.push_back(1);let mut iterator=values.iter();values.push_front(2);println!(\"{:?}\",iterator.next());}", "E0502"),
        ("fn main(){let mut values=TsrDeque::new();values.push_back(1);let first=values.front().unwrap();let moved=values;println!(\"{}\",first);drop(moved);}", "E0505"),
        ("struct Window<'a>(&'a [u8]);fn main(){let owner=[1];let mut values=TsrDeque::new();values.push_back(Window(&owner));let copied=values.clone();}", "E0599"),
        ("struct Window<'a>(&'a [u8]);fn leak()->TsrDeque<Window<'static>>{let owner=vec![1];let mut values=TsrDeque::new();values.push_back(Window(&owner));values}fn main(){}", "E0515"),
    ] {
        let directory = support::TempDirectory::new();
        let source = format!("{}\n{body}", include_str!("../src/stdlib/collections/deque_runtime.rs"));
        let path = directory.0.join("reject.rs");
        std::fs::write(&path, source).unwrap();
        let output = support::run(support::rustc().arg("--edition=2024").arg("--emit=metadata").arg(&path).arg("-o").arg(directory.0.join("reject.rmeta")), 30);
        assert!(!output.status.success(), "unsafe deque accepted: {body}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(code), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn ordered_backends_preserve_order_ownership_and_number_equivalence() {
    let directory=support::TempDirectory::new();
    let source=format!("{}\n{}",include_str!("../src/stdlib/collections/ordered_runtime.rs"),include_str!("fixtures/collections/ordered-backend.rs"));
    let path=directory.0.join("ordered.rs");std::fs::write(&path,source).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("ordered-{level}"));
        support::success(&support::run(support::rustc().arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),30));
    }
}

#[test]
fn ordered_backends_reject_invalidation_noncloneable_values_and_escape() {
    for (body,code) in [
        ("fn main(){let mut map=TsrOrderedMap::new();map.insert(1,2);let read=map.get(&1).unwrap();map.remove(&1);println!(\"{}\",read);}","E0502"),
        ("fn main(){let mut map=TsrOrderedMap::new();map.insert(1,2);let mut read=map.iter();map.clear();println!(\"{:?}\",read.next());}","E0502"),
        ("fn main(){let mut set=TsrOrderedSet::new();set.insert(1);let mut read=set.iter();set.insert(2);println!(\"{:?}\",read.next());}","E0502"),
        ("fn main(){let mut heap=TsrHeap::new();heap.push(1);let read=heap.peek().unwrap();heap.pop();println!(\"{}\",read);}","E0502"),
        ("struct Window<'a>(&'a [u8]);fn main(){let owner=[1];let mut map=TsrOrderedMap::new();map.insert(1,Window(&owner));let copied=map.clone();}","E0599"),
        ("struct Window<'a>(&'a [u8]);fn leak()->TsrOrderedMap<i64,Window<'static>>{let owner=vec![1];let mut map=TsrOrderedMap::new();map.insert(1,Window(&owner));map}fn main(){}","E0515"),
    ] {
        let directory=support::TempDirectory::new();
        let path=directory.0.join("reject.rs");
        std::fs::write(&path,format!("{}\n{body}",include_str!("../src/stdlib/collections/ordered_runtime.rs"))).unwrap();
        let output=support::run(support::rustc().arg("--edition=2024").arg("--emit=metadata").arg(&path).arg("-o").arg(directory.0.join("reject.rmeta")),30);
        assert!(!output.status.success(),"invalid ordered collection accepted: {body}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn heap_number_enum_order_and_invalid_access_are_checked() {
    let directory=support::TempDirectory::new();

    for body in [
        "const values=new Heap<int>(1);",
        "const values=new Heap<int>();values.push('bad');",
        "const values=new Heap<int>();values.size=3;",
        "const values=new Heap<int>();for(const value of values){values.push(1);}",
        "const values=new Heap<string>();values.push('a');const first=values.peek();values.clear();if(first!==undefined){console.log(first);}",
        "const values=new Heap<string>();const text='a';values.push(text);console.log(text);",
        "const values=new Heap<int[]>();",
    ] {
        let source=format!("import {{Heap}} from 'std:collections';function main(){{{body}}}");let path=directory.source(&support::format_round_trip(&source));
        assert!(!support::run(support::cli().arg("check").arg(path),30).status.success(),"invalid heap accepted: {body}");
    }
}

#[test]
fn heap_iteration_borrows_every_value_and_preserves_completion() {
    let source=r#"import {Heap} from 'std:collections';import {strictEqual} from 'std:testing';
function first(values:ReadOnly<Heap<int>>):int{for(const value of values){try{return value;}finally{strictEqual(values.size,3);}}return -1;}
function main(){const values=new Heap<int>();values.push(2);values.push(9);values.push(4);let count=0;let sum=0;for(const value of values){sum+=value;count++;}strictEqual(count,3);strictEqual(sum,15);strictEqual(values.size,3);strictEqual(first(values)>0,true);const words=new Heap<string>();words.push('abc');words.push('xy');let length=0;for(const word of words){length+=word.length;}strictEqual(length,5);strictEqual(words.size,2);words.clear();for(const word of words){throw 'empty heap visited';}}
"#;
    let directory=support::TempDirectory::new();

    for replacement in ["values.clear();sum+=value;","const moved=values;sum+=value;"] {
        let source=source.replace("sum+=value;",replacement);let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("iteration-conflict"),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn native_collection_spread_rejects_moves_and_heap_mutation() {
    let directory = support::TempDirectory::new();
    for source in [
        "import {Deque} from 'std:collections';function take(...words:string[]){}function main(){const values=new Deque<string>();values.pushBack('x');take(...values);console.log(values.size);}",
        "import {Heap} from 'std:collections';function take(...words:string[]){}function main(){const values=new Heap<string>();values.push('x');take(...values);console.log(values.size);}",
        "import {Heap} from 'std:collections';function change(value:Mutable<string>){}function main(){const values=new Heap<string>();values.push('x');change(...values);}",
        "import {Deque} from 'std:collections';function read(value:ReadOnly<string>,extra:int){}function change(values:Mutable<Deque<string>>):int{values.clear();return 0;}function main(){const values=new Deque<string>();values.pushBack('x');read(...values,change(values));}",
    ] {
        let path=directory.source(&support::format_round_trip(source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"invalid collection spread accepted: {source}");
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("transferred") || error.contains("collection-key") || error.contains("conflicting-access"),"{error}");
    }
}

#[test]
fn native_collection_spread_retains_view_owners() {
    let helpers=include_str!("fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
    let source=format!(r#"import {{Deque}} from 'std:collections';{helpers}
function read(value:ReadOnly<ByteView>):int{{return length(value);}}
function take(...values:ByteView[]):ByteView[]{{return values;}}
function identity(value:ByteView):ByteView{{return value;}}
function main(){{const owner=bytes('ab');const values=new Deque<ByteView>();values.pushBack(window(owner));strictEqual(read(...values),2);const parts=[...values];strictEqual(length(parts[0]),2);}}
"#);
    for source in [source.clone(),source.replace("const parts=[...values];","const parts=take(...values);"),source.replace("const parts=[...values];","const parts=[identity(...values)];")] {
    let directory=support::TempDirectory::new();


    let invalid=source.replace("strictEqual(length(parts[0]),2);","const moved=owner;strictEqual(length(parts[0]),2);");
    let path=directory.source(&support::format_round_trip(&invalid));
    let output=support::run(support::cli().arg("check").arg(path),30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("borrow"),"{}",String::from_utf8_lossy(&output.stderr));
}
}

#[test]
fn deque_borrowed_nested_indexing_reads_without_transfers() {
 let source=r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
function consume(value:string):string{return value;}
function change(values:Mutable<Deque<int[]>>):int{values.clear();return 0;}
function main(){const values=new Deque<int[]>();values.pushBack([4,5]);strictEqual(values.front()![0],4);strictEqual(values.back()![1],5);strictEqual(values.at(0)![0],4);strictEqual(values.size,1);const words=new Deque<string[]>();words.pushBack(['word']);strictEqual(words.front()![0].length,4);strictEqual(words.size,1);}
"#;
 let directory=support::TempDirectory::new();
 for invalid in [source.replace("strictEqual(values.front()![0],4);","values.front()![0]=9;"),source.replace("strictEqual(words.front()![0].length,4);","consume(words.front()![0]);"),source.replace("strictEqual(values.front()![0],4);","strictEqual(values.front()![change(values)],4);"),source.replace("strictEqual(words.front()![0].length,4);","const borrowed=words.front()![0];words.clear();console.log(borrowed);")] {
  let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"borrowed index accepted: {invalid}");
 }
}

#[test]
fn ordered_set_number_enum_and_ownership_boundaries() {
 let directory=support::TempDirectory::new();
 for body in ["const values=new OrderedSet<int[]>();","const values=new OrderedSet<int>();values.add('x');","const values=new OrderedSet<int>();values.size=3;","const values=new OrderedSet<int>();values.add(1);for(const item of values){values.clear();}","const values=new OrderedSet<string>();const word='x'.repeat(2);values.add(word);console.log(word);","const values=new OrderedSet<int>();values.has();","function read(values:ReadOnly<OrderedSet<int>>){values.add(1);}","function change(value:Mutable<string>){}const values=new OrderedSet<string>();values.add('x');change(...values);"] {
  let source=format!("import {{OrderedSet}} from 'std:collections';function main(){{{body}}}");let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"invalid ordered set accepted: {source}");
 }
}

#[test]
fn ordered_map_queries_borrow_noncloneable_keys_and_preserve_values() {
    let directory=support::TempDirectory::new();
    let source=format!("{}\n{}\n{}",include_str!("../src/stdlib/collections/ordered_runtime.rs"),include_str!("../src/stdlib/collections/ordered_key_runtime.rs"),include_str!("fixtures/collections/ordered-map-queries.rs"));
    let path=directory.0.join("ordered-queries.rs");std::fs::write(&path,source).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("queries-{level}"));support::success(&support::run(support::rustc().arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&binary),60));support::success(&support::run(&mut std::process::Command::new(binary),30));
    }
}

#[test]
fn ordered_map_source_queries_reject_borrow_invalidation_and_escape() {
    for (body,code) in [
        ("fn main(){let mut map=TsrOrderedMap::new();map.insert(TsrOrdered::new(1_i64),2);let read=map.get_copy_key(1).unwrap();map.remove_copy_key(1);println!(\"{}\",read);}","E0502"),
        ("fn main(){let mut map=TsrOrderedMap::new();map.insert(TsrOrdered::new(String::from(\"a\")),2);let query=String::from(\"a\");let read=map.get_string_key(&query).unwrap();map.remove_string_key(&query);println!(\"{}\",read);}","E0502"),
        ("fn main(){let mut map=TsrOrderedMap::new();map.insert(TsrOrdered::new(1_i64),2);let mut read=map.source_iter();map.clear();println!(\"{:?}\",read.next());}","E0502"),
        ("fn leak()->&'static i64{let mut map=TsrOrderedMap::new();map.insert(TsrOrdered::new(1_i64),2);map.get_copy_key(1).unwrap()}fn main(){}","E0515"),
    ] {
        let directory=support::TempDirectory::new();let path=directory.0.join("reject.rs");
        std::fs::write(&path,format!("{}\n{}\n{body}",include_str!("../src/stdlib/collections/ordered_runtime.rs"),include_str!("../src/stdlib/collections/ordered_key_runtime.rs"))).unwrap();
        let output=support::run(support::rustc().arg("--edition=2024").arg("--emit=metadata").arg(&path).arg("-o").arg(directory.0.join("reject.rmeta")),30);
        assert!(!output.status.success(),"invalid ordered map query accepted: {body}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn ordered_map_public_constructor_checks_keys_ownership_and_family() {
    let directory=support::TempDirectory::new();
    for body in [
        "const values=new OrderedMap<int[],int>();",
        "const values=new OrderedMap<int|string,int>();",
        "const values=new OrderedMap<int>();",
        "const values=new OrderedMap<int,int>(1);",
        "const values=new OrderedMap<int,int>();values.set('bad',1);",
        "const values=new OrderedMap<int,int>();values.size=2;",
        "const values=new OrderedMap<string,int>();const key='a';values.set(key,1);console.log(key);",
        "const values=new OrderedMap<int,string>();values.set(1,'a');const read=values.get(1);values.clear();console.log(read);",
        "const values=new OrderedMap<int,string>();values.set(1,'a');for(const entry of values){values.delete(1);console.log(entry[1]);}",
        "const values=new OrderedMap<int,int>();const ordinary:Map<int,int>=values;",
        "const ordinary=new Map<int,int>();const values:OrderedMap<int,int>=ordinary;",
    ] {
        let source=format!("import {{OrderedMap}} from 'std:collections';function main(){{{body}}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"invalid ordered map accepted: {body}");
    }
}

#[test]
fn ordered_map_array_from_preserves_view_payloads_and_copy_boundaries() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';
function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
type Row={data:ByteView;};
function main(){const owner=bytes();const values=new OrderedMap<int,Row>();values.set(2,{data:window(owner)});values.set(1,{data:window(owner)});const lengths=Array.from(values,(entry:ReadOnly<[int,Row]>,index:int):int=>length(entry[1].data)+entry[0]+index);strictEqual(lengths[0],3);strictEqual(lengths[1],5);strictEqual(values.size,2);const plain=new OrderedMap<string,string>();plain.set('z','last');plain.set('a','first');const copied=Array.from(plain);plain.clear();strictEqual(copied[0][0],'a');strictEqual(copied[1][1],'last');}
"#;
 let directory=support::TempDirectory::new();
 for invalid in [source.replace("length(entry[1].data)+entry[0]+index","{values.clear();return 0;}"),source.replace("const lengths=Array.from(values,","const forbidden=Array.from(values);const lengths=Array.from(values,")] {
  let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"unsafe mapping accepted");
 }
}

#[test]
fn ordered_map_owned_spread_retains_noncloneable_payloads() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function total(...entries:[int,ByteView][]):int{let sum=0;for(const entry of entries){sum+=entry[0]+length(entry[1]);}return sum;}function main(){const owner=bytes();const values=new OrderedMap<int,ByteView>();values.set(2,window(owner));values.set(1,window(owner));const entries=[...values];strictEqual(entries[0][0],1);strictEqual(length(entries[1][1]),2);const more=new OrderedMap<int,ByteView>();more.set(3,window(owner));strictEqual(total(...more),5);}"#;
 let directory=support::TempDirectory::new();
 for invalid in [source.replace("const entries=[...values];","const entries=[...values];console.log(values.size);"),source.replace("strictEqual(length(entries[1][1]),2);","const moved=owner;strictEqual(length(entries[1][1]),2);")] {
  let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"invalid owning spread accepted");
 }
}

#[test]
fn ordered_map_spread_returns_original_view_projection() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function project(entry:ReadOnly<[int,ByteView]>):ByteView{match(view(entry[1],0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function select(owner:ReadOnly<Bytes>):ByteView{const values=new OrderedMap<int,ByteView>();values.set(3,window(owner));return project(...values);}function main(){const owner=bytes();const result=select(owner);strictEqual(length(result),2);}"#;
 let directory=support::TempDirectory::new();
 let invalid=source.replace("strictEqual(length(result),2);","const moved=owner;strictEqual(length(result),2);");let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"escaped projection lost owner");
}

#[test]
fn borrowed_collection_queries_reject_owning_conditional_branches() {
 let directory=support::TempDirectory::new();
 for query in ["values.front()","values.back()","values.at(0)","mapped.get(1)","ordered.get(1)"] {
  for selection in [format!("true?{query}:'fresh'.repeat(1)"),format!("{query}||'fresh'.repeat(1)"),format!("{query}!")] {
  let source=format!("import {{Deque,OrderedMap}} from 'std:collections';function main(){{const values=new Deque<string>();values.pushBack('x');const mapped=new Map<int,string>();mapped.set(1,'x');const ordered=new OrderedMap<int,string>();ordered.set(1,'x');const result={selection};console.log(String(result));}}");
  let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"mixed borrowed and owning query accepted: {query}");assert!(String::from_utf8_lossy(&output.stderr).contains("borrowed-view"),"{}",String::from_utf8_lossy(&output.stderr));
 }
 }
}

#[test]
fn hash_buckets_avoid_single_key_allocations_and_preserve_collisions() {
    let directory = support::TempDirectory::new();
    let sets = runtime_sets::SET_RUNTIME;
    let inline_maps = format!("{}\n{}", runtime_maps::MAP_INLINE_FIRST, runtime_maps::MAP_INLINE_RUNTIME);
    for (kind, maps) in [runtime_maps::MAP_RUNTIME, inline_maps.as_str()].into_iter().enumerate() {
        let source = format!("#![allow(dead_code)]\ntype String=std::borrow::Cow<'static,str>;\n#[derive(Clone)] struct TsrNull;\n#[derive(Clone)] struct TsrUndefined;\n{sets}\n{maps}\n{}", include_str!("fixtures/collections/hash-buckets-backend.rs"));
        let path = directory.0.join("buckets.rs");
        std::fs::write(&path, source).unwrap();
        for level in ["0", "3"] {
            let binary = directory.0.join(format!("buckets-{kind}-{level}"));
            support::success(&support::run(support::rustc().arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&binary),60));
            support::success(&support::run(&mut std::process::Command::new(binary),30));
        }
    }
}


#[test]
fn registered_collection_runtime_contracts_execute(){
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let fixtures=root.join("crates/tsrust-compiler/tests/fixtures/collections/registered");
    for (name,source) in [
        ("deque-clone-is-deep-and-requires-cloneable-payloads.tsr",include_str!("fixtures/collections/registered/deque-clone-is-deep-and-requires-cloneable-payloads.tsr")),
        ("deque-callback-records-preserve-borrowed-captures.tsr",include_str!("fixtures/collections/registered/deque-callback-records-preserve-borrowed-captures.tsr")),
        ("deque-callback-records-join-distinct-environments.tsr",include_str!("fixtures/collections/registered/deque-callback-records-join-distinct-environments.tsr")),
        ("deque-callback-records-retain-native-view-captures.tsr",include_str!("fixtures/collections/registered/deque-callback-records-retain-native-view-captures.tsr")),
        ("returned-deque-callback-records-keep-parameter-owners.tsr",include_str!("fixtures/collections/registered/returned-deque-callback-records-keep-parameter-owners.tsr")),
        ("deque-iteration-borrows-in-front-to-back-order.tsr",include_str!("fixtures/collections/registered/deque-iteration-borrows-in-front-to-back-order.tsr")),
        ("deque-iteration-retains-views-and-handles-early-completion.tsr",include_str!("fixtures/collections/registered/deque-iteration-retains-views-and-handles-early-completion.tsr")),
        ("deque-end-operations-move-values-and-borrow-reads.tsr",include_str!("fixtures/collections/registered/deque-end-operations-move-values-and-borrow-reads.tsr")),
        ("deque-view-transport-keeps-owners-and-rejects-mutation-conflicts.tsr",include_str!("fixtures/collections/registered/deque-view-transport-keeps-owners-and-rejects-mutation-conflicts.tsr")),
        ("deque-construction-uses-import-identity-and-owned-native-storage-1.tsr",include_str!("fixtures/collections/registered/deque-construction-uses-import-identity-and-owned-native-storage-1.tsr")),
        ("deque-construction-uses-import-identity-and-owned-native-storage-2.tsr",include_str!("fixtures/collections/registered/deque-construction-uses-import-identity-and-owned-native-storage-2.tsr")),
        ("empty-deque-of-views-returns-without-inventing-an-owner.tsr",include_str!("fixtures/collections/registered/empty-deque-of-views-returns-without-inventing-an-owner.tsr")),
        ("maps-hold-borrowed-native-views-without-copying-and-clones-are-independent.tsr",include_str!("fixtures/collections/registered/maps-hold-borrowed-native-views-without-copying-and-clones-are-independent.tsr")),
        ("returned-map-views-keep-selected-owner-through-parameters-and-capture.tsr",include_str!("fixtures/collections/registered/returned-map-views-keep-selected-owner-through-parameters-and-capture.tsr")),
        ("projected-and-recursive-map-views-execute-with-real-owner-lifetimes-1.tsr",include_str!("fixtures/collections/registered/projected-and-recursive-map-views-execute-with-real-owner-lifetimes-1.tsr")),
        ("projected-and-recursive-map-views-execute-with-real-owner-lifetimes-2.tsr",include_str!("fixtures/collections/registered/projected-and-recursive-map-views-execute-with-real-owner-lifetimes-2.tsr")),
        ("heap-orders-scalars-and-transfers-owned-values.tsr",include_str!("fixtures/collections/registered/heap-orders-scalars-and-transfers-owned-values.tsr")),
        ("heap-number-enum-order-and-invalid-access-are-checked.tsr",include_str!("fixtures/collections/registered/heap-number-enum-order-and-invalid-access-are-checked.tsr")),
        ("heap-iteration-borrows-every-value-and-preserves-completion.tsr",include_str!("fixtures/collections/registered/heap-iteration-borrows-every-value-and-preserves-completion.tsr")),
        ("native-collection-spread-preserves-copy-borrow-and-move.tsr",include_str!("fixtures/collections/registered/native-collection-spread-preserves-copy-borrow-and-move.tsr")),
        ("native-collection-spread-retains-view-owners-1.tsr",include_str!("fixtures/collections/registered/native-collection-spread-retains-view-owners-1.tsr")),
        ("native-collection-spread-retains-view-owners-2.tsr",include_str!("fixtures/collections/registered/native-collection-spread-retains-view-owners-2.tsr")),
        ("native-collection-spread-retains-view-owners-3.tsr",include_str!("fixtures/collections/registered/native-collection-spread-retains-view-owners-3.tsr")),
        ("deque-borrowed-array-reads-support-non-null-projection.tsr",include_str!("fixtures/collections/registered/deque-borrowed-array-reads-support-non-null-projection.tsr")),
        ("deque-borrowed-nested-indexing-reads-without-transfers.tsr",include_str!("fixtures/collections/registered/deque-borrowed-nested-indexing-reads-without-transfers.tsr")),
        ("ordered-set-public-surface-preserves-order-and-borrowed-queries.tsr",include_str!("fixtures/collections/registered/ordered-set-public-surface-preserves-order-and-borrowed-queries.tsr")),
        ("ordered-set-number-enum-and-ownership-boundaries.tsr",include_str!("fixtures/collections/registered/ordered-set-number-enum-and-ownership-boundaries.tsr")),
        ("ordered-map-public-constructor-preserves-sorted-owned-storage.tsr",include_str!("fixtures/collections/registered/ordered-map-public-constructor-preserves-sorted-owned-storage.tsr")),
        ("map-set-copied-key-can-feed-the-value-expression.tsr",include_str!("fixtures/collections/registered/map-set-copied-key-can-feed-the-value-expression.tsr")),
        ("ordered-map-converts-to-independent-ordinary-map.tsr",include_str!("fixtures/collections/registered/ordered-map-converts-to-independent-ordinary-map.tsr")),
        ("ordered-map-array-from-borrows-sorted-entries.tsr",include_str!("fixtures/collections/registered/ordered-map-array-from-borrows-sorted-entries.tsr")),
        ("ordered-map-array-from-preserves-view-payloads-and-copy-boundaries.tsr",include_str!("fixtures/collections/registered/ordered-map-array-from-preserves-view-payloads-and-copy-boundaries.tsr")),
        ("ordered-map-owned-spread-transfers-sorted-entries.tsr",include_str!("fixtures/collections/registered/ordered-map-owned-spread-transfers-sorted-entries.tsr")),
        ("ordered-map-owned-spread-retains-noncloneable-payloads.tsr",include_str!("fixtures/collections/registered/ordered-map-owned-spread-retains-noncloneable-payloads.tsr")),
        ("ordered-map-readonly-spread-borrows-mixed-entry-sources.tsr",include_str!("fixtures/collections/registered/ordered-map-readonly-spread-borrows-mixed-entry-sources.tsr")),
        ("ordered-map-readonly-spread-keeps-view-owners.tsr",include_str!("fixtures/collections/registered/ordered-map-readonly-spread-keeps-view-owners.tsr")),
        ("ordered-map-spread-combines-readonly-and-owning-parameters.tsr",include_str!("fixtures/collections/registered/ordered-map-spread-combines-readonly-and-owning-parameters.tsr")),
        ("ordered-map-spread-handles-optional-readonly-entries.tsr",include_str!("fixtures/collections/registered/ordered-map-spread-handles-optional-readonly-entries.tsr")),
        ("ordered-map-spread-returns-original-view-projection-1.tsr",include_str!("fixtures/collections/registered/ordered-map-spread-returns-original-view-projection-1.tsr")),
        ("ordered-map-spread-returns-original-view-projection-2.tsr",include_str!("fixtures/collections/registered/ordered-map-spread-returns-original-view-projection-2.tsr")),
        ("ordered-map-spread-mixes-nullable-entry-sources.tsr",include_str!("fixtures/collections/registered/ordered-map-spread-mixes-nullable-entry-sources.tsr")),
        ("ordered-map-parameters-and-branches-preserve-family-and-order.tsr",include_str!("fixtures/collections/registered/ordered-map-parameters-and-branches-preserve-family-and-order.tsr")),
        ("collections.test.tsr",include_str!("fixtures/collections/registered/collections.test.tsr")),
    ]{
        assert_eq!(std::fs::read_to_string(fixtures.join(name)).unwrap(),source,"checked-in fixture changed after host compilation: {name}");
        support::format_round_trip(source);
    }
    let output=support::run(support::cli().arg("test").arg(fixtures.join("collections.test.tsr")).args(["--reporter=json","--profile"]).current_dir(&root),180);
    support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));print!("{}",String::from_utf8_lossy(&output.stdout));
    let rows=std::str::from_utf8(&output.stdout).unwrap().lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
    let expected=[
        "deque-clone-is-deep-and-requires-cloneable-payloads",
        "deque-callback-records-preserve-borrowed-captures",
        "deque-callback-records-join-distinct-environments",
        "deque-callback-records-retain-native-view-captures",
        "returned-deque-callback-records-keep-parameter-owners",
        "deque-iteration-borrows-in-front-to-back-order",
        "deque-iteration-retains-views-and-handles-early-completion",
        "deque-end-operations-move-values-and-borrow-reads",
        "deque-view-transport-keeps-owners-and-rejects-mutation-conflicts",
        "deque-construction-uses-import-identity-and-owned-native-storage-1",
        "deque-construction-uses-import-identity-and-owned-native-storage-2",
        "empty-deque-of-views-returns-without-inventing-an-owner",
        "maps-hold-borrowed-native-views-without-copying-and-clones-are-independent",
        "returned-map-views-keep-selected-owner-through-parameters-and-capture",
        "projected-and-recursive-map-views-execute-with-real-owner-lifetimes-1",
        "projected-and-recursive-map-views-execute-with-real-owner-lifetimes-2",
        "heap-orders-scalars-and-transfers-owned-values",
        "heap-number-enum-order-and-invalid-access-are-checked",
        "heap-iteration-borrows-every-value-and-preserves-completion",
        "native-collection-spread-preserves-copy-borrow-and-move",
        "native-collection-spread-retains-view-owners-1",
        "native-collection-spread-retains-view-owners-2",
        "native-collection-spread-retains-view-owners-3",
        "deque-borrowed-array-reads-support-non-null-projection",
        "deque-borrowed-nested-indexing-reads-without-transfers",
        "ordered-set-public-surface-preserves-order-and-borrowed-queries",
        "ordered-set-number-enum-and-ownership-boundaries",
        "ordered-map-public-constructor-preserves-sorted-owned-storage",
        "map-set-copied-key-can-feed-the-value-expression",
        "ordered-map-converts-to-independent-ordinary-map",
        "ordered-map-array-from-borrows-sorted-entries",
        "ordered-map-array-from-preserves-view-payloads-and-copy-boundaries",
        "ordered-map-owned-spread-transfers-sorted-entries",
        "ordered-map-owned-spread-retains-noncloneable-payloads",
        "ordered-map-readonly-spread-borrows-mixed-entry-sources",
        "ordered-map-readonly-spread-keeps-view-owners",
        "ordered-map-spread-combines-readonly-and-owning-parameters",
        "ordered-map-spread-handles-optional-readonly-entries",
        "ordered-map-spread-returns-original-view-projection-1",
        "ordered-map-spread-returns-original-view-projection-2",
        "ordered-map-spread-mixes-nullable-entry-sources",
        "ordered-map-parameters-and-branches-preserve-family-and-order"
    ];
    let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);
    for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{assert!(builds[0][field].as_f64().unwrap()>=0.0,"{field}");}
    let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),expected.len());
    for (row,name) in tests.iter().zip(expected){assert_eq!(row["name"],name);assert_eq!(row["status"],"passed");}
    assert!(!rows.iter().any(|row|row["event"]=="output"),"original positive cases produce no console output");
    let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);let summary=summaries[0];assert_eq!(rows.last().unwrap()["event"],"summary");
    for (field,value) in [("passed",42),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summary[field],value,"{field}");}
}
