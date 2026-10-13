//! Parser nesting witnesses on the ordinary test worker stack. The sizes are
//! test inputs, not support limits.
use super::*;

const DEEP: usize = 10_000;

fn nest(seed: &str, layers: usize, layer: impl Fn(&str) -> String) -> String {
    let mut value = seed.to_owned();
    for _ in 0..layers {
        value = layer(&value);
    }
    value
}

fn main_body(body: &str) -> String {
    format!("function identity(value:int):int{{return value;}}function main(){{{body}}}")
}

fn functions(source: &str) -> Vec<Function> {
    parse_unbounded(source).unwrap_or_else(|error| panic!("{}: {}", error.code, error.message)).functions
}

/// Count the expressions below `main` with the shared worklist walker.
fn expressions(source: &str) -> usize {
    let mut functions = functions(source);
    let mut count = 0;
    crate::source_walk::body(&mut functions[1].body, &mut |_| Ok(()), &mut |_| {
        count += 1;
        Ok(())
    })
    .unwrap();
    count
}

#[test]
fn suspended_scopes_of_every_grammar_family_stay_on_the_heap() {
    let repeat = |open: &str, seed: &str, close: &str| format!("{}{seed}{}", open.repeat(DEEP), close.repeat(DEEP));
    let sources = [
        main_body(&format!("console.log({});", repeat("(", "7", ")"))),
        main_body(&repeat("{", "console.log(7);", "}")),
        main_body(&format!("const v={};", repeat("[", "7", "]"))),
        main_body(&format!("const v={};", repeat("{a:", "7", "}"))),
        main_body(&format!("console.log({});", repeat("identity(", "7", ")"))),
        main_body(&format!("const v={};", repeat("a[", "0", "]"))),
        main_body(&format!("const v={};", repeat("`a${", "7", "}b`"))),
        main_body(&format!("const v={}a;", "!".repeat(DEEP))),
        main_body(&format!("const v={}1;", "a?b:".repeat(DEEP))),
        main_body(&format!("const v={};", repeat("a?", "b", ":c"))),
        main_body(&format!("const v={}1;", "a=>".repeat(DEEP))),
        main_body(&format!("const v={};", repeat("(():int=>{return ", "7", ";})()"))),
        main_body(&format!("const v={};", repeat("function(){return ", "1", ";}"))),
        main_body(&format!("const v={};", repeat("(function f(){return ", "1", ";})()"))),
        main_body(&format!("const v={};", repeat("((a:int=", "1", "):int=>a)()"))),
        main_body(&repeat("function next():int{", "return 1;", "}")),
        format!("function use(value:int={}):int{{return value;}}", repeat("f(", "1", ")")),
        main_body(&repeat("if(a){", "", "}")),
        main_body(&format!("{}b();", "if(a)".repeat(DEEP))),
        main_body(&format!("if(a){{}}{}", "else if(a){}".repeat(DEEP))),
        main_body(&repeat("for(const x of xs){while(a){do{for(;;){", "", "}}while(a);}}")),
        main_body(&repeat("try{", "throw 1;", "}catch(e){}finally{}")),
        main_body(&repeat("switch(a){case 1:", "break;", "}")),
        main_body(&repeat("match(v){Ok(x)=>{", "", "}}")),
        main_body(&format!("const {}=v;", repeat("[", "a", "]"))),
        main_body(&format!("const {}=v;", repeat("{a:", "b", "}"))),
        main_body(&format!("const {}=v;", nest("a", DEEP, |inner| format!("[{inner}=[1],b=2]")))),
        main_body(&format!("const f=({}:T)=>a;", repeat("[", "a", "]"))),
        main_body(&format!("{}=v;", repeat("[", "a", "]"))),
        main_body(&format!("const v:{}=x;", repeat("(", "int", ")"))),
        main_body(&format!("const v:{}=x;", repeat("Box<", "int", ">"))),
        main_body(&format!("const v:{}=x;", repeat("Function<int,[", "int", "]>"))),
        main_body(&format!("const v:{}int=x;", "()=>".repeat(DEEP))),
        main_body(&format!("const v=x as {};", repeat("[", "int", "]"))),
        format!("enum E{{A={},B={}1}}", repeat("(", "1", ")"), "-~".repeat(DEEP)),
    ];
    for source in &sources {
        let functions = functions(source);
        drop(functions);
        // The public entry keeps the documented admission bound.
        let bounded = parse_raw(source, false, Default::default(), Default::default());
        assert_eq!(bounded.err().map(|error| error.code), Some("unsupported"), "{}", &source[..source.len().min(96)]);
    }
}

#[test]
fn mixed_defaults_closures_bodies_annotations_and_literals_keep_their_structure() {
    let layers = 2_000;
    let value = nest("7", layers, |inner| {
        format!("((p:Map<string,Array<int>>=identity<Box<Box<int>>>({{k:[`a${{{inner}}}b`,1]}}.k[0])):int=>{{if(p){{for(const [q,{{r=1}}] of [p]){{try{{return q;}}finally{{}}}}}}return p;}})()")
    });
    let source = main_body(&format!("console.log({value});"));
    let functions = functions(&source);
    let main = &functions[1];
    assert_eq!(main.scope_span.end, source.len());
    let StmtKind::Expr(call) = &main.body[0].kind else { panic!("call statement") };
    assert_eq!(&source[call.span.start..call.span.end], format!("console.log({value})"));
    // Every layer contributes the same number of expressions; none is lost or repeated.
    let layer = |inner: &str| {
        format!("((p:Map<string,Array<int>>=identity<Box<Box<int>>>({{k:[`a${{{inner}}}b`,1]}}.k[0])):int=>{{if(p){{for(const [q,{{r=1}}] of [p]){{try{{return q;}}finally{{}}}}}}return p;}})()")
    };
    let zero = expressions(&main_body("console.log(7);"));
    let one = expressions(&main_body(&format!("console.log({});", nest("7", 1, layer))));
    assert_eq!(expressions(&source), zero + layers * (one - zero));
}

#[test]
fn both_policies_build_the_same_tree_below_the_admission_bound() {
    let value = nest("7", 12, |inner| {
        format!("((p:int=f<Box<Box<int>>>({{k:[`a${{{inner}}}b`]}}.k[0])):int=>{{switch(p){{case 1:{{for(const [q,{{r=1}}] of [p]){{try{{return q;}}finally{{}}}}break;}}default:}}function g([a,...b]:T=[]):int{{return 1;}}return p;}})()")
    });
    let source = main_body(&format!("label:while(a){{console.log({value});}}"));
    let bounded = parse_raw(&source, false, Default::default(), Default::default()).unwrap();
    let unbounded = parse_unbounded(&source).unwrap();
    assert_eq!(format!("{:?}", bounded.functions), format!("{:?}", unbounded.functions));
    assert_eq!(format!("{:?}", bounded.call_type_arguments), format!("{:?}", unbounded.call_type_arguments));
}

#[test]
fn deep_syntax_errors_keep_their_location_and_restore_parser_scopes() {
    // The error is at the deepest point of each kind of suspended scope.
    for (source, expected) in [
        (format!("{}1 +{}", "f(({v:[".repeat(DEEP), "]}).v[0])".repeat(DEEP)), "]"),
        (format!("{}return +;{}", "(():int=>{".repeat(DEEP), "})()".repeat(DEEP)), ";"),
        (format!("{}1 +{}", "((a:int=".repeat(DEEP), "):int=>a)()".repeat(DEEP)), ")"),
        (format!("{}function(x:int=,){{}}{}", "[".repeat(DEEP), "]".repeat(DEEP)), ","),
    ] {
        let mut parser = Parser::with_policy(&source, Default::default(), Default::default(), NestingPolicy::UNBOUNDED).unwrap();
        parser.pending.push(Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Return(None), span: Span::new(0, 0) });
        let error = parser.expression().expect_err("syntax error");
        assert_eq!(error.code, "syntax");
        assert_eq!(&source[error.span.start..error.span.end], expected);
        assert_eq!(error.span.start, source.find(expected).unwrap());
        assert_eq!(parser.depth, 0);
        assert!(!parser.scheduling);
        assert_eq!(parser.pending.len(), 1);
        assert!(matches!(parser.pending[0].kind, StmtKind::Return(None)));
    }
}

#[test]
fn recovery_resumes_in_each_enclosing_block_at_any_depth() {
    // Two malformed statements in every nested block: each block recovers
    // both, then continues with the next block.
    let layers = 31;
    let source = main_body(&nest("ok();", layers, |inner| format!("{{bad(;{inner}worse +;}}")));
    let errors = diagnostics_unbounded(&source);
    assert_eq!(errors.len(), 2 * layers);
    assert!(errors.iter().all(|error| error.code == "syntax"));
    let first = errors.iter().filter(|error| &source[error.span.start..error.span.end] == ";").count();
    assert_eq!(first, 2 * layers);
    // Recovery stops after 63 recovered errors; the next error ends the parse.
    let deep = main_body(&nest("ok();", DEEP, |inner| format!("const f=()=>{{bad(;{inner}}};")));
    let errors = diagnostics_unbounded(&deep);
    assert_eq!(errors.len(), 64);
    assert!(errors.windows(2).all(|pair| pair[0].span.start < pair[1].span.start));
    // Each unclosed block reports its missing brace until the same cap.
    let unclosed = main_body(&format!("{}ok(", "if(a){const f=x=>{".repeat(DEEP)));
    let errors = diagnostics_unbounded(&unclosed);
    assert_eq!(errors.len(), 64);
    assert!(errors.iter().all(|error| error.code == "syntax"));
    assert!(errors[1..].iter().all(|error| error.span.start == unclosed.len()));
}

#[test]
fn destructuring_keeps_source_order_for_hidden_names_and_errors() {
    let layers = 5_000;
    let nested = |leaf: &str| format!("{}{leaf}{}", "[x,".repeat(layers), "]".repeat(layers));
    let source = main_body(&format!("{}=v;", nested("y")));
    let functions = functions(&source);
    let body = &functions[1].body;
    // Each layer assigns `x`, then holds its nested position under a hidden name.
    assert_eq!(body.len(), 2 * layers);
    let mut holders = Vec::new();
    for statement in body {
        if let StmtKind::Let { name, .. } = &statement.kind {
            holders.push(name.text.trim_start_matches("pattern#").parse::<usize>().unwrap());
        }
    }
    assert_eq!(holders.len(), layers - 1);
    assert!(holders.windows(2).all(|pair| pair[0] + 1 == pair[1]));
    // The first invalid target in source order is reported.
    let invalid = main_body(&format!("{}=v;", nested("...r,z=1")));
    let error = parse_unbounded(&invalid).unwrap_err();
    assert_eq!(error.message, "A rest target must be last in a destructuring assignment.");
    assert_eq!(error.span.start, invalid.find("...r").unwrap());
}

#[test]
fn deep_annotations_keep_spans_access_wrappers_and_packed_closers() {
    use crate::type_syntax::{ValueAnnotationKind as Kind, parse_with};
    let layers = DEEP;
    for (open, close) in [("(", ")"), ("Box<", ">"), ("[", "]"), ("Function<int,[x:", "]>"), ("ReadOnly<Function<int,[", "]>>"), ("(a:", ")=>int")] {
        let source = format!("{}int{}", open.repeat(layers), close.repeat(layers));
        let annotation = parse_with(&source, NestingPolicy::UNBOUNDED).unwrap();
        assert_eq!(annotation.span, Span::new(0, source.len()));
        assert_eq!(annotation.clone(), annotation);
        assert_eq!(crate::type_syntax::parse(&source).unwrap_err().code, "unsupported");
    }
    let arrays = format!("int{}", "[]".repeat(layers));
    assert!(matches!(parse_with(&arrays, NestingPolicy::UNBOUNDED).unwrap().kind, Kind::Array(_)));
    let arrows = format!("{}int", "()=>".repeat(layers));
    assert!(matches!(parse_with(&arrows, NestingPolicy::UNBOUNDED).unwrap().kind, Kind::Callable { .. }));
    // A parameter access wrapper below deep nesting still becomes parameter access.
    let wrapped = format!("{}Mutable<int[]>{}", "Function<int,[value:".repeat(layers), "]>".repeat(layers));
    let mut annotation = &parse_with(&wrapped, NestingPolicy::UNBOUNDED).unwrap();
    let mut seen = 0;
    while let Kind::Callable { parameters, .. } = &annotation.kind {
        seen += 1;
        assert_eq!(parameters[0].access.is_some(), seen == layers);
        annotation = &parameters[0].annotation;
    }
    assert_eq!(seen, layers);
    // An error below deep nesting keeps its location and the retired-keyword check.
    let invalid = format!("{}read int{}", "Box<(".repeat(layers), ")>".repeat(layers));
    let error = parse_with(&invalid, NestingPolicy::UNBOUNDED).unwrap_err();
    assert_eq!(error.code, "retired-access-keyword");
    assert_eq!(error.span.start, invalid.find("read").unwrap());
}

#[test]
fn return_and_throw_never_become_labels() {
    for body in ["return :;", "throw :1;", "const f=()=>{return :;};", "if(a){return :;}"] {
        let error = parse(&main_body(body)).unwrap_err();
        assert_eq!((error.code, error.message.as_str()), ("syntax", "Expected an expression."), "{body}");
    }
    parse(&main_body("returns: for(;;){break returns;}")).unwrap();
}

#[test]
fn annotated_arrow_detection_leaves_packed_closers_for_the_real_parse() {
    // The probe splits `>>` while it looks for `=>`, then restores the stream.
    let source = main_body("const f=(a:Box<Box<int>>):Box<Box<int>>=>a;const g=(b)?(c):Box<Box<int>>>1;");
    let program = parse(&source).unwrap();
    let StmtKind::Let { value, .. } = &program.functions[1].body[1].kind else { panic!("second declaration") };
    assert!(matches!(value.kind, ExprKind::Conditional { .. }));
    let formatted = crate::format(&source).unwrap();
    assert_eq!(crate::format(&formatted).unwrap(), formatted);
}

#[test]
fn native_stack_use_does_not_grow_with_nesting() {
    // Every grammar scope together: statements, a function expression with a
    // pattern parameter, a default that holds a closure, a nested declaration
    // with a generic default, a template, an index, an assertion and a switch.
    let layer = |inner: &str| {
        format!(
            "label:if(a){{for(let i=0;i<1;i++){{switch(a){{case 1:{{try{{const f=function([b,{{c=((d:Map<string,Array<int>>=new Map<string,Array<int>>()):int=>{{{inner}}})()}}]:T,...r:int[]):int{{function g<U>(u:U=`x${{[r[0]?.k!,b as int]}}`):U{{return u;}}return g(1);}};}}finally{{}}break;}}default:}}}}}}else if(e){{}}else{{}}"
        )
    };
    let valid = main_body(&nest("return 7;", 1_000, layer));
    let invalid = main_body(&nest("return +;", 1_000, layer));
    let annotation = nest("int", 50_000, |inner| format!("Function<Box<({inner})[]>|null,[a:ReadOnly<Map<string,int>[]>,b?:(c:int)=>[int,string]|null]>"));
    // An eighth of the ordinary 2 MiB worker stack is enough at any depth.
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || {
            let program = parse_unbounded(&valid).unwrap();
            assert_eq!(program.functions[1].scope_span.end, valid.len());
            drop(program.clone());
            drop(program);
            // The innermost block recovers; every enclosing scope then completes.
            let errors = diagnostics_unbounded(&invalid);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].span.start, invalid.find("+;").unwrap() + 1);
            assert_eq!(parse_unbounded(&invalid).unwrap_err().span, errors[0].span);
            let parsed = crate::type_syntax::parse_with(&annotation, NestingPolicy::UNBOUNDED).unwrap();
            assert_eq!(parsed.span.end, annotation.len());
        })
        .unwrap()
        .join()
        .unwrap();
}

fn main_statements(source: &str) -> Vec<Stmt> {
    let mut functions = parse_raw(source, false, Default::default(), Default::default()).unwrap().functions;
    functions.pop().unwrap().body
}

fn names(body: &[Stmt]) -> Vec<String> {
    body.iter()
        .map(|statement| match &statement.kind {
            StmtKind::LocalFunction(function) => format!("function {}", crate::local_functions::written_name(&function.name.text)),
            StmtKind::Let { name, .. } => format!("let {}", name.text.split('#').next().unwrap()),
            StmtKind::If { .. } => "if".into(),
            StmtKind::While { .. } => "while".into(),
            StmtKind::ForOf { .. } => "for-of".into(),
            StmtKind::Loop { .. } => "loop".into(),
            StmtKind::Block(_) => "block".into(),
            StmtKind::Return(_) => "return".into(),
            StmtKind::Expr(_) | StmtKind::Assign { .. } => "expression".into(),
            _ => "other".into(),
        })
        .collect()
}

#[test]
fn a_hidden_declaration_follows_the_statement_that_holds_its_expression() {
    // The declaration of a named function expression belongs to the block
    // that holds the statement, not to that statement's body.
    let body = main_statements("function main(){if((function yes():boolean{return true;})()){a();b();}c();}");
    assert_eq!(names(&body), ["if", "function yes", "expression"]);
    let StmtKind::If { then_body, .. } = &body[0].kind else { panic!("if statement") };
    assert_eq!(names(then_body), ["expression", "expression"]);

    let body = main_statements("function main(){if((function no():boolean{return false;})())a();else b();while((function more():boolean{return false;})()){a();}}");
    assert_eq!(names(&body), ["if", "function no", "while", "function more"]);
    let StmtKind::If { then_body, else_body, .. } = &body[0].kind else { panic!("if statement") };
    assert_eq!((names(then_body), names(else_body)), (vec!["expression".to_owned()], vec!["expression".to_owned()]));

    let body = main_statements("function main(){for(const x of (function items():int[]{return [1];})()){a();}switch((function pick():int{return 1;})()){case (function one():int{return 1;})():a();break;default:b();}}");
    assert_eq!(names(&body), ["for-of", "function items", "let pattern", "function pick", "function one", "if"]);

    // The loop tests its condition inside the block that scopes the loop.
    let body = main_statements("function main(){for(let i=(function start():int{return 0;})();(function below():boolean{return false;})();i+=(function step():int{return 1;})()){a();}}");
    assert_eq!(names(&body), ["block"]);
    let StmtKind::Block(scoped) = &body[0].kind else { panic!("scoped loop block") };
    assert_eq!(names(scoped), ["let i", "function start", "function below", "loop"]);
    let StmtKind::Loop { body, update, .. } = &scoped[3].kind else { panic!("loop") };
    assert_eq!((names(body), names(update)), (vec!["expression".to_owned()], vec!["expression".to_owned(), "function step".to_owned()]));

    // An earlier argument's declaration does not move into a later callback body.
    let body = main_statements("function main(){f((function seven():int{return 7;})(),()=>{a();},function(){b();});}");
    assert_eq!(names(&body), ["expression", "function seven"]);
}

#[test]
fn parameter_patterns_open_the_body_of_their_own_callable() {
    let closure = |statement: &Stmt| {
        let (StmtKind::Let { value, .. } | StmtKind::Expr(value) | StmtKind::Return(Some(value))) = &statement.kind else { panic!("closure holder") };
        let ExprKind::Closure { params, body, .. } = &value.kind else { panic!("closure") };
        let inner = params.iter().find_map(|parameter| match &parameter.initializer {
            ParameterInitializer::Default { value, .. } => match &value.kind {
                ExprKind::Closure { body, .. } => Some(names(body)),
                _ => None,
            },
            _ => None,
        });
        (names(body), inner)
    };
    for source in [
        "function main(){const f=function([a,b]:T,g=function([c]:T){return c;}){return a;};}",
        "function main(){const f=([a,b]:T,g=function([c]:T){return c;})=>{return a;};}",
        "function main(){const f=function([a,b]:T,g=([c]:T)=>{return c;}){return a;};}",
    ] {
        let body = main_statements(source);
        let (outer, inner) = closure(&body[0]);
        assert_eq!(outer, ["let a", "let b", "return"], "{source}");
        assert_eq!(inner.unwrap(), ["let c", "return"], "{source}");
    }
    // A declaration with a pattern inside a default's closure body keeps the
    // outer parameter's pattern out of its own body.
    let body = main_statements("function main(){const f=([a]:T,h=(()=>{function inner([x]:T){return x;}return 1;})())=>{return a;};}");
    let (outer, _) = closure(&body[0]);
    assert_eq!(outer, ["let a", "return"]);
}


#[test]
fn wide_declarations_and_parameters_use_the_grammar_scheduler() {
    let declarations = (0..DEEP).map(|i| format!("v{i}:int")).collect::<Vec<_>>().join(",");
    let source = main_body(&format!("let {declarations};"));
    let program = parse_unbounded(&source).unwrap();
    let body = &program.functions[1].body;
    assert_eq!(body.len(), DEEP);
    for (i, statement) in body.iter().enumerate() {
        let StmtKind::Let { name, .. } = &statement.kind else { panic!("declaration") };
        assert_eq!(name.text, format!("v{i}"));
    }
    drop(program);
    for source in [
        format!("function many({declarations}):int{{return 1;}}"),
        main_body(&format!("const many=({declarations}):int=>1;")),
        main_body(&format!("const many=function({declarations}):int{{return 1;}};")),
    ] {
        drop(parse_unbounded(&source).unwrap());
    }
    // Scheduling the next required parameter must preserve trailing-comma errors.
    for source in [
        "function many(a:int,):int{return 1;}",
        "function main(){const many=(a:int,):int=>1;}",
        "function main(){const many=function(a:int,):int{return 1;};}",
    ] {
        let error = parse_unbounded(source).err().expect("required parameter after comma");
        assert_eq!(error.code, "syntax");
        assert_eq!(&source[error.span.start..error.span.end], ")");
    }
}
