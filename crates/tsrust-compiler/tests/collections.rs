mod support;

#[test]
fn deque_clone_is_deep_and_requires_cloneable_payloads() {
    let source = r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
type Item={values:int[];};
function main(){const original=new Deque<Item>();original.pushBack({values:[1,2]});const copied=clone(original);const item=copied.popFront()!;item.values.push(3);strictEqual(item.values.length,3);const source=original.popFront()!;strictEqual(source.values.length,2);strictEqual(source.values[0],1);const empty=new Deque<int>();strictEqual(clone(empty).size,0);}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
fn deque_callback_records_join_distinct_environments() {
    let source = r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
type Reader={read:ReadOnly<Function<int>>;};
function main(){const text='one';const other='seven';const values=new Deque<Reader>();values.pushBack({read:()=>text.length});values.pushBack({read:()=>other.length});let sum=0;for(const item of values){sum+=item.read();}strictEqual(sum,8);const first=values.popFront()!;const last=values.popBack()!;strictEqual(first.read(),3);strictEqual(last.read(),5);values.pushBack({read:()=>text.length+other.length});strictEqual(values.popFront()!.read(),8);}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
}

#[test]
fn deque_callback_records_retain_native_view_captures() {
    let helpers = include_str!("fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
    let source = format!(r#"import {{Deque}} from 'std:collections';{helpers}
type Reader={{read:ReadOnly<Function<int>>;}};
function main(){{const owner=bytes('ab');const part=window(owner);const values=new Deque<Reader>();values.pushBack({{read:()=>length(part)}});for(const item of values){{strictEqual(item.read(),2);}}const item=values.popFront()!;strictEqual(item.read(),2);}}
"#);
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
    let path = directory.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
    let source = source.replace("joined+=value;", "values.clear();joined+=value;");
    let path = directory.source(&support::format_round_trip(&source));
    let output = support::run(support::cli().arg("check").arg(path), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("iteration-conflict"));
}

#[test]
fn deque_iteration_retains_views_and_handles_early_completion() {
    let helpers = include_str!("fixtures/collections/projected-views.tsr").split("function fill(").next().unwrap();
    let source = format!(r#"import {{Deque}} from 'std:collections';{helpers}
function first(values:ReadOnly<Deque<ByteView>>):int{{for(const part of values){{return length(part);}}return 0;}}
function guarded(values:ReadOnly<Deque<ByteView>>):string{{let output='';try{{for(const part of values){{output+=text(part);return output;}}}}finally{{strictEqual(values.size,2);}}return output;}}
function main(){{const owner=bytes('ab');const values=new Deque<ByteView>();values.pushBack(window(owner));values.pushFront(window(owner));let sum=0;for(const part of values){{sum+=length(part);}}strictEqual(sum,4);strictEqual(first(values),2);strictEqual(guarded(values),'ab');values.clear();strictEqual(first(values),0);}}
"#);
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
}

#[test]
fn deque_end_operations_move_values_and_borrow_reads() {
    let source = r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
function main(){
 const values=new Deque<int>();strictEqual(values.size,0);strictEqual(values.popFront(),undefined);
 values.pushBack(2);values.pushFront(1);values.pushBack(3);
 strictEqual(values.size,3);strictEqual(values.front()!,1);strictEqual(values.back()!,3);
 strictEqual(values.at(1)!,2);strictEqual(values.at(-1),undefined);strictEqual(values.at(3),undefined);
 strictEqual(values.popFront()!,1);strictEqual(values.popBack()!,3);strictEqual(values.popBack()!,2);
 strictEqual(values.popBack(),undefined);values.pushBack(9);values.clear();strictEqual(values.size,0);
 const words=new Deque<string>();words.pushBack('owned');{const word=words.front();
 if(word!==undefined){strictEqual(word.length,5);}}const moved=words.popFront()!;strictEqual(moved,'owned');
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
}

#[test]
fn deque_view_transport_keeps_owners_and_rejects_mutation_conflicts() {
    let helpers = r#"import {Deque} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length,set} from 'std:bytes';import {strictEqual} from 'std:testing';
function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
function transfer(input:ByteView):ByteView{const values=new Deque<ByteView>();values.pushBack(input);return values.popFront()!;}
"#;
    let directory = support::TempDirectory::new();
    let good = format!("{helpers}function main(){{const owner=bytes();const result=transfer(window(owner));strictEqual(length(result),2);}}");
    let path = directory.source(&support::format_round_trip(&good));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
fn deque_construction_uses_import_identity_and_owned_native_storage() {
    for source in [
        "import {Deque as Queue} from 'std:collections';function create<T>():Queue<T>{return new Queue<T>();}function accept(value:Queue<int>){}function main(){const values=create<int>();accept(values);}",
        "import {Deque} from 'std:collections';function read(value:ReadOnly<Deque<int>>){}function write(value:Mutable<Deque<int>>){}function main(){read(new Deque<int>());write(new Deque<int>());}",
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(source));
        let output = support::run(support::cli().arg("run").arg(path), 120);
        support::success(&output);
        assert!(output.stdout.is_empty());
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
fn empty_deque_of_views_returns_without_inventing_an_owner() {
    let source = "import {Deque} from 'std:collections';import {ByteView} from 'std:bytes';function empty():Deque<ByteView>{return new Deque<ByteView>();}function accept(values:Deque<ByteView>){}function main(){accept(empty());}";
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
}

#[test]
fn maps_hold_borrowed_native_views_without_copying_and_clones_are_independent() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/collections/borrowed-values.tsr"));
    let path = directory.source(&source);
    let output = support::run(support::cli().arg("run").arg(path), 120);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
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
fn returned_map_views_keep_selected_owner_through_parameters_and_capture() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/collections/returned-views.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
fn projected_and_recursive_map_views_execute_with_real_owner_lifetimes() {
    for fixture in [include_str!("fixtures/collections/projected-views.tsr"), include_str!("fixtures/collections/recursive-map-views.tsr")] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(fixture));
        support::success(&support::run(support::cli().arg("run").arg(path), 120));
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
fn heap_orders_scalars_and_transfers_owned_values() {
    let source=r#"import {Heap as Queue} from 'std:collections';import {strictEqual} from 'std:testing';
function main(){const values=new Queue<int>();strictEqual(values.size,0);strictEqual(values.pop(),undefined);values.push(2);values.push(9);values.push(9);strictEqual(values.peek(),9);const copy=clone(values);strictEqual(copy.pop(),9);strictEqual(values.size,3);strictEqual(values.pop(),9);strictEqual(values.pop(),9);strictEqual(values.pop(),2);strictEqual(values.peek(),undefined);const words=new Queue<string>();words.push('a');words.push('🦀');words.push('界');{const top=words.peek();if(top!==undefined){strictEqual(top,'🦀');}}const word=words.pop()!;strictEqual(word,'🦀');words.clear();strictEqual(words.size,0);}
"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn heap_number_enum_order_and_invalid_access_are_checked() {
    let source=r#"import {Heap} from 'std:collections';import {strictEqual} from 'std:testing';
enum Priority{Low=1,High=9}
function main(){const values=new Heap<number>();values.push(-0.0);values.push(Infinity);values.push(NaN);values.push(-Infinity);strictEqual(Number.isNaN(values.pop()!),true);strictEqual(values.pop(),Infinity);strictEqual(Object.is(values.pop()!,0.0),true);strictEqual(values.pop(),-Infinity);const priorities=new Heap<Priority>();priorities.push(Priority.Low);priorities.push(Priority.High);strictEqual(priorities.pop(),Priority.High);const flags=new Heap<boolean>();flags.push(false);flags.push(true);strictEqual(flags.pop(),true);}
"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
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
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
    for replacement in ["values.clear();sum+=value;","const moved=values;sum+=value;"] {
        let source=source.replace("sum+=value;",replacement);let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("iteration-conflict"),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn native_collection_spread_preserves_copy_borrow_and_move() {
    let source = r#"import {Deque,Heap} from 'std:collections';import {strictEqual} from 'std:testing';
function sum(...values:int[]):int{let total=0;for(const value of values){total+=value;}return total;}
function read(value:ReadOnly<string>):int{return value.length;}
function take(...values:string[]):int{let total=0;for(const value of values){total+=value.length;}return total;}
function update(value:Mutable<int[]>){value.push(2);}
function change(values:Mutable<Deque<int>>):int{values.pushBack(9);return 4;}
function pair(a:int,b:int):int{return a+b;}
function main(){
 const numbers=new Deque<int>();numbers.pushBack(2);numbers.pushFront(1);strictEqual(sum(...numbers),3);strictEqual(numbers.size,2);const copied=[...numbers];strictEqual(copied[0],1);strictEqual(numbers.size,2);
 strictEqual(sum(...numbers,change(numbers)),7);strictEqual(numbers.size,3);let caught=false;try{pair(...numbers);}catch(error){caught=true;}strictEqual(caught,true);
 const words=new Deque<string>();words.pushBack('abc');strictEqual(read(...words),3);strictEqual(words.size,1);strictEqual(take(...words),3);
 const nested=new Deque<int[]>();nested.pushBack([1]);update(...nested);strictEqual(nested.popFront()!.length,2);
 const heap=new Heap<int>();heap.push(2);heap.push(1);strictEqual(sum(...heap),3);strictEqual(heap.size,2);const items=[...heap];strictEqual(sum(...items),3);strictEqual(heap.size,2);
 const strings=new Heap<string>();strings.push('abc');strictEqual(read(...strings),3);strictEqual(strings.size,1);strictEqual(take(...strings),3);
 const moving=new Deque<string>();moving.pushBack('a');const owned=[...moving];strictEqual(owned[0],'a');
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
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
    let path=directory.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
    let invalid=source.replace("strictEqual(length(parts[0]),2);","const moved=owner;strictEqual(length(parts[0]),2);");
    let path=directory.source(&support::format_round_trip(&invalid));
    let output=support::run(support::cli().arg("check").arg(path),30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("borrow"),"{}",String::from_utf8_lossy(&output.stderr));
}
}

#[test]
fn deque_borrowed_array_reads_support_non_null_projection() {
    let source=r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
function size(values:ReadOnly<Deque<int[]>>):int{return values.front()!.length;}
function main(){const values=new Deque<int[]>();values.pushBack([1,2]);strictEqual(values.front()!.length,2);strictEqual(values.back()!.length,2);strictEqual(values.at(0)!.length,2);strictEqual(size(values),2);strictEqual(values.size,1);}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn deque_borrowed_nested_indexing_reads_without_transfers() {
 let source=r#"import {Deque} from 'std:collections';import {strictEqual} from 'std:testing';
function consume(value:string):string{return value;}
function change(values:Mutable<Deque<int[]>>):int{values.clear();return 0;}
function main(){const values=new Deque<int[]>();values.pushBack([4,5]);strictEqual(values.front()![0],4);strictEqual(values.back()![1],5);strictEqual(values.at(0)![0],4);strictEqual(values.size,1);const words=new Deque<string[]>();words.pushBack(['word']);strictEqual(words.front()![0].length,4);strictEqual(words.size,1);}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
 for invalid in [source.replace("strictEqual(values.front()![0],4);","values.front()![0]=9;"),source.replace("strictEqual(words.front()![0].length,4);","consume(words.front()![0]);"),source.replace("strictEqual(values.front()![0],4);","strictEqual(values.front()![change(values)],4);"),source.replace("strictEqual(words.front()![0].length,4);","const borrowed=words.front()![0];words.clear();console.log(borrowed);")] {
  let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"borrowed index accepted: {invalid}");
 }
}

#[test]
fn ordered_set_public_surface_preserves_order_and_borrowed_queries() {
 let source=r#"import {OrderedSet as Tree} from 'std:collections';import {strictEqual} from 'std:testing';
function total(...values:int[]):int{let sum=0;for(const value of values){sum+=value;}return sum;}
function main(){const values=new Tree<int>();strictEqual(values.size,0);values.add(3);values.add(1);values.add(2);values.add(1);strictEqual(values.size,3);let ordered='';for(const value of values){ordered+=String(value);}strictEqual(ordered,'123');strictEqual(values.has(2),true);strictEqual(values.has(4),false);strictEqual(total(...values),6);const array=[...values];strictEqual(array[0],1);strictEqual(array[2],3);const copied=clone(values);strictEqual(copied.delete(2),true);strictEqual(copied.delete(2),false);strictEqual(values.size,3);copied.clear();strictEqual(copied.size,0);const words=new Tree<string>();const owned='z'.repeat(2);words.add(owned);words.add('a');const query='z'.repeat(2);strictEqual(words.has(query),true);strictEqual(query.length,2);strictEqual(words.delete(query),true);strictEqual(query.length,2);for(const word of words){strictEqual(word,'a');}}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_set_number_enum_and_ownership_boundaries() {
 let source=r#"import {OrderedSet} from 'std:collections';import {strictEqual} from 'std:testing';
enum Code{Low=1,High=4}
enum Label{Last='z',First='a'}
function take(...words:string[]):int{let size=0;for(const word of words){size+=word.length;}return size;}
function main(){const numbers=new OrderedSet<number>();numbers.add(-0.0);numbers.add(0.0);numbers.add(NaN);numbers.add(NaN);numbers.add(Infinity);numbers.add(-Infinity);strictEqual(numbers.size,4);strictEqual(numbers.has(NaN),true);strictEqual(numbers.has(-0.0),true);const sorted=[...numbers];strictEqual(sorted[0],-Infinity);strictEqual(1.0/sorted[1],Infinity);strictEqual(sorted[2],Infinity);strictEqual(Number.isNaN(sorted[3]),true);strictEqual(numbers.delete(NaN),true);strictEqual(numbers.has(NaN),false);const codes=new OrderedSet<Code>();codes.add(Code.High);codes.add(Code.Low);strictEqual(codes.has(Code.Low),true);strictEqual(codes.delete(Code.High),true);for(const code of codes){strictEqual(code,Code.Low);}const flags=new OrderedSet<boolean>();flags.add(true);flags.add(false);const ordered=[...flags];strictEqual(ordered[0],false);const labels=new OrderedSet<Label>();labels.add(Label.Last);labels.add(Label.First);strictEqual(labels.has(Label.First),true);strictEqual(labels.delete(Label.Last),true);const unicode=new OrderedSet<string>();unicode.add('😀');unicode.add('\uE000');const sequence=[...unicode];strictEqual(sequence[0],'\uE000');strictEqual(sequence[1],'😀');const words=new OrderedSet<string>();words.add('a');words.add('bb');strictEqual(take(...words),3);}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
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
fn ordered_map_public_constructor_preserves_sorted_owned_storage() {
    let source=r#"import {OrderedMap as Tree} from 'std:collections';import {strictEqual} from 'std:testing';
function main(){const values=new Tree<string,int>();values.set('z',3);values.set('a',1);values.set('m',2);let seen='';for(const entry of values){seen+=entry[0];}strictEqual(seen,'amz');strictEqual(values.get('m'),2);const copy=clone(values);strictEqual(copy.delete('a'),true);strictEqual(values.size,3);strictEqual(copy.size,2);}
"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
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
fn map_set_copied_key_can_feed_the_value_expression() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function main(){const key=2;const ordered=new OrderedMap<int,int>();ordered.set(key,key*3+1);strictEqual(ordered.get(key),7);const ordinary=new Map<int,int>();ordinary.set(key,key*4+1);strictEqual(ordinary.get(key),9);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_converts_to_independent_ordinary_map() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function main(){const ordered=new OrderedMap<string,string>();ordered.set('z','last');ordered.set('a','first');const ordinary=new Map<string,string>(ordered);let keys='';for(const entry of ordinary){keys+=entry[0];}strictEqual(keys,'az');ordinary.delete('a');strictEqual(ordered.size,2);strictEqual(ordinary.size,1);strictEqual(ordinary.get('z'),'last');}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_array_from_borrows_sorted_entries() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function main(){const values=new OrderedMap<string,string>();values.set('z','last');values.set('a','first');const mapped=Array.from(values,(entry:ReadOnly<[string,string]>,index:int):string=>entry[0]+String(index)+entry[1]);strictEqual(mapped[0],'a0first');strictEqual(mapped[1],'z1last');strictEqual(values.size,2);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_array_from_preserves_view_payloads_and_copy_boundaries() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';
function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
type Row={data:ByteView;};
function main(){const owner=bytes();const values=new OrderedMap<int,Row>();values.set(2,{data:window(owner)});values.set(1,{data:window(owner)});const lengths=Array.from(values,(entry:ReadOnly<[int,Row]>,index:int):int=>length(entry[1].data)+entry[0]+index);strictEqual(lengths[0],3);strictEqual(lengths[1],5);strictEqual(values.size,2);const plain=new OrderedMap<string,string>();plain.set('z','last');plain.set('a','first');const copied=Array.from(plain);plain.clear();strictEqual(copied[0][0],'a');strictEqual(copied[1][1],'last');}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
 for invalid in [source.replace("length(entry[1].data)+entry[0]+index","{values.clear();return 0;}"),source.replace("const lengths=Array.from(values,","const forbidden=Array.from(values);const lengths=Array.from(values,")] {
  let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"unsafe mapping accepted");
 }
}

#[test]
fn ordered_map_owned_spread_transfers_sorted_entries() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function collect(...entries:[string,string][]):string{let text='';for(const entry of entries){text+=entry[0]+entry[1];}return text;}function main(){const values=new OrderedMap<string,string>();values.set('z','last');values.set('a','first');const entries=[...values];strictEqual(entries[0][0],'a');strictEqual(entries[1][1],'last');const more=new OrderedMap<string,string>();more.set('b','2');more.set('a','1');strictEqual(collect(...more),'a1b2');}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_owned_spread_retains_noncloneable_payloads() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function total(...entries:[int,ByteView][]):int{let sum=0;for(const entry of entries){sum+=entry[0]+length(entry[1]);}return sum;}function main(){const owner=bytes();const values=new OrderedMap<int,ByteView>();values.set(2,window(owner));values.set(1,window(owner));const entries=[...values];strictEqual(entries[0][0],1);strictEqual(length(entries[1][1]),2);const more=new OrderedMap<int,ByteView>();more.set(3,window(owner));strictEqual(total(...more),5);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
 for invalid in [source.replace("const entries=[...values];","const entries=[...values];console.log(values.size);"),source.replace("strictEqual(length(entries[1][1]),2);","const moved=owner;strictEqual(length(entries[1][1]),2);")] {
  let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"invalid owning spread accepted");
 }
}

#[test]
fn ordered_map_readonly_spread_borrows_mixed_entry_sources() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function read(first:ReadOnly<[string,string]>,second:ReadOnly<[string,string]>):string{return first[0]+first[1]+second[0]+second[1];}function main(){const values=new OrderedMap<string,string>();values.set('b','2');values.set('a','1');strictEqual(read(...values),'a1b2');strictEqual(values.size,2);const one=new OrderedMap<string,string>();one.set('c','3');const pair:[string,string]=['d','4'];strictEqual(read(...one,pair),'c3d4');strictEqual(read(pair,...one),'d4c3');strictEqual(one.size,1);strictEqual(pair[0],'d');}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_readonly_spread_keeps_view_owners() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function read(entry:ReadOnly<[int,ByteView]>):int{return entry[0]+length(entry[1]);}function main(){const owner=bytes();const values=new OrderedMap<int,ByteView>();values.set(3,window(owner));strictEqual(read(...values),5);strictEqual(values.size,1);const entries:[int,ByteView][]=[[4,window(owner)]];strictEqual(read(...entries),6);strictEqual(entries.length,1);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_spread_combines_readonly_and_owning_parameters() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';type Row={text:string;};function mixed(first:ReadOnly<[string,Row]>,second:[string,Row]):string{return first[0]+first[1].text+second[0]+second[1].text;}function read(first:ReadOnly<[string,Row]>,second:ReadOnly<[string,Row]>):string{return first[0]+first[1].text+second[0]+second[1].text;}function main(){const values=new OrderedMap<string,Row>();values.set('b',{text:'2'});values.set('a',{text:'1'});strictEqual(read(...values),'a1b2');strictEqual(mixed(...values),'a1b2');}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_spread_handles_optional_readonly_entries() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function read(entry?:ReadOnly<[string,string]>):string{if(entry===undefined){return 'empty';}return entry[0]+entry[1];}function main(){const values=new OrderedMap<string,string>();strictEqual(read(...values),'empty');values.set('a','1');strictEqual(read(...values),'a1');strictEqual(values.size,1);}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_spread_returns_original_view_projection() {
 let source=r#"import {OrderedMap} from 'std:collections';import {Bytes,ByteView,fromUtf8,view,length} from 'std:bytes';import {strictEqual} from 'std:testing';function bytes():Bytes{match(fromUtf8('ab')){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function window(owner:ReadOnly<Bytes>):ByteView{match(view(owner,0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function project(entry:ReadOnly<[int,ByteView]>):ByteView{match(view(entry[1],0,2)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}function select(owner:ReadOnly<Bytes>):ByteView{const values=new OrderedMap<int,ByteView>();values.set(3,window(owner));return project(...values);}function main(){const owner=bytes();const result=select(owner);strictEqual(length(result),2);}"#;
 let directory=support::TempDirectory::new();
 for candidate in [source.to_owned(),source.replace("new OrderedMap<int,ByteView>()","new Map<int,ByteView>()")] {
  let path=directory.source(&support::format_round_trip(&candidate));support::success(&support::run(support::cli().arg("run").arg(path),120));
 }
 let invalid=source.replace("strictEqual(length(result),2);","const moved=owner;strictEqual(length(result),2);");let path=directory.source(&support::format_round_trip(&invalid));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"escaped projection lost owner");
}

#[test]
fn ordered_map_spread_mixes_nullable_entry_sources() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function text(entry:ReadOnly<[string,string]|null>):string{if(entry===null){return '-';}return entry[0]+entry[1];}function read(first:ReadOnly<[string,string]|null>,second:ReadOnly<[string,string]|null>):string{return text(first)+text(second);}function main(){const values=new OrderedMap<string,string>();values.set('a','1');const absent:([string,string]|null)[]=[null];strictEqual(read(...values,...absent),'a1-');strictEqual(read(...absent,...values),'-a1');strictEqual(values.size,1);const present:([string,string]|null)[]=[['b','2']];strictEqual(read(...values,...present),'a1b2');strictEqual(read(...present,...values),'b2a1');}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn ordered_map_parameters_and_branches_preserve_family_and_order() {
 let source=r#"import {OrderedMap} from 'std:collections';import {strictEqual} from 'std:testing';function build(flag:boolean):OrderedMap<string,int>{const left=new OrderedMap<string,int>();left.set('b',2);left.set('a',1);const right=new OrderedMap<string,int>();right.set('d',4);right.set('c',3);if(flag){return left;}return right;}function add(values:Mutable<OrderedMap<string,int>>){values.set('e',5);}function text(values:ReadOnly<OrderedMap<string,int>>):string{let result='';for(const entry of values){result+=entry[0]+String(entry[1]);}return result;}function main(){const first=build(true);const second=build(false);strictEqual(text(first),'a1b2');strictEqual(text(second),'c3d4');add(first);strictEqual(text(first),'a1b2e5');strictEqual(text(second),'c3d4');}"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),120));
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
    let sets = include_str!("../src/runtime/sets.rs").split_once("r#\"").unwrap().1.rsplit_once("\"#;").unwrap().0;
    let maps = include_str!("../src/runtime/maps.rs").split_once("r#\"").unwrap().1.rsplit_once("\"#;").unwrap().0;
    let source = format!("#![allow(dead_code)]\ntype String=std::borrow::Cow<'static,str>;\n#[derive(Clone)] struct TsrNull;\n#[derive(Clone)] struct TsrUndefined;\n{sets}\n{maps}\n{}", include_str!("fixtures/collections/hash-buckets-backend.rs"));
    let path = directory.0.join("buckets.rs");
    std::fs::write(&path, source).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("buckets-{level}"));
        support::success(&support::run(support::rustc().arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),30));
    }
}
