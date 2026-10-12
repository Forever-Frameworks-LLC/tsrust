//! Project imported native declarations into editor symbols using shared metadata.
use super::*;

impl SourceIndex {
    pub(super) fn deque_element<'a>(&self, ty: &'a str) -> Option<&'a str> {
        self.native_collection_element(ty,tsrust_compiler::native_signatures::NativeTypeId::Deque)
    }
    pub(super) fn heap_element<'a>(&self, ty: &'a str) -> Option<&'a str> {
        self.native_collection_element(ty,tsrust_compiler::native_signatures::NativeTypeId::Heap)
    }
    pub(super) fn ordered_set_element<'a>(&self,ty:&'a str)->Option<&'a str>{
        self.native_collection_element(ty,tsrust_compiler::native_signatures::NativeTypeId::OrderedSet)
    }
    pub(super) fn ordered_map_types<'a>(&self,ty:&'a str)->Option<(&'a str,&'a str)> {
        self.native_collection_element(ty,tsrust_compiler::native_signatures::NativeTypeId::OrderedMap)?.split_once(',').map(|(key,value)|(key.trim(),value.trim()))
    }
    pub(super) fn ordered_map_entry(&self,ty:&str)->Option<String>{self.ordered_map_types(ty).map(|(key,value)|format!("[{key}, {value}]"))}
    fn native_collection_element<'a>(&self, ty: &'a str, identity: tsrust_compiler::native_signatures::NativeTypeId) -> Option<&'a str> {
        let ty = ["ReadOnly<", "Mutable<", "Owned<"].into_iter()
            .find_map(|wrapper| ty.strip_prefix(wrapper).and_then(|inner| inner.strip_suffix('>')))
            .unwrap_or(ty);
        let (name, arguments) = ty.split_once('<')?;
        (self.native_type_aliases.get(name.trim()) == Some(&identity)).then(|| arguments.strip_suffix('>')).flatten()
    }
    pub(super) fn schema_element<'a>(&self, ty: &'a str) -> Option<&'a str> { self.native_collection_element(ty, tsrust_compiler::native_signatures::NativeTypeId::Schema) }
    pub(super) fn receiver_builtins(&self, ty: &str) -> Vec<Value> {
        if self.schema_element(ty) == Some("string") { return builtins_for("Schema"); }
        if self.schema_element(ty) == Some("null") { return builtins_for("SchemaNull"); }
        if self.schema_element(ty) == Some("number") { return builtins_for("SchemaNumber"); }
        if self.schema_element(ty) == Some("int") { return builtins_for("SchemaInt"); }
        if self.schema_element(ty) == Some("boolean") { return builtins_for("SchemaBoolean"); }
        if let Some(element)=self.schema_element(ty).filter(|element|element.ends_with("[]")) {
            let mut entries=builtins_for("SchemaArray");
            for entry in &mut entries {
                if let Some(signature)=entry["signature"].as_str(){entry["signature"]=json!(builtin_type_substitution(signature,&[("Output",element)]));}
                if let Some(parameters)=entry["parameters"].as_array_mut(){for parameter in parameters{if let Some(label)=parameter["label"].as_str(){parameter["label"]=json!(builtin_type_substitution(label,&[("Output",element)]));}}}
            }
            return entries;
        }
        if ty == "@namespace:std:schema#s" { return builtins_for("schemaBuilder"); }
        if ty=="__tsrust_json_string_read" {return builtins_for("string").into_iter().filter(|entry|matches!(entry["name"].as_str(),Some("length"|"startsWith"|"endsWith"|"includes"|"indexOf"|"lastIndexOf"))).collect();}
        if let Some((key,value))=self.ordered_map_types(ty){return builtins_for(&format!("OrderedMap<{key}, {value}>"));}
        let Some((family,element)) = self.deque_element(ty).map(|element|("Deque",element)).or_else(||self.heap_element(ty).map(|element|("Heap",element))).or_else(||self.ordered_set_element(ty).map(|element|("OrderedSet",element))) else { return builtins_for(ty); };
        let mut entries = builtins_for(family);
        for entry in &mut entries {
            if let Some(signature) = entry["signature"].as_str() {
                entry["signature"] = json!(builtin_type_substitution(signature, &[("Element", element)]));
            }
            if let Some(parameters) = entry["parameters"].as_array_mut() {
                for parameter in parameters {
                    if let Some(text) = parameter.as_str() {
                        *parameter = json!(builtin_type_substitution(text, &[("Element", element)]));
                    }
                }
            }
        }
        entries
    }

    pub(super) fn import_structured_native_alias(&mut self, dependency: &SourceIndex, declaration: &Symbol, local: &str, span: Span, uri: &str, types: &std::collections::BTreeMap<String, String>) -> bool {
        let mut mappings = types.clone();
        for (name, identity) in &dependency.native_type_aliases {
            mappings.insert(name.clone(), self.native_type_spelling(*identity));
        }
        let Some(ty) = remap_imported_type(&dependency.written_type(&declaration.ty), &mappings) else { return false; };
        self.type_aliases.insert(local.into(), ty.clone());
        let mut symbol = declaration.clone();
        symbol.name = local.into();
        symbol.ty = ty.clone();
        symbol.signature = format!("type {local} = {}", self.display_type(&ty));
        symbol.span = span;
        symbol.extent = span;
        self.foreign_definitions.insert(self.symbols.len(), json!({"uri":uri,"range":dependency.range(declaration.span)}));
        self.symbols.push(symbol);
        true
    }
    /// Materialize result payloads even when their type names were not imported.
    /// Private spelling prevents collisions with user declarations.
    pub(super) fn native_type_spelling(&mut self, identity: tsrust_compiler::native_signatures::NativeTypeId) -> String {
        if let Some((name, _)) = self.native_type_aliases.iter().find(|(name, id)| **id == identity && !name.starts_with("__tsrust_private_")) {
            return name.clone();
        }
        let declaration = tsrust_compiler::native_types::declaration(identity);
        let name = format!("__tsrust_private_native_{}", declaration.name);
        if !self.native_type_aliases.contains_key(&name) {
            self.import_native_type(declaration, &name, Span::new(0, 0));
            self.generic_display_names.insert(name.clone(), declaration.name.into());
        }
        name
    }
    pub(super) fn import_native_type(&mut self, declaration: &tsrust_compiler::native_types::Declaration, local: &str, span: Span) {
        let Some(catalog) = builtins_for(declaration.module).into_iter().find(|entry| entry["name"] == declaration.name) else { return; };
        self.native_type_aliases.insert(local.into(), declaration.id);
        let render = |signature: &tsrust_compiler::native_signatures::Signature| {
            signature.render(|id| Some(tsrust_compiler::native_types::declaration(id).name.into())).expect("registered native field signature").into_owned()
        };
        let fields = declaration.fields.iter().map(|field| format!("{}{}: {}", field.name, if field.optional { "?" } else { "" }, render(&field.signature))).collect::<Vec<_>>();
        let type_name = if declaration.type_parameters.is_empty() { local.to_owned() } else { format!("{local}<{}>", declaration.type_parameters.join(", ")) };
        let symbol = Symbol {
            name: local.into(), kind: 23, span, extent: span, scope: 0, ty: local.into(),
            signature: if fields.is_empty() { format!("type {type_name}") } else { format!("type {type_name} = {{ {}; }}", fields.join("; ")) },
            docs: catalog["documentation"].as_str().unwrap_or("").into(), parameters: Vec::new(), owner: None,
        };
        if let Some(existing) = self.symbols.iter_mut().find(|symbol| symbol.name == local && symbol.kind == 26 && symbol.owner.is_none()) {
            let mut replacement = symbol;
            replacement.extent = existing.extent;
            if !existing.docs.is_empty() { replacement.docs = format!("{}\n\n{}", existing.docs, replacement.docs); }
            *existing = replacement;
        } else { self.symbols.push(symbol); }
        for field in declaration.fields {
            let mut ty = render(&field.signature);
            if field.optional { ty.push_str(" | undefined"); }
            self.symbols.push(Symbol {
                name: field.name.into(), kind: 8, span, extent: span, scope: 0,
                signature: format!("(property) {local}.{}: {ty}", field.name), ty,
                docs: catalog["fields"][field.name]["documentation"].as_str().unwrap_or("").into(),
                parameters: Vec::new(), owner: Some(local.into()),
            });
        }
    }
    /// Native aliases retain declaration identity while navigation stays on the
    /// written alias declaration. Structured aliases are not collapsed to owners.
    pub(super) fn refresh_native_aliases(&mut self) {
        let aliases = self.type_aliases.iter().filter_map(|(name, target)| {
            let target = self.written_type(target);
            let identity = *self.native_type_aliases.get(&target)?;
            let symbol = self.symbols.iter().find(|symbol| symbol.name == *name && symbol.kind == 26 && symbol.owner.is_none())?;
            Some((name.clone(), identity, symbol.span))
        }).collect::<Vec<_>>();
        for (name, identity, span) in aliases {
            self.import_native_type(tsrust_compiler::native_types::declaration(identity), &name, span);
        }
    }

}
