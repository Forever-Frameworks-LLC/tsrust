//! Imported declared-source assistance, gated by the compiler's validated module graph.
use crate::{
    intelligence::SourceIndex,
    project_diagnostics::{file_uri, source_id},
};
use serde_json::Value;
use std::collections::BTreeMap;
use tsrust_compiler::project::{ProjectGraph, SourceId};

pub struct ProjectIndex {
    indices: BTreeMap<SourceId, SourceIndex>,
    references: BTreeMap<String, Vec<(Value, bool)>>,
}
impl ProjectIndex {
    pub fn checked(analysis: &tsrust_compiler::project::ProjectAnalysis) -> Self {
        let mut index = Self::build(&analysis.graph);
        let labels=analysis.checked_control_labels();
        let facts = analysis.checked_type_facts();
        let local_functions = analysis.checked_local_functions();
        for id in &analysis.graph.order {
            let mut source = index.indices.remove(id).expect("indexed project source");
            source.checked_control_labels=labels.iter().filter(|(span,definition)|span.source==*id && definition.source==*id).map(|(span,definition)|(span.span,definition.span)).collect();
            source.apply_project_checked_facts(&analysis.graph, id, &facts, &index.indices);
            source.apply_project_local_functions(&analysis.graph, id, &local_functions, &index.indices);
            index.indices.insert(id.clone(), source);
        }
        index.refresh_references();
        index
    }
    pub fn new(graph: &ProjectGraph) -> Self {
        let mut index = Self::build(graph);
        index.refresh_references();
        index
    }
    fn build(graph: &ProjectGraph) -> Self {
        let mut indices = BTreeMap::new();
        let mut declarations: BTreeMap<SourceId, SourceIndex> = BTreeMap::new();
        for id in &graph.order {
            let module = &graph.modules[id];
            let mut declaration = SourceIndex::with_entry_help(&module.declaration_source, false);
            let mut index = SourceIndex::with_entry_help(&module.source.text, id == &graph.entry);
            for import in &module.imports {
                if let Some(native) = import.native {
                    index.import_native_names(import.names, native);
                    for binding in &import.bindings {
                        for index in [&mut index, &mut declaration] {
                            index.import_native(
                                native,
                                &binding.exported,
                                &binding.local,
                                binding.span,
                            );
                        }
                    }
                    continue;
                }
                let Some(target) = &import.target else {
                    continue;
                };
                let dependency = &graph.modules[target];
                let foreign = &declarations[target];
                index.import_export_names(import.names, foreign, dependency.exports.keys());
                let Ok(uri) = file_uri(target) else {
                    continue;
                };
                let types = module
                    .imports
                    .iter()
                    .filter(|other| other.target.as_ref() == Some(target))
                    .flat_map(|other| other.bindings.iter())
                    .filter(|binding| foreign.is_record_declaration(&binding.exported))
                    .map(|binding| (binding.exported.clone(), binding.local.clone()))
                    .collect::<BTreeMap<_, _>>();
                for binding in &import.bindings {
                    if dependency.exports.contains_key(&binding.exported) {
                        for index in [&mut index, &mut declaration] {
                            index.import_declaration(
                                foreign,
                                &binding.exported,
                                &binding.local,
                                binding.span,
                                uri.as_str(),
                                &types,
                            );
                        }
                    }
                }
            }
            declaration.finish_imports();
            declarations.insert(id.clone(), declaration);
            index.finish_imports();
            indices.insert(id.clone(), index);
        }
        Self {
            indices,
            references: BTreeMap::new(),
        }
    }
    fn refresh_references(&mut self) {
        self.references.clear();
        for (id, index) in &self.indices {
            if let Ok(uri) = file_uri(id) {
                for (definition, location, declaration) in index.reference_locations(uri.as_str()) {
                    self.references
                        .entry(definition.to_string())
                        .or_default()
                        .push((location, declaration));
                }
            }
        }
    }
    pub fn request(
        &self,
        method: &str,
        params: &Value,
        uri: &str,
    ) -> Option<Result<Value, String>> {
        let id = source_id(uri).ok()?;
        if method == "textDocument/references" {
            let index = self.indices.get(&id)?;
            let canonical_uri = file_uri(&id).ok()?;
            return Some(
                index
                    .request("textDocument/definition", params, canonical_uri.as_str())
                    .map(|definition| {
                        let include = params
                            .pointer("/context/includeDeclaration")
                            .and_then(Value::as_bool)
                            .unwrap_or(true);
                        serde_json::json!(
                            self.references
                                .get(&definition.to_string())
                                .into_iter()
                                .flatten()
                                .filter(|(_, declaration)| include || !declaration)
                                .map(|(location, _)| location.clone())
                                .collect::<Vec<_>>()
                        )
                    }),
            );
        }
        self.indices
            .get(&id)
            .map(|index| index.request(method, params, uri))
    }
}
