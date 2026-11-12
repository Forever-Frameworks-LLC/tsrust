//! Imported declared-source assistance, gated by the compiler's validated module graph.
use crate::{
    intelligence::SourceIndex,
    project_diagnostics::{file_uri, source_id},
};
use serde_json::Value;
use std::collections::{BTreeMap,HashMap};
use std::sync::OnceLock;
use lsp_types::Location;
use tsrust_compiler::project::{ModuleImport, ModuleSource, ProjectGraph, SourceId};

pub struct ProjectIndex {
    declarations_only: bool,
    indices: BTreeMap<SourceId, SourceIndex>,
    entry_aliases: BTreeMap<SourceId,SourceId>,
    references: OnceLock<HashMap<Location, Vec<(Location, bool)>>>,
    /// The validated module graph of this index, for checks of a repaired module.
    graph: ProjectGraph,
}

struct BundledIndices {
    declaration_source: String,
    entry_help: bool,
    declaration: SourceIndex,
    body: SourceIndex,
}

fn reusable_bundle(module: &ModuleSource) -> bool {
    tsrust_compiler::project::bundled_source(&module.source.id) == Some(module.source.text.as_str())
        && module.rust.is_none() && module.data.is_none()
        && module.imports.iter().all(|import| import.native.is_some() && import.target.is_none())
}

fn link_native_import(import: &ModuleImport, native: &str, declaration: &mut SourceIndex, index: &mut SourceIndex) {
    index.import_native_names(import.names, native);
    for binding in &import.bindings {
        for index in [&mut *index, &mut *declaration] {
            index.import_native(native, &binding.exported, &binding.local, binding.span);
        }
    }
}

fn module_indices(module: &ModuleSource, entry_help: bool) -> (SourceIndex, SourceIndex) {
    let reusable = reusable_bundle(module);
    let build = || {
        let mut declaration = SourceIndex::for_project(&module.declaration_source, false);
        let mut index = SourceIndex::for_project(module.checked_source(), entry_help);
        if reusable {
            declaration.inherit_public_record_aliases(&index);
            for import in &module.imports {
                if let Some(native) = import.native { link_native_import(import, native, &mut declaration, &mut index); }
            }
            declaration.finish_imports();
            index.finish_imports();
        }
        (declaration, index)
    };
    // Only compiler bundles with no project dependencies retain linked state.
    if !reusable {
        return build();
    }
    static BUNDLED: std::sync::OnceLock<std::sync::Mutex<BTreeMap<SourceId, BundledIndices>>> = std::sync::OnceLock::new();
    let mut bundled = BUNDLED.get_or_init(Default::default).lock().expect("bundled declaration indices");
    let cached = bundled.entry(module.source.id.clone()).or_insert_with(|| {
        let (declaration, body) = build();
        BundledIndices { declaration_source: module.declaration_source.clone(), entry_help, declaration, body }
    });
    if cached.declaration_source != module.declaration_source || cached.entry_help != entry_help {
        let (declaration, body) = build();
        *cached = BundledIndices { declaration_source: module.declaration_source.clone(), entry_help, declaration, body };
    }
    (cached.declaration.clone(), cached.body.clone())
}

#[cfg(test)]
mod inherited_help_tests {
    use super::*;
    #[test]
    fn signal_operations_survive_bundled_project_alias_linking() {
        let root=crate::project_diagnostics::canonical_path(&std::env::temp_dir().join("tsrust-signal-alias-help"));
        let entry=SourceId(root.join("main.tsr").to_str().unwrap().into());
        let text="import {Signal as State} from 'std:ui';function inspect(value:State<string[]>){value.get();}function main(){}";
        let mut workspace=crate::workspace::Workspace::default();workspace.open(entry.clone(),1,text.into());
        let analysis=workspace.snapshot().check_with_diagnostics(&entry).unwrap();let uri=file_uri(&entry).unwrap();
        let offset=text.find("value.get").unwrap()+"value.".len();
        let params=serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":offset}});
        for (phase,index) in [("headers",ProjectIndex::new(&analysis.graph)),("checked",ProjectIndex::checked(&analysis))] {
            let result=index.request("textDocument/completion",&params,uri.as_str()).unwrap().unwrap();
            assert!(result.to_string().contains("get()")&&result.to_string().contains("set(value:"),"{phase}: {result}");
        }
    }
    #[test]
    fn imported_inherited_fields_keep_help_in_header_and_checked_indices() {
        let root = crate::project_diagnostics::canonical_path(&std::env::temp_dir().join("tsrust-inherited-help"));
        let mut workspace = crate::workspace::Workspace::default();
        let sources = [
            ("base.tsr", "export interface Base<T>{/** Base value contract. */value:T;}export type Alias<T>=Base<T>;"),
            ("leaf.tsr", "import {Alias as RenamedBase} from './base';export interface Middle<T> extends RenamedBase<T>{}export interface Leaf extends Middle<string>{}export interface Override{/** Local value contract. */value:string;}"),
            ("main.tsr", "import {Leaf,Override,Middle} from './leaf';function inspect(base:ReadOnly<Leaf>,own:ReadOnly<Override>,generic:ReadOnly<Middle<string>>,mutable:Mutable<Leaf>,owned:Owned<Leaf>){const first=base.value;const second=own.value;const third=generic.value;const fourth=clone(mutable.value);const fifth=owned.value;}function main(){}"),
        ];
        let mut text = String::new();
        for (name, source) in sources {
            let formatted = tsrust_compiler::format(source).unwrap();
            assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
            if name == "main.tsr" { text = formatted.clone(); }
            workspace.open(SourceId(root.join(name).to_str().unwrap().into()), 1, formatted);
        }
        let entry = SourceId(root.join("main.tsr").to_str().unwrap().into());
        let analysis = workspace.snapshot().check_with_diagnostics(&entry).unwrap();
        let uri = file_uri(&entry).unwrap();
        for (phase, index) in [("headers", ProjectIndex::new(&analysis.graph)), ("checked", ProjectIndex::checked(&analysis))] {
            for (needle, expected, excluded) in [("base.value", "Base value contract", "Local value contract"), ("own.value", "Local value contract", "Base value contract"), ("generic.value", "Base value contract", "Local value contract"), ("mutable.value", "Base value contract", "Local value contract"), ("owned.value", "Base value contract", "Local value contract")] {
                let at = text.find(needle).unwrap() + needle.find('.').unwrap() + 2;
                let start = text[..at].rfind('\n').map_or(0, |offset| offset + 1);
                let params = serde_json::json!({"textDocument":{"uri":uri},"position":{"line":text[..at].matches('\n').count(),"character":text[start..at].encode_utf16().count()}});
                let result = index.request("textDocument/hover", &params, uri.as_str()).unwrap().unwrap();
                assert!(result.to_string().contains(expected) && !result.to_string().contains(excluded), "{phase} {needle}: {result}; fields: {:?}", analysis.checked_type_facts().record_fields);
                let definition=index.request("textDocument/definition",&params,uri.as_str()).unwrap().unwrap();
                let target=if needle=="own.value"{"leaf.tsr"}else{"base.tsr"};
                assert_eq!(definition["uri"],file_uri(&SourceId(root.join(target).to_str().unwrap().into())).unwrap().as_str(),"{phase} {needle}: {definition}");
            }
        }
        // Redeclaring an inherited field is a compiler rejection, not a legal
        // way to override its documentation. Keep that boundary explicit.
        let invalid = tsrust_compiler::format("import {Base} from './base';export interface Middle<T> extends Base<T>{}export interface Override extends Middle<string>{/** Local value contract. */value:string;}").unwrap();
        assert_eq!(tsrust_compiler::format(&invalid).unwrap(),invalid);
        let leaf = SourceId(root.join("leaf.tsr").to_str().unwrap().into());
        workspace.change(&leaf, 2, invalid);
        assert!(workspace.snapshot().check_with_diagnostics(&leaf).err().unwrap().iter().any(|error|error.code=="duplicate" && error.message.contains("value")));
    }
}
impl ProjectIndex {
    pub fn checked(analysis: &tsrust_compiler::project::ProjectAnalysis) -> Self {
        let trace = std::env::var_os("TSRUST_LSP_TRACE_TIMINGS").is_some();
        let started = std::time::Instant::now();
        let mut index = Self::build(&analysis.graph);
        if trace { eprintln!("[index] build={}ms modules={}", started.elapsed().as_millis(), analysis.graph.order.len()); }
        let started = std::time::Instant::now();
        let labels=analysis.checked_control_labels();
        let facts = analysis.checked_type_facts();
        let local_functions = analysis.checked_local_functions();
        if trace { eprintln!("[index] facts={}ms", started.elapsed().as_millis()); }
        let started = std::time::Instant::now();
        for id in &analysis.graph.order {
            // Data has no executable body; its complete declarations come from the checked schema.
            if analysis.graph.modules[id].data.is_some(){index.indices.get_mut(id).expect("indexed data source").mark_checked();continue;}
            let module_started = std::time::Instant::now();
            let mut source = index.indices.remove(id).expect("indexed project source");
            source.checked_control_labels=labels.iter().filter(|(span,definition)|span.source==*id && definition.source==*id).map(|(span,definition)|(span.span,definition.span)).collect();
            source.apply_project_checked_facts(&analysis.graph, id, &facts, &index.indices);
            source.apply_project_local_functions(&analysis.graph, id, &local_functions, &index.indices);
            source.apply_project_function_results(analysis, id, &index.indices);
            source.mark_checked();
            index.indices.insert(id.clone(), source);
            if trace && module_started.elapsed().as_millis() >= 500 { eprintln!("[index] apply-module={}ms source={}", module_started.elapsed().as_millis(), id.0); }
        }
        if trace { eprintln!("[index] apply={}ms", started.elapsed().as_millis()); }
        index
    }
    /// Declared symbols from headers only; never repair a body by running the checker.
    pub fn declarations(graph: &ProjectGraph) -> Self {
        let trace = std::env::var_os("TSRUST_LSP_TRACE_TIMINGS").is_some();
        let started = std::time::Instant::now();
        let mut index = Self::build(graph);
        if trace { eprintln!("[index] declarations-build={}ms", started.elapsed().as_millis()); }
        index.declarations_only = true;
        index
    }
    pub fn new(graph: &ProjectGraph) -> Self { Self::with_diagnostics(graph, &[]) }
    pub fn with_diagnostics(graph: &ProjectGraph, errors: &[tsrust_compiler::project::ProjectDiagnostic]) -> Self {
        Self::with_diagnostics_cancellable(graph, errors, || false).expect("uncancelled index")
    }
    pub fn with_diagnostics_cancellable(graph: &ProjectGraph, errors: &[tsrust_compiler::project::ProjectDiagnostic], cancelled: impl Fn() -> bool) -> Option<Self> {
        if cancelled() { return None; }
        let mut index = Self::build(graph);
        for id in &graph.order {
            if cancelled() { return None; }
            if graph.modules[id].data.is_some(){continue;}
            let mut source=index.indices.remove(id).expect("indexed project source");
            source.recover_project_member_receivers_cancellable(graph,id,&cancelled);
            if cancelled() { return None; }
            if let Some(error)=errors.iter().filter(|error|error.code=="syntax" && error.location.source==*id)
                .min_by_key(|error|error.location.span.start)
            {
                source.recover_project_generic_prefix(graph,id,error.location.span.start,&index.indices);
            }
            index.indices.insert(id.clone(),source);
        }
        if cancelled() { return None; }
        Some(index)
    }
    fn build(graph: &ProjectGraph) -> Self {
        let trace = std::env::var_os("TSRUST_LSP_TRACE_TIMINGS").is_some();
        let mut indices = BTreeMap::new();
        let mut declarations: BTreeMap<SourceId, SourceIndex> = BTreeMap::new();
        let mut export_completions = BTreeMap::new();
        for id in &graph.order {
            let started = std::time::Instant::now();
            let module = &graph.modules[id];
            let text_asset=module.data.is_some()&&std::path::Path::new(&id.0).extension().is_some_and(|value|value=="txt");
            let (mut declaration,mut index)=if text_asset {let schema=module.data.as_ref().unwrap().schema();(SourceIndex::text_asset(&module.source.text,&schema),SourceIndex::text_asset(&module.source.text,&schema))}else{module_indices(module,id==&graph.entry)};
            if let Some(data) = &module.data && !text_asset {
                let schema = data.schema();
                declaration = SourceIndex::for_project(&module.source.text, false);
                declaration.add_data_schema(&schema);
                index.add_data_schema(&schema);
            }

            if !reusable_bundle(module) {
                // Declaration text blanks export markers; retain public aliases from the original source.
                declaration.inherit_public_record_aliases(&index);
                for import in &module.imports {
                    if let Some(native) = import.native {
                        link_native_import(import, native, &mut declaration, &mut index);
                        continue;
                    }
                    let Some(target) = &import.target else {
                        continue;
                    };
                    let dependency = &graph.modules[target];
                    let foreign = &declarations[target];
                    let items=export_completions.entry(target.clone()).or_insert_with(||std::sync::Arc::new(
                        foreign.export_completions(dependency.exports.keys().map(|name|(name.as_str(),dependency.exported_local(name))))
                    ));
                    index.import_export_names(import.names,items.clone());
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
                index.finish_imports();
            }
            declarations.insert(id.clone(), declaration);
            indices.insert(id.clone(), index);
            if trace && started.elapsed().as_millis() >= 50 { eprintln!("[index] module={}ms source={}", started.elapsed().as_millis(), id.0); }
        }
        Self {
            declarations_only: false,
            indices,
            entry_aliases: BTreeMap::new(),
            references: OnceLock::new(),
            graph: graph.clone(),
        }
    }
    pub fn contains(&self, id: &SourceId) -> bool {
        self.indices.contains_key(self.entry_aliases.get(id).unwrap_or(id))
    }
    pub(crate) fn alias_entry(&mut self,physical:SourceId,logical:SourceId) {
        if logical==self.graph.entry && self.indices.contains_key(&logical) && physical!=logical {
            self.entry_aliases.insert(physical,logical);
        }
    }
    fn references(&self) -> &HashMap<Location, Vec<(Location, bool)>> {
        self.references.get_or_init(|| {
            let started = std::time::Instant::now();
            let mut references: HashMap<Location, Vec<(Location, bool)>> = HashMap::new();
            for (id, index) in &self.indices {
                if let Ok(uri) = file_uri(id) {
                    for (definition, location, declaration) in index.reference_locations(&uri) {
                        references.entry(definition).or_default().push((location, declaration));
                    }
                }
            }
            if std::env::var_os("TSRUST_LSP_TRACE_TIMINGS").is_some() { eprintln!("[index] references={}ms", started.elapsed().as_millis()); }
            references
        })
    }
    pub fn request(
        &self,
        method: &str,
        params: &Value,
        uri: &str,
    ) -> Option<Result<Value, String>> {
        let physical = source_id(uri).ok()?;
        let id=self.entry_aliases.get(&physical).unwrap_or(&physical).clone();
        if self.graph.modules.get(&id).is_some_and(|module|module.rust.is_some()){return None;}
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
                        let definition=serde_json::from_value::<Location>(definition).ok();
                        let mut locations=serde_json::json!(
                            definition.as_ref().and_then(|definition|self.references().get(definition))
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
        if method == "textDocument/completion"
            && !self.declarations_only
            && !index.checked()
            && let Some(items) = self.repaired_member_completion(&id, index, params)
        {
            return Some(Ok(items));
        }
        let answer = if self.declarations_only {
            index.request_declarations(method, params, uri)
        } else {
            index.request(method, params, uri)
        };
        Some(answer.map(|mut answer| {
            self.rust_locations(&mut answer);
            if self.declarations_only && method == "textDocument/completion" && answer.is_array() {
                serde_json::json!({"isIncomplete":true,"items":answer})
            } else { answer }
        }))
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
        let access = index.member_access_range(separator, offset);
        let receiver = index.receiver_text(separator);
        index.declaration_prefix_repairs(offset).into_iter().chain(index.member_repairs(offset)).find_map(|repair| {
            let mut graph = self.graph.clone();
            let module = graph.modules.get_mut(id)?;
            module.declaration_source = format!("{}{}", module.declaration_source.get(..repair.start)?, &repair.text[repair.start..]);
            module.source.text = repair.text;
            let analysis = tsrust_compiler::project::check_graph(graph).ok()?;
            Self::checked(&analysis).indices.get(id)?.members_ending_at(repair.receiver_end, prefix, replace, access, &receiver, index.optional_separator(separator))
        })
    }
}

#[cfg(test)]
mod generic_prefix_recovery_tests {
    use super::*;
    #[test]
    fn completed_generic_call_hover_survives_a_broken_sibling_without_accepting_invalid_prefixes() {
        let root=std::env::temp_dir().canonicalize().unwrap().join("tsrust-generic-prefix-recovery");
        let id=SourceId(root.join("main.tsr").to_string_lossy().into_owned());
        let uri=file_uri(&id).unwrap().to_string();
        let valid="import {deepStrictEqual} from 'std:testing';function main(){deepStrictEqual([[1]],[[1]]);}";
        let mut workspace=crate::workspace::Workspace::default();
        workspace.open(id.clone(),1,valid.into());
        let analysis=workspace.snapshot().check_with_diagnostics(&id).unwrap();
        let checked=ProjectIndex::checked(&analysis);
        let params=serde_json::json!({"position":{"line":0,"character":valid.rfind("deepStrictEqual").unwrap()+2}});
        let expected=checked.request("textDocument/hover",&params,&uri).unwrap().unwrap();
        assert!(expected.to_string().contains("ReadOnly<int[][]>"));
        let broken=format!("{valid}function broken(){{const = ;}}");
        assert!(workspace.change(&id,2,broken));
        let snapshot=workspace.snapshot();
        let errors=snapshot.check_with_diagnostics(&id).err().expect("original syntax error");
        assert!(errors.iter().any(|error|error.code=="syntax"));
        let graph=tsrust_compiler::project::load_project_headers(&snapshot,&id.0).unwrap();
        let recovered=ProjectIndex::with_diagnostics(&graph,&errors);
        assert_eq!(recovered.request("textDocument/hover",&params,&uri).unwrap().unwrap(),expected);
        assert!(snapshot.check_with_diagnostics(&id).is_err(),"index recovery never repairs the document");

        let invalid="import {deepStrictEqual,trackedResource} from 'std:testing';function main(){deepStrictEqual([trackedResource()],[trackedResource()]);}function broken(){const = ;}";
        assert!(workspace.change(&id,3,invalid.into()));
        let snapshot=workspace.snapshot();
        let errors=snapshot.check_with_diagnostics(&id).err().expect("invalid prefix and sibling");
        let graph=tsrust_compiler::project::load_project_headers(&snapshot,&id.0).unwrap();
        let recovered=ProjectIndex::with_diagnostics(&graph,&errors);
        let params=serde_json::json!({"position":{"line":0,"character":invalid.rfind("deepStrictEqual").unwrap()+2}});
        let hover=recovered.request("textDocument/hover",&params,&uri).unwrap().unwrap();
        assert!(hover.to_string().contains("deepStrictEqual<T>"),"invalid prefixes cannot supply checked specialization: {hover}");
    }
}

#[cfg(test)]
mod declaration_only_tests {
    use super::*;
    #[test]
    fn bundled_parse_reuse_is_finite_isolated_and_sensitive_to_source_and_mode() {
        let root = std::env::temp_dir().canonicalize().unwrap().join("tsrust-bundled-index-reuse");
        let entry = SourceId(root.join("main.tsr").to_string_lossy().into_owned());
        let mut workspace = crate::workspace::Workspace::default();
        workspace.open(entry.clone(), 1, "import {test} from 'std:testing';function main(){}".into());
        let graph = tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &entry.0).unwrap();
        let module = &graph.modules[&SourceId("std:testing".into())];
        assert!(reusable_bundle(module));
        let (mut declaration, mut body) = module_indices(module, false);
        declaration.mark_checked();
        body.mark_checked();
        let builds = || crate::intelligence::PROJECT_SOURCE_BUILDS.with(|count| count.get());
        let before = builds();
        let (declaration, body) = module_indices(module, false);
        assert_eq!(builds(), before, "unchanged bundle must reuse both parses");
        assert!(!declaration.checked() && !body.checked(), "project mutations must not contaminate the cache");
        module_indices(module, true);
        assert_eq!(builds(), before + 2, "entry help mode must be respected");
        let mut changed = module.clone();
        changed.declaration_source.push_str("\nfunction cacheWitness(value: int): int { return value; }\n");
        let (declaration, _) = module_indices(&changed, false);
        assert_eq!(builds(), before + 4);
        let start = changed.declaration_source.find("cacheWitness").unwrap();
        let prefix = &changed.declaration_source[..start];
        let params = serde_json::json!({"position":{"line":prefix.bytes().filter(|&b| b == b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}});
        assert!(declaration.request_declarations("textDocument/hover", &params, "file:///bundle.tsr").unwrap().to_string().contains("cacheWitness(value: int)"));
        let before = builds();
        let (restored, _) = module_indices(module, false);
        assert!(builds() <= before + 2, "another test thread may already have restored the immutable bundle");
        assert!(!restored.request_declarations("textDocument/documentSymbol", &serde_json::json!({}), "file:///bundle.tsr").unwrap().to_string().contains("cacheWitness"), "original declarations must be restored");
        // Even a known bundle ID cannot cache substituted workspace source.
        changed.source.text.push_str("\n// private workspace override");
        let before = builds();
        module_indices(&changed, false);
        module_indices(&changed, false);
        assert_eq!(builds(), before + 4);
        let mut dependent = module.clone();
        dependent.imports[0].native = None;
        dependent.imports[0].target = Some(entry);
        assert!(!reusable_bundle(&dependent), "project-dependent links must never enter the bundle cache");
        let before = builds();
        module_indices(&dependent, false);
        module_indices(&dependent, false);
        assert_eq!(builds(), before + 4);
    }

    #[test]
    fn preliminary_help_uses_declared_imports_and_marks_completions_incomplete() {
        let root = std::env::temp_dir().canonicalize().unwrap().join("tsrust-declaration-help");
        let id = SourceId(root.join("main.tsr").to_string_lossy().into_owned());
        let dep = SourceId(root.join("dep.tsr").to_string_lossy().into_owned());
        let uri = file_uri(&id).unwrap().to_string();
        let source = "import {Row,render} from './dep.tsr';function main(row:Row){render(row);row.;}";
        let mut workspace = crate::workspace::Workspace::default();
        workspace.open(dep, 1, "export interface Row{value:string;}/** Render a row. */export function render(row:Row):string{return 4;}".into());
        workspace.open(id.clone(), 1, source.into());
        let graph = tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &id.0).unwrap();
        let index = ProjectIndex::declarations(&graph);
        let ask = |method: &str, offset| index.request(method, &serde_json::json!({"position":{"line":0,"character":offset}}), &uri).unwrap().unwrap();
        let hover = ask("textDocument/hover", source.rfind("render(").unwrap()+1);
        assert!(hover.to_string().contains("Render a row"), "{hover}");
        let signatures = ask("textDocument/signatureHelp", source.rfind("render(").unwrap()+7);
        assert!(signatures.to_string().contains("Row"), "{signatures}");
        let repairs = crate::intelligence::MEMBER_REPAIR_ATTEMPTS.with(|count| count.get());
        let completion = ask("textDocument/completion", source.rfind("row.").unwrap()+4);
        assert_eq!(completion["isIncomplete"], true);
        assert!(completion["items"].as_array().unwrap().iter().any(|item|item["label"]=="value"), "{completion}");
        assert!(index.indices.values().all(|index| !index.checked()));
        assert_eq!(crate::intelligence::MEMBER_REPAIR_ATTEMPTS.with(|count| count.get()), repairs, "declaration reads must not run cursor repair on the protocol thread");
        let source_index = index.indices.get(&id).unwrap();
        source_index.request("textDocument/completion", &serde_json::json!({"position":{"line":0,"character":source.rfind("row.").unwrap()+4}}), &uri).unwrap();
        assert!(crate::intelligence::MEMBER_REPAIR_ATTEMPTS.with(|count| count.get()) > repairs, "ordinary unchecked requests retain cursor repair");
    }
}

#[cfg(test)]
mod compact_location_tests {
    use super::*;
    #[test]
    fn compact_locations_keep_unicode_alias_references_and_snapshot_identity() {
        use crate::{LineIndex,workspace::Workspace};
        use tsrust_compiler::{Span,project::load_project_headers};
        let root=std::env::temp_dir().canonicalize().unwrap().join("tsrust location é");
        let entry=SourceId(root.join("main.tsr").to_string_lossy().into_owned());
        let dependency=SourceId(root.join("lib é.tsr").to_string_lossy().into_owned());
        let auxiliary=SourceId(root.join("aux.tsr").to_string_lossy().into_owned());
        let source="/*😀*/ export function source():int{return 1;}";
        let main="import {source as renamed} from './lib é.tsr';import {forward} from './aux.tsr';function main(){console.log(renamed(),forward(),renamed());}";
        let mut workspace=Workspace::default();
        workspace.open(entry.clone(),1,main.into());
        workspace.open(dependency.clone(),1,source.into());
        workspace.open(auxiliary,1,"import {source as original} from './lib é.tsr';export function forward():int{return original();}".into());
        let index=ProjectIndex::declarations(&load_project_headers(&workspace.snapshot(),&entry.0).unwrap());
        assert!(index.references.get().is_none(), "reference tables must not delay preliminary help");
        let uri=file_uri(&entry).unwrap();
        let at=main.find("renamed()").unwrap();
        let params=serde_json::json!({"position":{"line":0,"character":main[..at].encode_utf16().count()},"context":{"includeDeclaration":true}});
        let declaration=source.find("source()").unwrap();
        let expected=serde_json::json!({"uri":file_uri(&dependency).unwrap(),"range":LineIndex::new(source).range(source,Span::new(declaration,declaration+6)).unwrap()});
        assert!(expected["uri"].as_str().unwrap().contains("%20"));
        assert!(index.request("textDocument/hover",&params,uri.as_str()).unwrap().unwrap().to_string().contains("int"));
        assert_eq!(index.request("textDocument/definition",&params,uri.as_str()).unwrap().unwrap(),expected);
        assert!(index.references.get().is_none(), "hover and definition do not need a reference table");
        let all=index.request("textDocument/references",&params,uri.as_str()).unwrap().unwrap();
        assert!(index.references.get().is_some(), "references initialize on demand");
        assert!(all.as_array().unwrap().contains(&expected));
        assert!(all.as_array().unwrap().len()>4);
        let mut without=params.clone();without["context"]["includeDeclaration"]=false.into();
        let references=index.request("textDocument/references",&without,uri.as_str()).unwrap().unwrap();
        let expected_references=all.as_array().unwrap().iter().filter(|location|**location!=expected).cloned().collect::<Vec<_>>();
        assert_eq!(references,serde_json::json!(expected_references));
        for location in all.as_array().unwrap() {
            let typed=serde_json::from_value::<Location>(location.clone()).unwrap();
            assert_eq!(serde_json::to_value(typed).unwrap(),*location);
        }
        assert!(workspace.change(&dependency,2,format!("\r\n{source}")));
        let newer=ProjectIndex::declarations(&load_project_headers(&workspace.snapshot(),&entry.0).unwrap());
        let moved=newer.request("textDocument/definition",&params,uri.as_str()).unwrap().unwrap();
        assert_eq!(moved["range"]["start"]["line"],1);
        assert_eq!(moved["range"]["start"]["character"],expected["range"]["start"]["character"]);
        assert_eq!(index.request("textDocument/definition",&params,uri.as_str()).unwrap().unwrap(),expected,"older snapshot stays immutable");
    }
}
