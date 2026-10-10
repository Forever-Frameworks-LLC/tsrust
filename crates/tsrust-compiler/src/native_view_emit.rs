//! Native byte-view storage is a real slice lifetime, including generic wrappers.
use crate::{analysis::CheckedProgram, native_return_borrows::BorrowAccess, syntax::Type};

/// Reverse reachability makes nested storage queries constant-time, including cycles.
#[derive(Default)]
pub(super) struct Shapes {
    records: Vec<bool>,
    unions: Vec<bool>,
}

impl Shapes {
    pub(super) fn new(program: &CheckedProgram) -> Self {
        let count = program.records.len();
        let seeds = program
            .records
            .iter()
            .enumerate()
            .filter_map(|(id, record)| {
                record
                    .native_type
                    .and_then(|native| crate::native_types::declaration(native).borrow_access())
                    .map(|_| id)
            })
            .collect::<Vec<_>>();
        if seeds.is_empty() {
            return Self::default();
        }
        let mut reverse = vec![Vec::new(); count + program.unions.len()];
        for (id, record) in program.records.iter().enumerate() {
            let mut edge = |ty| {
                if let Some(child) = Self::node(ty, count) {
                    reverse[child].push(id);
                }
            };
            for field in &record.fields {
                edge(field.ty.expect("checked field"));
            }
            if let Some(map) = &record.map_definition {
                edge(map.key);
                edge(map.value);
            }
            if let Some(set) = &record.set_definition {
                edge(set.key);
            }
        }
        for (id, union) in program.unions.iter().enumerate() {
            for &member in &union.members {
                if let Some(child) = Self::node(member, count) {
                    reverse[child].push(count + id);
                }
            }
        }
        let mut found = vec![false; reverse.len()];
        let mut pending = seeds;
        while let Some(id) = pending.pop() {
            if std::mem::replace(&mut found[id], true) {
                continue;
            }
            pending.extend(reverse[id].iter().copied());
        }
        Self {
            records: found[..count].to_vec(),
            unions: found[count..].to_vec(),
        }
    }

    fn node(ty: Type, count: usize) -> Option<usize> {
        match ty {
            Type::Record(id) | Type::Map(id) | Type::Set(id) => Some(id),
            Type::Union(id) | Type::BorrowedUnion(id) => Some(count + id.index()),
            Type::Array(element) => Self::node(element.with_depth(0).ty(), count),
            _ => None,
        }
    }

    pub(super) fn contains(&self, ty: Type) -> bool {
        Self::node(ty, self.records.len()).is_some_and(|id| {
            if id < self.records.len() {
                self.records[id]
            } else {
                self.unions
                    .get(id - self.records.len())
                    .copied()
                    .unwrap_or(false)
            }
        })
    }
}

pub(super) fn access(program: &CheckedProgram, id: usize) -> Option<BorrowAccess> {
    program.records[id]
        .native_type
        .and_then(|native| crate::native_types::declaration(native).borrow_access())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_storage_reachability_handles_cycles_and_array_ranks() {
        let mut parsed = crate::syntax::parse("type Window={};type Ring={next:Ring[];view:Window[][];optional:Window|null;};type Plain={value:int;};function main(){}").unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked = crate::analysis::analyze(parsed).unwrap();
        let shapes = Shapes::new(&checked);
        let ring = checked
            .records
            .iter()
            .position(|record| record.name.text == "Ring")
            .unwrap();
        let plain = checked
            .records
            .iter()
            .position(|record| record.name.text == "Plain")
            .unwrap();
        assert!(shapes.contains(Type::Record(ring)));
        for field in &checked.records[ring].fields {
            assert!(shapes.contains(field.ty.unwrap()));
        }
        assert!(!shapes.contains(Type::Record(plain)));
        let ordinary = crate::analysis::analyze(
            crate::syntax::parse("type Plain={value:int;};function main(){}").unwrap(),
        )
        .unwrap();
        let shapes = Shapes::new(&ordinary);
        assert!(shapes.records.is_empty() && shapes.unions.is_empty());
        assert!(!shapes.contains(Type::Record(0)));
    }

    #[test]
    fn native_view_records_and_wrappers_emit_real_slice_lifetimes() {
        let source = "type ReadWindow={};type WriteWindow={};type Shelf={item:ReadWindow;items:ReadWindow[];};type Outcome=Result<ReadWindow,int>;type Carrier={result:Outcome;};function inspect(input:ReadOnly<Outcome>):void{}function main(){}";
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
        for record in &mut parsed.records {
            record.native_type = match record.name.text.as_str() {
                "ReadWindow" => Some(crate::native_signatures::NativeTypeId::ByteView),
                "WriteWindow" => Some(crate::native_signatures::NativeTypeId::ByteViewMut),
                _ => None,
            };
        }
        let checked = crate::analysis::analyze(parsed).unwrap();
        let record = |name: &str| {
            checked
                .records
                .iter()
                .position(|record| record.name.text == name)
                .unwrap()
        };
        let read = record("ReadWindow");
        let write = record("WriteWindow");
        let shelf = record("Shelf");
        let carrier = record("Carrier");
        let outcome = checked.records[carrier].fields[0]
            .ty
            .unwrap()
            .union()
            .unwrap();
        let members = &checked.unions[outcome.index()].members;
        let [ok, err] = crate::result_identity::variants(&checked.records, members).unwrap();
        let ok_tag = members
            .iter()
            .position(|member| *member == Type::Record(ok))
            .unwrap();
        let err_tag = members
            .iter()
            .position(|member| *member == Type::Record(err))
            .unwrap();
        let rust = crate::emit::emit(&checked);
        assert!(
            rust.contains(&format!("struct Record{read}<'a> {{ bytes: &'a [u8] }}")),
            "borrowed native representation missing"
        );
        assert!(
            rust.contains(&format!(
                "struct Record{write}<'a> {{ bytes: &'a mut [u8] }}"
            )),
            "exclusive native representation missing"
        );
        let rust = rust.replace("fn main()", "fn unused_source_main()")
            + &format!(
                r#"
fn main() {{
    let mut owner = vec![0u8, 65, 66, 255];
    let pointer = owner.as_ptr();
    {{
        let shelf = Record{shelf} {{
            field0: Record{read} {{ bytes: &owner[..2] }},
            field1: RecordArray{read}::from([Record{read} {{ bytes: &owner[2..] }}]),
        }};
        assert_eq!(shelf.field0.bytes.as_ptr(), pointer);
        assert_eq!(shelf.field1[0].bytes, [66, 255]);
    }}
    {{
        let mutable = Record{write} {{ bytes: &mut owner[1..3] }};
        mutable.bytes[0] = 7;
    }}
    assert_eq!(owner, [0, 7, 66, 255]);
    let outcome: Union{union}<Record{ok}<Record{read}<'_>>, Record{err}> =
        Union{union}::V{ok_tag}(Record{ok} {{ field0: Record{read} {{ bytes: &owner[1..] }} }});
    let carrier = Record{carrier} {{ field0: outcome }};
    match carrier.field0.tsr_view() {{
        ViewUnion{union}::V{ok_tag}(value) => assert_eq!(value.field0.bytes, [7, 66, 255]),
        ViewUnion{union}::V{err_tag}(_) => panic!("unexpected error"),
    }}
}}
"#,
                union = outcome.index()
            );
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "tsrust-native-view-abi-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let input = directory.join("views.rs");
        std::fs::write(&input, rust).unwrap();
        for level in [0, 3] {
            let output = directory.join(format!("views-{level}"));
            let compiled = std::process::Command::new("rustup")
                .args([
                    "run",
                    crate::RUST_BACKEND_VERSION,
                    "rustc",
                    "--edition=2024",
                    &format!("-Copt-level={level}"),
                ])
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
            let run = std::process::Command::new(output).output().unwrap();
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
    #[test]
    fn captured_native_view_return_preserves_outer_origin() {
        let source = "type Window={};function captured(unused:int,input:Window):Window{const alias=input;const callback=():Window=>alias;return callback();}function main(){}";
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let checked = crate::analysis::analyze(parsed).unwrap();
        let function = checked.functions.iter().find(|function| function.name == "captured").unwrap();
        let origins = function.native_returns.as_ref().expect("captured return must retain an input lifetime");
        assert_eq!(origins.len(), 1);
        assert_eq!(origins[0].root, 1);
    }
    #[test]
    fn returned_native_views_compile_with_distinct_input_lifetimes() {
        let source="type Window={};type WriteWindow={};type Shelf={item:Window;};type Pair={one:Window;two:Window;};type Outcome=Result<Window,int>;function capturedPair(input:Pair):Window{const callback=():Window=>input.one;return callback();}function capturedWrite(input:WriteWindow):WriteWindow{const callback=():WriteWindow=>input;return callback();}function captured(input:Window):Window{const callback=():Window=>input;return callback();}function identity(input:Window):Window{return input;}function relay(input:Window):Window{return identity(input);}function arrowChoose(left:Window,right:Window,flag:boolean):Window{const callback=(first:Window,second:Window):Window=>{if(flag){return first;}return second;};return callback(left,right);}function arrowRelay(input:Window):Window{const identity=(value:Window):Window=>value;return identity(input);}function savedRelay(input:Window):Window{const saved=identity;return saved(input);}function outcomeIdentity(input:Outcome):Outcome{return input;}function pairFirst(input:Pair):Window{return input.one;}function unwrap(input:Shelf):Window{return input.item;}function writeIdentity(input:WriteWindow):WriteWindow{return input;}function arrayIdentity(input:Window[]):Window[]{return input;}function first(left:Window,right:Window):Window{return left;}function choose(left:Window,right:Window,flag:boolean):Window{if(flag){return left;}return right;}function main(){}";
        let formatted=crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(),formatted);
        let mut parsed=crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type=Some(crate::native_signatures::NativeTypeId::ByteView);
        parsed.records[1].native_type=Some(crate::native_signatures::NativeTypeId::ByteViewMut);
        let checked=crate::analysis::analyze(parsed).unwrap();
        let captured_pair=checked.functions.iter().position(|function|function.name=="capturedPair").unwrap();
        let captured_write=checked.functions.iter().position(|function|function.name=="capturedWrite").unwrap();
        let captured=checked.functions.iter().position(|function|function.name=="captured").unwrap();
        let relay=checked.functions.iter().position(|function|function.name=="relay").unwrap();
        let arrow_choose=checked.functions.iter().position(|function|function.name=="arrowChoose").unwrap();
        let arrow_relay=checked.functions.iter().position(|function|function.name=="arrowRelay").unwrap();
        let saved_relay=checked.functions.iter().position(|function|function.name=="savedRelay").unwrap();
        let outcome_identity=checked.functions.iter().position(|function|function.name=="outcomeIdentity").unwrap();
        let outcome=checked.functions[outcome_identity].result.union().unwrap();
        let members=&checked.unions[outcome.index()].members;
        let [ok,_]=crate::result_identity::variants(&checked.records,members).unwrap();
        let ok_tag=members.iter().position(|member|*member==Type::Record(ok)).unwrap();
        let outcome_id=outcome.index();
        let pair_first=checked.functions.iter().position(|function|function.name=="pairFirst").unwrap();
        let unwrap=checked.functions.iter().position(|function|function.name=="unwrap").unwrap();
        let write_identity=checked.functions.iter().position(|function|function.name=="writeIdentity").unwrap();
        let array_identity=checked.functions.iter().position(|function|function.name=="arrayIdentity").unwrap();
        let first=checked.functions.iter().position(|function|function.name=="first").unwrap();
        let choose=checked.functions.iter().position(|function|function.name=="choose").unwrap();
        let mut rust=crate::emit::emit(&checked).replace("fn main()", "fn unused_source_main()");
        let base=rust.clone();
        rust.push_str(&format!(r#"
fn main() {{
    let owner=vec![1u8,2,3];
    assert_eq!(function{captured}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{relay}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{arrow_relay}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    assert_eq!(function{saved_relay}(Record0{{bytes:&owner}}).bytes.as_ptr(),owner.as_ptr());
    let returned;
    {{ let short=vec![9u8]; returned=function{first}(Record0{{bytes:&owner}}, &Record0{{bytes:&short}}); }}
    assert_eq!(returned.bytes.as_ptr(),owner.as_ptr());
    let captured_field;
    {{ let short=vec![8u8];captured_field=function{captured_pair}(Record3{{field0:Record0{{bytes:&owner}},field1:Record0{{bytes:&short}}}}); }}
    assert_eq!(captured_field.bytes.as_ptr(),owner.as_ptr());
    let field;
    {{ let short=vec![8u8];field=function{pair_first}(Record3{{field0:Record0{{bytes:&owner}},field1:Record0{{bytes:&short}}}}); }}
    assert_eq!(field.bytes.as_ptr(),owner.as_ptr());
    let wrapped=function{unwrap}(Record2{{field0:Record0{{bytes:&owner}}}});
    assert_eq!(wrapped.bytes.as_ptr(),owner.as_ptr());
    let values=function{array_identity}(RecordArray0::from([Record0{{bytes:&owner}}]));
    assert_eq!(values[0].bytes.as_ptr(),owner.as_ptr());
    let outcome=function{outcome_identity}(Union{outcome_id}::V{ok_tag}(Record{ok}{{field0:Record0{{bytes:&owner}}}}));
    match outcome {{ Union{outcome_id}::V{ok_tag}(value)=>assert_eq!(value.field0.bytes.as_ptr(),owner.as_ptr()),_=>panic!("unexpected error") }}
    let mut writable=vec![4u8,5];
    {{ let returned=function{write_identity}(Record1{{bytes:&mut writable}});returned.bytes[0]=8; }}
    assert_eq!(writable[0],8);
    {{ let returned=function{captured_write}(Record1{{bytes:&mut writable}});returned.bytes[1]=6; }}
    assert_eq!(writable[1],6);
    let other=vec![7u8];
    assert_eq!(function{arrow_choose}(Record0{{bytes:&owner}},Record0{{bytes:&other}},false).bytes.as_ptr(),other.as_ptr());
    let selected=function{choose}(Record0{{bytes:&owner}},Record0{{bytes:&other}},false);
    assert_eq!(selected.bytes.as_ptr(),other.as_ptr());
}}
"#));
        let directory=std::env::temp_dir().join(format!("tsrust-returned-view-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());
        let input=directory.join("returns.rs");std::fs::write(&input,rust).unwrap();
        let rejected=directory.join("escape.rs");
        std::fs::write(&rejected,base+&format!(r#"
fn main() {{
    let owner=vec![1u8];
    let escaped;
    {{ let short=vec![2u8]; escaped=function{choose}(Record0{{bytes:&owner}},Record0{{bytes:&short}},false); }}
    assert_eq!(escaped.bytes[0],2);
}}
"#)).unwrap();
        let rejected_output=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&rejected).arg("-o").arg(directory.join("escape")).output().unwrap();
        assert!(!rejected_output.status.success(),"escaping selected input unexpectedly compiled");
        assert!(String::from_utf8_lossy(&rejected_output.stderr).contains("does not live long enough"));
        let captured_escape=directory.join("captured-escape.rs");
        std::fs::write(&captured_escape,crate::emit::emit(&checked).replace("fn main()", "fn unused_source_main()")+&format!(r#"
fn main() {{
    let escaped;
    {{ let short=vec![2u8]; escaped=function{captured}(Record0{{bytes:&short}}); }}
    assert_eq!(escaped.bytes[0],2);
}}
"#)).unwrap();
        let captured_rejected=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&captured_escape).arg("-o").arg(directory.join("captured-escape")).output().unwrap();
        assert!(!captured_rejected.status.success(),"captured view escaped its original owner");
        assert!(String::from_utf8_lossy(&captured_rejected.stderr).contains("does not live long enough"));

        for level in [0,3] {
            let output=directory.join(format!("returns-{level}"));
            let compiled=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024",&format!("-Copt-level={level}")]).arg(&input).arg("-o").arg(&output).output().unwrap();
            assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
            assert!(std::process::Command::new(output).status().unwrap().success());
        }
    }

}
