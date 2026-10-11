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
    // A result that may also carry the first input needs both owners for its whole life.
    let joined=checked.functions[relay].native_returns.as_deref().is_some_and(|loans|loans.iter().any(|loan|loan.root==0));
    if generated.contains(&format!("fn tsr_result_function{relay}")) {
        let (generics,result)=if joined {("<'a:'out,'b:'out,'out>","'out")}else{("<'a,'b>","'b")};
        generated.push_str(&format!("fn function{relay}{generics}(first:{first_access}Record{view}<'a>,second:Record{view}<'b>)->Record{view}<{result}>{{match tsr_result_function{relay}(first,second){{Ok(value)=>value,Err(_)=>panic!(\"unexpected callback failure\")}}}}"));
    }
    let probe = if joined {format!(
        "fn carried_probe(){{let mut owner=vec![7u8,8];let pointer=owner.as_ptr();{{let mut short=vec![1u8];let mut returned=function{relay}({first_access}Record{view}{{bytes:{reference} short}},Record{view}{{bytes:{reference} owner}});assert_eq!(returned.bytes.as_ptr(),pointer);assert_eq!(returned.bytes,&[7,8]);{write}}}assert_eq!(owner[0],{expected});}}"
    )} else {format!(
        "fn carried_probe(){{let mut owner=vec![7u8,8];let pointer=owner.as_ptr();{{let mut returned;{{let mut short=vec![1u8];returned=function{relay}({first_access}Record{view}{{bytes:{reference} short}},Record{view}{{bytes:{reference} owner}});}}assert_eq!(returned.bytes.as_ptr(),pointer);assert_eq!(returned.bytes,&[7,8]);{write}}}assert_eq!(owner[0],{expected});}}"
    )};
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
    if joined {
        // The first input is a possible origin too, so it cannot end before the result.
        let negative = format!(
            "{generated}\n{probe}\nfn invalid_first_escape(){{let mut second=vec![7u8,8];let escaped;{{let mut first=vec![1u8];escaped=function{relay}({first_access}Record{view}{{bytes:{reference} first}},Record{view}{{bytes:{reference} second}});}}assert_eq!(escaped.bytes.len(),2);}}"
        );
        std::fs::write(&input, negative).unwrap();
        let rejected = std::process::Command::new("rustc").arg("--edition=2024").arg(&input).arg("-o").arg(directory.join("rejected")).output().unwrap();
        assert!(!rejected.status.success(), "joined result escaped its first input owner");
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not live long enough"));
    }
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
    assert!(origins.iter().any(|loan|loan.root==1),"{label}: {origins:?}");
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

/// Invocation contracts and replacement forms of stored view callbacks. Every row
/// runs natively at two optimization levels for readonly and exclusive views.
#[test]
fn native_view_storage_invocation_and_replacement_matrix() {
    let once="read:Once<Function<Window>>;marker:int;";
    let plain="return {read:()=>view,marker:1};";
    let cases:&[(&str,&str,&str,&str)]=&[
        ("nullable-present-then-absent","read:Once<Function<Window>>|undefined;marker:int;",plain,"let holder=save(first);holder.read=undefined;const other=save(second);return (other.read!)();"),
        ("field-replacement",once,plain,"let holder=save(first);holder.read=()=>second;return holder.read();"),
        ("field-replacement-from-record",once,plain,"let holder=save(first);const other=save(second);holder.read=other.read;return holder.read();"),
        ("replacement-in-block",once,plain,"let holder=save(first);{holder=save(second);}return holder.read();"),
        ("replacement-in-branch",once,plain,"let holder=save(first);if(holder.marker>0){holder=save(second);}return holder.read();"),
        ("replacement-in-both-branches",once,plain,"let holder=save(first);if(holder.marker>0){holder=save(second);}else{const other=save(second);holder=other;}return holder.read();"),
        ("argument-int","read:Once<Function<Window,[int]>>;marker:int;","return {read:(n:int)=>view,marker:1};","const holder=save(second);return holder.read(3);"),
        ("argument-borrowed-string","read:Once<Function<Window,[ReadOnly<string>]>>;marker:int;","return {read:(text:string)=>view,marker:1};","const holder=save(second);const t=\"x\";return holder.read(t);"),
        ("argument-owned-string","read:Once<Function<Window,[Owned<string>]>>;marker:int;","return {read:(text:Owned<string>)=>view,marker:1};","const holder=save(second);return holder.read(\"x\");"),
        ("optional-parameter","read:Once<Function<Window,[n?:int]>>;marker:int;","return {read:(n?:int)=>view,marker:1};","const holder=save(second);return holder.read();"),
        ("default-parameter","read:Once<Function<Window,[n?:int]>>;marker:int;","return {read:(n:int=2)=>view,marker:1};","const holder=save(second);return holder.read();"),
        ("rest-parameter","read:Once<Function<Window,[...values:int[]]>>;marker:int;","return {read:(...values:int[])=>view,marker:1};","const holder=save(second);return holder.read(1,2);"),
        ("returns-int","read:Once<Function<int>>;marker:int;","return {read:()=>{const kept=view;return 7;},marker:1};","const holder=save(first);const n=holder.read();return second;"),
        ("returns-void","read:Once<Function<void>>;marker:int;","return {read:()=>{const kept=view;},marker:1};","const holder=save(first);holder.read();return second;"),
        ("throwing-success","read:Once<Function<int,[boolean]>>;marker:int;","return {read:(fail:boolean)=>{const kept=view;if(fail){throw 3;}return 4;},marker:1};","const holder=save(first);let seen=0;try{seen=holder.read(false);}catch(error){seen=9;}return second;"),
        ("throwing-failure","read:Once<Function<Window,[boolean]>>;marker:int;","return {read:(fail:boolean)=>{if(fail){throw 3;}return view;},marker:1};","const holder=save(first);let failed=false;try{const lost=holder.read(true);}catch(error){failed=true;}return second;"),
        ("throwing-void-failure","read:Once<Function<void,[boolean]>>;marker:int;","return {read:(fail:boolean)=>{const kept=view;if(fail){throw 3;}},marker:1};","const holder=save(first);let failed=false;try{holder.read(true);}catch(error){failed=true;}return second;"),
        ("throwing-mixed-alternatives","read:Once<Function<int,[boolean]>>;marker:int;","return {read:marker()>0?(fail:boolean)=>{const kept=view;if(fail){throw 3;}return 1;}:(fail:boolean)=>{const kept=view;return 2;},marker:1};","const holder=save(first);let seen=0;try{seen=holder.read(false);}catch(error){seen=9;}return second;"),
        ("argument-owned-view-ignored","read:Once<Function<Window,[Owned<Window>]>>;marker:int;","return {read:(other:Owned<Window>)=>view,marker:1};","const holder=save(second);return holder.read(first);"),
        ("replacement-in-loop",once,plain,"let holder=save(first);for(let i=0;i<1;i+=1){holder=save(second);break;}return holder.read();"),
        ("array-push-alternative","read:Once<Function<Window>>[];marker:int;","const list:Once<Function<Window>>[]=[];list.push(()=>view);return {read:list,marker:1};","const holder=save(second);const action=holder.read.pop()!;return action();"),
        ("stored-family-capture",once,plain,"const holder=save(second);const outer:Once<Function<Window>>=()=>holder.read();return outer();"),
        ("stored-family-capture-field",once,plain,"const holder=save(second);const action=holder.read;const outer:Once<Function<Window>>=()=>action();return outer();"),
        ("stored-family-recapture-record",once,plain,"const holder=save(second);const next:Holder={read:()=>holder.read(),marker:2};return next.read();"),
        ("throwing-view-success","read:Once<Function<Window,[boolean]>>;marker:int;","return {read:(fail:boolean)=>{if(fail){throw 3;}return view;},marker:1};","const holder=save(second);try{return holder.read(false);}catch(error){return first;}"),
        ("array-push-two-alternatives","read:Once<Function<Window>>;marker:int;","return {read:()=>view,marker:1};","const list:Once<Function<Window>>[]=[];list.push(()=>first);list.push(()=>second);const action=list.pop()!;return action();"),
        ("array-literal-then-push","read:Once<Function<Window>>;marker:int;","return {read:()=>view,marker:1};","const list:Once<Function<Window>>[]=[()=>first];list.push(()=>second);const action=list.pop()!;return action();"),
        ("array-two-literal-alternatives","read:Once<Function<Window>>;marker:int;","return {read:()=>view,marker:1};","const list:Once<Function<Window>>[]=[()=>first,()=>second];const action=list.pop()!;return action();"),
        ("reusable-read","read:Function<int>;marker:int;","return {read:()=>size(view),marker:1};","const holder=save(first);const total=holder.read()+holder.read();return second;"),
        ("reusable-mutable","read:Function<int>;marker:int;","let calls=0;return {read:()=>{calls+=1;return calls+size(view);},marker:1};","let holder=save(first);const total=holder.read()+holder.read();return second;"),
        ("reusable-argument","read:Function<int,[ReadOnly<string>]>;marker:int;","return {read:(text:string)=>size(view)+text.length,marker:1};","const holder=save(first);const total=holder.read(\"ab\")+holder.read(\"c\");return second;"),
        ("reusable-saved","read:Function<int>;marker:int;","return {read:()=>size(view),marker:1};","const holder=save(first);const action=holder.read;const total=action()+action();return second;"),
        ("array-two-views","read:Once<Function<Window>>[];marker:int;","return {read:[()=>view],marker:1};","const holder=save(second);const other=save(first);holder.read.push(other.read.pop()!);const action=holder.read.shift()!;return action();"),
        ("nullable-branch-store","read:Once<Function<Window>>|undefined;marker:int;",plain,"let holder:Holder={read:undefined,marker:1};if(marker()>0){holder.read=()=>second;}const action=holder.read;if(action!==undefined){return action();}return first;"),
        ("nullable-loop-store","read:Once<Function<Window>>|undefined;marker:int;",plain,"let holder:Holder={read:undefined,marker:1};for(let i=0;i<1;i+=1){holder.read=()=>second;break;}const action=holder.read;if(action!==undefined){return action();}return first;"),
        ("nullable-present-absent-present","read:Once<Function<Window>>|undefined;marker:int;",plain,"let holder=save(first);holder.read=undefined;holder.read=()=>second;return (holder.read!)();"),
        ("nullable-two-alternatives-branch","read:Once<Function<Window>>|undefined;marker:int;",plain,"let holder=save(first);if(marker()>0){holder.read=()=>second;}return (holder.read!)();"),
        ("nullable-absent-then-present","read:Once<Function<Window>>|undefined;marker:int;","return {read:undefined,marker:1};","let holder=save(first);holder.read=()=>second;return (holder.read!)();"),
        ("nullable-conditional","read:Once<Function<Window>>|undefined;marker:int;","return {read:true?()=>view:undefined,marker:1};","const holder=save(second);return (holder.read!)();"),
        ("field-replacement-in-branch",once,plain,"let holder=save(first);if(holder.marker>0){holder.read=()=>second;}return holder.read();"),
        ("argument-owned-view","read:Once<Function<Window,[Owned<Window>]>>;marker:int;","return {read:(other:Owned<Window>)=>other,marker:1};","const holder=save(first);return holder.read(second);"),
    ];
    for (label,holder,save,relay) in cases {
        for mutable in [false,true] {
            let source=format!("type Window={{}};type Holder={{{holder}}};function save(view:Window):Holder{{{save}}}function relay(first:Window,second:Window):Window{{{relay}}}function marker():int{{return 1;}}function size(view:ReadOnly<Window>):int{{return 2;}}function main(){{}}");
            assert_stored_source_access(&source,label,mutable);
        }
    }
}

#[test]
fn native_view_capturing_callback_returns_its_argument() {
    let source="type Window={};type Holder={read:Once<Function<Window,[Owned<Window>]>>;marker:int;};function save(view:Window):Holder{return {read:(other:Owned<Window>)=>{const kept=view;return other;},marker:1};}function relay(first:Window,second:Window):Window{const holder=save(first);return holder.read(second);}function main(){}";
    for mutable in [false,true] {
        let source=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&source).unwrap(),source);
        let mut parsed=crate::syntax::parse(&source).unwrap();
        parsed.records[0].native_type=Some(if mutable{NativeTypeId::ByteViewMut}else{NativeTypeId::ByteView});
        let checked=crate::analysis::analyze(parsed).unwrap();
        let relay=checked.functions.iter().position(|function|function.name=="relay").unwrap();
        assert!(checked.functions[relay].native_returns.as_ref().unwrap().iter().all(|loan|loan.root==1));
        assert_record_execution(&checked,relay,&format!("captured-argument-{mutable}"));
    }
}

#[test]
fn native_view_capturing_argument_matrix() {
    for (label,contract,body,call) in [
        ("later-argument", "Function<Window,[int,Owned<Window>]>", "(n:int,other:Owned<Window>)=>{size(view);return other;}", "return holder.read(1,second);"),
        ("nested-argument", "Once<Function<Window,[Owned<Envelope>]>>", "(other:Owned<Envelope>)=>{const kept=view;return other.data;}", "return holder.read({data:second});"),
        ("read", "Function<Window,[Owned<Window>]>", "(other:Owned<Window>)=>{size(view);return other;}", "return holder.read(second);"),
        ("mutate", "Function<Window,[Owned<Window>]>", "(other:Owned<Window>)=>{calls+=1;size(view);return other;}", "return holder.read(second);"),
        ("joined", "Once<Function<Window,[Owned<Window>,boolean]>>", "(other:Owned<Window>,choose:boolean)=>{if(choose){return view;}return other;}", "return holder.read(second,false);"),
        ("throw", "Once<Function<Window,[Owned<Window>,boolean]>>", "(other:Owned<Window>,fail:boolean)=>{const kept=view;if(fail){throw 1;}return other;}", "return holder.read(second,false);"),
        ("alternatives", "Once<Function<Window,[Owned<Window>]>>", "flag()?(other:Owned<Window>)=>{const kept=view;return other;}:(other:Owned<Window>)=>{return view;}", "return holder.read(second);"),
    ] {
        for mutable in [false,true] {
            let source=format!("type Window={{}};type Envelope={{data:Window;}};type Holder={{read:{contract};marker:int;}};function size(view:ReadOnly<Window>):int{{return 2;}}function flag():boolean{{return true;}}function save(view:Window):Holder{{let calls=0;return {{read:{body},marker:1}};}}function relay(first:Window,second:Window):Window{{let holder=save(first);{call}}}function main(){{}}");
            assert_stored_source_access(&source,&format!("capturing-argument-{label}-{mutable}"),mutable);
        }
    }
}

#[test]
fn native_view_recursive_construction_and_mutation_execute() {
    for (label, body) in [
        ("literal", "const leaf=save(second);const root:Holder={view:first,children:[leaf]};const child=root.children.pop()!;return child.view;"),
        ("replace", "const root=save(first);root.view=second;return root.view;"),
        ("append", "const root=save(first);root.children.push(save(second));const child=root.children.pop()!;return child.view;"),
        ("forward", "const leaf=save(second);const root:Holder={view:first,children:[leaf]};const moved=identity(root);const child=moved.children.pop()!;return child.view;"),
        ("outer-sibling", "const pair:Pair={chain:save(first),other:second};return outside(pair);"),
        ("borrow-root", "const root=save(second);inspect(root);return root.view;"),
        ("borrow-nested", "const pair:Pair={chain:save(first),other:second};inspect(pair.chain);return outside(pair);"),
    ] {
        for mutable in [false, true] {
            let source = format!("type Window={{}};type Holder={{view:Window;children:Holder[];}};type Pair={{chain:Holder;other:Window;}};function inspect(input:ReadOnly<Holder>):int{{return input.children.length;}}function outside(pair:Pair):Window{{return pair.other;}}function save(view:Window):Holder{{return {{view:view,children:[]}};}}function identity(input:Holder):Holder{{return input;}}function relay(first:Window,second:Window):Window{{{body}}}function main(){{}}");
            let formatted = crate::formatting::format(&source).unwrap();
            assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
            let mut parsed = crate::syntax::parse(&formatted).unwrap();
            parsed.records[0].native_type = Some(if mutable { NativeTypeId::ByteViewMut } else { NativeTypeId::ByteView });
            let checked = crate::analysis::analyze(parsed).unwrap_or_else(|error| panic!("{label}/{mutable}: {error:?}"));
            let relay = checked.functions.iter().position(|function| function.name == "relay").unwrap();
            assert_record_execution(&checked, relay, &format!("recursive-{label}-{mutable}"));
        }
    }
}

#[test]
fn native_view_nullable_recursive_links_preserve_all_owners() {
    let source="type Window={};type Holder={view:Window;next?:Holder;};function observe(input:ReadOnly<Holder|undefined>):int{return input===undefined?0:1;}function wrap(input:Holder|undefined):Holder|undefined{return input;}function relay(first:Window,second:Window):Window{const root:Holder={view:first,next:{view:second}};const seen=observe(root.next);const child=wrap(root.next)!;return child.view;}function main(){}";
    for mutable in [false,true] {
        assert_stored_source_access(source,&format!("nullable-recursive-{mutable}"),mutable);
    }
}
