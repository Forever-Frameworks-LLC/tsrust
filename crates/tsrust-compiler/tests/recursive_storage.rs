mod support;
use support::{execute, success};

#[test]
fn batched_mutual_forests_preserve_root_order_nested_rows_and_independence() {
    execute(
        r#"type A = { value: int; payload: B; rows: A[][]; };
type B = { value: int; back: A[]; };
function main() {
 const values: A[] = [
  { value: 1, payload: { value: 2, back: [] }, rows: [[], []] },
  { value: 3, payload: { value: 4, back: [
   { value: 5, payload: { value: 6, back: [
    { value: 7, payload: { value: 8, back: [] }, rows: [] }
   ] }, rows: [] }
  ] }, rows: [[{ value: 9, payload: { value: 10, back: [] }, rows: [] }]] },
  { value: 11, payload: { value: 12, back: [] }, rows: [] }
 ];
 const copy = clone(values); values[0].value = 99;
 values[1].payload.back[0].payload.back[0].value = 88;
 values[1].rows[0][0].value = 77;
 console.log(copy[0].value); console.log(copy[0].rows.length);
 console.log(copy[0].rows[1].length); console.log(copy[1].value);
 console.log(copy[1].payload.back[0].payload.back[0].value);
 console.log(copy[1].rows[0][0].value); console.log(copy[2].value);
 const members: B[] = [clone(values[0].payload), clone(values[1].payload)];
 const saved = clone(members); members[1].back[0].value = 66;
 console.log(saved[0].value); console.log(saved[1].value);
 console.log(saved[1].back[0].value);
}"#,
        "1\n2\n0\n3\n7\n9\n11\n2\n4\n5\n",
        None,
    );
}

#[test]
fn shallow_and_deep_clone_chunks_keep_every_value_and_independent_storage() {
    let source = support::format_round_trip(include_str!("../../../benchmarks/runtime/workloads/recursive_clone.tsr"))
        .replace(support::format_round_trip("function main() {}").trim(), "function main() { const rows = workload(5, 7); for (const row of rows) { console.log(row.length); for (const value of row) { console.log(value); } } }");
    assert!(source.contains("const rows = workload(5, 7);"), "workload entry point changed");
    execute(
        &source,
        concat!(
            "6\n9\n8\n7\n0\n7\n8\n",
            "4\n1\n10\n11\n12\n",
            "6\n9\n2\n13\n14\n15\n16\n",
            "3\n11\n10\n3\n",
            "5\n13\n12\n11\n4\n19\n",
            "6\n2000\n3007\n7\n0\n7\n8\n",
            "4\n2001\n3007\n11\n12\n",
            "6\n2002\n3007\n13\n14\n15\n16\n",
            "3\n2003\n3007\n3\n",
            "5\n2004\n3007\n11\n4\n19\n"
        ),
        None,
    );
}

#[test]
fn shared_inline_schemas_generate_compact_recursive_helpers() {
    fn schema(depth: usize) -> String {
        let mut source = format!("type R0 = {{ value: int; next: R{depth}[]; }};\n");
        for index in 1..=depth {
            source.push_str(&format!(
                "type R{index} = {{ left: R{}; right: R{}; }};\n",
                index - 1,
                index - 1
            ));
        }
        source.push_str("function main() {}");
        source
    }
    let small = tsrust_compiler::check(&schema(8)).unwrap().emit_rust();
    let large = tsrust_compiler::check(&schema(16)).unwrap().emit_rust();
    assert!(
        large.len() < small.len() * 3,
        "shared schema expansion must follow declarations, not unfolded storage"
    );
    assert!(large.len() < 100_000);
    assert!(!large.contains("unsafe"));
}

#[test]
fn deep_automatic_clone_and_cleanup_preserve_field_moves_in_debug_and_release() {
    let source = r#"type Tree = { value: int; children: Tree[]; };
function main() {
 let tree: Tree = { value: -1, children: [] }; let i = 0;
 while (i < 100000) { tree = { value: i, children: [tree] }; i = i + 1; }
 const saved = clone(tree); tree.value = 7;
 const children = tree.children; tree.children = [];
 console.log(tree.value); console.log(children[0].value);
 console.log(saved.value); console.log(saved.children[0].value);
 const forest: Tree[] = [clone(saved), { value: -2, children: [] }];
 const forest_copy = clone(forest); forest[0].value = 8;
 console.log(forest_copy[0].value); console.log(forest_copy[0].children[0].value);
 console.log(forest_copy[1].value);
}"#;
    let rust = execute(source, "7\n99998\n99999\n99998\n99999\n99998\n-2\n", None);
    assert!(!rust.contains("impl Drop for Record0"));
    assert!(!rust.contains("unsafe"));
    let dir = support::TempDirectory::new();
    let path = dir.0.join("deep.rs");
    let binary = dir.0.join("deep");
    std::fs::write(&path, rust).unwrap();
    success(&support::run(
        support::rustc()
            .arg("--edition=2024")
            .arg("-C")
            .arg("opt-level=3")
            .arg(&path)
            .arg("-o")
            .arg(&binary),
        30,
    ));
    let output = support::run(&mut std::process::Command::new(binary), 10);
    success(&output);
    assert_eq!(output.stdout, b"7\n99998\n99999\n99998\n99999\n99998\n-2\n");
}

#[test]
fn mutual_components_direct_fields_nested_rows_and_separate_components_clone_in_order() {
    execute(
        r#"type A = { name: string; payload: B; others: A[][]; };
type B = { value: int; back: A[]; };
type C = { value: int; next: C[]; };
type Holder = { a: A; c: C; };
function main() {
 const a: A = { name: "root", payload: { value: 1, back: [
  { name: "first", payload: { value: 2, back: [] }, others: [] },
  { name: "second", payload: { value: 3, back: [] }, others: [] }
 ] }, others: [[], [
  { name: "row first", payload: { value: 4, back: [] }, others: [] },
  { name: "row second", payload: { value: 5, back: [] }, others: [] }
 ]] };
 const holder: Holder = { a: a, c: { value: 6, next: [{ value: 7, next: [] }] } };
 const copy = clone(holder); holder.a.payload.back[0].name = "changed";
 holder.a.others[1][1].payload.value = 99; holder.c.next[0].value = 88;
 const moved = holder.a.payload; holder.a.payload = { value: 0, back: [] };
 console.log(moved.back[0].name); console.log(copy.a.payload.back[0].name);
 console.log(copy.a.payload.back[1].name); console.log(copy.a.others[1][0].name);
 console.log(copy.a.others[1][1].payload.value); console.log(copy.c.next[0].value);
 const row = holder.a.others.remove(1); console.log(row[1].payload.value);
}"#,
        "changed\nfirst\nsecond\nrow first\n5\n7\n99\n",
        None,
    );
}

#[test]
fn recursive_growth_preserves_small_and_large_storage_and_owned_clone_resources() {
    let pads = (0..140)
        .map(|index| format!("pad{index}: int;"))
        .collect::<String>();
    let source = format!(
        "type Tiny = {{ value: int; children: Tiny[]; }};\n\
         type Medium = {{ value: int; label: string; scores: int[]; children: Medium[]; }};\n\
         type Large = {{ value: int; label: string; children: Large[]; {pads} }};\n\
         function main() {{}}"
    );
    let rust = tsrust_compiler::check(&source)
        .unwrap()
        .emit_rust()
        .replace("fn main()", "fn source_main()");
    let pad_values = (0..140)
        .map(|index| format!("field{}: {index},", index + 3))
        .collect::<String>();
    let pad_checks = (0..140)
        .map(|index| format!("assert_eq!(saved[0].field{}, {index});", index + 3))
        .collect::<String>();
    let model = r#"
fn main() {
 assert!(std::mem::size_of::<Record0>() <= 64);
 assert!((65..=128).contains(&std::mem::size_of::<Record1>()));
 assert!(std::mem::size_of::<Record2>() > 1024);
 {
  let mut values = RecordArray0::from([]); let mut model = Vec::new();
  for i in 0..80 {
   values.push(Record0 { field0: i, field1: RecordArray0::from([]) });
   model.push(Record0 { field0: i, field1: RecordArray0::from([]) });
   if i < 4 { assert_eq!(values.capacity(), model.capacity()); }
   assert_eq!(values.len(), model.len());
   for (actual, expected) in values.iter().zip(&model) { assert_eq!(actual.field0, expected.field0); }
  }
  let capacity = values.capacity();
  while !model.is_empty() { assert_eq!(values.remove(0).field0, model.remove(0).field0); }
  assert_eq!(values.capacity(), capacity);
  values.push(Record0 { field0: -1, field1: RecordArray0::from([]) });
  assert_eq!(values.capacity(), capacity); assert_eq!(values[0].field0, -1);
 }
 let label = String::from("mixed Label".to_owned());
 assert!(matches!(&label, String::Owned(_)));
 {
  let make = |i| Record1 { field0: i, field1: label.clone(), field2: NumericArray::from([i, 1, 2, 3, 4]), field3: RecordArray1::from([]) };
  let mut values = RecordArray1::from([]); let mut model = Vec::new();
  for i in 0..80 {
   values.push(make(i)); model.push(make(i));
   if i < 4 { assert_eq!(values.capacity(), model.capacity()); }
   for (actual, expected) in values.iter().zip(&model) {
    assert_eq!(actual.field0, expected.field0);
    assert_eq!(actual.field1.as_ref(), expected.field1.as_ref());
    assert_eq!(actual.field2.as_slice(), expected.field2.as_slice());
   }
  }
  assert_eq!(label.as_ref(), "mixed Label");
  let saved = values.clone(); assert_ne!(saved[0].field1.as_ptr(), values[0].field1.as_ptr());
  values[0].field0 = -1; values[0].field2.as_mut_slice()[0] = -1; values[0].field1.to_mut().push('!'); assert_eq!(saved[0].field1.as_ref(), "mixed Label");
  assert_eq!(saved[0].field0, 0); assert_eq!(saved[0].field2.as_slice()[0], 0);
  drop(saved); assert_eq!(label.as_ref(), "mixed Label");
  model[0].field0 = -1; model[0].field2.as_mut_slice()[0] = -1;
  while !model.is_empty() {
   let actual = values.remove(0); let expected = model.remove(0);
   assert_eq!(actual.field0, expected.field0); assert_eq!(actual.field2.as_slice(), expected.field2.as_slice());
  }
  assert_eq!(label.as_ref(), "mixed Label");
 }
 {
  let make = |i| Record2 { field0: i, field1: label.clone(), field2: RecordArray2::from([]), PAD_VALUES };
  let mut values = RecordArray2::from([]); let mut model = Vec::new();
  for i in 0..80 {
   values.push(make(i)); model.push(make(i));
   assert_eq!(values.capacity(), model.capacity());
   assert_eq!(values.len(), model.len());
  }
  let saved = values.clone(); values[0].field0 = -1;
  assert_eq!(saved[0].field0, 0); PAD_CHECKS
  assert_eq!(label.as_ref(), "mixed Label");
  drop(values); assert_eq!(label.as_ref(), "mixed Label");
  drop(saved); assert_eq!(label.as_ref(), "mixed Label");
  drop(model); assert_eq!(label.as_ref(), "mixed Label");
 }
}
"#
    .replace("PAD_VALUES", &pad_values)
    .replace("PAD_CHECKS", &pad_checks);
    let dir = support::TempDirectory::new();
    let path = dir.0.join("growth-models.rs");
    std::fs::write(&path, format!("{rust}\n{model}")).unwrap();
    for optimization in [0, 3] {
        let binary = dir.0.join(format!("growth-models-{optimization}"));
        success(&support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={optimization}"))
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            30,
        ));
        success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}
