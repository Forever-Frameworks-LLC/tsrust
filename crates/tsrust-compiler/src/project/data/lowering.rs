//! Project validated data directly into checked-language AST declarations.
use crate::{data_module::Value, syntax::{Expr, Record}, type_syntax::ValueAnnotation, Diagnostic};
pub(crate) struct Lowered { pub records: Vec<Record>, pub annotation: ValueAnnotation, pub value: Expr }
use crate::{data_module::Kind, data_shape::Shape, Span,
    syntax::{ExprKind, RecordField, Token, TokenKind},
    type_syntax::{TypeName, ValueAnnotationKind as Annotation}};

pub(crate) fn lower(value: &Value, identity: &str) -> Result<Lowered, Diagnostic> {
    let shape = crate::data_shape::infer(value).map_err(|error| Diagnostic::new("data-type", error.message, error.span))?;
    let mut records = Vec::new();
    let annotation = annotation(&shape, &[value], identity, &mut records);
    Ok(Lowered { records, annotation, value: expression(value, &shape) })
}
fn named(text: &str, span: Span) -> ValueAnnotation {
    ValueAnnotation { kind: Annotation::Named(TypeName { text: text.into(), span }), span }
}
fn accepts(shape: &Shape, value: &Value) -> bool {
    matches!((shape, &value.kind),
        (Shape::Null, Kind::Null) | (Shape::Boolean, Kind::Boolean(_))
        | (Shape::String, Kind::String(_)) | (Shape::Int | Shape::Number, Kind::Number(_))
        | (Shape::Array(_), Kind::Array(_)) | (Shape::Object(_), Kind::Object(_)))
}
fn annotation(shape: &Shape, samples: &[&Value], identity: &str, records: &mut Vec<Record>) -> ValueAnnotation {
    let span = samples.first().expect("inferred shape has an input").span;
    let kind = match shape {
        Shape::Null => return named("null", span),
        Shape::Boolean => return named("boolean", span),
        Shape::Int => return named("int", span),
        Shape::Number => return named("number", span),
        Shape::String => return named("string", span),
        Shape::Array(Some(element)) => {
            let children: Vec<_> = samples.iter().flat_map(|value| match &value.kind { Kind::Array(values) => values.iter().collect::<Vec<_>>(), _ => vec![] }).collect();
            Annotation::Array(Box::new(annotation(element, &children, identity, records)))
        }
        Shape::Array(None) => unreachable!("inference rejects unconstrained empty arrays"),
        Shape::Nullable(inner) => {
            let present: Vec<_> = samples.iter().copied().filter(|value| value.kind != Kind::Null).collect();
            Annotation::Union(vec![annotation(inner, &present, identity, records), named("null", span)])
        }
        Shape::Union(alternatives) => Annotation::Union(alternatives.iter().map(|shape| {
            let selected: Vec<_> = samples.iter().copied().filter(|value| accepts(shape, value)).collect();
            annotation(shape, &selected, identity, records)
        }).collect()),
        Shape::Object(fields) => {
            let index = records.len();
            let name = format!("@data-record:{identity}:{index}");
            records.push(Record { native_type: None, set_definition: None, map_definition: None, enum_definition: None,literal_definition:None,
                application: None, name: Token { text: name.clone(), span, kind: TokenKind::Symbol }, type_parameters: vec![], fields: vec![] });
            let fields = fields.iter().map(|(name, field)| {
                let properties: Vec<_> = samples.iter().filter_map(|value| match &value.kind { Kind::Object(values) => values.get(name), _ => None }).collect();
                let values: Vec<_> = properties.iter().map(|property| &property.value).collect();
                let name_span = properties[0].name_span;
                let mut annotation = annotation(&field.shape, &values, identity, records);
                if field.optional { annotation = ValueAnnotation { kind: Annotation::Union(vec![annotation, named("undefined", name_span)]), span: name_span }; }
                RecordField { name: Token { text: name.clone(), span: name_span, kind: TokenKind::Name }, annotation,
                    ty: None, callable_declaration: None, optional: field.optional }
            }).collect();
            records[index].fields = fields;
            return named(&name, span);
        }
    };
    ValueAnnotation { kind, span }
}
fn expression(value: &Value, shape: &Shape) -> Expr {
    let shape = match shape {
        Shape::Nullable(inner) if value.kind != Kind::Null => inner.as_ref(),
        other => other,
    };
    if let Shape::Union(alternatives) = shape {
        return expression(value, alternatives.iter().find(|shape| accepts(shape, value)).expect("inferred alternative"));
    }
    let kind = match &value.kind {
        Kind::Null => ExprKind::Null,
        Kind::Boolean(value) => ExprKind::Boolean(*value),
        Kind::String(value) => ExprKind::String(value.clone()),
        Kind::Number(text) if *shape == Shape::Int => ExprKind::Integer(text.clone()),
        Kind::Number(text) => ExprKind::Number(text.parse().expect("validated finite JSON number")),
        Kind::Array(values) => {
            let Shape::Array(Some(element)) = shape else { unreachable!("inferred array") };
            ExprKind::Array(values.iter().map(|value| expression(value, element)).collect())
        }
        Kind::Object(values) => {
            let Shape::Object(fields) = shape else { unreachable!("inferred object") };
            ExprKind::Record(values.iter().map(|(name, property)| {
                (Token { text: name.clone(), kind: TokenKind::Name, span: property.name_span }, expression(&property.value, &fields[name].shape))
            }).collect())
        }
    };
    Expr::new(kind, value.span)
}

pub(crate) const OWNER: &str = "@data-owner";
pub(crate) const ACCESS: &str = "@data:";
pub(crate) fn program(value: &Value, identity: &str) -> Result<crate::syntax::Program, Diagnostic> {
    use crate::syntax::{Program, Function, ResultAnnotation, Stmt, StmtKind};
    let lowered = lower(value, identity)?;
    let span = value.span;
    Ok(Program {
        diagnostic_first: None, diagnostic_skips: Default::default(), diagnostic_contracts: Default::default(), unresolved_local_function: None,
        local_declarations: vec![], local_capture_plans: vec![], type_argument_checks: vec![], call_type_arguments: vec![],
        module_bindings: vec![],
        module_owner_bindings: vec![],
        records: lowered.records, type_aliases: Default::default(), unresolved_bases: vec![],
        functions: vec![Function { application: None, scope_span: span,
            name: Token { text: OWNER.into(), span, kind: TokenKind::Symbol }, type_parameters: vec![], params: vec![],
            parameter_initializers: vec![], parameter_access: vec![], rest_parameter: false,
            result: Some(ResultAnnotation::Named(Box::new(lowered.annotation))),
            body: vec![Stmt { kind: StmtKind::Return(Some(lowered.value)), span, control_target: None, labels: vec![] }],
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn checked(json: &str, statements: &str) -> Result<crate::analysis::CheckedProgram, Diagnostic> {
        let data = crate::data_module::parse(json, false).unwrap();
        let lowered = lower(&data, "test")?;
        let source = format!("function main(): void {{ const data = null; {statements} }}");
        let formatted = crate::formatting::format(&source)?;
        assert_eq!(crate::formatting::format(&formatted)?, formatted);
        let mut program = crate::syntax::parse(&formatted)?;
        program.records.extend(lowered.records);
        let crate::syntax::StmtKind::Let { annotation, value, .. } = &mut program.functions[0].body[0].kind else { panic!("let") };
        *annotation = Some(lowered.annotation); *value = lowered.value;
        crate::analysis::analyze(program)
    }
    #[test]
    fn data_lowering_checks_nested_records_and_unknown_fields() {
        checked(r#"{"server":{"port":8080,"host":"local"}}"#, "console.log(data.server.host); console.log(data.server.port);").unwrap();
        assert!(checked(r#"{"server":{"port":8080}}"#, "console.log(data.server.missing);").is_err());
        let parsed = crate::data_module::parse(r#"{"a-b":1}"#, false).unwrap();
        let lowered = lower(&parsed, "other").unwrap();
        assert_eq!(lowered.records[0].fields[0].name.text, "a-b");
        assert_eq!(lowered.records[0].fields[0].name.span, crate::Span::new(1,6));
    }
    #[test]
    fn data_lowering_executes_optional_union_and_nested_array_values() {
        let json = r#"{"rows":[{"x":1},{"x":"s","extra":true}],"matrix":[[],[1,2]],"nullable":[null,3],"numbers":[1,2.5],"server":{"host":"local","port":8080}}"#;
        let checked = checked(json, "for (const row of data.rows) { console.log(row.x); console.log(row.extra); } for (const row of data.matrix) { for (const value of row) { console.log(value); } } for (const value of data.nullable) { console.log(value); } for (const value of data.numbers) { console.log(value); } console.log(data.server.host); console.log(data.server.port);").unwrap();
        let source = crate::emit::emit(&checked);
        let directory = std::env::temp_dir().join(format!("tsrust-data-lowering-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(directory.clone());
        let input = directory.join("main.rs"); let output = directory.join("program");
        std::fs::write(&input, source).unwrap();
        let built = std::process::Command::new("rustup").args(["run", crate::RUST_BACKEND_VERSION, "rustc", "--edition=2024"]).arg(&input).arg("-o").arg(&output).output().unwrap();
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        let run = std::process::Command::new(&output).output().unwrap();
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_eq!(String::from_utf8(run.stdout).unwrap(), "1\nundefined\ns\ntrue\n1\n2\nnull\n3\n1\n2.5\nlocal\n8080\n");
    }
}
