//! Lexical type variables remain separate from explicit function type parameters.
use crate::syntax::Program;
use crate::type_syntax::TypeName;
use std::collections::HashMap;

pub(crate) fn inherited(program: &Program) -> HashMap<String, Vec<TypeName>> {
    let functions = program
        .functions
        .iter()
        .map(|function| (function.name.text.as_str(), function))
        .collect::<HashMap<_, _>>();
    let declarations = program
        .local_declarations
        .iter()
        .map(|declaration| (declaration.function_owner, declaration))
        .collect::<HashMap<_, _>>();
    let mut result = HashMap::new();
    for declaration in &program.local_declarations {
        let function = functions[declaration.emitted_name.as_str()];
        // Walk nearest scope first. Declaration spans retain variable identity
        // when two nested scopes use the same spelling.
        let mut shadowed = function
            .type_parameters
            .iter()
            .map(|parameter| parameter.text.as_str())
            .collect::<std::collections::HashSet<_>>();
        let mut owners = Vec::new();
        let mut owner = declaration.enclosing_owner;
        loop {
            let parent = if let Some(parent) = declarations.get(&owner) {
                owner = parent.enclosing_owner;
                functions[parent.emitted_name.as_str()]
            } else {
                let root = &program.functions[owner];
                owners.push(root);
                break;
            };
            owners.push(parent);
        }
        let mut scopes = Vec::new();
        for parent in owners {
            scopes.push(
                parent
                    .type_parameters
                    .iter()
                    .filter(|parameter| shadowed.insert(parameter.text.as_str()))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
        // Outer scopes precede inner scopes for stable specialization keys.
        result.insert(
            declaration.emitted_name.clone(),
            scopes.into_iter().rev().flatten().collect(),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Program {
        let formatted = crate::format(source).unwrap();
        assert_eq!(crate::format(&formatted).unwrap(), formatted);
        crate::syntax::parse(&formatted).unwrap()
    }

    #[test]
    fn lifted_functions_retain_lexical_identity_and_shadowing() {
        let source = "function outer<T>(value:T){function leaf():T{return value;}function middle<U>(other:U){function inner<T>(own:T):U{return other;}}}function separate<V>(){function sibling():V{return undefined;}}function main(){}";
        let program = parse(source);
        let scopes = inherited(&program);
        let scope = |name: &str| {
            let declaration = program
                .local_declarations
                .iter()
                .find(|item| item.source_name.text == name)
                .unwrap();
            &scopes[&declaration.emitted_name]
        };
        let outer_t = &program
            .functions
            .iter()
            .find(|item| item.name.text == "outer")
            .unwrap()
            .type_parameters[0];
        assert_eq!(scope("leaf"), &[outer_t.clone()]);
        assert_eq!(scope("middle"), &[outer_t.clone()]);
        let middle = program
            .functions
            .iter()
            .find(|item| item.name.text.ends_with("#middle"))
            .unwrap();
        assert_eq!(scope("inner"), &[middle.type_parameters[0].clone()]);
        assert_eq!(scope("sibling")[0].text, "V");
        assert!(!scopes.contains_key("main"));
        let inner = program
            .functions
            .iter()
            .find(|item| item.name.text.ends_with("#inner"))
            .unwrap();
        assert_eq!(
            inner.type_parameters.len(),
            1,
            "inherited variables must not change explicit arity"
        );
    }

    #[test]
    fn capture_free_declarations_inherit_types_without_value_capture_plans() {
        let program =
            parse("function outer<T>(){function inner(value:T):T{return value;}}function main(){}");
        assert!(program.local_capture_plans.is_empty());
        let scopes = inherited(&program);
        let declaration = &program.local_declarations[0];
        assert_eq!(scopes[&declaration.emitted_name][0].text, "T");
    }
}
