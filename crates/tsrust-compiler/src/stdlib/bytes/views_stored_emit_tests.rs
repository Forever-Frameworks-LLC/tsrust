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
        .position(|record| record.native_type == Some(NativeTypeId::ByteView))
        .unwrap();
    let mut generated = crate::emit::emit(&checked);
    let main = generated.find("fn main()").unwrap();
    let opening = main + generated[main..].find('{').unwrap() + 1;
    generated.insert_str(opening, "carried_probe();");
    let probe = format!(
        "fn carried_probe(){{let owner=vec![7u8,8];let returned;{{let short=vec![1u8];returned=function{relay}(&Record{view}{{bytes:&short}},Record{view}{{bytes:&owner}});}}assert_eq!(returned.bytes.as_ptr(),owner.as_ptr());assert_eq!(returned.bytes,&[7,8]);}}"
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
        "{generated}\n{probe}\nfn invalid_source_escape(){{let first=vec![1u8];let escaped;{{let second=vec![7u8,8];escaped=function{relay}(&Record{view}{{bytes:&first}},Record{view}{{bytes:&second}});}}assert_eq!(escaped.bytes.len(),2);}}"
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
fn native_view_stored_callback_reports_missing_lifetime_storage() {
    let formatted=crate::formatting::format(STORED_SOURCE).unwrap();
    assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
    let mut parsed=crate::syntax::parse(&formatted).unwrap();
    parsed.records.iter_mut().find(|record|record.name.text=="Window").unwrap().native_type=Some(NativeTypeId::ByteView);
    let error=crate::analysis::analyze(parsed).err().expect("stored payload lifetime planning is still guarded");
    assert_eq!(error.code,"native-view-callable-storage");
    assert!(error.message.contains("owns its environment"));
    assert!(error.message.contains("not implemented"));
    assert!(!error.related.is_empty());
}
