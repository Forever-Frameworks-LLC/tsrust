mod support;
use tsrust_compiler::{check, format};
#[test]
fn exception_handlers_stay_with_their_closing_block() {
    let source="function nestedFinal(trace:int[]):int{try{try{return 4;}\n\nfinally{trace.push(5);}}\n\nfinally{trace.push(6);}}function main(){try{nestedFinal([]);}\n\ncatch(error){throw error;}\n\nfinally{}}";
    let result=format(source).unwrap();
    assert!(result.contains("} finally {"),"{result}");
    assert!(result.contains("} catch (error) {"),"{result}");
    assert!(!result.contains("\n\n  finally"),"{result}");
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
fn cli_format_check_write_and_run_work_with_space_paths() {
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
        support::cli().arg("check").arg(&path).arg("--json"),
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
    assert!(formatted.contains(";\n\n  console.log(user.name)"));
    assert!(formatted.contains("}\n\n  console.log(2)"));
    assert_eq!(format(&formatted).unwrap(), formatted);
    assert!(check(&formatted).is_ok());
}
#[test]
fn check_retains_contract_details_and_lint_is_removed() {
    let dir = support::TempDirectory::new();
    let path = dir.source("function main(){}");
    let checked_text = support::run(support::cli().arg("check").arg(&path), 10);
    support::success(&checked_text);
    assert!(String::from_utf8_lossy(&checked_text.stdout).contains("Check passed."));
    assert!(checked_text.stderr.is_empty());
    let removed=support::run(support::cli().arg("lint").arg(&path),10);
    assert!(!removed.status.success());
    assert!(String::from_utf8_lossy(&removed.stderr).contains("Unknown command: lint"));
}

#[test]
fn module_import_braces_are_bindings_not_statement_blocks() {
    for (source, expected) in [
        (
            "import { left } from \"./left.tsr\";\nimport { right } from './right.tsr';\n\nfunction main(){}",
            "import { left } from \"./left.tsr\";\nimport { right } from './right.tsr';\n\nfunction main() {}\n",
        ),
        (
            "import {\n  value as sharedValue\n}\n\nfrom \"./shared.tsr\";\n\nexport function left():int{return sharedValue;}",
            "import {\n  value as sharedValue\n} from \"./shared.tsr\";\n\nexport function left(): int {\n  return sharedValue;\n}\n",
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
            "  const a = 4, [b, c] = [5, 6], { x, y: z } = o;\n",
        ),
        (
            "type P={x:int;y:int;};function f({x,y}:P,n:int):int{return x+y+n;}function main(){}",
            "function f({ x, y }: P, n: int): int {\n",
        ),
        (
            "type R={id:int;};function main(){const rows:R[]=[{id:1}];const ids=rows.map(({id})=>id);const g=(z:int):int=>{return z;};console.log(ids.length,g(1));}",
            "  const ids = rows.map(({ id }) => id);\n",
        ),
        (
            "function main(){const a=2;const k=a-(1+a)*(a-3);const u=-(a);console.log(k,u,a*-(a));}",
            "  const k = a - (1 + a) * (a - 3);\n  const u = -(a);\n  console.log(k, u, a * -(a));\n",
        ),
        (
            "function main(){let t=0;for(let i=0,j=3;i<j;i++,j--){t+=1;}console.log(t);}",
            "  for (let i = 0, j = 3; i < j; i++, j--) {\n",
        ),
        (
            "function main(){const g=(z:int):int=>{return z;};console.log(g(1));}",
            "  const g = (z: int): int => {\n",
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
    let expected = "function name(code: int, flag: boolean): string {\n  switch (code) {\n    case 1:\n      return \"one\";\n    case 2:\n    case 3:\n      return flag ? \"few\" : \"some\";\n    default:\n      switch (code % 2) {\n        case 0:\n          return \"even\";\n        default:\n          return \"odd\";\n      }\n  }\n}\n";
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
    let expected = "function main() {\n  let a: int;\n  let b: string, c: int = 2;\n  if (c > 1) {\n    a = 1;\n    b = \"x\";\n  } else {\n    a = 2;\n    b = \"y\";\n  }\n\n  console.log(a, b);\n}\n";
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

#[test]
fn formatting_nested_declarations_keeps_blocks_and_blank_lines() {
    let source="function main(){\nconsole.log(twice(3));\n\nfunction twice(value:int):int{return value+value;}\n\n{function triple(value:int):int{return value*3;}console.log(triple(2));}\n}";
    let formatted = format(source).unwrap();
    assert!(formatted.contains("  function twice(value: int): int {"),"{formatted}");
    assert!(formatted.contains("  }\n\n  {"),"{formatted}");
    assert_eq!(format(&formatted).unwrap(),formatted);
    check(&formatted).unwrap();
}

#[test]
fn interfaces_keep_generic_brackets_and_separate_comma_fields() {
    let source = "interface Named{name:string;} interface Positioned{x:int;y?:int;} interface Player extends Named, Positioned{score:int,tags:string[]} interface Box < T >{value:T;label?:string;} function main(){const values=new Map < string, int >();values.set(\"a\",1);console.log(values.size);}";
    let result = format(source).unwrap();
    assert!(result.contains("interface Box<T>"), "{result}");
    assert!(result.contains("new Map<string, int>()"), "{result}");
    assert!(result.contains("  score: int,\n  tags: string[]\n"), "{result}");
    assert!(result.contains("extends Named, Positioned"), "{result}");
    assert_eq!(format(&result).unwrap(), result);
    assert!(check(&result).is_ok(), "{result}");
}

#[test]
fn assertion_continuations_and_generic_calls_keep_type_spacing() {
    let source="type Row={value:int;};interface Box<T>{value:T;}function identity<T>(value:T):T{return value;}function main(){const row={value:4}\n\nsatisfies Row;const callback=((x:int):int=>x+1) satisfies(value:int)=>int;const asserted=((x:int):int=>x+2) as(value:int)=>int;const boxed={value:9}\n\nas Box < int >;const value=identity < int >(boxed.value);console.log(row.value,callback(value),asserted(4));}";
    let formatted=format(source).unwrap();
    for expected in ["} satisfies Row;","satisfies (value: int) => int;","as (value: int) => int;","} as Box<int>;","identity<int>(boxed.value)"] {
        assert!(formatted.contains(expected),"Missing {expected:?}:\n{formatted}");
    }
    assert_eq!(format(&formatted).unwrap(),formatted);
    assert_eq!(check(source).unwrap().contracts(),check(&formatted).unwrap().contracts());
}

#[test]
fn formatted_generic_calls_do_not_change_comparisons_or_nested_types() {
    let source="function identity<T>(value:T):T{return value;}function main(){const a=1;const b=2;const lower=a < b;const higher=b > a;const shift=8 >> 1;const values=identity < int[] >([1]);console.log(lower,higher,shift,values[0]);}";
    let formatted=format(source).unwrap();
    for expected in ["identity<int[]>([1])","a < b","b > a","8 >> 1"] {assert!(formatted.contains(expected),"{formatted}");}
    assert_eq!(format(&formatted).unwrap(),formatted);
    check(&formatted).unwrap();
}

#[test]
fn nested_array_types_and_chained_indexes_stay_attached() {
    let source="interface Box<T>{value:T;}function main(){const matrix:Box<int>[][]=[[{value:1}]];console.log(matrix[0][0].value);}";
    let formatted=format(source).unwrap();
    assert!(formatted.contains("matrix: Box<int>[][]"),"{formatted}");
    assert!(formatted.contains("matrix[0][0].value"),"{formatted}");
    assert_eq!(format(&formatted).unwrap(),formatted);
    check(&formatted).unwrap();
}

#[test]
fn cli_format_recursively_writes_project_sources_from_working_directory() {
    let dir=support::TempDirectory::new();
    std::fs::create_dir_all(dir.0.join("src/nested")).unwrap();
    std::fs::create_dir_all(dir.0.join("target")).unwrap();
    std::fs::create_dir_all(dir.0.join("node_modules")).unwrap();
    let source="function main(){console.log(1);}";
    for path in ["root.tsr","src/nested/main.tsr","target/generated.tsr","node_modules/vendor.tsr"] {
        std::fs::write(dir.0.join(path),source).unwrap();
    }
    std::fs::write(dir.0.join("notes.txt"),source).unwrap();
    let output=support::run(support::cli().arg("format").current_dir(&dir.0),10);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),"Formatted 2 files.\n");
    for path in ["root.tsr","src/nested/main.tsr"] {
        assert_eq!(std::fs::read_to_string(dir.0.join(path)).unwrap(),tsrust_compiler::format(source).unwrap());
    }
    for path in ["target/generated.tsr","node_modules/vendor.tsr","notes.txt"] {
        assert_eq!(std::fs::read_to_string(dir.0.join(path)).unwrap(),source);
    }
    support::success(&support::run(support::cli().arg("format").arg("--check").current_dir(&dir.0),10));
}

#[test]
fn cli_recursive_format_check_is_read_only_and_continues_after_bad_files() {
    let dir=support::TempDirectory::new();
    let source="function main(){console.log(1);}";
    std::fs::write(dir.0.join("good.tsr"),source).unwrap();
    let check=support::run(support::cli().args(["format","--check"]).current_dir(&dir.0),10);
    assert_eq!(check.status.code(),Some(1));
    assert_eq!(std::fs::read_to_string(dir.0.join("good.tsr")).unwrap(),source);
    std::fs::write(dir.0.join("bad.tsr"),"function main(){const text='unterminated;}").unwrap();
    let output=support::run(support::cli().arg("format").arg(&dir.0),10);
    assert_eq!(output.status.code(),Some(1));
    assert!(String::from_utf8(output.stderr).unwrap().contains("bad.tsr"));
    assert_eq!(std::fs::read_to_string(dir.0.join("good.tsr")).unwrap(),tsrust_compiler::format(source).unwrap());
}

#[test]
fn recursive_format_deduplicates_inputs_and_ignores_directory_symlinks() {
    let dir=support::TempDirectory::new();
    std::fs::create_dir_all(dir.0.join("src")).unwrap();
    let path=dir.0.join("src/main.tsr");
    let source="function main(){console.log(2);}";
    std::fs::write(&path,source).unwrap();
    #[cfg(unix)] {
        std::os::unix::fs::symlink(&dir.0,dir.0.join("src/cycle")).unwrap();
    }
    let output=support::run(support::cli().arg("format").arg(&dir.0).arg(&path),10);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),"Formatted 1 file.\n");
    assert_eq!(std::fs::read_to_string(&path).unwrap(),tsrust_compiler::format(source).unwrap());
    let unchanged=support::run(support::cli().arg("format").arg(&dir.0),10);
    support::success(&unchanged);
    assert_eq!(String::from_utf8(unchanged.stdout).unwrap(),"Formatted 0 files.\n");
}

#[test]
fn generic_types_stay_tight_inside_unions_tuples_and_callback_results() {
    let source="interface Box<T>{value:T;}type Scores=int|Map < string,int >;interface Holder{rows:[Box < int >,Map < string,int >];make:(value:int)=>Map < string,int >;}function main(){}";
    let formatted=format(source).unwrap();
    for expected in ["int | Map<string, int>","[Box<int>, Map<string, int>]","(value: int) => Map<string, int>"] {
        assert!(formatted.contains(expected),"Missing {expected}:\n{formatted}");
    }
    assert_eq!(format(&formatted).unwrap(),formatted);
    check(&formatted).unwrap();
}

#[test]
fn generic_alias_closer_is_separated_from_assignment() {
    for source in [
        "type Box<T>={value:T;};",
        "type Box<T> = {value:T;};",
        "type Box<T >= {value:T;};",
    ] {
        let formatted = format(source).unwrap();
        assert!(formatted.starts_with("type Box<T> = {"), "{formatted}");
        assert_eq!(format(&formatted).unwrap(), formatted);
        check(&format!("{formatted}function main(){{}} ")).unwrap();
    }
    let source = "type Nested<T>={value:T;};function main(){const value=2;console.log(value>=1,value>>1);}";
    let formatted = format(source).unwrap();
    assert!(formatted.contains("type Nested<T> = {"), "{formatted}");
    assert!(formatted.contains("value >= 1"), "{formatted}");
    assert!(formatted.contains("value >> 1"), "{formatted}");
    support::execute("type Box<T >= {value:T;};function main(){const box:Box<int>={value:7};console.log(box.value);}", "7\n", None);
    assert_eq!(format(&formatted).unwrap(), formatted);
    check(&formatted).unwrap();
}

#[test]
fn unary_signs_attach_to_their_operands() {
    let source = "function main(){const a=- 1;const b=[- 2,+ 3];let i=0;const c=i++ + - a;const d=a - - b[0];const e=a- 1;const f=[- a,- (a+1)];const g=a>- 1?- a:+ a;const h=a*- 1;const k=i-- - 1;const m=- - a;console.log(a,b,c,d,e,f,g,h,k,m,i);return;}";
    let formatted = format(source).unwrap();
    for expected in [
        "const a = -1;",
        "const b = [-2, +3];",
        "const c = i++ + -a;",
        "const d = a - -b[0];",
        "const e = a - 1;",
        "const f = [-a, -(a + 1)];",
        "const g = a > -1 ? -a : +a;",
        "const h = a * -1;",
        "const k = i-- - 1;",
        "const m = - -a;",
    ] {
        assert!(formatted.contains(expected), "{expected}\n{formatted}");
    }
    assert_eq!(format(&formatted).unwrap(), formatted);
    check(&formatted).unwrap();
}

#[test]
fn optional_chains_stay_attached_to_their_operands() {
    let source = "type Item={label:string;next:Item|undefined;};function main(){const items:Item[]=[{label:\"a\",next:undefined}];const first:Item|undefined=items[0];console.log(first?.label??\"none\",first?.next?.label??\"end\",items[0]?.label.length??-1);}";
    let formatted = format(source).unwrap();
    for expected in ["first?.label ?? \"none\"", "first?.next?.label ?? \"end\"", "items[0]?.label.length ?? -1"] {
        assert!(formatted.contains(expected), "{expected}\n{formatted}");
    }
    assert_eq!(format(&formatted).unwrap(), formatted);
}

#[test]
fn chained_logical_not_stays_attached_without_changing_postfix_spacing() {
    let source = "import {strictEqual} from 'std:testing';function truth(text:string):boolean{return ! ! text;}function main(){const text='hello';strictEqual(!! text,true);strictEqual(! ! ! text,false);strictEqual(!! (text),true);strictEqual(truth(text),true);const maybe:int|undefined=1;strictEqual(maybe! + 1,2);let index=0;strictEqual(++index + 1,2);}";
    let formatted = support::format_round_trip(source);
    for expected in ["strictEqual(!!text, true)", "strictEqual(!!!text, false)", "strictEqual(!!(text), true)", "return !!text;", "strictEqual(maybe! + 1, 2)", "strictEqual(++index + 1, 2)"] {
        assert!(formatted.contains(expected), "{expected}\n{formatted}");
    }
    let directory = support::TempDirectory::new();
    let path = directory.source(&formatted);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn empty_delimiters_stay_tight_without_losing_comments_or_behavior() {
    for gap in ["", " ", "\n", "\n\n"] {
        let source = format!("type Empty={{{gap}}};function noop({gap}){{{gap}}}function main({gap}){{const object:Empty={{{gap}}};const values:int[]=[{gap}];const action=({gap}):void=>{{{gap}}};action();noop({gap});if(false){{{gap}}}console.log(values.length);}}");
        let formatted = format(&source).unwrap();
        for expected in ["type Empty = {};", "function noop() {}", "object: Empty = {};", "values: int[] = [];", "(): void => {};", "if (false) {}"] {
            assert!(formatted.contains(expected), "missing {expected}: {formatted}");
        }
        assert_eq!(format(&formatted).unwrap(), formatted);
        assert_eq!(check(&source).unwrap().contracts(), check(&formatted).unwrap().contracts());
    }
    for source in ["function main(){/* keep */}", "function main(){\n// keep\n}"] {
        let formatted = format(source).unwrap();
        assert!(formatted.contains(if source.contains("/*") { "{/* keep */}" } else { "{\n// keep\n}" }), "{formatted}");
        assert_eq!(format(&formatted).unwrap(), formatted);
        assert!(check(&formatted).is_ok());
    }
    support::execute("function noop(){}function main(){const values:int[]=[];noop();console.log(values.length);}", "0\n", None);
}

#[test]
fn record_type_declaration_commas_end_lines_like_interfaces() {
    let source = "type Point = { x: int, y: [int, string], f: (a: int, b: int) => int }\ninterface Pair { a: int, b: int }\nfunction main() {}\n";
    let result = format(source).unwrap();
    assert_eq!(result, "type Point = {\n  x: int,\n  y: [int, string],\n  f: (a: int, b: int) => int\n}\n\ninterface Pair {\n  a: int,\n  b: int\n}\n\nfunction main() {}\n");
    assert_eq!(format(&result).unwrap(), result);
}
