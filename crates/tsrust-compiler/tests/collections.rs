mod support;

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
