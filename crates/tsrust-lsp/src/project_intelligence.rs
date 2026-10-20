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
    /// The validated module graph of this index, for checks of a repaired module.
    graph: ProjectGraph,
}
impl ProjectIndex {
    pub fn checked(analysis: &tsrust_compiler::project::ProjectAnalysis) -> Self {
        let mut index = Self::build(&analysis.graph);
        let labels=analysis.checked_control_labels();
        let facts = analysis.checked_type_facts();
        let local_functions = analysis.checked_local_functions();
        for id in &analysis.graph.order {
            // Data has no executable body; its complete declarations come from the checked schema.
            if analysis.graph.modules[id].data.is_some(){index.indices.get_mut(id).expect("indexed data source").mark_checked();continue;}
            let mut source = index.indices.remove(id).expect("indexed project source");
            source.checked_control_labels=labels.iter().filter(|(span,definition)|span.source==*id && definition.source==*id).map(|(span,definition)|(span.span,definition.span)).collect();
            source.apply_project_checked_facts(&analysis.graph, id, &facts, &index.indices);
            source.apply_project_local_functions(&analysis.graph, id, &local_functions, &index.indices);
            source.mark_checked();
            index.indices.insert(id.clone(), source);
        }
        index.refresh_references();
        index
    }
    pub fn new(graph: &ProjectGraph) -> Self {
        let mut index = Self::build(graph);
        for (id,source) in &mut index.indices {if graph.modules[id].data.is_none(){source.recover_project_member_receivers(graph,id);}}
        index.refresh_references();
        index
    }
    fn build(graph: &ProjectGraph) -> Self {
        let mut indices = BTreeMap::new();
        let mut declarations: BTreeMap<SourceId, SourceIndex> = BTreeMap::new();
        for id in &graph.order {
            let module = &graph.modules[id];
            let text_asset=module.data.is_some()&&std::path::Path::new(&id.0).extension().is_some_and(|value|value=="txt");
            let (mut declaration,mut index)=if text_asset {let schema=module.data.as_ref().unwrap().schema();(SourceIndex::text_asset(&module.source.text,&schema),SourceIndex::text_asset(&module.source.text,&schema))}else{(SourceIndex::with_entry_help(&module.declaration_source,false),SourceIndex::with_entry_help(module.checked_source(),id==&graph.entry))};
            if let Some(data) = &module.data && !text_asset {
                let schema = data.schema();
                declaration = SourceIndex::with_entry_help(&module.source.text, false);
                declaration.add_data_schema(&schema);
                index.add_data_schema(&schema);
                index.theme_document=tsrust_compiler::ui_project::theme_document(std::path::Path::new(&id.0),&module.source.text).is_some();
            }

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
                index.import_export_names(import.names, foreign, dependency.exports.keys().map(|name|(name.as_str(),dependency.exported_local(name))));
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
                                dependency.exported_local(&binding.exported),
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
            graph: graph.clone(),
        }
    }
    pub fn contains(&self, id: &SourceId) -> bool {
        self.indices.contains_key(id)
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
        if self.graph.modules.get(&id).is_some_and(|module|module.rust.is_some()){return None;}
        if method=="textDocument/definition" {
            let index=self.indices.get(&id)?;
            let position:lsp_types::Position=serde_json::from_value(params.get("position")?.clone()).ok()?;
            if let Some((class,_))=index.ui_class_at(index.offset(position)?) {
                let paths=tsrust_compiler::ui_project::utility_theme_paths(&class);
                let mut locations=vec![];
                for (source,module) in &self.graph.modules {
                    let Some((theme,prefix))=tsrust_compiler::ui_project::theme_document(std::path::Path::new(&source.0),&module.source.text) else{continue;};
                    if tsrust_compiler::ui_project::theme::Theme::parse(&theme.to_string()).is_err(){continue;}
                    let Some(data)=&module.data else{continue;};let schema=data.schema();
                    for path in &paths {
                        let mut annotation=&schema.root;let mut span=None;
                        for key in prefix.iter().chain(path) {
                            let tsrust_compiler::type_syntax::ValueAnnotationKind::Named(name)=&annotation.kind else{span=None;break;};
                            let Some(field)=schema.records.iter().find(|record|record.name==name.text).and_then(|record|record.fields.iter().find(|field|field.name==*key)) else{span=None;break;};
                            span=Some(field.span);annotation=&field.annotation;
                        }
                        if let Some(span)=span {let range=crate::LineIndex::new(&module.source.text).range(&module.source.text,span).ok()?;locations.push(serde_json::json!({"uri":file_uri(source).ok()?,"range":range}));}
                    }
                }
                return Some(Ok(serde_json::json!(locations)));
            }
        }
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
                        let mut locations=serde_json::json!(
                            self.references
                                .get(&definition.to_string())
                                .into_iter()
                                .flatten()
                                .filter(|(_, declaration)| include || !declaration)
                                .map(|(location, _)| location.clone())
                                .collect::<Vec<_>>()
                        );
                        self.rust_locations(&mut locations);locations
                    }),
            );
        }
        let index = self.indices.get(&id)?;
        let answer = index.request(method, params, uri);
        if method == "textDocument/completion"
            && !index.checked()
            && answer.as_ref().is_ok_and(|items| items.as_array().is_some_and(Vec::is_empty))
            && let Some(items) = self.repaired_member_completion(&id, index, params)
        {
            return Some(Ok(items));
        }
        Some(answer.map(|mut answer| { self.rust_locations(&mut answer); answer }))
    }

    fn rust_locations(&self, value: &mut Value) {
        if let Some(uri)=value.get("uri").and_then(Value::as_str)
            && let Ok(id)=source_id(uri)
            && let Some(module)=self.graph.modules.get(&id)
            && let Some(rust)=&module.rust {
            let position=|text:&str,offset:usize| {
                let prefix=&text[..offset];serde_json::json!({"line":prefix.bytes().filter(|&byte|byte==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap_or("").encode_utf16().count()})
            };
            for (declaration,native) in rust.exports.iter().map(|export|(&export.declaration,&export.native)).chain(rust.locations.iter().map(|(declaration,native)|(declaration,native))) {
                if value.pointer("/range/start")==Some(&position(&module.declaration_source,declaration.start)) {
                    value["uri"]=serde_json::Value::String(file_uri(&rust.source.id).unwrap().to_string());value["range"]=serde_json::json!({"start":position(&rust.source.text,native.start),"end":position(&rust.source.text,native.end)});return;
                }
            }
        }
        match value {Value::Array(values)=>for value in values{self.rust_locations(value)},Value::Object(values)=>for value in values.values_mut(){self.rust_locations(value)},_=>{}}
    }

    /// Answer an unfinished member access from a project check of the module
    /// without that access. Headers and import identities stay those of the
    /// same source snapshot; only the module body changes.
    fn repaired_member_completion(&self, id: &SourceId, index: &SourceIndex, params: &Value) -> Option<Value> {
        let offset = index.position_offset(params)?;
        let (separator, prefix_start) = index.member_position(offset)?;
        let source = &self.graph.modules.get(id)?.source.text;
        let prefix = source.get(prefix_start..offset)?;
        let replace = index.source_range(prefix_start, offset);
        let receiver = index.receiver_text(separator);
        index.declaration_prefix_repairs(offset).into_iter().chain(index.member_repairs(offset)).find_map(|repair| {
            let mut graph = self.graph.clone();
            let module = graph.modules.get_mut(id)?;
            module.declaration_source = format!("{}{}", module.declaration_source.get(..repair.start)?, &repair.text[repair.start..]);
            module.source.text = repair.text;
            let analysis = tsrust_compiler::project::check_graph(graph).ok()?;
            Self::checked(&analysis).indices.get(id)?.members_ending_at(repair.receiver_end, prefix, replace, &receiver, index.optional_separator(separator))
        })
    }
}
