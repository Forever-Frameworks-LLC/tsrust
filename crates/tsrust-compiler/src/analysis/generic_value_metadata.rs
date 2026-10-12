//! Nominal source identity inside unresolved generic value signatures.
use super::*;
use crate::type_syntax::ValueAnnotationKind as Kind;

impl Analyzer {
    pub(super) fn deferred_generic_nominal_uses(&self) -> Vec<(Span, Span)> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for source in self.deferred_generic_values.values() {
            if !seen.insert(&source.declaration) { continue; }
            let function = self.generic_functions.template(&source.declaration)
                .expect("deferred source template");
            let mut annotations = function.params.iter().map(|(_, annotation, callback)|
                callback.as_ref().map_or_else(|| annotation.clone(), |callback| callback.value_annotation()))
                .collect::<Vec<_>>();
            annotations.extend(function.result.as_ref().map(|result| result.value_annotation()));
            let mut pending = annotations.iter().collect::<Vec<_>>();
            while let Some(annotation) = pending.pop() {
                let name = match &annotation.kind {
                    Kind::Polymorphic { .. } => None,
                    Kind::Named(name) => Some(name),
                    Kind::Applied { name, arguments } => { pending.extend(arguments); Some(name) }
                    Kind::Array(inner) | Kind::Group(inner) => { pending.push(inner); None }
                    Kind::Union(members) | Kind::Tuple(members) => { pending.extend(members); None }
                    Kind::Callable { parameters, result, .. } => {
                        pending.extend(parameters.iter().map(|parameter| &parameter.annotation));
                        pending.push(result); None
                    }
                };
                let Some(name) = name else { continue; };
                if function.type_parameters.iter().any(|parameter| parameter.text == name.text) { continue; }
                let declaration = self.record_names.get(&name.text).map(|id| self.program.records[*id].name.span)
                    .or_else(|| self.generic_records.declaration(&name.text));
                if let Some(declaration) = declaration {
                    let fact = (name.span, declaration);
                    if !result.contains(&fact) { result.push(fact); }
                }
            }
        }
        result
    }
}
