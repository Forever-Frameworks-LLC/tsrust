//! Editor symbols derived from validated compiler data schemas.
use super::*;
use tsrust_compiler::{project::DataSchema, type_syntax::{ValueAnnotation, ValueAnnotationKind}};
fn spelling(value: &ValueAnnotation) -> String {
    match &value.kind {
        ValueAnnotationKind::Named(name) => name.text.clone(),
        ValueAnnotationKind::Array(inner) => { let text = spelling(inner); if text.contains('|') { format!("({text})[]") } else { format!("{text}[]") } },
        ValueAnnotationKind::Union(values) => values.iter().map(spelling).collect::<Vec<_>>().join(" | "),
        _ => unreachable!("data schema only contains named, array and union types"),
    }
}
impl SourceIndex {
    pub(crate) fn add_data_schema(&mut self, schema: &DataSchema) {
        self.symbols.clear();
        for record in &schema.records {
            self.symbols.push(Symbol { name: record.name.clone(), kind: 23, span: record.span, extent: record.span, scope: 0,
                ty: record.name.clone(), signature: format!("type {}", record.name), docs: "Inferred from validated JSON data.".into(), parameters: vec![], owner: None });
            for field in &record.fields {
                let ty = spelling(&field.annotation);
                self.symbols.push(Symbol { name: field.name.clone(), kind: 8, span: field.span, extent: field.span, scope: 0,
                    signature: format!("{}{}: {ty}", field.name, if field.optional { "?" } else { "" }), ty,
                    docs: "Inferred from validated JSON data. Imported data is readonly; use clone() for an owned copy.".into(), parameters: vec![], owner: Some(record.name.clone()) });
            }
        }
        let ty = spelling(&schema.root);
        self.symbols.push(Symbol { name: "@data-owner".into(), kind: 13, span: schema.root.span, extent: schema.root.span, scope: 0,
            signature: format!("const @data-owner: {ty}"), ty,
            docs: "Validated and embedded at build time. One immutable value per data file; reads do not copy. Use clone() for an owned copy before mutation.".into(), parameters: vec![], owner: None });
    }
    pub(super) fn import_data_value(&mut self, dependency: &SourceIndex, declaration: &Symbol, local: &str, span: Span, uri: &str) -> bool {
        let encoded = uri.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let types: std::collections::BTreeMap<_, _> = dependency.symbols.iter().filter(|symbol| symbol.kind == 23 && symbol.owner.is_none())
            .map(|symbol| (symbol.name.clone(), format!("__tsrust_private_{encoded}_{}", symbol.name))).collect();
        for (name, identity) in &types {
            self.generic_display_names.insert(identity.clone(), name.clone());
            self.import_record_fields(dependency, name, identity, span, uri, &types);
        }
        let mut symbol = declaration.clone();
        symbol.name = local.into(); symbol.span = span; symbol.extent = span;
        symbol.ty = remap_imported_type(&symbol.ty, &types).unwrap_or_default();
        symbol.signature = format!("const {local}: {}", self.display_type(&symbol.ty));
        self.foreign_definitions.insert(self.symbols.len(), json!({"uri":uri,"range":dependency.range(declaration.span)}));
        self.symbols.push(symbol);
        true
    }
}
