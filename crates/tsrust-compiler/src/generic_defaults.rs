//! Declaration-time constraints for defaults with type-variable-dependent slots.
//! Concrete specializations still use the ordinary expression/ownership checker.
use crate::Diagnostic;
use crate::syntax::{
    BinaryOp, Expr, ExprKind, Function, ParameterInitializer, Program, ResultAnnotation, UnaryOp,
};
use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind as Kind};
use std::collections::{HashMap, HashSet};

#[path = "generic_default_scope.rs"]
mod scope;

#[derive(Clone, PartialEq, Eq)]
enum Shape {
    Named(String),
    Array(Box<Shape>),
    Applied(String, Vec<Shape>),
    Union(Vec<Shape>),
    Tuple(Vec<Shape>),
    Callable(Vec<Shape>, Box<Shape>),
    Empty,
}
fn annotation(value: &ValueAnnotation) -> Shape {
    match &value.kind {
        Kind::Named(name) => Shape::Named(name.text.clone()),
        Kind::Group(inner) => annotation(inner),
        Kind::Array(inner) => Shape::Array(Box::new(annotation(inner))),
        Kind::Applied { name, arguments } => Shape::Applied(
            name.text.clone(),
            arguments.iter().map(annotation).collect(),
        ),
        Kind::Tuple(members) => Shape::Tuple(members.iter().map(annotation).collect()),
        Kind::Union(members) => Shape::Union(members.iter().map(annotation).collect()),
        Kind::Callable {
            parameters, result, ..
        } => Shape::Callable(
            parameters
                .iter()
                .map(|parameter| annotation(&parameter.annotation))
                .collect(),
            Box::new(annotation(result)),
        ),
    }
}
fn variable(value: &Shape, variables: &HashSet<&str>) -> bool {
    match value {
        Shape::Named(name) => variables.contains(name.as_str()),
        Shape::Array(inner) => variable(inner, variables),
        Shape::Applied(_, items) | Shape::Union(items) | Shape::Tuple(items) => {
            items.iter().any(|item| variable(item, variables))
        }
        Shape::Callable(params, result) => {
            params.iter().any(|param| variable(param, variables)) || variable(result, variables)
        }
        _ => false,
    }
}
fn accepts(expected: &Shape, actual: &Shape) -> bool {
    if expected == actual {
        return true;
    }
    if let Shape::Union(members) = actual {
        return members.iter().all(|actual| accepts(expected, actual));
    }
    if let Shape::Union(members) = expected {
        return members.iter().any(|expected| accepts(expected, actual));
    }
    match (expected, actual) {
        (Shape::Named(expected), Shape::Named(actual)) if actual == "<integer literal>" => {
            matches!(expected.as_str(), "int" | "number")
        }
        (Shape::Array(_), Shape::Array(inner)) if **inner == Shape::Empty => true,
        (Shape::Array(expected), Shape::Array(actual)) => accepts(expected, actual),
        (Shape::Callable(x, a), Shape::Callable(y, b)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| accepts(x, y)) && accepts(a, b)
        }
        (Shape::Applied(a, x), Shape::Applied(b, y)) => {
            a == b && x.len() == y.len() && x.iter().zip(y).all(|(x, y)| accepts(x, y))
        }
        _ => false,
    }
}
fn substitute(value: Shape, bindings: &HashMap<String, Shape>) -> Shape {
    match value {
        Shape::Named(name) => bindings.get(&name).cloned().unwrap_or(Shape::Named(name)),
        Shape::Array(inner) => Shape::Array(Box::new(substitute(*inner, bindings))),
        Shape::Applied(name, args) => Shape::Applied(
            name,
            args.into_iter()
                .map(|arg| substitute(arg, bindings))
                .collect(),
        ),
        Shape::Union(args) => Shape::Union(
            args.into_iter()
                .map(|arg| substitute(arg, bindings))
                .collect(),
        ),
        Shape::Callable(params, result) => Shape::Callable(
            params
                .into_iter()
                .map(|param| substitute(param, bindings))
                .collect(),
            Box::new(substitute(*result, bindings)),
        ),
        other => other,
    }
}
fn infer(
    pattern: &Shape,
    actual: &Shape,
    variables: &HashSet<&str>,
    bindings: &mut HashMap<String, Shape>,
) {
    match (pattern, actual) {
        (Shape::Named(name), actual) if variables.contains(name.as_str()) => {
            bindings
                .entry(name.clone())
                .or_insert_with(|| actual.clone());
        }
        (Shape::Array(x), Shape::Array(y)) => infer(x, y, variables, bindings),
        (Shape::Applied(a, x), Shape::Applied(b, y)) if a == b => {
            for (x, y) in x.iter().zip(y) {
                infer(x, y, variables, bindings);
            }
        }
        _ => {}
    }
}
fn result(value: &ResultAnnotation) -> Shape {
    match value {
        ResultAnnotation::Inferred(_) => Shape::Empty,
        ResultAnnotation::Named(value) => annotation(value),
        ResultAnnotation::Callable {
            params,
            result: value,
            ..
        } => Shape::Callable(
            params
                .iter()
                .map(|param| match &param.ty {
                    crate::syntax::ParameterAnnotation::Named(value) => annotation(value),
                    crate::syntax::ParameterAnnotation::Callback(callback) => {
                        annotation(&callback.value_annotation())
                    }
                    crate::syntax::ParameterAnnotation::Inferred(_) => Shape::Empty,
                })
                .collect(),
            Box::new(result(value)),
        ),
    }
}
fn absent(shape: &Shape) -> bool {
    matches!(shape,Shape::Named(name) if name=="null" || name=="undefined")
}
fn with_undefined(shape: Shape) -> Shape {
    let mut members = match shape {
        Shape::Union(items) => items,
        item => vec![item],
    };
    let undefined = Shape::Named("undefined".into());
    if !members.contains(&undefined) {
        members.push(undefined);
    }
    if members.len() == 1 {
        members.pop().unwrap()
    } else {
        Shape::Union(members)
    }
}
fn literal_depth(value: &Expr) -> Option<i64> {
    match &value.kind {
        ExprKind::Integer(text) => text.parse().ok(),
        ExprKind::Unary {
            op: UnaryOp::Positive,
            value,
            ..
        } => literal_depth(value),
        ExprKind::Unary {
            op: UnaryOp::Negate,
            value,
            ..
        } => match &value.kind {
            ExprKind::Integer(text) => format!("-{text}").parse().ok(),
            _ => literal_depth(value)?.checked_neg(),
        },
        _ => None,
    }
}
struct Types<'a> {
    functions: HashMap<&'a str, &'a Function>,
    records: HashMap<&'a str, &'a crate::syntax::Record>,
    calls: &'a [(crate::Span, Vec<ValueAnnotation>)],
}
impl Types<'_> {
    // Logical result shapes mirror the ordinary builtin checker. This pass
    // constrains unused generic defaults, not argument/ownership validity.
    fn method_result(
        &self,
        receiver: Shape,
        method: &str,
        args: &[Expr],
        names: &HashMap<String, Shape>,
    ) -> Option<Shape> {
        let named = |name: &str| Some(Shape::Named(name.into()));
        match receiver {
            Shape::Named(ref name) if name == "string" => match method {
                "repeat" | "concat" | "slice" | "substring" | "substr" | "toUpperCase"
                | "toLocaleUpperCase" | "toLocaleLowerCase" | "toLowerCase" | "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight"
                | "replace" | "replaceAll" | "charAt" | "padStart" | "padEnd" | "toWellFormed"
                | "toString" | "valueOf" => named("string"),
                "includes" | "startsWith" | "endsWith" | "isWellFormed" => named("boolean"),
                "indexOf" | "lastIndexOf" => named("int"),
                "charCodeAt" => named("number"),
                "split" => Some(Shape::Array(Box::new(Shape::Named("string".into())))),
                method if crate::syntax::string_html_target(method).is_some() => named("string"),
                _ => None,
            },
            Shape::Named(ref name)
                if matches!(
                    name.as_str(),
                    "int" | "<integer literal>" | "number" | "boolean"
                ) =>
            {
                match method {
                    "valueOf" => Some(receiver),
                    "toString" if name != "number" => named("string"),
                    "toFixed" | "toExponential" | "toPrecision" if name == "number" => {
                        named("string")
                    }
                    _ => None,
                }
            }
            Shape::Array(element) => match method {
                "join" | "toString" => named("string"),
                "some" | "every" | "includes" => named("boolean"),
                "findIndex" | "findLastIndex" | "indexOf" | "lastIndexOf" | "unshift" => {
                    named("int")
                }
                "push" | "reverse" | "sort" | "fill" | "copyWithin" | "forEach" => named("void"),
                "at" | "find" | "findLast" | "pop" | "shift" => Some(with_undefined(*element)),
                "remove" => Some(*element),
                "slice" | "concat" | "filter" | "toReversed" | "toSorted" | "with"
                | "toSpliced" | "splice" => Some(Shape::Array(element)),
                "flat" => {
                    let mut output = Shape::Array(element);
                    let depth = match args.first() {
                        None => 1,
                        Some(value) => literal_depth(value).or_else(|| matches!(&output,Shape::Array(inner) if !matches!(**inner,Shape::Array(_))).then_some(0))?,
                    };
                    for _ in 0..depth.max(0).min(127) {
                        output = match output {
                            Shape::Array(inner) => match *inner {
                                Shape::Array(inner) => Shape::Array(inner),
                                leaf => {
                                    output = Shape::Array(Box::new(leaf));
                                    break;
                                }
                            },
                            _ => unreachable!(),
                        };
                    }
                    Some(output)
                }
                "map" | "flatMap" | "reduce" | "reduceRight" => {
                    let Shape::Callable(_, result) = self.expression(args.first()?, names)? else {
                        return None;
                    };
                    Some(match method {
                        "map" => Shape::Array(result),
                        "flatMap" if !matches!(*result, Shape::Array(_)) => Shape::Array(result),
                        _ => *result,
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }
    fn chain_may_skip(&self, mut value: &Expr, names: &HashMap<String, Shape>) -> bool {
        loop {
            match &value.kind {
                ExprKind::Unary {
                    op: UnaryOp::OptionalBoundary,
                    value: base,
                    ..
                } => {
                    if self
                        .expression(base, names)
                        .is_some_and(|shape| match shape {
                            Shape::Union(members) => members.iter().any(absent),
                            member => absent(&member),
                        })
                    {
                        return true;
                    }
                    value = base;
                }
                ExprKind::Field(base, _) | ExprKind::Index(base, _) | ExprKind::Call(base, _) => {
                    value = base
                }
                _ => return false,
            }
        }
    }
    fn expression(&self, value: &Expr, names: &HashMap<String, Shape>) -> Option<Shape> {
        let named = |name: &str| Some(Shape::Named(name.into()));
        match &value.kind {
            ExprKind::Integer(_) => named("<integer literal>"),
            ExprKind::Number(_) => named("number"),
            ExprKind::String(_) | ExprKind::Template(_) => named("string"),
            ExprKind::Boolean(_) => named("boolean"),
            ExprKind::Null => named("null"),
            ExprKind::Undefined => named("undefined"),
            ExprKind::Name(name) => names
                .get(name)
                .cloned()
                .or_else(|| {
                    self.functions.get(name.as_str()).map(|function| {
                        Shape::Callable(
                            function
                                .params
                                .iter()
                                .map(|(_, annotation_value, _)| annotation(annotation_value))
                                .collect(),
                            Box::new(
                                function
                                    .result
                                    .as_ref()
                                    .map(result)
                                    .unwrap_or_else(|| Shape::Named("void".into())),
                            ),
                        )
                    })
                })
                .or_else(|| {
                    crate::native_bindings::by_symbol(name).map(|(_, binding)| {
                        let shape = |ty| {
                            Shape::Named(
                                match ty {
                                    crate::native_bindings::NativeType::String => "string",
                                    crate::native_bindings::NativeType::Boolean => "boolean",
                                    crate::native_bindings::NativeType::Int => "int",
                                    crate::native_bindings::NativeType::Number => "number",
                                    crate::native_bindings::NativeType::Void => "void",
                                    crate::native_bindings::NativeType::OwnedVoidCallback | crate::native_bindings::NativeType::SharedCallback => "Owned<Function<void>>",
                                    crate::native_bindings::NativeType::ObservedValue => "unknown",
                                }
                                .into(),
                            )
                        };
                        Shape::Callable(
                            binding
                                .parameters
                                .iter()
                                .map(|parameter| shape(parameter.ty))
                                .collect(),
                            Box::new(shape(binding.result)),
                        )
                    })
                })
                .or_else(|| {
                    (name == "undefined" && !self.functions.contains_key(name.as_str()))
                        .then(|| Shape::Named("undefined".into()))
                }),
            ExprKind::Closure {
                params,
                result: value,
                ..
            } => {
                let params = params
                    .iter()
                    .map(|param| match &param.ty {
                        crate::syntax::ParameterAnnotation::Named(value) => Some(annotation(value)),
                        crate::syntax::ParameterAnnotation::Callback(callback) => {
                            Some(annotation(&callback.value_annotation()))
                        }
                        crate::syntax::ParameterAnnotation::Inferred(_) => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Shape::Callable(params, Box::new(result(value))))
            }
            ExprKind::Array(values) => {
                let mut items = values.iter().map(|value| self.expression(value, names));
                let first = items.next().unwrap_or(Some(Shape::Empty))?;
                if items.all(|item| item.as_ref() == Some(&first)) {
                    Some(Shape::Array(Box::new(first)))
                } else {
                    None
                }
            }
            ExprKind::ArrayOf(values, explicit) => {
                if let Some(explicit) = explicit {
                    Some(Shape::Array(Box::new(annotation(explicit))))
                } else {
                    let first = values
                        .first()
                        .and_then(|value| self.expression(value, names))?;
                    if values
                        .iter()
                        .skip(1)
                        .all(|value| self.expression(value, names).as_ref() == Some(&first))
                    {
                        Some(Shape::Array(Box::new(first)))
                    } else {
                        None
                    }
                }
            }
            ExprKind::TypeAssertion { value, annotation: target, satisfies } => {
                if *satisfies { self.expression(value, names) } else { Some(annotation(target)) }
            }
            ExprKind::OptionalChain(body) => {
                let actual = self.expression(body, names)?;
                if self.chain_may_skip(body, names) {
                    Some(with_undefined(actual))
                } else {
                    Some(actual)
                }
            }
            ExprKind::Unary { op, value, .. } => match op {
                UnaryOp::Not => named("boolean"),
                UnaryOp::Typeof => named("string"),
                UnaryOp::Void => named("undefined"),
                UnaryOp::OptionalBoundary => {
                    let value = self.expression(value, names)?;
                    match value {
                        Shape::Union(members) => {
                            let mut payloads = members
                                .into_iter()
                                .filter(|item| !absent(item))
                                .collect::<Vec<_>>();
                            if payloads.len() == 1 {
                                payloads.pop()
                            } else {
                                None
                            }
                        }
                        other if absent(&other) => None,
                        other => Some(other),
                    }
                }
                UnaryOp::Negate | UnaryOp::Positive | UnaryOp::BitNot => {
                    self.expression(value, names)
                }
                _ => None,
            },
            ExprKind::Binary {
                op, left, right, ..
            } => {
                match op {
                    BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::Equal
                    | BinaryOp::NotEqual => named("boolean"),
                    BinaryOp::And | BinaryOp::Or => {
                        let a = self.expression(left, names)?;
                        let b = self.expression(right, names)?;
                        if a == b { Some(a) } else { None }
                    }
                    BinaryOp::Coalesce | BinaryOp::PatternDefault => {
                        let left = self.expression(left, names)?;
                        let mut members = match left {
                            Shape::Union(items) => items,
                            other => vec![other],
                        };
                        let undefined_only=*op==BinaryOp::PatternDefault;
                        let absent=members.iter().any(|item|matches!(item,Shape::Named(name) if name=="undefined"||(!undefined_only && name=="null")));
                        members.retain(|item|!matches!(item,Shape::Named(name) if name=="undefined"||(!undefined_only && name=="null")));
                        if absent {
                            let right = self.expression(right, names)?;
                            for item in match right {
                                Shape::Union(items) => items,
                                other => vec![other],
                            } {
                                if !members.contains(&item) {
                                    members.push(item);
                                }
                            }
                        }
                        if members.len() == 1 {
                            members.pop()
                        } else {
                            Some(Shape::Union(members))
                        }
                    }
                    _ => {
                        let a = self.expression(left, names)?;
                        let b = self.expression(right, names)?;
                        if a == b {
                            Some(a)
                        } else if matches!((&a,&b),(Shape::Named(a),Shape::Named(b)) if (a=="int"&&b=="<integer literal>")||(b=="int"&&a=="<integer literal>"))
                        {
                            named("int")
                        } else {
                            None
                        }
                    }
                }
            }
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                let a = self.expression(consequent, names)?;
                let b = self.expression(alternate, names)?;
                if a == b {
                    Some(a)
                } else {
                    Some(Shape::Union(vec![a, b]))
                }
            }
            ExprKind::Field(base, field) => {
                if let ExprKind::Name(namespace) = &base.kind
                    && !names.contains_key(namespace)
                    && !self.functions.contains_key(namespace.as_str())
                    && crate::syntax::NumericConstant::lookup(namespace, field).is_some()
                {
                    return named("number");
                }
                let base = self.expression(base, names)?;
                if field == "length"
                    && (matches!(&base, Shape::Array(_))
                        || matches!(&base,Shape::Named(name) if name=="string"))
                {
                    return named("int");
                }
                let (name, args) = match base {
                    Shape::Named(name) => (name, Vec::new()),
                    Shape::Applied(name, args) => (name, args),
                    _ => return None,
                };
                let record = self.records.get(name.as_str())?;
                let field = record
                    .fields
                    .iter()
                    .find(|candidate| candidate.name.text == *field)?;
                let bindings = record
                    .type_parameters
                    .iter()
                    .zip(args)
                    .map(|(parameter, argument)| (parameter.text.clone(), argument))
                    .collect();
                Some(substitute(annotation(&field.annotation), &bindings))
            }
            ExprKind::Index(base, _) => {
                if let Shape::Array(inner) = self.expression(base, names)? {
                    Some(*inner)
                } else {
                    None
                }
            }
            ExprKind::Call(callee, args) => {
                if let ExprKind::Name(name) = &callee.kind {
                    if let Some(bound) = names.get(name) {
                        return match bound {
                            Shape::Callable(_, result) => Some((**result).clone()),
                            _ => None,
                        };
                    }
                    if let Some((_, binding)) = crate::native_bindings::by_symbol(name) {
                        use crate::native_bindings::NativeType;
                        return named(match binding.result {
                            NativeType::String => "string",
                            NativeType::Boolean => "boolean",
                            NativeType::Int => "int",
                            NativeType::Number => "number",
                            NativeType::Void => "void",
                            NativeType::OwnedVoidCallback | NativeType::SharedCallback | NativeType::ObservedValue => return None,
                        });
                    }
                    if name == "clone" && !self.functions.contains_key(name.as_str()) {
                        return args.first().and_then(|arg| self.expression(arg, names));
                    }
                    if let Some(function) = self.functions.get(name.as_str()) {
                        let result = function
                            .result
                            .as_ref()
                            .map(result)
                            .unwrap_or_else(|| Shape::Named("void".into()));
                        let variables = function
                            .type_parameters
                            .iter()
                            .map(|name| name.text.as_str())
                            .collect::<HashSet<_>>();
                        let mut bindings = HashMap::new();
                        if let Some((_, arguments)) =
                            self.calls.iter().find(|(span, _)| *span == value.span)
                        {
                            for (parameter, argument) in
                                function.type_parameters.iter().zip(arguments)
                            {
                                bindings.insert(parameter.text.clone(), annotation(argument));
                            }
                        }
                        for ((_, pattern, _), argument) in function.params.iter().zip(args) {
                            if let Some(actual) = self.expression(argument, names) {
                                infer(&annotation(pattern), &actual, &variables, &mut bindings);
                            }
                        }
                        return Some(substitute(result, &bindings));
                    }
                    return match name.as_str() {
                        "String" => named("string"),
                        "Boolean" => named("boolean"),
                        "Number" => named("number"),
                        _ => None,
                    };
                }
                if let ExprKind::Field(base, field) = &callee.kind {
                    if let ExprKind::Name(namespace) = &base.kind
                        && !names.contains_key(namespace)
                        && !self.functions.contains_key(namespace.as_str())
                    {
                        if namespace == "console"
                            && matches!(
                                field.as_str(),
                                "log"
                                    | "info"
                                    | "debug"
                                    | "warn"
                                    | "error"
                                    | "table"
                                    | "assert"
                                    | "dir"
                                    | "dirxml"
                                    | "group"
                                    | "groupCollapsed"
                                    | "groupEnd"
                                    | "count"
                                    | "countReset"
                                    | "time"
                                    | "timeLog"
                                    | "timeEnd"
                                    | "clear"
                                    | "trace"
                                    | "profile"
                                    | "profileEnd"
                                    | "timeStamp"
                            )
                        {
                            return named("void");
                        }
                        if namespace == "Math" && crate::analysis::math_target(field).is_some() {
                            return named("number");
                        }
                        if namespace == "Number" && matches!(field.as_str(), "parseFloat" | "parseInt") {
                            return named("number");
                        }
                        if namespace == "Number"
                            && crate::analysis::number_predicate_target(field).is_some()
                        {
                            return named("boolean");
                        }
                        if namespace == "String" && matches!(field.as_str(), "fromCodePoint" | "fromCharCode") {
                            return named("string");
                        }
                        if namespace == "Object" && field == "is" {
                            return named("boolean");
                        }
                        if namespace == "Array" && field == "isArray" {
                            return named("boolean");
                        }
                    }
                    // A nominal callable field takes precedence over builtin method names.
                    if let Some(Shape::Callable(_, result)) = self.expression(callee, names) {
                        return Some(*result);
                    }
                    return self.method_result(self.expression(base, names)?, field, args, names);
                }
                if let Some(Shape::Callable(_, result)) = self.expression(callee, names) {
                    return Some(*result);
                }
                None
            }
            _ => None,
        }
    }
}
pub(crate) fn validate(program: &Program) -> Result<(), Diagnostic> {
    scope::validate(program)?;
    let types = Types {
        functions: program
            .functions
            .iter()
            .map(|function| (function.name.text.as_str(), function))
            .collect(),
        records: program
            .records
            .iter()
            .map(|record| (record.name.text.as_str(), record))
            .collect(),
        calls: &program.call_type_arguments,
    };
    for function in &program.functions {
        if function.type_parameters.is_empty() {
            continue;
        }
        let variables = function
            .type_parameters
            .iter()
            .map(|name| name.text.as_str())
            .collect::<HashSet<_>>();
        let mut names = HashMap::new();
        for ((name, expected, _), initializer) in
            function.params.iter().zip(&function.parameter_initializers)
        {
            let expected = annotation(expected);
            if variable(&expected, &variables)
                && let ParameterInitializer::Default { value, .. } = initializer
                && let Some(actual) = types.expression(value, &names)
                && !accepts(&expected, &actual)
            {
                return Err(Diagnostic::new(
                    "type",
                    "Default initializer is not assignable to every type permitted by its unconstrained generic parameter.",
                    value.span,
                ));
            }
            names.insert(name.text.clone(), expected);
        }
    }
    Ok(())
}
