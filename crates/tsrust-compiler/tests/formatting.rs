mod support;
use tsrust_compiler::{check, format};
#[test]
fn exception_handlers_stay_with_their_closing_block() {
    let source="function nestedFinal(trace:int[]):int{try{try{return 4;}\n\nfinally{trace.push(5);}}\n\nfinally{trace.push(6);}}function main(){try{nestedFinal([]);}\n\ncatch(error){throw error;}\n\nfinally{}}";
    let result=format(source).unwrap();
    assert!(result.contains("} finally {"),"{result}");
    assert!(result.contains("} catch (error) {"),"{result}");
    assert!(!result.contains("\n\n    finally"),"{result}");
    assert_eq!(format(&result).unwrap(),result);
    // Exception handling is enabled in source; the formatted program checks.
    assert!(check(&result).is_ok());
}
#[test]
fn formatting_preserves_literals_comments_templates_and_is_idempotent() {
    for source in [
        "function main(){const text=\"😀 a  b\";console.log(text);}",
        "function main(){const text='a\\\nb';/* keep  spacing */console.log(text); // keep\n}",
        "type User={name:string;};function main(){const user:User={name:\"Ada\"};console.log(user.name);}",
        "function main(){for(let i=0;i<2;i++){console.log(i);}}",
        "function main(){const text=`a  ${1+2}  b`;console.log(text);}",
        "function main(){let n=-1;console.log(!false,n);}",
    ] {
        let result = format(source).unwrap();
        assert_eq!(format(&result).unwrap(), result, "{source}");
        assert!(check(&result).is_ok(), "{result}");
        let before = check(source).unwrap().contracts();
        assert_eq!(before, check(&result).unwrap().contracts());
    }
}
#[test]
fn cli_format_check_write_lint_and_run_work_with_space_paths() {
    let dir = support::TempDirectory::new();
    let path = dir.0.join("hello world.tsr");
    std::fs::write(&path, "function main(){console.log(7);}").unwrap();
    assert_eq!(
        support::run(support::cli().arg("format").arg(&path).arg("--check"), 10)
            .status
            .code(),
        Some(1)
    );
    support::success(&support::run(
        support::cli().arg("format").arg(&path).arg("--write"),
        10,
    ));
    support::success(&support::run(
        support::cli().arg("format").arg(&path).arg("--check"),
        10,
    ));
    support::success(&support::run(
        support::cli().arg("lint").arg(&path).arg("--json"),
        10,
    ));
    let output = support::run(support::cli().arg("run").arg(&path), 60);
    support::success(&output);
    assert_eq!(output.stdout, b"7\n");
}
#[test]
fn cloned_transfer_retains_original_record_after_mutating_copy() {
    let source = r#"type User={name:string;};function take(user:User):User{return user;}function main(){const user:User={name:"Pat".repeat(2)};const saved=take(user);saved.name="Sam";console.log(user.name,saved.name);}"#;
    let error = check(source).err().unwrap();
    let fix = &error.fixes[0];
    let mut edited = source.to_owned();
    edited.replace_range(fix.span.start..fix.span.end, &fix.replacement);
    support::execute(&edited, "PatPat Sam\n", None);
}
#[test]
fn blank_lines_and_block_separation_are_preserved_idempotently() {
    let source = "type User={name:string;};\n\n\nfunction take(user:User):User{return user;}function main(){\nconst user:User={name:\"Pat\"};\n\nconsole.log(user.name);\nif(true){console.log(1);}console.log(2);\n}";
    let formatted = format(source).unwrap();
    assert!(formatted.contains("};\n\n\nfunction take"));
    assert!(formatted.contains("}\n\nfunction main"));
    assert!(formatted.contains(";\n\n    console.log(user.name)"));
    assert!(formatted.contains("}\n\n    console.log(2)"));
    assert_eq!(format(&formatted).unwrap(), formatted);
    assert!(check(&formatted).is_ok());
}
#[test]
fn successful_lint_is_silent_and_check_retains_contract_details() {
    let dir = support::TempDirectory::new();
    let path = dir.source("function main(){}");
    let lint = support::run(support::cli().arg("lint").arg(&path), 10);
    support::success(&lint);
    assert!(lint.stdout.is_empty());
    assert!(lint.stderr.is_empty());
    let check = support::run(support::cli().arg("check").arg(&path), 10);
    support::success(&check);
    assert!(
        String::from_utf8(check.stdout)
            .unwrap()
            .contains("Check passed.")
    );
}

#[test]
fn module_import_braces_are_bindings_not_statement_blocks() {
    for (source, expected) in [
        (
            "import { left } from \"./left.tsr\";\nimport { right } from './right.tsr';\n\nfunction main(){}",
            "import { left } from \"./left.tsr\";\nimport { right } from './right.tsr';\n\nfunction main() {\n}\n",
        ),
        (
            "import {\n    value as sharedValue\n}\n\nfrom \"./shared.tsr\";\n\nexport function left():int{return sharedValue;}",
            "import {\n    value as sharedValue\n} from \"./shared.tsr\";\n\nexport function left(): int {\n    return sharedValue;\n}\n",
        ),
    ] {
        let formatted = format(source).unwrap();
        assert_eq!(formatted, expected);
        assert_eq!(format(&formatted).unwrap(), formatted);
    }
    let commented =
        "import { value /* keep */ as local } from './dep.tsr';\nexport type User={name:string;};";
    let formatted = format(commented).unwrap();
    assert!(formatted.contains("/* keep */"));
    assert_eq!(format(&formatted).unwrap(), formatted);
}

#[test]
fn function_type_generics_are_tight_without_changing_expression_operators() {
    let source = "type Node = { value: int; }; function visit(node: Node, action: Mutable < Function < void, [value: int] >>) { action(node.value); } function main() { const small = 1 < 2; const big = 3 > 2; const shift = 8 >> 1; visit({ value: 7 }, (value: int): void => { console.log(value); }); }";
    let result = format(source).unwrap();
    assert!(result.contains("action: Mutable<Function<void, [value: int]>>"), "{result}");
    for expression in ["1 < 2", "3 > 2", "8 >> 1"] { assert!(result.contains(expression), "{result}"); }
    assert_eq!(format(&result).unwrap(), result);
    assert!(check(&result).is_ok(), "{result}");
    let source = "type Step = { run: Function < int, [label: ReadOnly < string >, count?: int] >; }; function main() {}";
    let result = format(source).unwrap();
    assert!(result.contains("Function<int, [label: ReadOnly<string>, count?: int]>"), "{result}");
    assert_eq!(format(&result).unwrap(), result);
    assert!(check(&result).is_ok(), "{result}");
}

#[test]
fn spread_and_rest_prefixes_stay_attached_to_their_operands() {
    let source = "function pack(... items: int[]): int[] { return items; } function main(){const source=[1,2];const values=[0,... source,3];console.log(pack(... values).length);}";
    let formatted = tsrust_compiler::format(source).unwrap();
    assert!(formatted.contains("[0, ...source, 3]"), "{formatted}");
    assert!(formatted.contains("pack(...values)"), "{formatted}");
    assert!(formatted.contains("pack(...items: int[])"), "{formatted}");
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    assert!(tsrust_compiler::check(&formatted).is_ok());
}

#[test]
fn readonly_tuple_generic_delimiters_stay_tight() {
    let source = "function names(pair: ReadOnly < [string, string] >): int { const [left, right] = pair; return left.length + right.length; } function main() { console.log(names([\"a\", \"bc\"])); }";
    let formatted = format(source).unwrap();
    assert!(formatted.contains("ReadOnly<[string, string]>"), "{formatted}");
    assert_eq!(format(&formatted).unwrap(), formatted);
    support::execute(&formatted, "3\n", None);
}

// Declaration lists, patterns in later declarators and parameter lists, comma
// updates, and spacing around parentheses after operators and signs.
#[test]
fn formatting_keeps_declaration_lists_patterns_and_grouping_readable() {
    for (source, expected) in [
        (
            "type P={x:int;y:int;};function main(){const o:P={x:1,y:2};const a=4,[b,c]=[5,6],{x,y:z}=o;let t=0;for(let i=0,j=3;i<j;i++,j--){t+=a+b+c+x+z;}}",
            "    const a = 4, [b, c] = [5, 6], { x, y: z } = o;\n",
        ),
        (
            "type P={x:int;y:int;};function f({x,y}:P,n:int):int{return x+y+n;}function main(){}",
            "function f({ x, y }: P, n: int): int {\n",
        ),
        (
            "type R={id:int;};function main(){const rows:R[]=[{id:1}];const ids=rows.map(({id})=>id);const g=(z:int):int=>{return z;};console.log(ids.length,g(1));}",
            "    const ids = rows.map(({ id }) => id);\n",
        ),
        (
            "function main(){const a=2;const k=a-(1+a)*(a-3);const u=-(a);console.log(k,u,a*-(a));}",
            "    const k = a - (1 + a) * (a - 3);\n    const u = -(a);\n    console.log(k, u, a * -(a));\n",
        ),
        (
            "function main(){let t=0;for(let i=0,j=3;i<j;i++,j--){t+=1;}console.log(t);}",
            "    for (let i = 0, j = 3; i < j; i++, j--) {\n",
        ),
        (
            "function main(){const g=(z:int):int=>{return z;};console.log(g(1));}",
            "    const g = (z: int): int => {\n",
        ),
    ] {
        let result = format(source).unwrap();
        assert!(result.contains(expected), "{result}");
        assert_eq!(format(&result).unwrap(), result, "{source}");
        assert!(check(&result).is_ok(), "{result}");
        assert_eq!(check(source).unwrap().contracts(), check(&result).unwrap().contracts());
    }
}

// Switch clauses put each label on its own line with its statements one level
// deeper, also when nested; a conditional colon has a space on both sides.
#[test]
fn formatting_lays_out_switch_clauses_and_conditional_colons() {
    let source = "function name(code:int,flag:boolean):string{switch(code){case 1:return \"one\";case 2:case 3:return flag?\"few\":\"some\";default:switch(code%2){case 0:return \"even\";default:return \"odd\";}}}function main(){console.log(name(1,true));}";
    let result = format(source).unwrap();
    let expected = "function name(code: int, flag: boolean): string {\n    switch (code) {\n        case 1:\n            return \"one\";\n        case 2:\n        case 3:\n            return flag ? \"few\" : \"some\";\n        default:\n            switch (code % 2) {\n                case 0:\n                    return \"even\";\n                default:\n                    return \"odd\";\n            }\n    }\n}\n";
    assert!(result.starts_with(expected), "{result}");
    assert_eq!(format(&result).unwrap(), result);
    assert!(check(&result).is_ok(), "{result}");
    assert_eq!(check(source).unwrap().contracts(), check(&result).unwrap().contracts());
    // Optional parameters and record fields keep their tight colon.
    let other = format("type P={a:int;};function f(x?:int):int{const p:P={a:x?1:2};return p.a;}function main(){}").unwrap();
    assert!(other.contains("function f(x?: int): int {") && other.contains("a: x ? 1 : 2"), "{other}");
}

// A declaration without an initializer keeps its annotation and semicolon,
// alone and inside a declaration list.
#[test]
fn formatting_keeps_declarations_without_initializers() {
    let source = "function main(){let a:int;let b:string,c:int=2;if(c>1){a=1;b=\"x\";}else{a=2;b=\"y\";}console.log(a,b);}";
    let result = format(source).unwrap();
    let expected = "function main() {\n    let a: int;\n    let b: string, c: int = 2;\n    if (c > 1) {\n        a = 1;\n        b = \"x\";\n    } else {\n        a = 2;\n        b = \"y\";\n    }\n\n    console.log(a, b);\n}\n";
    assert_eq!(result, expected);
    assert_eq!(format(&result).unwrap(), result);
    assert_eq!(check(source).unwrap().contracts(), check(&result).unwrap().contracts());
}

#[test]
fn in_operator_and_for_in_format_preserves_contracts() {
 let source="type P={x:int;};function main(){const p:P={x:1};const yes=\"x\" in p;for(const key in p)console.log(key,yes);}";
 let formatted=format(source).unwrap();assert!(formatted.contains("key in p"),"{formatted}");
 assert_eq!(format(&formatted).unwrap(),formatted);
 assert_eq!(check(source).unwrap().contracts(),check(&formatted).unwrap().contracts());
}

#[test]
fn labeled_control_formatting_keeps_targets_and_updates() {
 let source="function main(){outer:for(let i=0;i<2;i++){while(true){continue outer;}}done:{break done;}}";
 let formatted=format(source).unwrap();assert_eq!(format(&formatted).unwrap(),formatted);
 assert_eq!(check(source).unwrap().contracts(),check(&formatted).unwrap().contracts());
 assert!(formatted.contains("continue outer;"),"{formatted}");
}
