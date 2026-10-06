use std::collections::BTreeMap;
use tsrust_compiler::project::{
    ProjectAnalysis, ProjectExpressionTypeInfo, ProjectType, SourceId, SourceProvider, check_graph,
    load_project,
};

struct Memory(BTreeMap<String, String>);
impl SourceProvider for Memory {
    fn resolve(&self, _importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let name = path.trim_start_matches("./");
        self.0
            .contains_key(name)
            .then(|| SourceId(name.into()))
            .ok_or_else(|| format!("Missing {name}"))
    }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        self.0
            .get(&id.0)
            .cloned()
            .ok_or_else(|| format!("Missing {}", id.0))
    }
}
fn project(entry: &str, dependency: &str) -> ProjectAnalysis {
    let memory = Memory(
        [
            ("main.tsr".into(), entry.into()),
            ("dep.tsr".into(), dependency.into()),
        ]
        .into_iter()
        .collect(),
    );
    check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|error| panic!("{error:?}"))
}
fn text<'a>(entry: &'a str, fact: &ProjectExpressionTypeInfo) -> &'a str {
    assert_eq!(fact.location.source.0, "main.tsr");
    &entry[fact.location.span.start..fact.location.span.end]
}
fn record(ty: &ProjectType, dependency: &str) {
    let ProjectType::Named {
        name,
        declaration: Some(origin),
        arguments,
    } = ty
    else {
        panic!("expected nominal record: {ty:?}")
    };
    assert_eq!(name, "User");
    assert!(arguments.is_empty());
    assert_eq!(origin.source.0, "dep.tsr");
    assert_eq!(&dependency[origin.span.start..origin.span.end], "User");
}
#[test]
fn imported_nullable_views_and_optional_results_keep_source_identity() {
    let entry = "// 🦀\nimport {User as Person,make} from './dep.tsr';function main(){const user=make(true);if(user!==null){console.log(user.name);}const label=user?.name;console.log(String(label));}";
    let dependency = "export type User={name:string;};export function make(flag:boolean):User|null{return flag?{name:'Pat'}:null;}";
    let checked = project(entry, dependency);
    let facts = checked.checked_type_facts();
    let view = facts
        .projections
        .iter()
        .find(|fact| {
            fact.location.source.0 == "main.tsr"
                && text(entry, fact) == "user"
                && matches!(fact.ty, ProjectType::Named { .. })
        })
        .expect("checked strict guard payload");
    record(&view.ty, dependency);
    let optional = facts
        .optional_receivers
        .iter()
        .find(|fact| fact.location.source.0 == "main.tsr" && text(entry, fact) == "user")
        .expect("checked optional receiver payload");
    record(&optional.ty, dependency);
    let result = facts
        .method_results
        .iter()
        .find(|fact| fact.location.source.0 == "main.tsr" && text(entry, fact) == "user?.name")
        .expect("optional hop result");
    let ProjectType::Union(members) = &result.ty else {
        panic!("{result:?}")
    };
    assert!(
        members
            .iter()
            .any(|member| matches!(member,ProjectType::Named{name,..} if name=="string"))
    );
    assert!(
        members
            .iter()
            .any(|member| matches!(member,ProjectType::Named{name,..} if name=="undefined"))
    );
    let call = facts
        .method_results
        .iter()
        .find(|fact| fact.location.source.0 == "main.tsr" && text(entry, fact) == "make(true)")
        .expect("checked imported function result");
    let ProjectType::Union(members) = &call.ty else {
        panic!("{call:?}")
    };
    record(&members[0], dependency);
    assert!(matches!(&members[1],ProjectType::Named{name,..} if name=="null"));
    assert_eq!(facts, checked.checked_type_facts());
    assert_eq!(facts.generic_calls, checked.generic_calls());
}
#[test]
fn direct_callable_results_keep_original_expression_coordinates() {
    let entry = "import {User as Person} from './dep.tsr';function main(){const action=(person:Person):Person=>{return person;};const result=action({name:'Pat'});console.log(result.name);}";
    let dependency = "export type User={name:string;};";
    let checked = project(entry, dependency);
    let facts = checked.checked_type_facts();
    let result = facts
        .method_results
        .iter()
        .find(|fact| {
            fact.location.source.0 == "main.tsr" && text(entry, fact) == "action({name:'Pat'})"
        })
        .expect("checked direct callable result");
    record(&result.ty, dependency);
}

#[test]
fn specialized_arrow_callback_results_keep_the_checked_expression_type() {
    let entry = "function main(){const apply=(inner:ReadOnly<Function<int, [value:int]>>,value:int):int=>{return inner(value);};const increment=(value:int):int=>{return value+1;};const result=apply(increment,4);console.log(result);}";
    let checked = project(entry, "");
    let facts = checked.checked_type_facts();
    let result = facts
        .method_results
        .iter()
        .find(|fact| {
            fact.location.source.0 == "main.tsr" && text(entry, fact) == "apply(increment,4)"
        })
        .expect("checked specialized arrow invocation result");
    assert!(matches!(&result.ty,ProjectType::Named{name,declaration:None,..} if name=="int"));
}

#[test]
fn checked_callable_array_result_groups_payload_signature() {
    let source = "function empty():((value:int)=>int)[]{return [];}function main(){const actions=empty();console.log(actions.length);}";
    let checked = tsrust_compiler::check(source).unwrap();
    let start = source.find("empty();").unwrap();
    let facts = checked.checked_type_facts();
    let fact = facts
        .method_results
        .iter()
        .find(|fact| fact.span.start == start)
        .expect("checked array result");
    assert_eq!(fact.ty, "Function<int, [value: int]>[]");
}

#[test]
fn optional_callable_result_keeps_absence_without_hiding_nested_call_facts() {
    let entry = "function step(value:int):int{return value+1;}function main(){const action:(Function<int, [value:int]>)|null=step;const result=action?.(step(3));console.log(String(result));}";
    let checked = project(entry, "");
    let facts = checked.checked_type_facts();
    let results = facts
        .method_results
        .iter()
        .filter(|fact| text(entry, fact) == "action?.(step(3))")
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1, "{results:?}");
    let ProjectType::Union(members) = &results[0].ty else {
        panic!("{results:?}")
    };
    assert!(
        members
            .iter()
            .any(|member| matches!(member,ProjectType::Named{name,..} if name=="int"))
    );
    assert!(
        members
            .iter()
            .any(|member| matches!(member,ProjectType::Named{name,..} if name=="undefined"))
    );
    assert!(
        facts
            .method_results
            .iter()
            .any(|fact| text(entry, fact) == "step(3)"
                && matches!(&fact.ty,ProjectType::Named{name,..} if name=="int"))
    );
    assert!(facts.optional_receivers.iter().any(
        |fact| text(entry, fact) == "action" && matches!(fact.ty, ProjectType::Callable { .. })
    ));
}

#[test]
fn checked_polymorphic_callback_projection_keeps_declared_nested_contract() {
    let entry = "function invoke(action:(ReadOnly<Function<string, [direct:ReadOnly<Function<string, [text:ReadOnly<string>]>>]>>)|null):string{if(action===null){return 'absent';}const normalize=(text:string):string=>{return text.trim();};return action(normalize);}function main(){const action:(Function<string, [direct:ReadOnly<Function<string, [text:ReadOnly<string>]>>]>)|null=(operation:ReadOnly<Function<string, [input:ReadOnly<string>]>>):string=>{return operation(' Pat ');};console.log(invoke(action));}";
    let checked = project(entry, "");
    let facts = checked.checked_type_facts();
    let projected = facts
        .projections
        .iter()
        .filter(|fact| text(entry, fact) == "action")
        .collect::<Vec<_>>();
    assert_eq!(projected.len(), 1, "{projected:?}");
    let ProjectType::Callable {
        parameters, result, ..
    } = &projected[0].ty
    else {
        panic!("{projected:?}")
    };
    assert!(matches!(&**result,ProjectType::Named{name,..} if name=="string"));
    assert_eq!(parameters.len(), 1);
    assert!(matches!(
        parameters[0].ty,
        Some(ProjectType::Callable { .. })
    ));
}

#[test]
fn canonical_polymorphic_projection_has_one_callback_ownership_prefix() {
    let source = "function invoke(action:(ReadOnly<Function<string, [direct:ReadOnly<Function<string, [text:ReadOnly<string>]>>]>>)|null):string{if(action===null){return 'absent';}const normalize=(text:string):string=>{return text.trim();};return action(normalize);}function main(){const action:(Function<string, [direct:ReadOnly<Function<string, [text:ReadOnly<string>]>>]>)|null=(operation:ReadOnly<Function<string, [input:ReadOnly<string>]>>):string=>{return operation(' Pat ');};console.log(invoke(action));}";
    let facts = tsrust_compiler::check(source).unwrap().checked_type_facts();
    let span = source.find("action(normalize)").unwrap();
    let projection = facts
        .projections
        .iter()
        .find(|fact| fact.span.start == span)
        .expect("checked concrete declaration");
    assert_eq!(
        projection.ty,
        "Function<string, [operation: ReadOnly<Function<string, [input: ReadOnly<string>]>>]>"
    );
}
