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
}
