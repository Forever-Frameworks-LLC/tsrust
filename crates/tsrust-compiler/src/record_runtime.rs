//! Iterative ownership operations for recursive record components. The record
//! itself has no Drop implementation, so source field moves remain valid.
use crate::syntax::{ArrayElement, Record, Type};
use std::fmt::Write;

pub(crate) struct RecordRuntime {
    pub(crate) ordered: std::collections::HashSet<usize>,
    groups: Vec<Vec<usize>>,
    membership: Vec<Option<usize>>,
    unions: Vec<crate::unions::UnionType>,
    cloneability: crate::cloneability::Cloneability,
}

impl RecordRuntime {
    pub(crate) fn new(records: &[Record], unions: &[crate::unions::UnionType]) -> Self {
        let edges = records
            .iter()
            .map(|record| {
                record
                    .fields
                    .iter()
                    .flat_map(|field| {
                        let actual = field.ty.expect("checked field");
                        if let Type::Union(id) = actual { unions[id.index()].members.clone() } else { vec![actual] }
                    })
                    .filter_map(|payload| match payload {
                        Type::Record(index) | Type::Array(ArrayElement::Record(index, _)) => Some(index),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut reverse = vec![Vec::new(); records.len()];
        for (parent, children) in edges.iter().enumerate() {
            for &child in children {
                reverse[child].push(parent);
            }
        }
        let mut seen = vec![false; records.len()];
        let mut order = Vec::new();
        for root in 0..records.len() {
            if seen[root] {
                continue;
            }
            seen[root] = true;
            let mut stack = vec![(root, 0)];
            while let Some((node, next)) = stack.last_mut() {
                if let Some(&child) = edges[*node].get(*next) {
                    *next += 1;
                    if !seen[child] {
                        seen[child] = true;
                        stack.push((child, 0));
                    }
                } else {
                    order.push(*node);
                    stack.pop();
                }
            }
        }
        seen.fill(false);
        let mut groups = Vec::new();
        for &root in order.iter().rev() {
            if seen[root] {
                continue;
            }
            seen[root] = true;
            let mut stack = vec![root];
            let mut component = Vec::new();
            while let Some(node) = stack.pop() {
                component.push(node);
                for &child in &reverse[node] {
                    if !seen[child] {
                        seen[child] = true;
                        stack.push(child);
                    }
                }
            }
            if component.len() > 1 || edges[root].contains(&root) {
                component.sort_unstable();
                groups.push(component);
            }
        }
        groups.sort_by_key(|group| group[0]);
        let mut membership = vec![None; records.len()];
        for (group, members) in groups.iter().enumerate() {
            for &member in members {
                membership[member] = Some(group);
            }
        }
        Self {
            ordered: std::collections::HashSet::new(),
            cloneability: crate::cloneability::Cloneability::new(records, |id| {
                unions[id.index()].members.clone()
            }),
            groups,
            membership,
            unions: unions.to_vec(),
        }
    }

    pub(crate) fn cloneable(&self, record: usize) -> bool {
        self.cloneability.record(record)
    }

    pub(crate) fn recursive(&self, record: usize) -> bool {
        self.membership[record].is_some()
    }

    fn seed(&self, group: usize, ty: Type, expression: &str) -> String {
        match ty {
            Type::Union(id) => {
                let arms = self.unions[id.index()]
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, member)| {
                        format!(
                            "Union{}::V{index}(value)=>Union{}::V{index}({})",
                            id.index(),
                            id.index(),
                            self.seed(group, *member, "*value")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("match &({expression}) {{ {arms} }}")
            }
            Type::Record(index) if self.membership[index] == Some(group) => {
                format!("seed_record{group}_{index}(&({expression}))")
            }
            Type::Array(ArrayElement::Record(index, depth))
                if self.membership[index] == Some(group) =>
            {
                if depth == 0 {
                    format!("RecordArray{index}(Vec::with_capacity(({expression}).len()))")
                } else {
                    let variable = format!("seed_row{depth}");
                    format!(
                        "({expression}).iter().map(|{variable}| {}).collect()",
                        self.seed(
                            group,
                            Type::Array(ArrayElement::Record(index, depth - 1)),
                            &variable
                        )
                    )
                }
            }
            _ => format!("({expression}).clone()"),
        }
    }

    fn visit(&self, group: usize, ty: Type, expression: &str, action: Action) -> String {
        match ty {
            Type::Union(id) => {
                let arms = self.unions[id.index()]
                    .members
                    .iter()
                    .enumerate()
                    .filter_map(|(index, member)| {
                        let body = self.visit(group, *member, "*value", action);
                        (!body.is_empty())
                            .then(|| format!("Union{}::V{index}(value)=>{{ {body} }}", id.index()))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if arms.is_empty() {
                    String::new()
                } else {
                    format!(
                        "match {}({expression}) {{ {arms}, _=>{{}} }}\n",
                        if matches!(action, Action::Drop) {
                            "&mut "
                        } else {
                            "&"
                        }
                    )
                }
            }
            Type::Record(index) if self.membership[index] == Some(group) => match action {
                Action::Drop => {
                    format!("drain_record{group}_{index}(&mut ({expression}), stack);\n")
                }
                Action::Enter => format!("enter_record{group}_{index}(&({expression}), tasks);\n"),
                Action::Count => {
                    format!("count += count_record{group}_{index}(&({expression}));\n")
                }
                Action::Shallow => format!(
                    "if !shallow_record{group}_{index}(&({expression})) {{ return false; }}\n"
                ),
            },
            Type::Array(ArrayElement::Record(index, depth))
                if self.membership[index] == Some(group) =>
            {
                if depth == 0 {
                    match action {
                        Action::Drop => format!(
                            "if !({expression}).0.is_empty() {{ stack.extend(std::mem::take(&mut ({expression}).0).into_iter().map(DropNode{group}::R{index})); }}\n"
                        ),
                        Action::Enter => format!(
                            "for child in ({expression}).iter().rev() {{ tasks.push(CloneTask{group}::Enter{index}(child)); }}\n"
                        ),
                        Action::Count => format!("count += ({expression}).len();\n"),
                        Action::Shallow => format!(
                            "if ({expression}).iter().any(|child| count_record{group}_{index}(child) != 0) {{ return false; }}\n"
                        ),
                    }
                } else {
                    let variable = format!("row{depth}");
                    let iterator = match action {
                        Action::Drop => format!("std::mem::take(&mut ({expression}))"),
                        Action::Enter => format!("({expression}).iter().rev()"),
                        Action::Count | Action::Shallow => format!("({expression}).iter()"),
                    };
                    let mutable = if matches!(action, Action::Drop) {
                        "mut "
                    } else {
                        ""
                    };
                    format!(
                        "for {mutable}{variable} in {iterator} {{ {} }}\n",
                        self.visit(
                            group,
                            Type::Array(ArrayElement::Record(index, depth - 1)),
                            &variable,
                            action
                        )
                    )
                }
            }
            _ => String::new(),
        }
    }

    fn fill(&self, group: usize, ty: Type, original: &str, output: &str, shallow: bool) -> String {
        match ty {
            Type::Union(id) => {
                let arms=self.unions[id.index()].members.iter().enumerate().filter_map(|(index,member)|{
                    let body=self.fill(group,*member,"*input","*result",shallow);
                    (!body.is_empty()).then(||format!("(Union{}::V{index}(input),Union{}::V{index}(result))=>{{ {body} }}",id.index(),id.index()))
                }).collect::<Vec<_>>().join(", ");
                if arms.is_empty() {
                    String::new()
                } else {
                    format!("match (&({original}), &mut ({output})) {{ {arms}, _=>{{}} }}\n")
                }
            }
            Type::Record(index) if self.membership[index] == Some(group) => {
                if shallow {
                    format!("fill_shallow_record{group}_{index}(&({original}), &mut ({output}));\n")
                } else {
                    format!(
                        "fill_record{group}_{index}(&({original}), &mut ({output}), children);\n"
                    )
                }
            }
            Type::Array(ArrayElement::Record(index, depth))
                if self.membership[index] == Some(group) =>
            {
                if depth == 0 {
                    if shallow {
                        return format!(
                            "({output}).0.extend(({original}).iter().map(seed_record{group}_{index}));\n"
                        );
                    }
                    let mismatch = if self.groups[group].len() > 1 {
                        " else { unreachable!() }"
                    } else {
                        ""
                    };
                    format!(
                        "for _ in 0..({original}).len() {{ let CloneNode{group}::R{index}(child) = children.next().expect(\"completed clone child\"){mismatch}; ({output}).0.push(child); }}\n"
                    )
                } else {
                    let input = format!("input_row{depth}");
                    let result = format!("output_row{depth}");
                    format!(
                        "for ({input}, {result}) in ({original}).iter().zip(({output}).iter_mut()) {{ {} }}\n",
                        self.fill(
                            group,
                            Type::Array(ArrayElement::Record(index, depth - 1)),
                            &input,
                            &result,
                            shallow
                        )
                    )
                }
            }
            _ => String::new(),
        }
    }

    pub(crate) fn emit(&self, records: &[Record]) -> String {
        let mut code = String::new();
        for (group, members) in self.groups.iter().enumerate() {
            let variants = members
                .iter()
                .map(|index| format!("R{index}(Record{index})"))
                .collect::<Vec<_>>()
                .join(", ");
            let cloneable = members.iter().all(|&index| self.cloneable(index));
            let _ = writeln!(code, "enum DropNode{group} {{ {variants} }}");
            if cloneable {
                let _ = writeln!(
                    code,
                    "enum CloneNode{group} {{ {variants} }}\nenum CloneTask{group}<'a> {{"
                );
                for &index in members {
                    let _ = writeln!(
                        code,
                        "Enter{index}(&'a Record{index}), Finish{index}(&'a Record{index}, Record{index}),"
                    );
                }
                code.push_str("}\n");
            }
            for &index in members {
                let mismatch = if members.len() > 1 {
                    " else { unreachable!() }"
                } else {
                    ""
                };
                let _ = writeln!(code, "struct RecordArray{index}(Vec<Record{index}>);
impl RecordArray{index} {{
#[inline]
fn push(&mut self, value: Record{index}) {{
let bytes = std::mem::size_of::<Record{index}>();
let target = if bytes <= 64 {{ 16 }} else if bytes <= 128 {{ 8 }} else {{ 4 }};
let capacity = self.0.capacity();
if self.0.len() == capacity && capacity >= 4 && capacity < target {{ self.0.reserve_exact(target - self.0.len()); }}
self.0.push(value);
}}
}}
impl std::ops::Deref for RecordArray{index} {{ type Target = Vec<Record{index}>; fn deref(&self) -> &Self::Target {{ &self.0 }} }}
impl std::ops::DerefMut for RecordArray{index} {{ fn deref_mut(&mut self) -> &mut Self::Target {{ &mut self.0 }} }}
impl<const N: usize> From<[Record{index}; N]> for RecordArray{index} {{ fn from(values: [Record{index}; N]) -> Self {{ Self(Vec::from(values)) }} }}
impl Drop for RecordArray{index} {{ fn drop(&mut self) {{ if !self.0.is_empty() {{ drop_record_graph{group}(std::mem::take(&mut self.0).into_iter().map(DropNode{group}::R{index}).collect()); }} }} }}");
                if cloneable {
                    let _ = writeln!(code, "impl Clone for RecordArray{index} {{ fn clone(&self) -> Self {{
if self.0.iter().all(shallow_record{group}_{index}) {{ return Self(self.0.iter().map(clone_shallow_record{group}_{index}).collect()); }}
Self(clone_record_forest{group}(self.0.iter().map(CloneTask{group}::Enter{index})).into_iter().map(|node| {{ let CloneNode{group}::R{index}(record) = node{mismatch}; record }}).collect())
}} }}
impl Clone for Record{index} {{ fn clone(&self) -> Self {{ if shallow_record{group}_{index}(self) {{ return clone_shallow_record{group}_{index}(self); }} let CloneNode{group}::R{index}(result) = clone_record_graph{group}(CloneTask{group}::Enter{index}(self)){mismatch}; result }} }}");
                }
            }
            // Each schema node gets a shallow helper. Expanding direct field trees
            // at every use would make emission exponential for shared schemas.
            for &index in members {
                let fields = &records[index].fields;
                let drain = fields
                    .iter()
                    .enumerate()
                    .map(|(field, value)| {
                        self.visit(
                            group,
                            value.ty.unwrap(),
                            &format!("value.field{field}"),
                            Action::Drop,
                        )
                    })
                    .collect::<String>();
                let _ = writeln!(
                    code,
                    "fn drain_record{group}_{index}(value: &mut Record{index}, stack: &mut Vec<DropNode{group}>) {{ {drain} }}"
                );
                if cloneable {
                    let seed = fields
                        .iter()
                        .enumerate()
                        .map(|(field, value)| {
                            format!(
                                "field{field}: {}",
                                self.seed(
                                    group,
                                    value.ty.unwrap(),
                                    &format!("original.field{field}")
                                )
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let mut count = String::new();
                    let mut fill = String::new();
                    let mut enter = String::new();
                    let mut shallow = String::new();
                    let mut shallow_fill = String::new();
                    for (field, value) in fields.iter().enumerate() {
                        let ty = value.ty.unwrap();
                        count.push_str(&self.visit(
                            group,
                            ty,
                            &format!("original.field{field}"),
                            Action::Count,
                        ));
                        fill.push_str(&self.fill(
                            group,
                            ty,
                            &format!("original.field{field}"),
                            &format!("output.field{field}"),
                            false,
                        ));
                        shallow.push_str(&self.visit(
                            group,
                            ty,
                            &format!("original.field{field}"),
                            Action::Shallow,
                        ));
                        shallow_fill.push_str(&self.fill(
                            group,
                            ty,
                            &format!("original.field{field}"),
                            &format!("output.field{field}"),
                            true,
                        ));
                    }
                    for (field, value) in fields.iter().enumerate().rev() {
                        enter.push_str(&self.visit(
                            group,
                            value.ty.unwrap(),
                            &format!("original.field{field}"),
                            Action::Enter,
                        ));
                    }
                    let seed = if self.ordered.contains(&index) { format!("{seed}, key_order: original.key_order") } else { seed };
                    let _ = writeln!(code, "fn seed_record{group}_{index}(original: &Record{index}) -> Record{index} {{ Record{index} {{ {seed} }} }}
fn shallow_record{group}_{index}(original: &Record{index}) -> bool {{ {shallow} true }}
fn fill_shallow_record{group}_{index}(original: &Record{index}, output: &mut Record{index}) {{ {shallow_fill} }}
fn clone_shallow_record{group}_{index}(original: &Record{index}) -> Record{index} {{ let mut output = seed_record{group}_{index}(original); fill_shallow_record{group}_{index}(original, &mut output); output }}
fn count_record{group}_{index}(original: &Record{index}) -> usize {{ let mut count = 0; {count} count }}
fn enter_record{group}_{index}<'a>(original: &'a Record{index}, tasks: &mut Vec<CloneTask{group}<'a>>) {{ {enter} }}
fn fill_record{group}_{index}(original: &Record{index}, output: &mut Record{index}, children: &mut std::vec::Drain<'_, CloneNode{group}>) {{ {fill} }}");
                }
            }
            let _ = writeln!(
                code,
                "fn drop_record_graph{group}(mut stack: Vec<DropNode{group}>) {{ while let Some(node) = stack.pop() {{ match node {{"
            );
            for &index in members {
                let body = format!("drain_record{group}_{index}(&mut value, &mut stack);");
                let _ = writeln!(
                    code,
                    "DropNode{group}::R{index}(mut value) => {{ {body} }},"
                );
            }
            code.push_str("} } }\n");
            if cloneable {
                let _ = writeln!(
                code,
                "fn clone_record_graph{group}(initial: CloneTask{group}<'_>) -> CloneNode{group} {{ clone_record_forest{group}(std::iter::once(initial)).pop().expect(\"completed record clone\") }}
fn clone_record_forest{group}<'a>(roots: impl ExactSizeIterator<Item = CloneTask{group}<'a>>) -> Vec<CloneNode{group}> {{ let mut tasks = Vec::new(); let mut completed = Vec::with_capacity(roots.len()); for root in roots {{ tasks.push(root); while let Some(task) = tasks.pop() {{ match task {{"
            );
                for &index in members {
                    let seed = self.seed(group, Type::Record(index), "original");
                    let enter = format!("enter_record{group}_{index}(original, &mut tasks);");
                    let count = format!("count += count_record{group}_{index}(original);");
                    let fill = format!(
                        "fill_record{group}_{index}(original, &mut output, &mut children);"
                    );
                    let _ = writeln!(code, "CloneTask{group}::Enter{index}(original) if shallow_record{group}_{index}(original) => {{ completed.push(CloneNode{group}::R{index}(clone_shallow_record{group}_{index}(original))); }},
CloneTask{group}::Enter{index}(original) => {{ let output = {seed}; tasks.push(CloneTask{group}::Finish{index}(original, output)); {enter} }},
CloneTask{group}::Finish{index}(original, mut output) => {{ let mut count = 0_usize; {count} {{ let start = completed.len() - count; let mut children = completed.drain(start..); {fill} }} completed.push(CloneNode{group}::R{index}(output)); }},");
                }
                code.push_str("} } } completed }\n");
            }
        }
        code
    }
}
#[derive(Clone, Copy)]
enum Action {
    Drop,
    Enter,
    Count,
    Shallow,
}

#[cfg(test)]
mod cloneability_tests {
    use super::*;
    use crate::cloneability::tests::record;

    #[test]
    fn noncloneable_recursive_records_emit_drop_without_clone_and_dispose_all_payloads() {
        let records = [record(&[
            Type::Array(ArrayElement::Record(0, 0)),
            Type::StoredCallable(0),
        ])];
        let runtime = RecordRuntime::new(&records, &[]);
        assert!(runtime.recursive(0));
        assert!(!runtime.cloneable(0));
        let emitted = runtime.emit(&records);
        assert!(!emitted.contains("CloneNode"));
        assert!(!emitted.contains("CloneTask"));
        assert!(!emitted.contains("impl Clone"));
        assert!(emitted.contains("impl Drop for RecordArray0"));
        let source = format!(
            r#"#![allow(dead_code, unused_variables, unused_mut)]
use std::sync::atomic::{{AtomicUsize, Ordering}};
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct NoClone;
impl Drop for NoClone {{ fn drop(&mut self) {{ DROPS.fetch_add(1, Ordering::SeqCst); }} }}
struct Record0 {{ field0: RecordArray0, field1: NoClone }}
{emitted}
fn main() {{
    let mut root = Record0 {{ field0: RecordArray0(Vec::new()), field1: NoClone }};
    for _ in 0..10000 {{ root = Record0 {{ field0: RecordArray0(vec![root]), field1: NoClone }}; }}
    drop(root);
    println!("{{}}", DROPS.load(Ordering::SeqCst));
}}
"#
        );
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "tsrust-nonclone-drop-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let input = directory.join("main.rs");
        std::fs::write(&input, source).unwrap();
        for optimization in ["0", "3"] {
            let output = directory.join(format!("program-{optimization}"));
            let compiled = std::process::Command::new("rustup")
                .args([
                    "run",
                    crate::RUST_BACKEND_VERSION,
                    "rustc",
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={optimization}"),
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
            let run = std::process::Command::new(&output).output().unwrap();
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            assert_eq!(run.stdout, b"10001\n");
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
