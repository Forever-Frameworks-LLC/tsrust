//! Project imported native declarations into editor symbols using shared metadata.
use super::*;

impl SourceIndex {
    pub(super) fn import_native_type(&mut self, declaration: &tsrust_compiler::native_types::Declaration, local: &str, span: Span) {
        let Some(catalog) = builtins_for(declaration.module).into_iter().find(|entry| entry["name"] == declaration.name) else { return; };
        self.native_type_aliases.insert(local.into(), declaration.id);
        let render = |signature: &tsrust_compiler::native_signatures::Signature| {
            signature.render(|id| Some(tsrust_compiler::native_types::declaration(id).name.into())).expect("registered native field signature").into_owned()
        };
        let fields = declaration.fields.iter().map(|field| format!("{}{}: {}", field.name, if field.optional { "?" } else { "" }, render(&field.signature))).collect::<Vec<_>>();
        self.symbols.push(Symbol {
            name: local.into(), kind: 23, span, extent: span, scope: 0, ty: local.into(),
            signature: if fields.is_empty() { format!("type {local}") } else { format!("type {local} = {{ {}; }}", fields.join("; ")) },
            docs: catalog["documentation"].as_str().unwrap_or("").into(), parameters: Vec::new(), owner: None,
        });
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
}
