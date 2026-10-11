mod support;

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
