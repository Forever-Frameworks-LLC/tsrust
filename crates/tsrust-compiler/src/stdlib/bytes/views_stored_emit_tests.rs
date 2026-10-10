//! Complete source/native controls for views carried in stored callback fields.
use super::*;

const STORED_SOURCE:&str = "type Window={};type Holder={read:Once<Function<Window>>;marker:int;};function save(view:Window):Holder{return {read:()=>view,marker:1};}function relay(first:Window,second:Window):Window{const holder=save(second);return holder.read();}function main(){}";

#[test]
fn native_view_record_callback_preserves_selected_input() {
    let source = STORED_SOURCE;
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=Some(NativeTypeId::ByteView);
    let checked=crate::analysis::analyze(parsed).unwrap();
    let relay=checked.functions.iter().position(|function|function.name=="relay").unwrap();
    let origins=checked.functions.iter().find(|f|f.name=="relay").unwrap().native_returns.as_ref().expect("stored callback needs caller origin");
    assert_eq!(origins.len(),1);
    assert_eq!(origins[0].root,1);
    assert_record_execution(&checked,relay,"callback");
}

fn assert_record_execution(checked:&crate::analysis::CheckedProgram,relay:usize,label:&str) {
    let view = checked
        .records
        .iter()
        .position(|record| matches!(record.native_type,Some(NativeTypeId::ByteView|NativeTypeId::ByteViewMut)))
        .unwrap();
    let mut generated = crate::emit::emit(&checked);
    let main = generated.find("fn main()").unwrap();
    let opening = main + generated[main..].find('{').unwrap() + 1;
    generated.insert_str(opening, "carried_probe();");
    let mutable=checked.records[view].native_type==Some(NativeTypeId::ByteViewMut);
    let reference=if mutable {"&mut"}else{"&"};
    let write=if mutable {"returned.bytes[0]+=10;"}else{""};
    let expected=if mutable {17}else{7};
    let first_access=if matches!(checked.functions[relay].bindings[0].mode,crate::Mode::Read|crate::Mode::Mutate){"&"}else{""};
    let probe = format!(
        "fn carried_probe(){{let mut owner=vec![7u8,8];let pointer=owner.as_ptr();{{let mut returned;{{let mut short=vec![1u8];returned=function{relay}({first_access}Record{view}{{bytes:{reference} short}},Record{view}{{bytes:{reference} owner}});}}assert_eq!(returned.bytes.as_ptr(),pointer);assert_eq!(returned.bytes,&[7,8]);{write}}}assert_eq!(owner[0],{expected});}}"
    );
    let directory = std::env::temp_dir().join(format!(
        "tsrust-record-carried-view-environment-{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    let input = directory.join("carried.rs");
    std::fs::write(&input, format!("{generated}\n{probe}")).unwrap();
    for level in ["0", "3"] {
        let output = directory.join(format!("carried-{level}"));
        let compiled = std::process::Command::new("rustc")
            .arg("--edition=2024")
            .arg("-C")
            .arg(format!("opt-level={level}"))
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        assert!(
            std::process::Command::new(output)
                .status()
                .unwrap()
                .success()
        );
    }
    let negative = format!(
        "{generated}\n{probe}\nfn invalid_source_escape(){{let mut first=vec![1u8];let escaped;{{let mut second=vec![7u8,8];escaped=function{relay}({first_access}Record{view}{{bytes:{reference} first}},Record{view}{{bytes:{reference} second}});}}assert_eq!(escaped.bytes.len(),2);}}"
    );
    std::fs::write(&input, negative).unwrap();
    let rejected = std::process::Command::new("rustc")
        .arg("--edition=2024")
        .arg(&input)
        .arg("-o")
        .arg(directory.join("rejected"))
        .output()
        .unwrap();
    assert!(
        !rejected.status.success(),
        "carried capture escaped its selected input owner"
    );
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not live long enough"));
}

#[test]
fn native_view_mixed_record_preserves_selected_input() {
    let source="type Window={};type Holder={view:Window;read:()=>int;};function save(view:Window):Holder{return {view:view,read:()=>7};}function relay(first:Window,second:Window):Window{const holder=save(second);holder.read();return holder.view;}function main(){}";
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=Some(NativeTypeId::ByteView);
    let checked=crate::analysis::analyze(parsed).unwrap();
    let relay=checked.functions.iter().position(|function|function.name=="relay").unwrap();
    let origins=checked.functions[relay].native_returns.as_ref().expect("mixed record needs caller origin");
    assert_eq!(origins.len(),1);
    assert_eq!(origins[0].root,1);
    assert_record_execution(&checked,relay,"mixed");
}

#[test]
fn native_view_stored_callback_rejects_unplanned_abi_at_source() {
    for (result, parameters, body, expected) in [
        ("Window", "text:string", "return view;", "arguments"),
        ("void", "", "const discarded=view;", "returned capture"),
        ("Window", "", "if(fail){throw 3;}return view;", "Fallible"),
    ] {
        let source=format!("type Window={{}};type Holder={{read:Once<Function<{result}{}>>;}};function save(view:Window,fail:boolean):Holder{{return {{read:({parameters}):{result}=>{{{body}}}}};}}function main(){{}}",if parameters.is_empty(){""}else{",[ReadOnly<string>]"});
        let formatted=crate::formatting::format(&source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=Some(NativeTypeId::ByteView);
        let error=crate::analysis::analyze(parsed).err().expect("unsupported ABI needs a source diagnostic");
        assert_eq!(error.code,"native-view-family-emission","{error:?}");
        assert!(error.message.contains(expected),"{error:?}");
        assert_ne!(error.span,crate::Span::default());
    }
}

fn assert_stored_source_case(source:&str,label:&str) {
    assert_stored_source_access(source,label,false);
}
fn assert_stored_source_access(source:&str,label:&str,mutable:bool) {
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=Some(if mutable {NativeTypeId::ByteViewMut}else{NativeTypeId::ByteView});
    let checked=crate::analysis::analyze(parsed).unwrap_or_else(|error|panic!("{label}: {error:?}"));
    let relay=checked.functions.iter().position(|function|function.name=="relay" && !function.template).unwrap();
    let origins=checked.functions[relay].native_returns.as_ref().expect("stored callback needs caller origins");
    assert!(!origins.is_empty(),"{label}");
    assert!(origins.iter().all(|loan|loan.root==1),"{label}: {origins:?}");
    assert_record_execution(&checked,relay,label);
}

macro_rules! stored_source_case {
    ($name:ident,$holder:literal,$save:literal,$relay:literal) => {
        #[test]
        fn $name() {
            assert_stored_source_case(concat!("type Window={};type Holder={",$holder,"};function save(view:Window):Holder{",$save,"}function relay(first:Window,second:Window):Window{",$relay,"}function main(){}"),stringify!($name));
        }
    };
}

stored_source_case!(native_view_storage_field_move,
    "read:Once<Function<Window>>;marker:int;",
    "return {read:()=>view,marker:1};",
    "const holder=save(second);const action=holder.read;return action();");
stored_source_case!(native_view_storage_saved_move,
    "read:Once<Function<Window>>;marker:int;",
    "const action:Once<Function<Window>>=()=>view;return {read:action,marker:1};",
    "const holder=save(second);return holder.read();");
stored_source_case!(native_view_storage_replacement,
    "read:Once<Function<Window>>;marker:int;",
    "return {read:()=>view,marker:1};",
    "let holder=save(first);holder=save(second);return holder.read();");
stored_source_case!(native_view_storage_array,
    "read:Once<Function<Window>>[];marker:int;",
    "return {read:[()=>view],marker:1};",
    "const holder=save(second);const action=holder.read.pop()!;return action();");
stored_source_case!(native_view_storage_nullable,
    "read:Once<Function<Window>>|undefined;marker:int;",
    "return {read:()=>view,marker:1};",
    "const holder=save(second);return (holder.read!)();");
stored_source_case!(native_view_storage_nested_capture,
    "read:Once<Function<Window>>;marker:int;",
    "const action=()=>view;return {read:()=>action(),marker:1};",
    "const holder=save(second);return holder.read();");

#[test]
fn native_view_storage_mutable_containers() {
    for (label,holder,save,relay) in [
        ("mutable-move","read:Once<Function<Window>>;marker:int;","return {read:()=>view,marker:1};","const holder=save(second);const action=holder.read;return action();"),
        ("mutable-array","read:Once<Function<Window>>[];marker:int;","return {read:[()=>view],marker:1};","const holder=save(second);const action=holder.read.pop()!;return action();"),
        ("mutable-nullable","read:Once<Function<Window>>|undefined;marker:int;","return {read:()=>view,marker:1};","const holder=save(second);return (holder.read!)();"),
        ("mutable-replacement","read:Once<Function<Window>>;marker:int;","return {read:()=>view,marker:1};","let holder=save(first);holder=save(second);return holder.read();"),
        ("mutable-nested","read:Once<Function<Window>>;marker:int;","const action=()=>view;return {read:()=>action(),marker:1};","const holder=save(second);return holder.read();"),
    ] {
        let source=format!("type Window={{}};type Holder={{{holder}}};function save(view:Window):Holder{{{save}}}function relay(first:Window,second:Window):Window{{{relay}}}function main(){{}}");
        assert_stored_source_access(&source,label,true);
    }
}

#[test]
fn native_view_storage_record_alternatives() {
    for flag in ["true","false"] {
        let source=format!("type Window={{}};type Holder={{read:Once<Function<Window>>;marker:int;}};function choose(view:Window,flag:boolean):Holder{{if(flag){{return {{read:()=>view,marker:1}};}}return {{read:()=>view,marker:2}};}}function relay(first:Window,second:Window):Window{{const holder=choose(second,{flag});return holder.read();}}function main(){{}}");
        for mutable in [false,true] {assert_stored_source_access(&source,&format!("record-alternatives-{flag}-{mutable}"),mutable);}
    }
}

stored_source_case!(native_view_storage_field_alternatives,
    "read:Once<Function<Window>>;marker:int;",
    "return {read:true?()=>view:()=>view,marker:1};",
    "const holder=save(second);return holder.read();");

stored_source_case!(native_view_storage_partial_move_replacement,
    "read:Once<Function<Window>>;marker:int;",
    "return {read:()=>view,marker:1};",
    "let holder=save(first);const previous=holder.read;holder=save(second);const discarded=previous();return holder.read();");

#[test]
fn native_view_storage_rejects_consumed_callback_reuse() {
    let source="type Window={};type Holder={read:Once<Function<Window>>;marker:int;};function save(view:Window):Holder{return {read:()=>view,marker:1};}function relay(first:Window,second:Window):Window{const holder=save(second);const action=holder.read;const discarded=action();return action();}function main(){}";
    let formatted=crate::formatting::format(source).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records[0].native_type=Some(NativeTypeId::ByteView);
    let error=crate::analysis::analyze(parsed).err().expect("consumed callback cannot be invoked again");
    assert_eq!(error.code,"closure-consumed","{error:?}");
    assert!(!error.related.is_empty());
}
