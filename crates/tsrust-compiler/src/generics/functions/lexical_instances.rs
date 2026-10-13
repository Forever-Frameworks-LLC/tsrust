//! Contextual nested declarations retain explicit arity and lexical provenance.
use super::*;

#[derive(Default)]
pub(super) struct Instances {
    inherited: HashMap<String, Vec<crate::type_syntax::TypeName>>,
    children: HashMap<String, Vec<String>>,
    deferred: HashMap<String, Function>,
    bindings: HashMap<String, HashMap<String, ValueAnnotation>>,
}

pub(crate) fn declaration(name: &str) -> &str {
    name.split("<lexical#").next().unwrap_or(name)
}

impl Instances {
    pub(super) fn new(program: &Program) -> Self {
        let inherited = crate::lexical_generic_scopes::inherited(program);
        let declarations = program
            .local_declarations
            .iter()
            .map(|item| (item.function_owner, item))
            .collect::<HashMap<_, _>>();
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        for item in &program.local_declarations {
            if inherited.get(&item.emitted_name).is_none_or(Vec::is_empty) {
                continue;
            }
            let parent = declarations.get(&item.enclosing_owner).map_or_else(
                || program.functions[item.enclosing_owner].name.text.clone(),
                |parent| parent.emitted_name.clone(),
            );
            children
                .entry(parent)
                .or_default()
                .push(item.emitted_name.clone());
        }
        Self {
            inherited,
            children,
            ..Self::default()
        }
    }
    pub(super) fn needs_context(&self, name: &str) -> bool {
        self.inherited
            .get(name)
            .is_some_and(|variables| !variables.is_empty())
    }
    pub(super) fn inherited(&self, name: &str) -> Vec<crate::type_syntax::TypeName> {
        self.inherited.get(name).cloned().unwrap_or_default()
    }
    pub(super) fn defer(&mut self, function: Function) {
        self.deferred.insert(function.name.text.clone(), function);
    }
    pub(super) fn bindings(&self, name: &str) -> &HashMap<String, ValueAnnotation> {
        static EMPTY: std::sync::LazyLock<HashMap<String, ValueAnnotation>> =
            std::sync::LazyLock::new(HashMap::new);
        self.bindings.get(name).unwrap_or(&EMPTY)
    }
}

impl Cache {
    pub(super) fn contextualize(
        &mut self,
        function: &mut Function,
        bindings: &HashMap<String, ValueAnnotation>,
    ) -> Result<(), Diagnostic> {
        let original = function
            .application
            .as_ref()
            .map_or(function.name.text.as_str(), |application| {
                application.declaration.text.as_str()
            });
        let Some(children) = self.lexical.children.get(declaration(original)).cloned() else {
            return Ok(());
        };
        let names = children
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    format!("{name}<lexical#{}>", function.name.text),
                )
            })
            .collect::<HashMap<_, _>>();
        // Install all peer identities before visiting any body, including recursion.
        for name in children {
            let mut child = self.lexical.deferred[&name].clone();
            let replacements = self.lexical.inherited[&name]
                .iter()
                .map(|variable| {
                    let argument = bindings.get(&variable.text).cloned().ok_or_else(|| {
                        Diagnostic::new(
                            "generic-inference",
                            "Enclosing generic type binding is unavailable.",
                            variable.span,
                        )
                    })?;
                    Ok((variable.text.clone(), argument))
                })
                .collect::<Result<HashMap<_, _>, Diagnostic>>()?;
            let mut substitute = |annotation: &mut ValueAnnotation| {
                crate::type_substitution::apply(annotation, &replacements)
            };
            crate::source_walk::function_annotations(&mut child, &mut substitute)?;
            crate::source_walk::function_body(&mut child, &mut substitute, &mut |_| Ok(()))?;
            rewrite(&mut child, &names)?;
            child.name.text = names[&name].clone();
            self.lexical
                .bindings
                .insert(child.name.text.clone(), replacements.clone());
            child.application = Some(Box::new(FunctionApplication {
                lexical_owner: Some(function.name.text.clone()),
                declaration: self.lexical.deferred[&name].name.clone(),
                variables: self.lexical.inherited[&name].clone(),
                arguments: self.lexical.inherited[&name]
                    .iter()
                    .map(|variable| replacements[&variable.text].clone())
                    .collect(),
                patterns: self.lexical.deferred[&name]
                    .params
                    .iter()
                    .map(|(_, annotation, _)| annotation.clone())
                    .collect(),
            }));
            if child.type_parameters.is_empty() {
                self.pending.push(Pending {
                    function: child,
                    bindings: replacements,
                });
            } else {
                self.templates.insert(child.name.text.clone(), child);
            }
        }
        rewrite(function, &names)
    }
}

fn rewrite(function: &mut Function, names: &HashMap<String, String>) -> Result<(), Diagnostic> {
    crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
        if let ExprKind::Name(name) = &mut expression.kind {
            if let Some(replacement) = names.get(name) {
                *name = replacement.clone();
            }
        }
        Ok(())
    })
}
