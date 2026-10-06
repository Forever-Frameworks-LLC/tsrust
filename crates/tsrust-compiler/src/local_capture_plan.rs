//! Lexical capture dependencies; checked types and access modes are assigned later.
use crate::Span;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BindingId {
    pub owner: usize,
    pub declaration_start: usize,
}
#[derive(Clone, Debug)]
pub(crate) struct Capture {
    pub binding: BindingId,
    pub name: String,
    pub declaration: Span,
    pub usage: Span,
}
#[derive(Default)]
pub(crate) struct Planner {
    captures: Vec<BTreeMap<BindingId, Capture>>,
    edges: Vec<Vec<usize>>,
}
#[derive(Clone, Debug)]
pub(crate) struct Group {
    pub members: Vec<usize>,
    pub captures: Vec<Capture>,
    pub dependencies: Vec<usize>,
}
#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pub node_start: usize,
    pub direct: Vec<Vec<Capture>>,
    pub groups: Vec<Group>,
    pub group_of: Vec<usize>,
}
impl Planner {
    pub fn declaration(&mut self) -> usize {
        let id = self.edges.len();
        self.edges.push(Vec::new());
        self.captures.push(BTreeMap::new());
        id
    }
    pub fn capture(&mut self, function: usize, capture: Capture) {
        let entry = self.captures[function]
            .entry(capture.binding)
            .or_insert_with(|| capture.clone());
        if capture.usage.start < entry.usage.start {
            *entry = capture;
        }
    }
    pub fn dependency(&mut self, function: usize, target: usize) {
        self.edges[function].push(target);
    }
    pub fn has_captures(&self) -> bool {
        self.captures.iter().any(|captures| !captures.is_empty())
    }
    pub fn finish(mut self) -> Plan {
        // Iterative Kosaraju traversal keeps graph depth off the compiler stack.
        let direct = self
            .captures
            .iter()
            .map(|captures| captures.values().cloned().collect())
            .collect();
        let count = self.edges.len();
        let mut reverse = vec![Vec::new(); count];
        for (source, edges) in self.edges.iter_mut().enumerate() {
            edges.sort_unstable();
            edges.dedup();
            for &target in edges.iter() {
                reverse[target].push(source);
            }
        }
        let mut seen = vec![false; count];
        let mut order = Vec::with_capacity(count);
        for root in 0..count {
            if seen[root] {
                continue;
            }
            seen[root] = true;
            let mut stack = vec![(root, 0)];
            while let Some((node, next)) = stack.last_mut() {
                if *next < self.edges[*node].len() {
                    let target = self.edges[*node][*next];
                    *next += 1;
                    if !seen[target] {
                        seen[target] = true;
                        stack.push((target, 0));
                    }
                } else {
                    order.push(*node);
                    stack.pop();
                }
            }
        }
        let mut group_of = vec![usize::MAX; count];
        let mut groups = Vec::new();
        for root in order.into_iter().rev() {
            if group_of[root] != usize::MAX {
                continue;
            }
            let id = groups.len();
            group_of[root] = id;
            let mut pending = vec![root];
            let mut members = Vec::new();
            let mut captures: BTreeMap<BindingId, Capture> = BTreeMap::new();
            while let Some(node) = pending.pop() {
                members.push(node);
                for capture in std::mem::take(&mut self.captures[node]).into_values() {
                    let entry = captures
                        .entry(capture.binding)
                        .or_insert_with(|| capture.clone());
                    if capture.usage.start < entry.usage.start {
                        *entry = capture;
                    }
                }
                for &source in &reverse[node] {
                    if group_of[source] == usize::MAX {
                        group_of[source] = id;
                        pending.push(source);
                    }
                }
            }
            members.sort_unstable();
            groups.push(Group {
                members,
                captures: captures.into_values().collect(),
                dependencies: Vec::new(),
            });
        }
        for (id, group) in groups.iter_mut().enumerate() {
            let mut dependencies = Vec::new();
            for &member in &group.members {
                dependencies.extend(
                    self.edges[member]
                        .iter()
                        .map(|&target| group_of[target])
                        .filter(|&target| target != id),
                );
            }
            dependencies.sort_unstable();
            dependencies.dedup();
            group.dependencies = dependencies;
        }
        Plan {
            node_start: 0,
            direct,
            groups,
            group_of,
        }
    }
}
impl Plan {
    pub fn rebase(&mut self, declarations: usize, owners: usize, source: usize) {
        self.node_start += declarations;
        for captures in &mut self.direct {
            for capture in captures {
                capture.binding.owner += owners;
                capture.binding.declaration_start += source;
                capture.declaration.start += source;
                capture.declaration.end += source;
                capture.usage.start += source;
                capture.usage.end += source;
            }
        }
        for group in &mut self.groups {
            for member in &mut group.members {
                *member += declarations;
            }
            for capture in &mut group.captures {
                capture.binding.owner += owners;
                capture.binding.declaration_start += source;
                capture.declaration.start += source;
                capture.declaration.end += source;
                capture.usage.start += source;
                capture.usage.end += source;
            }
        }
    }
    pub fn pending_diagnostic(&self) -> Option<crate::Diagnostic> {
        let group = self
            .groups
            .iter()
            .find(|group| !group.captures.is_empty())?;
        let capture = self
            .requirements(group.members[0])
            .into_iter()
            .min_by_key(|capture| capture.usage.start)?;
        Some(
            crate::Diagnostic::new(
                "unsupported",
                format!(
                    "Nested function capture of {:?} is pending capture-ownership implementation.",
                    capture.name
                ),
                capture.usage,
            )
            .related(capture.declaration, "Captured binding declared here."),
        )
    }

    pub fn requirements(&self, function: usize) -> Vec<Capture> {
        let mut pending = vec![self.group_of[function - self.node_start]];
        let mut seen = vec![false; self.groups.len()];
        let mut result: BTreeMap<BindingId, Capture> = BTreeMap::new();
        while let Some(group) = pending.pop() {
            if seen[group] {
                continue;
            }
            seen[group] = true;
            for capture in &self.groups[group].captures {
                let entry = result
                    .entry(capture.binding)
                    .or_insert_with(|| capture.clone());
                if capture.usage.start < entry.usage.start {
                    *entry = capture.clone();
                }
            }
            pending.extend(self.groups[group].dependencies.iter().copied());
        }
        result.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn capture(owner: usize, start: usize, usage: usize) -> Capture {
        Capture {
            binding: BindingId {
                owner,
                declaration_start: start,
            },
            name: "value".into(),
            declaration: Span::new(start, start + 5),
            usage: Span::new(usage, usage + 5),
        }
    }
    #[test]
    fn recursive_groups_deduplicate_bindings_and_forward_requirements() {
        let mut planner = Planner::default();
        for _ in 0..4 {
            planner.declaration();
        }
        planner.dependency(0, 1);
        planner.dependency(1, 0);
        planner.dependency(2, 0);
        planner.capture(0, capture(8, 10, 90));
        planner.capture(1, capture(8, 10, 70));
        planner.capture(3, capture(9, 20, 100));
        let plan = planner.finish();
        assert_eq!(plan.groups.len(), 3);
        assert_eq!(plan.group_of[0], plan.group_of[1]);
        assert_ne!(plan.group_of[0], plan.group_of[2]);
        let requirements = plan.requirements(2);
        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].usage.start, 70);
        assert_eq!(plan.groups[plan.group_of[0]].members, vec![0, 1]);
    }
    #[test]
    fn shadowed_spellings_remain_distinct_binding_requirements() {
        let mut planner = Planner::default();
        planner.declaration();
        planner.capture(0, capture(2, 10, 30));
        planner.capture(0, capture(3, 20, 40));
        let plan = planner.finish();
        assert_eq!(plan.requirements(0).len(), 2);
    }
    #[test]
    fn long_dependency_chain_uses_no_recursive_graph_frames() {
        let mut planner = Planner::default();
        for _ in 0..4096 {
            planner.declaration();
        }
        for id in 0..4095 {
            planner.dependency(id, id + 1);
        }
        planner.capture(4095, capture(99, 5, 10));
        let plan = planner.finish();
        assert_eq!(plan.groups.len(), 4096);
        assert_eq!(plan.requirements(0).len(), 1);
    }
    #[test]
    fn module_rebasing_preserves_group_queries_and_binding_locations() {
        let mut planner = Planner::default();
        planner.declaration();
        planner.declaration();
        planner.dependency(0, 1);
        planner.capture(1, capture(2, 10, 30));
        let mut plan = planner.finish();
        plan.rebase(7, 12, 100);
        let captures = plan.requirements(7);
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].binding.owner, 14);
        assert_eq!(plan.direct[1][0].binding.owner, 14);
        assert_eq!(plan.direct[1][0].usage.start, 130);
        assert_eq!(captures[0].binding.declaration_start, 110);
        assert_eq!(captures[0].declaration, Span::new(110, 115));
        assert_eq!(captures[0].usage, Span::new(130, 135));
        assert!(
            plan.groups
                .iter()
                .all(|group| group.members.iter().all(|&member| member >= 7))
        );
        assert_eq!(plan.pending_diagnostic().unwrap().span, Span::new(130, 135));
    }
}
