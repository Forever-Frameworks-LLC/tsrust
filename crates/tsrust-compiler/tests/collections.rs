mod support;

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
