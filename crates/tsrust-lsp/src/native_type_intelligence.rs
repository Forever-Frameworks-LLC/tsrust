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
    pub(super) fn validation_context_element<'a>(&self, ty: &'a str) -> Option<&'a str> { self.native_collection_element(ty, tsrust_compiler::native_signatures::NativeTypeId::ValidationContext) }
    fn schema_integer_alternative(element:&str)->Option<String> {
        let mut leaf=element.trim();let mut depth=0;
        while let Some(inner)=leaf.strip_suffix("[]"){leaf=inner.trim();depth+=1;}
        if let Some(inner)=leaf.strip_prefix('(').and_then(|inner|inner.strip_suffix(')')){leaf=inner;}
        let members=leaf.split('|').map(str::trim).collect::<Vec<_>>();
        (members.contains(&"number")&&members.iter().all(|member|matches!(*member,"number"|"null"|"undefined"))).then(||format!("int{}","[]".repeat(depth)))
    }
    fn schema_scalar_input_text(&self,element:&str)->String {
        use tsrust_compiler::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind};
        fn collect(value:&ValueAnnotation,source:&str,enums:&std::collections::BTreeMap<String,String>,replacements:&mut Vec<(usize,usize,String)>){
            match &value.kind {
                Kind::Literal(_)=>{if let Some(base)=literal_scalar_family(&source[value.span.start..value.span.end]){replacements.push((value.span.start,value.span.end,base.into()));}},
                Kind::Named(name)=>{if let Some(base)=enums.get(&name.text){replacements.push((value.span.start,value.span.end,format!("({}|{})",name.text,if base=="number"{"number|int"}else{base})));}},
                Kind::Array(inner)|Kind::Group(inner)=>collect(inner,source,enums,replacements),
                Kind::Union(members)=>for member in members{collect(member,source,enums,replacements);},
                _=>{},
            }
        }
        let Ok(annotation)=tsrust_compiler::type_syntax::parse(element)else{return element.into()};
        let mut replacements=Vec::new();collect(&annotation,element,&self.enum_scalar_types,&mut replacements);
        let mut result=element.to_owned();for(start,end,replacement)in replacements.into_iter().rev(){result.replace_range(start..end,&replacement);}generic_intelligence::canonical(&result).unwrap_or(result)
    }
    fn schema_is_string_enum(element:&str)->bool {
        use tsrust_compiler::type_syntax::{ValueAnnotation,ValueAnnotationKind as Kind,LiteralAnnotation};
        fn all_strings(value:&ValueAnnotation)->bool{match &value.kind{Kind::Literal(LiteralAnnotation::String(_))=>true,Kind::Group(inner)=>all_strings(inner),Kind::Union(members)=>members.iter().all(all_strings),_=>false}}
        tsrust_compiler::type_syntax::parse(element).is_ok_and(|value|matches!(value.kind,Kind::Union(_))&&all_strings(&value))
    }
    pub(super) fn receiver_builtins(&self, ty: &str) -> Vec<Value> {
        if let Some((input, output, object_policy, json_input, text_input, scalar_constraints)) = self.schema_pipeline_types.get(ty) {
            let accepted = if *json_input {format!("{input} | JsonValue")}else{input.clone()};
            let mut entries = self.schema_pipeline_entries(&accepted, input, output);
            if *scalar_constraints {
                let catalog = match output.as_str() { "int" => "SchemaInt", "number" => "SchemaNumber", _ => "Schema" };
                entries.extend(builtins_for(catalog).into_iter().filter(|entry|
                    matches!(entry["name"].as_str(), Some("min" | "max" | "gt" | "lt"))));
            }
            if !*text_input { entries.retain(|entry|entry["name"]!="safeParseJson"); }
            if *object_policy {
                for mut entry in builtins_for("SchemaObject").into_iter().filter(|entry| matches!(entry["name"].as_str(), Some("strip" | "retain"))) {
                    if let Some(signature) = entry["signature"].as_str() {
                        entry["signature"] = json!(builtin_type_substitution(signature, &[("Output", output)]));
                    }
                    entries.push(entry);
                }
            }
            return entries;
        }
        if let Some(output) = self.validation_context_element(ty) {
            let mut entries = builtins_for("ValidationContext");
            for entry in &mut entries {
                if let Some(signature) = entry["signature"].as_str() { entry["signature"] = json!(builtin_type_substitution(signature, &[("T", output)])); }
                if let Some(parameters) = entry["parameters"].as_array_mut() {
                    for parameter in parameters { if let Some(label) = parameter["label"].as_str() { parameter["label"] = json!(builtin_type_substitution(label, &[("T", output)])); } }
                }
            }
            return entries;
        }
        let mut entries = self.receiver_base_builtins(ty);
        if let Some(output) = self.schema_element(ty) {
            let input = entries.iter().find(|entry| entry["name"] == "safeParse")
                .and_then(|entry| entry["parameters"][0]["label"].as_str())
                .and_then(|label| label.strip_prefix("input: "))
                .map(str::to_owned).unwrap_or_else(|| format!("{output} | JsonValue"));
            entries.extend(self.schema_pipeline_entries(&input, output, output).into_iter()
                .filter(|entry| matches!(entry["name"].as_str(),Some("refine"|"superRefine"|"transform"|"coerce"|"safeParseJson"|"toJsonSchema"))));
        }
        entries
    }
    fn schema_pipeline_entries(&self, input: &str, raw_input: &str, output: &str) -> Vec<Value> {
        let mut entries = builtins_for("SchemaPipeline");
        for entry in &mut entries {
            if let Some(signature) = entry["signature"].as_str() {
                entry["signature"] = json!(builtin_type_substitution(signature, &[("Input", input), ("RawInput", raw_input), ("Output", output)]));
            }
            if let Some(parameters) = entry["parameters"].as_array_mut() {
                for parameter in parameters {
                    if let Some(label) = parameter["label"].as_str() {
                        parameter["label"] = json!(builtin_type_substitution(label, &[("Input", input), ("RawInput", raw_input), ("Output", output)]));
                    }
                }
            }
        }
        entries
    }
    fn receiver_base_builtins(&self, ty: &str) -> Vec<Value> {
        if self.schema_element(ty) == Some("string") { return builtins_for("Schema"); }
        if self.schema_element(ty) == Some("null") { return builtins_for("SchemaNull"); }
        if self.schema_element(ty) == Some("number") { return builtins_for("SchemaNumber"); }
        if self.schema_element(ty) == Some("int") { return builtins_for("SchemaInt"); }
        if self.schema_element(ty) == Some("boolean") { return builtins_for("SchemaBoolean"); }
        if let Some(element)=self.schema_element(ty) && let Some(base)=literal_scalar_family(element) {
            let input=if base=="number"{"number | int | JsonValue"}else{base};
            let input=if base=="number"{input.to_owned()}else{format!("{input} | JsonValue")};
            let mut entries=builtins_for("SchemaLiteral");
            for entry in &mut entries {
                if let Some(signature)=entry["signature"].as_str(){entry["signature"]=json!(builtin_type_substitution(signature,&[("Output",element),("Input",&input)]));}
                if let Some(parameters)=entry["parameters"].as_array_mut(){for parameter in parameters{if let Some(label)=parameter["label"].as_str(){parameter["label"]=json!(builtin_type_substitution(label,&[("Output",element),("Input",&input)]));}}}
            }
            return entries;
        }
        if let Some(element)=self.schema_element(ty).filter(|element|element.ends_with("[]")) {
            let scalar=self.schema_scalar_input_text(element);
            let input=if let Some(integer)=Self::schema_integer_alternative(&scalar) {format!("{scalar} | {integer} | JsonValue")}else{format!("{scalar} | JsonValue")};
            let mut entries=builtins_for("SchemaArray");
            for entry in &mut entries {
                if let Some(signature)=entry["signature"].as_str(){entry["signature"]=json!(builtin_type_substitution(signature,&[("Output",element),("Input",&input)]));}
                if let Some(parameters)=entry["parameters"].as_array_mut(){for parameter in parameters{if let Some(label)=parameter["label"].as_str(){parameter["label"]=json!(builtin_type_substitution(label,&[("Output",element),("Input",&input)]));}}}
            }
            return entries;
        }
        if let Some(element)=self.schema_element(ty) {
            let retained=element.starts_with("__tsrust_private_schema_retained_");
            let raw=if retained {self.symbols.iter().find(|symbol|symbol.owner.as_deref()==Some(element)&&symbol.name=="value").map(|symbol|symbol.ty.as_str()).unwrap_or(element)}else{element};
            let scalar=self.schema_scalar_input_text(raw);
            let input=if let Some(integer)=Self::schema_integer_alternative(&scalar) {format!("{scalar} | {integer} | JsonValue")}else{format!("{scalar} | JsonValue")};
            let mut entries=builtins_for(if retained {"SchemaRetained"}else if self.enum_scalar_types.contains_key(element){"SchemaDeclaredEnum"}else if Self::schema_is_string_enum(element){"SchemaEnum"}else if element.contains('|') {"SchemaUnion"}else{"SchemaObject"});
            for entry in &mut entries {
                if let Some(signature)=entry["signature"].as_str(){entry["signature"]=json!(builtin_type_substitution(signature,&[("Output",element),("Input",&input)]));}
                if let Some(parameters)=entry["parameters"].as_array_mut(){for parameter in parameters{if let Some(label)=parameter["label"].as_str(){parameter["label"]=json!(builtin_type_substitution(label,&[("Output",element),("Input",&input)]));}}}
            }
            return entries;
        }
        let (base, arguments) = ty.find('<').map_or((ty, ""), |start| (&ty[..start], &ty[start..]));
        let identity=self.native_type_aliases.get(base).copied().or_else(|| {
            let (module,name)=base.strip_prefix("__tsrust_module_native#")?.rsplit_once('#')?;
            tsrust_compiler::native_types::lookup(module,name).map(|declaration|declaration.id)
        });
        if let Some(identity) = identity {
            let declaration = tsrust_compiler::native_types::declaration(identity);
            if declaration.module == "std:http" {return builtins_for(&format!("Http{}",declaration.name));}
            if declaration.module == "std:futures" {return builtins_for(&format!("{}{arguments}",match declaration.id {tsrust_compiler::native_signatures::NativeTypeId::AsyncChannel=>"AsyncChannel",tsrust_compiler::native_signatures::NativeTypeId::AsyncSender=>"AsyncSender",_=>declaration.name}));}
            if declaration.module == "std:threads" { return builtins_for(&format!("{}{arguments}", declaration.name)); }
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
        if declaration.id == tsrust_compiler::native_signatures::NativeTypeId::HttpApp { self.native_type_spelling(tsrust_compiler::native_signatures::NativeTypeId::HttpTlsOptions); }
        if declaration.id == tsrust_compiler::native_signatures::NativeTypeId::HttpClient { self.native_type_spelling(tsrust_compiler::native_signatures::NativeTypeId::HttpClientOptions); }
        if declaration.id == tsrust_compiler::native_signatures::NativeTypeId::ThreadPool { self.native_type_spelling(tsrust_compiler::native_signatures::NativeTypeId::ThreadPoolOptions); }
        let Some(catalog) = builtins_for(declaration.module).into_iter().find(|entry| entry["name"] == declaration.name) else { return; };
        self.native_type_aliases.insert(local.into(), declaration.id);
        if !declaration.type_parameters.is_empty() {
            self.generic_parameters.insert(local.into(), declaration.type_parameters.iter().map(|name| (*name).to_owned()).collect());
        }
        let render = |signature: &tsrust_compiler::native_signatures::Signature| {
            signature.render_with_parameters(|id| Some(tsrust_compiler::native_types::declaration(id).name.into()), |index| declaration.type_parameters.get(index).map(|name| (*name).into())).expect("registered native field signature").into_owned()
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
        // A field can hold another native record. Declare it too, so its
        // members are known where the document does not import its name.
        let mut mentioned = Vec::new();
        for field in declaration.fields {
            let _ = field.signature.render(|id| {
                mentioned.push(id);
                Some(tsrust_compiler::native_types::declaration(id).name.into())
            });
        }
        for id in mentioned {
            if id != declaration.id && !self.native_type_aliases.values().any(|known| *known == id) {
                self.native_type_spelling(id);
            }
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
