//! Local named function facts and hierarchical source outlines.
use super::*;

impl SourceIndex {
    pub(super) fn apply_local_function_result(
        &mut self,
        span: Span,
        name: &str,
        result: Option<String>,
    ) {
        let Some(result) = result else {
            return;
        };
        if let Some(symbol) = self.symbols.iter_mut().find(|symbol| {
            symbol.span == span && symbol.name == name && symbol.signature.starts_with("function ")
        }) {
            if symbol.ty.is_empty() {
                symbol.ty = result.clone();
                symbol.signature.push_str(&format!(": {result}"));
            }
        }
    }

    /// A top-level function without a written result type shows its checked one.
    pub(super) fn apply_function_result(&mut self, name: &str, result: &str) {
        if let Some(symbol) = self.symbols.iter_mut().find(|symbol| {
            symbol.scope == 0 && symbol.name == name && symbol.signature.starts_with("function ") && symbol.ty.is_empty()
        }) {
            symbol.ty = result.to_owned();
            symbol.signature.push_str(&format!(": {result}"));
        }
    }

    pub(crate) fn apply_project_local_functions(
        &mut self,
        graph: &tsrust_compiler::project::ProjectGraph,
        current: &tsrust_compiler::project::SourceId,
        facts: &[tsrust_compiler::project::ProjectLocalFunctionInfo],
        dependencies: &std::collections::BTreeMap<tsrust_compiler::project::SourceId, SourceIndex>,
    ) {
        for fact in facts
            .iter()
            .filter(|fact| fact.declaration.source == *current)
        {
            let result = fact
                .result
                .as_ref()
                .and_then(|ty| self.project_type_spelling(graph, current, ty, dependencies));
            self.apply_local_function_result(fact.declaration.span, &fact.name, result);
        }
    }

    pub(crate) fn apply_project_function_results(
        &mut self,
        analysis: &tsrust_compiler::project::ProjectAnalysis,
        current: &tsrust_compiler::project::SourceId,
        dependencies: &std::collections::BTreeMap<tsrust_compiler::project::SourceId, SourceIndex>,
    ) {
        let functions = self.symbols.iter().filter(|symbol| symbol.scope == 0
            && symbol.ty.is_empty() && symbol.signature.starts_with("function "))
            .map(|symbol| symbol.name.clone()).collect::<Vec<_>>();
        for name in functions {
            if let Some(result) = analysis.checked_function_result(current, &name)
                .and_then(|ty| self.project_type_spelling(&analysis.graph, current, &ty, dependencies))
                && result != "void"
            {
                self.apply_function_result(&name, &result);
            }
        }
    }

    pub(super) fn document_outline(&self) -> Value {
        let eligible = |symbol: &Symbol| {
            symbol.owner.is_none()
                && ((symbol.scope == 0 && matches!(symbol.kind, 12 | 23 | 13 | 26))
                    || symbol.signature.starts_with("function "))
        };
        let mut children = vec![Vec::new(); self.symbols.len()];
        let mut roots = Vec::new();
        let mut declarations = self
            .symbols
            .iter()
            .enumerate()
            .filter(|(_, symbol)| eligible(symbol))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        declarations.sort_by_key(|&id| {
            (
                self.symbols[id].extent.start,
                std::cmp::Reverse(self.symbols[id].extent.end),
            )
        });
        let mut parents: Vec<usize> = Vec::new();
        for id in declarations {
            let symbol = &self.symbols[id];
            while parents.last().is_some_and(|&parent| {
                let extent = self.symbols[parent].extent;
                !(extent.start < symbol.extent.start && symbol.extent.end <= extent.end)
            }) {
                parents.pop();
            }
            if let Some(&parent) = parents.last() {
                children[parent].push(id);
            } else {
                roots.push(id);
            }
            if symbol.signature.starts_with("function ") {
                parents.push(id);
            }
        }
        // Build children before parents without recursive JSON construction.
        let mut pending = roots.iter().map(|&id| (id, false)).collect::<Vec<_>>();
        let mut results = vec![Value::Null; self.symbols.len()];
        while let Some((id, visited)) = pending.pop() {
            if !visited {
                pending.push((id, true));
                pending.extend(children[id].iter().map(|&child| (child, false)));
                continue;
            }
            let symbol = &self.symbols[id];
            let mut value = json!({"name":symbol.name,"detail":symbol.signature,"kind":symbol.kind,
                "range":self.range(symbol.extent),"selectionRange":self.range(symbol.span)});
            if !children[id].is_empty() {
                value["children"] = json!(children[id]
                    .iter()
                    .map(|&child| std::mem::take(&mut results[child]))
                    .collect::<Vec<_>>());
            }
            results[id] = value;
        }
        json!(roots
            .into_iter()
            .map(|id| std::mem::take(&mut results[id]))
            .collect::<Vec<_>>())
    }
}
