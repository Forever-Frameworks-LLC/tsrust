mod support;

#[test]
fn native_byte_type_imports_and_result_payloads_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/values.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_callable_and_generic_aggregate_storage_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/storage.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn exported_result_aliases_keep_builtin_identity_and_native_payloads() {
    let directory = support::TempDirectory::new();
    let library = support::format_round_trip("import {Bytes,ByteError,fromArray} from 'std:bytes';export type Outcome=Result<Bytes,ByteError>;export function build():Outcome{return fromArray([1,2]);}export type Plain=Result<int,string>;");
    std::fs::write(directory.0.join("types.tsr"), library).unwrap();
    let main = support::format_round_trip("import {Outcome as Batch,Plain,build} from './types.tsr';import {length} from 'std:bytes';import {strictEqual} from 'std:testing';function inspect(result:ReadOnly<Batch>):int{match(result){Ok(bytes)=>{return length(bytes);},Err(error)=>{return -1;}}}function main(){const result:Batch=build();strictEqual(inspect(result),2);const plain:Plain=Ok(4);match(plain){Ok(value)=>{strictEqual(value,4);},Err(error)=>{throw error;}}}");
    let path = directory.source(&main);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_access_and_mutation_validate_before_writing() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/access.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_allocation_and_resize_preserve_defaults_and_atomic_errors() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/growth.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_append_is_atomic_and_range_copies_are_independent() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/copy-append.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_clone_storage_capture_and_evaluation_contracts_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/ownership.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn native_byte_utf8_round_trips_and_strict_error_offsets_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/utf8.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn byte_views_read_and_write_through_public_imports() {
    for fixture in [
        include_str!("fixtures/native-byte-views/read.tsr"),
        include_str!("fixtures/native-byte-views/values.tsr"),
        include_str!("fixtures/native-byte-views/callbacks.tsr"),
        include_str!("fixtures/native-byte-views/recursive-arrays.tsr"),
        include_str!("fixtures/native-byte-views/recursive-links.tsr"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(fixture));
        support::success(&support::run(support::cli().arg("run").arg(path), 120));
    }
}

#[test]
fn capturing_callbacks_return_views_of_borrowed_owner_arguments() {
    let directory=support::TempDirectory::new();
    let fixture=include_str!("fixtures/native-byte-views/callback-owners.tsr");
    let path=directory.source(&support::format_round_trip(fixture));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
    let helpers=fixture.split("function main()").next().unwrap();
    let source=format!("{helpers}function leak(captured:ByteView):ByteView{{const owner=bytes(\"short\");const picker=saveRead(captured);return picker.pick(owner);}}function main(){{}}");
    let path=directory.source(&support::format_round_trip(&source));
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("native-view-escape"));
}

#[test]
fn byte_view_ownership_rules_are_source_diagnostics() {
    let imports = "import { Bytes, ByteView, ByteViewMut, alloc, view, viewMut, length, set, resize } from \"std:bytes\";";
    let inside = |body: &str| format!("function main() {{ match (alloc(2, 1)) {{ Ok(bytes) => {{ {body} }}, Err(error) => {{}}, }} }}");
    let with_view = |factory: &str, body: &str| inside(&format!("match ({factory}(bytes, 0, 1)) {{ Ok(part) => {{ {body} }}, Err(error) => {{}}, }}"));
    for (label, source, code, text) in [
        ("returned view of a local owner", "function leak(): ByteView { match (alloc(2, 1)) { Ok(bytes) => { match (view(bytes, 0, 1)) { Ok(part) => { return part; }, Err(error) => { throw error.message; }, } }, Err(error) => { throw error.message; }, } } function main() { console.log(length(leak())); }".to_owned(), "native-view-escape", "local owner"),
        ("owner write while a view is live", with_view("view", "match (set(bytes, 0, 9)) { Ok(_) => {}, Err(error) => {}, } console.log(length(part));"), "native-view-conflict", "bytes"),
        ("owner resize while a view is live", with_view("view", "match (resize(bytes, 9)) { Ok(_) => {}, Err(error) => {}, } console.log(length(part));"), "native-view-conflict", "bytes"),
        ("owner move while a view is live", with_view("view", "const moved = bytes; console.log(length(part));"), "native-view-conflict", "bytes"),
        ("owner read while an exclusive view is live", with_view("viewMut", "console.log(length(bytes)); console.log(length(part));"), "native-view-conflict", "bytes"),
        ("write through a readonly view", with_view("view", "match (set(part, 0, 9)) { Ok(_) => {}, Err(error) => {}, }"), "type", "Expected Bytes, found ByteView"),
        ("exclusive view of a readonly view", with_view("view", "match (viewMut(part, 0, 1)) { Ok(inner) => {}, Err(error) => {}, }"), "type", "Expected Bytes, found ByteView"),
        ("clone of a view", with_view("view", "const other = clone(part); console.log(length(other));"), "clone-type", "copy(view, start, end)"),
        ("constructed view", "function main() { const made: ByteView = { }; }".to_owned(), "native-opaque", "constructors"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&format!("{imports}\n{source}"));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success(), "{label}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("error[{code}]")) && stderr.contains(text), "{label}: {stderr}");
    }
}

#[test]
fn recursive_byte_views_reject_owner_conflicts_and_escape() {
    let helpers = include_str!("fixtures/native-byte-views/recursive-arrays.tsr").split("function main()").next().unwrap();
    let tree = "const first=bytes(3);const second=bytes(7);const root=leaf(read(first));root.children.push(leaf(read(second)));";
    for (label, body, expected) in [
        ("owner move", format!("{tree}const moved=first;console.log(length(root.data));"), "native-view-conflict"),
        ("owner write", format!("{tree}set(second,0,9);console.log(length(root.data));"), "native-view-conflict"),
        ("removed child keeps shared origin", format!("{tree}const child=root.children.pop()!;set(second,0,9);console.log(length(root.data));"), "native-view-conflict"),
        ("short owner", "const first=bytes(3);let escaped:Chunk;{const second=bytes(7);const root=leaf(read(first));root.children.push(leaf(read(second)));escaped=root;}console.log(length(escaped.data));".into(), "native-view-escape"),
        ("exclusive owner read", "const first=bytes(3);const root=writable(write(first));console.log(length(first));console.log(length(root.data));".into(), "native-view-conflict"),
    ] {
        let directory = support::TempDirectory::new();
        let source = support::format_round_trip(&format!("{helpers}function main(){{{body}}}"));
        let path = directory.source(&source);
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success(), "{label}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("error[{expected}]")), "{label}: {stderr}");
    }
}

#[test]
fn nullable_recursive_byte_views_keep_ownership_diagnostics() {
    let helpers=include_str!("fixtures/native-byte-views/recursive-links.tsr").split("function main()").next().unwrap();
    let tree="const first=bytes(3);const second=bytes(7);const root=leaf(read(first));root.next=leaf(read(second));";
    for (label,body,expected) in [
        ("owner move",format!("{tree}const moved=first;console.log(length(root.data));"),"native-view-conflict"),
        ("owner write",format!("{tree}set(second,0,9);console.log(length(root.data));"),"native-view-conflict"),
        ("extracted link",format!("{tree}const child=root.next!;set(second,0,9);console.log(length(root.data));"),"native-view-conflict"),
        ("short owner","const first=bytes(3);let escaped:Chunk;{const second=bytes(7);const root=leaf(read(first));root.next=leaf(read(second));escaped=root;}console.log(length(escaped.data));".into(),"native-view-escape"),
        ("exclusive owner read","const first=bytes(3);const root=writable(write(first));console.log(length(first));console.log(length(root.data));".into(),"native-view-conflict"),
        ("clone view chain",format!("{tree}const copied=clone(root);"),"clone-type"),
    ] {
        let directory=support::TempDirectory::new();
        let source=support::format_round_trip(&format!("{helpers}function main(){{{body}}}"));
        let path=directory.source(&source);
        let output=support::run(support::cli().arg("check").arg(path),60);
        assert!(!output.status.success(),"{label}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{expected}]")),"{label}: {}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn temporary_callback_record_preserves_borrowed_owner_arguments() {
    let fixture=include_str!("fixtures/native-byte-views/callback-owners.tsr");
    let source=fixture.replace("const picker = saveRead(slice(first));\n    const selected = picker.pick(second);","const selected = saveRead(slice(first)).pick(second);");
    assert_ne!(source,fixture);
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn native_view_constructor_inside_stored_callback_keeps_argument_ownership() {
    let fixture=include_str!("fixtures/native-byte-views/callback-owners.tsr");
    let source=fixture.replace("return writable(other);","match(viewMut(other,1,3)){Ok(value)=>{return value;},Err(error)=>{throw error.message;},}");
    assert_ne!(source,fixture);
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(&source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
}

#[test]
fn recursive_records_store_view_capturing_once_callbacks() {
    let source = include_str!("fixtures/native-byte-views/recursive-callbacks.tsr");
    let mutual = source.replace("next?: Holder;", "next?: Link;")
        .replace("function bytes", "type Link={value:Holder;};\nfunction bytes")
        .replace("holder.next = build(read(second), read(first));", "holder.next = {value:build(read(second),read(first))};")
        .replace("const child = holder.next!;", "const child = holder.next!.value;");
    let arrays = source.replace("next?: Holder;", "next: Holder[];")
        .replace("data: first, read: () => second", "data: first, read: () => second, next: []")
        .replace("holder.next = build(read(second), read(first));", "holder.next.push(build(read(second),read(first)));")
        .replace("const child = holder.next!;", "const child = holder.next.pop()!;");
    let mutable = source.replace("ByteView", "ByteViewMut")
        .replace("alloc, view,", "alloc, viewMut,")
        .replace("length, at }", "length, at, set }")
        .replace("ReadOnly<Bytes>", "Mutable<Bytes>")
        .replace("match (view(", "match (viewMut(")
        .replace("const holder =", "const third=bytes(11);const fourth=bytes(13);const holder =")
        .replace("holder.next = build(read(second), read(first));", "holder.next = build(read(third), read(fourth));")
        .replace("strictEqual(value, 3);", "strictEqual(value, 13);")
        .replace("const result = holder.read();", "const result = holder.read();set(result,0,19);")
        .replace("strictEqual(value, 7);", "strictEqual(value, 19);");
    let alternatives = source.replace("function main()", "function other(first:ByteView,second:ByteView):Holder{return {data:first,read:()=>second};}\nfunction main()")
        .replace("holder.next = build(read(second), read(first));", "holder.next = other(read(second), read(first));");
    let replaced = source.replace("  const result = holder.read();", "  const old=holder.read();strictEqual(length(old),2);const replacement=read(first);holder.read=()=>replacement;const result = holder.read();")
        .replace("strictEqual(value, 7);", "strictEqual(value, 3);");
    for source in [source.to_owned(), mutual, arrays, mutable, alternatives, replaced] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&source));
        support::success(&support::run(support::cli().arg("run").arg(path),120));
    }
}

#[test]
fn recursive_view_callbacks_preserve_owner_and_clone_rejections() {
    let helpers = include_str!("fixtures/native-byte-views/recursive-callbacks.tsr").split("function main()").next().unwrap();
    let helpers = format!("{helpers}function other(first:ByteView,second:ByteView):Holder{{return {{data:first,read:()=>second}};}}");
    let setup = "const first=bytes(3);const second=bytes(7);const holder=build(read(first),read(second));holder.next=build(read(second),read(first));";
    for (body, expected) in [
        ("const first=bytes(3);let escaped:Holder;{const second=bytes(7);const root=build(read(first),read(first));root.next=other(read(first),read(second));escaped=root;}const result=escaped.read();console.log(length(result));".into(), "native-view-escape"),
        (format!("{setup}const moved=first;const result=holder.read();console.log(length(result));"), "native-view-conflict"),
        (format!("{setup}const copied=clone(holder);"), "clone-type"),
        (format!("{setup}const a=holder.read();const b=holder.read();console.log(length(a),length(b));"), "partial-transfer"),
        ("const first=bytes(3);let escaped:Holder;{const second=bytes(7);escaped=build(read(first),read(second));}const result=escaped.read();console.log(length(result));".into(), "native-view-escape"),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&format!("{helpers}function main(){{{body}}}")));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("error[{expected}]")), "{stderr}");
    }
}

#[test]
fn nullable_recursive_records_execute_beside_and_with_owned_callbacks() {
    for source in [
        "import {strictEqual} from 'std:testing';type Link={value:int;next?:Link;};type Holder={read:Function<int>;};function main(){const link:Link={value:7,next:{value:11}};const holder:Holder={read:():int=>3};strictEqual(link.next!.value,11);strictEqual(holder.read(),3);}",
        "import {strictEqual} from 'std:testing';type Link={value:int;read:Function<int>;next?:Link;};function main(){const link:Link={value:7,read:():int=>3,next:{value:11,read:():int=>5}};strictEqual(link.read(),3);const child=link.next!;strictEqual(child.read(),5);}",
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(source));
        for optimized in [false, true] {
            let mut command = support::cli();
            command.arg("run").arg(&path);
            if optimized { command.arg("--release"); }
            support::success(&support::run(&mut command, 120));
        }
    }
}

#[test]
fn recursive_callbacks_return_extracted_families_with_shared_owner_lifetimes() {
    let helpers = include_str!("fixtures/native-byte-views/recursive-callbacks.tsr").split("function main()").next().unwrap();
    let take = "function other(first:ByteView,second:ByteView):Holder{return {data:first,read:()=>second};}function take(first:ByteView,second:ByteView,flag:boolean){const holder=flag?build(first,second):other(first,second);return holder.read;}";
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(&format!("{helpers}{take}function main(){{const first=bytes(3);const second=bytes(7);const callback=take(read(first),read(second),true);const result=callback();strictEqual(length(result),2);const otherCallback=take(read(first),read(second),false);const otherResult=otherCallback();strictEqual(length(otherResult),2);}}"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path),120));
    let source = support::format_round_trip(&format!("{helpers}{take}function leak(second:ByteView){{const first=bytes(3);return take(read(first),second,false);}}function main(){{const second=bytes(7);const callback=leak(read(second));const result=callback();console.log(length(result));}}"));
    let path = directory.source(&source);
    let output = support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[native-view-escape]"),"{stderr}");
}

#[test]
fn recursive_callback_record_parameters_return_the_stored_family() {
    let helpers = include_str!("fixtures/native-byte-views/recursive-callbacks.tsr").split("function main()").next().unwrap();
    let declarations = "function take(holder:Holder){return holder.read;}function size(holder:ReadOnly<Holder>):int{return length(holder.data);}function relay(holder:Holder):Holder{return holder;}";
    let main = "function main(){const first=bytes(3);const second=bytes(7);const holder=relay(build(read(first),read(second)));strictEqual(size(holder),2);const callback=take(holder);const result=callback();match(at(result,0)){Ok(value)=>{strictEqual(value,7);},Err(error)=>{throw error.message;},}}";
    let direct = helpers.split("function build(").next().unwrap();
    let mutual_helpers = helpers.replace("next?: Holder;", "next?: Link;").replace("function bytes", "type Link={value:Holder;};function bytes");
    let mutual = format!("{mutual_helpers}{declarations}function takeLink(link:Link){{return take(link.value);}}{}", main.replace("const callback=take(holder);", "holder.next={value:build(read(first),read(second))};const callback=takeLink(holder.next!);"));
    let sources = [
        mutual,
        format!("{helpers}{}{main}", declarations.replace("function take(holder:Holder)", "function take(holder:Holder):Once<Function<ByteView>>")),
        format!("{helpers}function other(first:ByteView,second:ByteView):Holder{{return {{data:first,read:()=>second}};}}{declarations}{}", main.replace("build(read(first),read(second))", "other(read(first),read(second))")),
        format!("{helpers}{declarations}{main}"),
        format!("{declarations}{main}{helpers}"),
        format!("{direct}{declarations}{}", main.replace("const holder=", "const selected=read(second);const holder=").replace("build(read(first),read(second))", "{data:read(first),read:()=>selected}")),
        format!("{helpers}{declarations}{}", main.replace("const callback=take(holder);", "const child:Holder=build(read(first),read(second));holder.next=child;const callback=take(holder.next!);")),
    ];
    for source in sources {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&source));
        for optimized in [false, true] {
            let mut command = support::cli();
            command.arg("run").arg(&path);
            if optimized { command.arg("--release"); }
            support::success(&support::run(&mut command,120));
        }
    }
}

#[test]
fn recursive_callback_record_parameters_preserve_every_owner() {
    let helpers = include_str!("fixtures/native-byte-views/recursive-callbacks.tsr").split("function main()").next().unwrap();
    for arguments in ["read(short),other", "other,read(short)"] {
        let source = format!("{helpers}function take(holder:Holder){{return holder.read;}}function leak(other:ByteView){{const short=bytes(3);return take(build({arguments}));}}function main(){{const owner=bytes(7);const callback=leak(read(owner));const result=callback();console.log(length(result));}}");
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&source));
        let output = support::run(support::cli().arg("check").arg(path),60);
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("error[native-view-escape]"),"{stderr}");
    }
}

#[test]
fn recursive_records_with_only_captured_views_execute() {
    let source = include_str!("fixtures/native-byte-views/recursive-captured-views.tsr");
    let mutual = source.replace("next?: Holder;", "next?: Link;")
        .replace("function bytes", "type Link={value:Holder;};function bytes")
        .replace("holder.next = build(read(second), read(first));", "holder.next={value:build(read(second),read(first))};")
        .replace("const child = holder.next!;", "const child=holder.next!.value;");
    let arrays = source.replace("next?: Holder;", "next:Holder[];")
        .replace("read: () => second", "read: () => second, next: []")
        .replace("holder.next = build(read(second), read(first));", "holder.next.push(build(read(second),read(first)));")
        .replace("const child = holder.next!;", "const child=holder.next.pop()!;");
    let nullable = source.replace("read: Once<Function<ByteView>>;", "read?: Once<Function<ByteView>>;")
        .replace("child.read()", "child.read!()").replace("holder.read()", "holder.read!()");
    let parameters = source.replace("function main()", "function take(holder:Holder){return holder.read;}function main()")
        .replace("const result = holder.read();", "holder.next=undefined;const callback=take(holder);const result=callback();");
    let captured = source.replace("const result = holder.read();", "holder.next=undefined;const action=()=>{const moved=holder;return moved.read();};const result=action();");
    let mutable = source.replace("ByteView", "ByteViewMut").replace("alloc, view,", "alloc, viewMut,")
        .replace("length, at }", "length, at, set }").replace("ReadOnly<Bytes>", "Mutable<Bytes>").replace("match (view(", "match (viewMut(")
        .replace("const holder =", "const third=bytes(11);const fourth=bytes(13);const holder =")
        .replace("holder.next = build(read(second), read(first));", "holder.next=build(read(third),read(fourth));")
        .replace("strictEqual(value, 3);", "strictEqual(value, 13);").replace("const result = holder.read();", "const result=holder.read();set(result,0,19);")
        .replace("strictEqual(value, 7);", "strictEqual(value, 19);");
    let replaced = source.replace("const result = holder.read();", "const old=holder.read();strictEqual(length(old),2);const replacement=read(first);holder.read=()=>replacement;const result=holder.read();")
        .replace("strictEqual(value, 7);", "strictEqual(value, 3);");
    let helpers = source.split("function main()").next().unwrap();
    let scalar = helpers.replace("Once<Function<ByteView>>", "Once<Function<int>>").replace("read: () => second", "read: () => {const kept=second;return length(kept);}");
    let scalar = format!("{scalar}function main(){{const first=bytes(3);const second=bytes(7);const holder=build(read(first),read(second));holder.next=build(read(second),read(first));const child=holder.next!;strictEqual(child.read(),2);strictEqual(holder.read(),2);}}");
    let unused = format!("{helpers}function keep(second:ByteView):Holder{{const unused=bytes(3);return build(read(unused),second);}}function main(){{const owner=bytes(7);const holder=keep(read(owner));const result=holder.read();strictEqual(length(result),2);}}");
    for source in [source.to_owned(),mutual,arrays,nullable,parameters,scalar,unused,mutable,replaced,captured] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&source));
        for optimized in [false,true] {
            let mut command=support::cli();command.arg("run").arg(&path);
            if optimized {command.arg("--release");}
            support::success(&support::run(&mut command,120));
        }
    }
    let source=format!("{helpers}function leak(first:ByteView):Holder{{const short=bytes(7);return build(first,read(short));}}function main(){{const owner=bytes(3);const holder=leak(read(owner));const result=holder.read();console.log(length(result));}}");
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains("error[native-view-escape]"),"{stderr}");
}

#[test]
fn uninhabited_record_callback_still_requires_a_resolved_catch_contract() {
    let source="import {ByteView} from 'std:bytes';type Unknown={read:Once<Function<int>>;};function invoke(holder:Unknown):int{try{return holder.read();}catch(error){return 0;}}function main(){}";
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains("error[error-effect]"),"{stderr}");
}

#[test]
fn nullable_recursive_callbacks_join_factories() {
    let source=include_str!("fixtures/native-byte-views/recursive-callbacks.tsr")
        .replace("read: Once<Function<ByteView>>;", "read?: Once<Function<ByteView>>;")
        .replace("child.read()", "child.read!()").replace("holder.read()", "holder.read!()")
        .replace("function main()", "function other(first:ByteView,second:ByteView):Holder{return {data:first,read:()=>second};}function main()")
        .replace("holder.next = build(read(second), read(first));", "holder.next=other(read(second),read(first));");
    let only=source.replace("  data: ByteView;\n", "").replace("data: first, read:", "read:").replace("data:first,read:","read:").replace("  strictEqual(length(holder.data), 2);\n", "");
    let null=source.replace("read?: Once<Function<ByteView>>;", "read:Once<Function<ByteView>>|null;");
    let replacement=source.replace("const result = holder.read!();", "const previous=holder.read!();strictEqual(length(previous),2);holder.read=undefined;strictEqual(holder.read===undefined,true);const value=read(first);holder.read=()=>value;const result=holder.read!();")
        .replace("strictEqual(value, 7);", "strictEqual(value, 3);");
    let parameter=source.replace("function main()", "function take(holder:Holder){return holder.read;}function main()")
        .replace("const result = holder.read!();", "holder.next=undefined;const callback=take(holder);const result=callback!();");
    let helpers=source.split("function main()").next().unwrap();
    let empty=format!("{helpers}function empty(first:ByteView):Holder{{return {{data:first}};}}function main(){{const first=bytes(3);const second=bytes(7);const holder=build(read(first),read(second));holder.next=empty(read(first));const child=holder.next!;strictEqual(child.read===undefined,true);const result=holder.read!();strictEqual(length(result),2);}}");
    let empty_only=empty.replace("  data: ByteView;\n", "").replace("data: first, read:", "read:").replace("data:first,read:","read:").replace("return {data:first};", "return {};");
    let empty_helpers=empty_only.split("function main()").next().unwrap();
    let independent=format!("{empty_helpers}function vacant():Holder{{const short=bytes(9);return empty(read(short));}}function main(){{const owner=bytes(7);const holder=vacant();strictEqual(holder.read===undefined,true);const value=read(owner);holder.read=()=>value;const result=holder.read!();strictEqual(length(result),2);}}");
    let reverse=format!("{}{}", &empty_only[empty_helpers.len()..],empty_helpers);
    for source in [source,only.clone(),null,replacement,parameter,empty,empty_only,independent,reverse,include_str!("fixtures/native-byte-views/nullable-recursive-callbacks.tsr").to_owned()] {
        let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
        for optimized in [false,true] {
            let mut command=support::cli();command.arg("run").arg(&path);if optimized {command.arg("--release");}
            support::success(&support::run(&mut command,120));
        }
    }
    let helpers=only.split("function main()").next().unwrap();
    for body in ["return other(value,read(short));", "const holder:Holder={};const captured=read(short);holder.read=()=>captured;return holder;"] {
        let escaping=format!("{helpers}function leak(value:ByteView):Holder{{const short=bytes(3);{body}}}function main(){{const owner=bytes(7);const holder=leak(read(owner));const result=holder.read!();console.log(length(result));}}");
        let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&escaping));
        let output=support::run(support::cli().arg("check").arg(path),60);
        assert!(!output.status.success());
        let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains("error[native-view-escape]"),"{stderr}");
    }
}
