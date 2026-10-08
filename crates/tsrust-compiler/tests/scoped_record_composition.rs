mod support;

#[test]
fn caught_error_logging_ignores_uninspectable_callback_records() {
    support::execute("type Holder={run:ReadOnly<Function<int>>;};function fail():void{throw 7;}function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{fail();console.log(first[0]);}catch(error){console.log('caught',error);}console.log(handlers.size);}", "caught 7\n2\n", None);
}

#[test]
fn finally_without_catch_renews_nonleading_map_storage() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function scenario(fail:boolean):void{const old=[3];const next=[5];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before');try{failIf(fail);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('normal');}finally{console.log('cleanup',handlers.size);handlers.clear();console.log('cleared');}old[0]=9;next[0]=11;console.log('after');}function main():void{scenario(false);try{scenario(true);}catch(error){console.log('caught',error);}}");
    verify_disposal(&source, 2, "before\ndrop 0\nnormal\ncleanup 1\ndrop 1\ncleared\nafter\nbefore\ncleanup 1\ndrop 0\ncleared\ncaught 7\n");
    let overriding = source.replace("console.log('cleared');", "console.log('cleared'); throw 9;")
        .replace("old[0] = 9;", "").replace("next[0] = 11;", "").replace("console.log('after');", "")
        .replace("scenario(false);", "try { scenario(false); } catch (error) { console.log('caught', error); }");
    verify_disposal(&support::format_round_trip(&overriding), 2, "before\ndrop 0\nnormal\ncleanup 1\ndrop 1\ncleared\ncaught 9\nbefore\ncleanup 1\ndrop 0\ncleared\ncaught 9\n");
    let field = source.replace("function scenario", "type Box = { handlers: Map<string, Holder>; marker: int; }; function scenario")
        .replace("console.log('before');", "const box: Box = { handlers: handlers, marker: 23 }; console.log('before');")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("handlers.set('next'", "box.handlers.set('next'")
        .replace("handlers.size", "box.handlers.size")
        .replace("old[0] = 9;", "if (box.marker !== 23) throw 99; old[0] = 9;");
    verify_disposal(&support::format_round_trip(&field), 2, "before\ndrop 0\nnormal\ncleanup 1\ndrop 1\ncleared\nafter\nbefore\ncleanup 1\ndrop 0\ncleared\ncaught 7\n");
}

#[test]
fn conditional_transfer_replacement_preserves_disposal_without_try() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const old=[3];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before');if(branch){handlers.clear();const moved=handlers;console.log('moved');}else{console.log('kept');}console.log('replace');handlers=new Map<string,Holder>();console.log('restored',handlers.size);old[0]=9;console.log('after');}function main():void{scenario(true);scenario(false);}");
    let expected = "before\ndrop 0\nmoved\nreplace\nrestored 0\nafter\nbefore\nkept\nreplace\ndrop 0\nrestored 0\nafter\n";
    verify_disposal(&source, 1, expected);
    let nested = source.replace("if (branch) {", "{ if (branch) {").replace("console.log('replace');", "} console.log('replace');");
    verify_disposal(&support::format_round_trip(&nested), 1, expected);
    let mut field = source.replace("function scenario", "type Box = { handlers: Map<string, Holder>; marker: int; }; function scenario")
        .replace("console.log('before');", "const box: Box = { handlers: handlers, marker: 23 }; console.log('before');")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("const moved = handlers;", "const moved = box.handlers;")
        .replace("handlers.size", "box.handlers.size")
        .replace("old[0] = 9;", "if (box.marker !== 23) throw 99; old[0] = 9;");
    let restore = field.rfind("handlers = new Map<string, Holder>();").unwrap();
    field.replace_range(restore..restore+"handlers".len(), "box.handlers");
    verify_disposal(&support::format_round_trip(&field), 1, expected);
    let moved_again = source.replace("console.log('restored', handlers.size);", "const movedAgain = handlers; handlers = new Map<string, Holder>(); console.log('restored', handlers.size);");
    verify_disposal(&support::format_round_trip(&moved_again), 1, expected);
}

#[test]
fn conditional_transfer_error_preserves_survivor_until_restoration() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const old=[3];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before');try{if(branch){handlers.clear();const moved=handlers;console.log('moved');}else{console.log('kept');}console.log('throw');throw 7;}catch(error){console.log('caught');handlers=new Map<string,Holder>();console.log('restored',handlers.size);}old[0]=9;console.log('after');}function main():void{scenario(true);scenario(false);}");
    let expected = "before\ndrop 0\nmoved\nthrow\ncaught\nrestored 0\nafter\nbefore\nkept\nthrow\ncaught\ndrop 0\nrestored 0\nafter\n";
    verify_disposal(&source, 1, expected);
    let nested = source.replace("try {", "try { {").replace("console.log('throw');", "} console.log('throw');");
    assert_ne!(nested, source);
    verify_disposal(&support::format_round_trip(&nested), 1, expected);
    let mut field = source.replace("function scenario", "type Box = { handlers: Map<string, Holder>; marker: int; }; function scenario")
        .replace("    console.log('before');", "    const box: Box = { handlers: handlers, marker: 23 }; console.log('before');")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("const moved = handlers;", "const moved = box.handlers;")
        .replace("handlers.size", "box.handlers.size")
        .replace("    old[0] = 9;", "    if (box.marker !== 23) throw 99; old[0] = 9;");
    let restore = field.rfind("handlers = new Map<string, Holder>();").expect("catch restoration");
    field.replace_range(restore..restore+"handlers".len(), "box.handlers");
    verify_disposal(&support::format_round_trip(&field), 1, expected);
    let moved_again = source.replace("console.log('restored', handlers.size);", "const movedAgain = handlers; console.log('restored', movedAgain.size);");
    verify_disposal(&support::format_round_trip(&moved_again), 1, expected);
    let restored_again = moved_again.replace("console.log('restored', movedAgain.size);", "handlers = new Map<string, Holder>(); console.log('restored', handlers.size);");
    verify_disposal(&support::format_round_trip(&restored_again), 1, expected);
}

#[test]
fn conditional_transfer_normal_completion_preserves_survivor_scope() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function scenario(branch:boolean):void{const old=[3];console.log('before');{let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});try{failIf(false);if(branch){handlers.clear();const moved=handlers;console.log('moved');}else{console.log('kept');}console.log('normal');}catch(error){console.log('caught');handlers=new Map<string,Holder>();}console.log('inside');}old[0]=9;console.log('after');}function main():void{scenario(true);scenario(false);}");
    verify_disposal(&source, 1, "before\ndrop 0\nmoved\nnormal\ninside\nafter\nbefore\nkept\nnormal\ninside\ndrop 0\nafter\n");
}

#[test]
fn expression_transfer_before_failure_restores_map_storage() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function fail(): int { throw 7; }
        function main(): void {
            const old = [3];
            let handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            let saved = new Map<string, Holder>();
            let observed = -1;
            try {
                handlers.clear();
                observed = (saved = handlers, fail());
                throw 8;
            } catch (error) {
                handlers = new Map<string, Holder>();
            }
            old[0] = 9;
            strictEqual(observed, -1);
            strictEqual(saved.size, 0);
            strictEqual(handlers.size, 0);
            strictEqual(old[0], 9);
        }
    "#;
    project(source);
    project(&source.replace("(saved = handlers, fail())", "(fail(), saved = handlers, 1)"));
    project(&source.replace("(saved = handlers, fail())", "(saved = handlers, fail()) + fail()"));
    project(&source.replace("let observed = -1;", "const values = [1]; let observed = -1;")
        .replace("(saved = handlers, fail())", "(saved = handlers, values[2])"));
    project(&source.replace("function fail(): int { throw 7; }", "function fail(): int { throw 7; } function failArray(): int[] { throw 7; }")
        .replace("(saved = handlers, fail())", "failArray()[(saved = handlers, 0)]"));
}

#[test]
fn expression_transfer_failure_preserves_disposal() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function fail():int{throw 7;}function main():void{const old=[3];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});let saved=new Map<string,Holder>();let observed=-1;console.log('before');try{handlers.clear();observed=(saved=handlers,fail());throw 8;}catch(error){console.log('caught');handlers=new Map<string,Holder>();}old[0]=9;console.log('after',observed,saved.size,handlers.size);}");
    verify_disposal(&source, 1, "before\ndrop 0\ncaught\nafter -1 0 0\n");
}

#[test]
fn restored_error_storage_preserves_transfer_disposal() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):int{if(fail)throw 5;return 1;}function scenario(fail:boolean):void{const old=[3];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});let observed=-1;console.log('before');try{const path=failIf(fail);console.log('normal');handlers.clear();const moved=handlers;observed=moved.size;throw 7;}catch(error){console.log('caught');handlers=new Map<string,Holder>();observed+=handlers.size;}old[0]=9;console.log('after',observed);}function main():void{scenario(false);scenario(true);}");
    verify_disposal(&source, 1, "before\nnormal\ndrop 0\ncaught\nafter 0\nbefore\ncaught\ndrop 0\nafter -1\n");
}

#[test]
fn error_after_map_transfer_restores_storage_in_catch() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function failIf(fail: boolean): int { if (fail) throw 5; return 1; }
        function scenario(fail: boolean): void {
            const old = [3];
            let handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            let observed = -1;
            try {
                const path = failIf(fail);
                handlers.clear();
                const moved = handlers;
                observed = moved.size;
                throw 7;
            } catch (error) {
                handlers = new Map<string, Holder>();
                observed += handlers.size;
            }
            old[0] = 9;
            strictEqual(observed, fail ? -1 : 0);
            strictEqual(handlers.size, 0);
            strictEqual(old[0], 9);
        }
        function main(): void { scenario(false); scenario(true); }
    "#;
    project(source);
    project(&source.replace("                throw 7;", "                const later = failIf(true); throw 7;"));
    project(&source.replace("                throw 7;", "                const late = [1]; const failed = late[2]; throw 7;"));
    let field = source.replace("function scenario", "type Box = { handlers: Map<string, Holder>; marker: int; }; function scenario")
        .replace("            let observed = -1;", "            const box: Box = { handlers: handlers, marker: 23 }; let observed = -1;")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("const moved = handlers;", "const moved = box.handlers;")
        .replace("                handlers = new", "                box.handlers = new")
        .replace("handlers.size", "box.handlers.size")
        .replace("strictEqual(old[0], 9);", "strictEqual(old[0], 9); strictEqual(box.marker, 23);");
    project(&field);
}

#[test]
fn consumed_map_on_normal_and_caught_paths_releases_capture() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function failIf(fail: boolean): int { if (fail) throw 5; return 1; }
        function scenario(fail: boolean): void {
            const old = [3];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            let path = 0;
            let observed = -1;
            try {
                path = failIf(fail);
                handlers.clear();
                const moved = handlers;
                observed = moved.size;
            } catch (error) {
                path = 2;
                handlers.clear();
                const moved = handlers;
                observed = moved.size;
            }
            old[0] = 9;
            strictEqual(path, fail ? 2 : 1);
            strictEqual(observed, 0);
            strictEqual(old[0], 9);
        }
        function main(): void { scenario(false); scenario(true); }
    "#;
    project(source);
    let unavailable = source.replace("import { strictEqual } from \"std:testing\";", "").replace("            old[0] = 9;", "            console.log(handlers.size); old[0] = 9;");
    let error = tsrust_compiler::check(&unavailable).err().expect("internal empty storage cannot revive a consumed binding");
    assert_eq!(error.code, "transferred");
    let field = source.replace("function scenario", "type Box = { handlers: Map<string, Holder>; marker: int; }; function scenario")
        .replace("            let path = 0;", "            const box: Box = { handlers: handlers, marker: 23 }; let path = 0;")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("const moved = handlers;", "const moved = box.handlers;")
        .replace("strictEqual(observed, 0);", "strictEqual(observed, 0); strictEqual(box.marker, 23);");
    project(&field);
}

#[test]
fn consumed_exception_maps_add_no_capture_disposal() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):int{if(fail)throw 5;return 1;}function scenario(fail:boolean):void{const old=[3];const handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});let observed=-1;console.log('before');try{const path=failIf(fail);console.log('normal');handlers.clear();const moved=handlers;observed=moved.size;}catch(error){console.log('caught');handlers.clear();const moved=handlers;observed=moved.size;}console.log('moved',observed);old[0]=9;console.log('after');}function main():void{scenario(false);scenario(true);}");
    verify_disposal(&source, 1, "before\nnormal\ndrop 0\nmoved 0\nafter\nbefore\ncaught\ndrop 0\nmoved 0\nafter\n");
}

#[test]
fn nested_exception_branches_preserve_map_renewal() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function scenario(fail: boolean): void {
            const old = [3];
            const current = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            let path = 0;
            let cleaned = 0;
            try {
                if (fail) {
                    throw 5;
                } else {
                    path = 1;
                    handlers.clear();
                    handlers.set("new", { run: (): int => current[0] });
                }
            } catch (error) {
                {
                    path = 2;
                    handlers.clear();
                    handlers.set("new", { run: (): int => current[0] });
                }
            } finally {
                cleaned = handlers.size;
            }
            old[0] = 9;
            strictEqual(path, fail ? 2 : 1);
            strictEqual(cleaned, 1);
            {
                const entry = handlers.get("new");
                if (entry !== undefined) strictEqual(entry.run(), 7);
            }
            handlers.clear();
            current[0] = 11;
            strictEqual(old[0] + current[0], 20);
        }
        function main(): void { scenario(false); scenario(true); }
    "#;
    project(source);
    project(&source.replace("                    throw 5;", "                    handlers.clear(); handlers.set(\"new\", { run: (): int => current[0] }); throw 5;"));
    project(&source.replace("cleaned = handlers.size;", "if (fail) { cleaned = handlers.size; } else { cleaned = handlers.size; }"));
    let field = source.replace("function scenario", "type Box = { handlers: Map<string, Holder>; marker: int; }; function scenario")
        .replace("            let path = 0;", "            const box: Box = { handlers: handlers, marker: 23 }; let path = 0;")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("handlers.set(\"new\"", "box.handlers.set(\"new\"")
        .replace("handlers.size", "box.handlers.size")
        .replace("handlers.get(\"new\")", "box.handlers.get(\"new\")")
        .replace("strictEqual(cleaned, 1);", "strictEqual(cleaned, 1); strictEqual(box.marker, 23);");
    project(&field);
}

#[test]
fn finally_map_storage_runs_after_handler_failure_and_preserves_disposal() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function failIf(fail: boolean): int { if (fail) throw 5; return 1; }
        function scenario(fail: boolean): void {
            const old = [3];
            const current = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            console.log("before");
            try {
                const path = failIf(fail);
                console.log("normal");
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            } catch (error) {
                console.log("caught");
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
                throw 9;
            } finally {
                console.log("finally", handlers.size);
                handlers.clear();
                console.log("cleared");
            }
            old[0] = 9;
            current[0] = 11;
            console.log("after");
        }
        function main(): void {
            try { scenario(false); } catch (error) { console.log("escaped"); }
            try { scenario(true); } catch (error) { console.log("escaped"); }
        }
    "#);
    verify_disposal(&source, 3, "before\nnormal\ndrop 0\nfinally 1\ndrop 1\ncleared\nafter\nbefore\ncaught\ndrop 0\nfinally 1\ndrop 2\ncleared\nescaped\n");
    let nested = support::format_round_trip(&source.replace("const path = failIf(fail);", "if (fail) { throw 5; } else { const marker = 1; }"));
    verify_disposal(&nested, 3, "before\nnormal\ndrop 0\nfinally 1\ndrop 1\ncleared\nafter\nbefore\ncaught\ndrop 0\nfinally 1\ndrop 2\ncleared\nescaped\n");
    let start = source.find("    } catch (error) {").unwrap();
    let end = start + source[start..].find("    } finally {").unwrap();
    let before_clear = support::format_round_trip(&format!("{}    }} catch (error) {{ console.log(\"caught\"); throw 9;\n{}", &source[..start], &source[end..]));
    verify_disposal(&before_clear, 2, "before\nnormal\ndrop 0\nfinally 1\ndrop 1\ncleared\nafter\nbefore\ncaught\nfinally 1\ndrop 0\ncleared\nescaped\n");
    let overridden = source.replace("console.log(\"cleared\");", "console.log(\"cleared\"); throw 11;")
        .replace("old[0] = 9;", "").replace("current[0] = 11;", "").replace("console.log(\"after\");", "")
        .replace("console.log(\"escaped\");", "if (error === 11) console.log(\"override\"); else console.log(\"wrong\");");
    verify_disposal(&support::format_round_trip(&overridden), 3, "before\nnormal\ndrop 0\nfinally 1\ndrop 1\ncleared\noverride\nbefore\ncaught\ndrop 0\nfinally 1\ndrop 2\ncleared\noverride\n");
}

#[test]
fn fallible_prefix_map_refill_preserves_finally_storage() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function failIf(fail: boolean): int { if (fail) throw 5; return 1; }
        function scenario(fail: boolean): void {
            const old = [3];
            const current = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            let path = 0;
            let cleaned = 0;
            try {
                path = failIf(fail);
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            } catch (error) {
                path = 2;
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            } finally {
                cleaned = handlers.size;
            }
            old[0] = 9;
            strictEqual(path, fail ? 2 : 1);
            strictEqual(cleaned, 1);
            {
                const entry = handlers.get("new");
                if (entry !== undefined) strictEqual(entry.run(), 7);
            }
            handlers.clear();
            current[0] = 11;
            strictEqual(old[0] + current[0], 20);
        }
        function main(): void { scenario(false); scenario(true); }
    "#);
}

#[test]
fn fallible_prefix_map_renewal_disposes_at_each_path_clear() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function failIf(fail: boolean): int { if (fail) throw 5; return 1; }
        function scenario(fail: boolean): void {
            const old = [3];
            const current = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            console.log("before");
            try {
                const path = failIf(fail);
                console.log("normal");
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            } catch (error) {
                console.log("caught");
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            }
            old[0] = 9;
            console.log("after");
            handlers.clear();
            current[0] = 11;
            console.log("done");
        }
        function main(): void { scenario(false); scenario(true); }
    "#);
    verify_disposal(&source, 3, "before\nnormal\ndrop 0\nafter\ndrop 1\ndone\nbefore\ncaught\ndrop 0\nafter\ndrop 2\ndone\n");
}

#[test]
fn map_refill_after_fallible_prefix_preserves_both_completion_paths() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function failIf(fail: boolean): int {
            if (fail) throw 5;
            return 1;
        }
        function scenario(fail: boolean): void {
            const old = [3];
            const current = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => old[0] });
            let path = 0;
            try {
                path = failIf(fail);
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            } catch (error) {
                path = 2;
                handlers.clear();
                handlers.set("new", { run: (): int => current[0] });
            }
            old[0] = 9;
            strictEqual(path, fail ? 2 : 1);
            {
                const entry = handlers.get("new");
                if (entry !== undefined) strictEqual(entry.run(), 7);
            }
            handlers.clear();
            current[0] = 11;
            strictEqual(old[0] + current[0], 20);
        }
        function main(): void { scenario(false); scenario(true); }
    "#);
}

#[test]
fn map_clear_refill_survives_caught_failure() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main(): void {
            const first = [3];
            const second = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => first[0] });
            try {
                handlers.clear();
                handlers.set("new", { run: (): int => second[0] });
                throw 5;
            } catch (error) {
                strictEqual(handlers.size, 1);
            }
            first[0] = 9;
            {
                const current = handlers.get("new");
                if (current !== undefined) strictEqual(current.run(), 7);
            }
            handlers.clear();
            second[0] = 11;
            strictEqual(first[0] + second[0], 20);
        }
    "#;
    project(source);
    project(&source.replace("throw 5;", "strictEqual(handlers.size, 1);"));
    let field = source.replace("function main(): void", "type Box = { handlers: Map<string, Holder>; marker: int; }; function main(): void")
        .replace("            try {", "            const box: Box = { handlers: handlers, marker: 23 }; try {")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("handlers.set(\"new\"", "box.handlers.set(\"new\"")
        .replace("handlers.size", "box.handlers.size")
        .replace("handlers.get(\"new\")", "box.handlers.get(\"new\")")
        .replace("strictEqual(first[0] + second[0], 20);", "strictEqual(first[0] + second[0], 20); strictEqual(box.marker, 23);");
    project(&field);
}

#[test]
fn leading_try_clear_preserves_catch_finally_disposal_order() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function main(): void {
            const first = [3];
            const second = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => first[0] });
            console.log("before");
            try {
                handlers.clear();
                console.log("cleared");
                handlers.set("new", { run: (): int => second[0] });
                throw 5;
            } catch (error) {
                console.log("caught", handlers.size);
            } finally {
                console.log("finally", handlers.size);
            }
            first[0] = 9;
            console.log("after");
            handlers.clear();
            second[0] = 11;
            console.log("done");
        }
    "#);
    verify_disposal(&source, 2, "before\ndrop 0\ncleared\ncaught 1\nfinally 1\nafter\ndrop 1\ndone\n");
}

#[test]
fn one_branch_map_consumption_preserves_survivor_disposal_boundary() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const values=[3];{const handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>values[0]});if(branch){handlers.clear();const moved=handlers;console.log('moved');}else{console.log('kept');}console.log('inside');}values[0]=9;console.log('outside');}function main():void{scenario(true);scenario(false);}");
    verify_disposal(&source, 1, "drop 0\nmoved\ninside\noutside\nkept\ninside\ndrop 0\noutside\n");
}

#[test]
fn renewed_map_can_be_consumed_inside_its_clear_block() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main(): void {
            const values = [3];
            const handlers = new Map<string, Holder>();
            handlers.set("old", { run: (): int => values[0] });
            {
                handlers.clear();
                const moved = handlers;
                strictEqual(moved.size, 0);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#;
    project(source);
    let branches = source.replace("            {\n                handlers.clear();\n                const moved = handlers;\n                strictEqual(moved.size, 0);\n            }", "            if (values[0] === 3) { handlers.clear(); const moved = handlers; strictEqual(moved.size, 0); } else { handlers.clear(); const moved = handlers; strictEqual(moved.size, 0); }");
    assert_ne!(branches, source);
    project(&branches);
    project(&branches.replace("values[0] === 3", "values[0] === 4"));
    let one_path = branches.replace("else { handlers.clear(); const moved = handlers; strictEqual(moved.size, 0); }", "else { handlers.clear(); }");
    project(&one_path);
    project(&one_path.replace("values[0] === 3", "values[0] === 4"));
    let restored = source.replace("const handlers =", "let handlers =")
        .replace("values[0] = 9;", "handlers = new Map<string, Holder>(); strictEqual(handlers.size, 0); values[0] = 9;");
    project(&restored);
    let restored_inside = source.replace("const handlers =", "let handlers =")
        .replace("strictEqual(moved.size, 0);", "strictEqual(moved.size, 0); handlers = new Map<string, Holder>();")
        .replace("strictEqual(values[0], 9);", "strictEqual(values[0], 9); strictEqual(handlers.size, 0);");
    project(&restored_inside);
    let field = source.replace("function main()", "type Box = { handlers: Map<string, Holder>; marker: int; }; function main()")
        .replace("handlers.clear();", "box.handlers.clear();").replace("const moved = handlers;", "const moved = box.handlers;")
        .replace("            {\n                box.handlers.clear();", "            const box: Box = { handlers: handlers, marker: 23 }; {\n                box.handlers.clear();")
        .replace("strictEqual(values[0], 9);", "strictEqual(values[0], 9); strictEqual(box.marker, 23);");
    project(&field);
}

#[test]
fn renewed_map_field_keeps_uncleared_sibling_capture_protected() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};type Pair={left:Map<string,Holder>;right:Map<string,Holder>;};function scenario(branch:boolean):void{const first=[3];const second=[7];const left=new Map<string,Holder>();left.set('old',{run:():int=>first[0]});const right=new Map<string,Holder>();right.set('old',{run:():int=>second[0]});const pair:Pair={left:left,right:right};";
    for tail in [
        "if(branch){pair.left.clear();}else{pair.left.clear();}second[0]=9;console.log(pair.right.size);",
        "{if(branch){pair.left.clear();}else{pair.left.clear();}}second[0]=9;console.log(pair.right.size);",
        "if(branch){pair.left.clear();}first[0]=9;console.log(pair.left.size);",
        "if(branch){pair.left.clear();pair.right.clear();}else{pair.left.clear();}second[0]=9;console.log(pair.right.size);",
    ] {
        let source = format!("{prefix}{tail}}}function main():void{{scenario(false);}}");
        let error = tsrust_compiler::check(&source).err().expect("uncleared payload protects its capture");
        assert_eq!(error.code, "capture-conflict", "{}", error.message);
    }
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Pair = { left: Map<string, Holder>; right: Map<string, Holder>; };
        function main(): void {
            const first = [3];
            const second = [7];
            const left = new Map<string, Holder>();
            left.set("old", { run: (): int => first[0] });
            const right = new Map<string, Holder>();
            right.set("old", { run: (): int => second[0] });
            const pair: Pair = { left: left, right: right };
            if (first[0] === 3) { pair.left.clear(); } else { pair.left.clear(); }
            first[0] = 9;
            pair.left.set("new", { run: (): int => first[0] });
            const sibling = pair.right.get("old");
            if (sibling !== undefined) strictEqual(sibling.run(), 7);
            const current = pair.left.get("new");
            if (current !== undefined) strictEqual(current.run(), 9);
        }
    "#);
}

#[test]
fn both_branches_renew_independent_map_fields_without_losing_siblings() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Pair = { left: Map<string, Holder>; right: Map<string, Holder>; marker: int; };
        function scenario(branch: boolean): void {
            const first = [3];
            const second = [7];
            const third = [11];
            const left = new Map<string, Holder>();
            left.set("old", { run: (): int => first[0] });
            const right = new Map<string, Holder>();
            right.set("old", { run: (): int => second[0] });
            const pair: Pair = { left: left, right: right, marker: 23 };
            if (branch) {
                pair.left.clear();
                strictEqual(pair.marker, 23);
                pair.right.clear();
            } else {
                pair.right.clear();
                strictEqual(pair.marker, 23);
                pair.left.clear();
            }
            pair.left.set("new", { run: (): int => third[0] });
            first[0] = 5;
            second[0] = 9;
            strictEqual(first[0] + second[0], 14);
            {
                const current = pair.left.get("new");
                if (current !== undefined) strictEqual(current.run(), 11);
            }
            strictEqual(pair.right.size, 0);
            strictEqual(pair.marker, 23);
            pair.left.clear();
            third[0] = 13;
            strictEqual(third[0], 13);
        }
        function main(): void { scenario(true); scenario(false); }
    "#;
    project(source);
    let different = source.replace("pair.right.clear();\n                strictEqual(pair.marker, 23);", "strictEqual(pair.marker, 23);")
        .replace("second[0] = 9;", "")
        .replace("strictEqual(first[0] + second[0], 14);", "strictEqual(first[0] + second[0], 12);")
        .replace("strictEqual(pair.right.size, 0);", "if (branch) strictEqual(pair.right.size, 0); else strictEqual(pair.right.size, 1);");
    project(&different);
}

#[test]
fn cleared_map_refill_tracks_only_current_capture_owners() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const first = [3];
            const second = [7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => first[0] });
            handlers.clear();
            handlers.set("second", { run: (): int => second[0] });
            first[0] = 9;
            strictEqual(first[0], 9);
            {
                const result = handlers.get("second");
                if (result !== undefined) strictEqual(result.run(), 7);
            }
            handlers.clear();
            second[0] = 11;
            strictEqual(second[0], 11);
        }
    "#;
    project(source);
    project(&source.replace("handlers.clear();", "{ handlers.clear(); }"));
    project(&source.replace("handlers.clear();", "if (first[0] === 3) { handlers.clear(); } else { handlers.clear(); }"));
    project(&source.replace("handlers.clear();", "{ const marker = 2; if (marker === 2) { if (first[0] === 3) { handlers.clear(); } else { handlers.clear(); } } else { handlers.clear(); } strictEqual(handlers.size, 0); }"));
    project(&source.replace("handlers.clear();", "{ const marker = 2; strictEqual(marker, 2); handlers.clear(); strictEqual(handlers.size, 0); }"));

    let field = source.replace("function main()", "type Box = { handlers: Map<string, Holder>; marker: int; }; function main()")
        .replace("handlers.set", "box.handlers.set").replace("handlers.clear", "box.handlers.clear").replace("handlers.get", "box.handlers.get")
        .replace("const handlers = new Map<string, Holder>();", "const handlers = new Map<string, Holder>(); const box: Box = { handlers: handlers, marker: 23 };")
        .replace("strictEqual(second[0], 11);", "strictEqual(second[0], 11); strictEqual(box.marker, 23);");
    project(&field);
    project(&field.replace("box.handlers.clear();", "{ { box.handlers.clear(); } }"));
    project(&field.replace("box.handlers.clear();", "{ const marker = 2; strictEqual(marker, 2); { box.handlers.clear(); } strictEqual(box.marker, 23); }"));

    let conflict = source.replace("import { strictEqual } from \"std:testing\";", "function strictEqual(a: int, b: int): void {}")
        .replace("first[0] = 9;", "second[0] = 9;");
    let error = tsrust_compiler::check(&conflict).err().expect("refill retains new capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn clearing_one_scoped_map_releases_only_its_capture_owner() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const first = [3];
            const second = [7];
            const left = new Map<string, Holder>();
            left.set("value", { run: (): int => first[0] });
            const right = new Map<string, Holder>();
            right.set("value", { run: (): int => second[0] });
            left.clear();
            first[0] = 9;
            strictEqual(first[0], 9);
            const remaining = right.get("value");
            if (remaining !== undefined) strictEqual(remaining.run(), 7);
        }
    "#;
    project(source);
    let fields = source.replace("left.clear();", "const box: Pair = { left: left, right: right }; box.left.clear();")
        .replace("right.get(\"value\")", "box.right.get(\"value\")").replace("function main()", "type Pair = { left: Map<string, Holder>; right: Map<string, Holder>; }; function main()");
    project(&fields);
    let conditional = source.replace("import { strictEqual } from \"std:testing\";", "function strictEqual(a: int, b: int): void {}")
        .replace("function main()", "function verify(clear: boolean)")
        .replace("left.clear();", "if (clear) left.clear();") + "function main(): void { verify(false); }";
    let error = tsrust_compiler::check(&conditional).err().expect("uncleared branch retains captures");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let shared = source.replace("second[0]", "first[0]").replace("import { strictEqual } from \"std:testing\";", "function strictEqual(a: int, b: int): void {}");
    let error = tsrust_compiler::check(&shared).err().expect("live sibling Map retains shared capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);

}

#[test]
fn removed_record_map_preserves_owned_callback_environment() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const first = new Map<string, Holder>();
            first.set("value", { run: (): int => values[0] });
            const second = new Map<string, Holder>();
            second.set("value", { run: (): int => values[1] });
            const boxes: Box[] = [{ handlers: first }, { handlers: second }];
            const removed = boxes.pop();
            if (removed !== undefined) {
                const result = removed.handlers.get("value");
                if (result !== undefined) strictEqual(result.run(), 7);
            }
            const remaining = boxes[0].handlers.get("value");
            if (remaining !== undefined) strictEqual(remaining.run(), 3);
        }
    "#;
    project(source);
    let escaped = source.replace("const first = new Map", "let escaped: Box | undefined; { const first = new Map")
        .replace("const removed = boxes.pop();", "escaped = boxes.pop(); const removed = boxes.pop();")
        .replace("const remaining = boxes[0].handlers.get(\"value\");", "} const remaining = escaped!.handlers.get(\"value\");")
        .replace("remaining.run(), 3", "remaining.run(), 7").replace("result.run(), 7", "result.run(), 3");
    project(&escaped);
}

#[test]
fn pushed_record_map_elements_preserve_independent_environments() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const first = new Map<string, Holder>();
            first.set("value", { run: (): int => values[0] });
            const boxes: Box[] = [{ handlers: first }];
            const second = new Map<string, Holder>();
            second.set("value", { run: (): int => values[1] });
            boxes.push({ handlers: second });
            const left = boxes[0].handlers.get("value");
            const right = boxes[1].handlers.get("value");
            if (left !== undefined) strictEqual(left.run(), 3);
            if (right !== undefined) strictEqual(right.run(), 7);
        }
    "#;
    project(source);
    project(&source.replace("boxes.push({ handlers: second });", "boxes.unshift({ handlers: second });")
        .replace("left.run(), 3", "left.run(), 7").replace("right.run(), 7", "right.run(), 3"));
    project(&source.replace("boxes.push({ handlers: second });", "boxes.splice(1, 0, { handlers: second });"));
}

#[test]
fn separate_map_elements_keep_key_contracts_and_both_capture_owners() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const a=[3];const b=[7];const first=new Map<string,Holder>();first.set('value',{run:():int=>a[0]});const second=new Map<string,Holder>();second.set('value',{run:():int=>b[0]});const boxes:Box[]=[{handlers:first},{handlers:second}];";
    for owner in ["a", "b"] {
        let source = format!("{prefix}{owner}[0]=9;console.log(boxes[0].handlers.size,boxes[1].handlers.size);}}");
        let error = tsrust_compiler::check(&source).err().expect("each Map retains its own capture owner");
        assert_eq!(error.code, "capture-conflict", "{}", error.message);
    }
    let wrong_key = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const values=[3];const first=new Map<string,Holder>();first.set('value',{run:():int=>values[0]});const second=new Map<int,Holder>();second.set(1,{run:():int=>values[0]});const boxes:Box[]=[{handlers:first},{handlers:second}];console.log(boxes.length);}";
    let error = tsrust_compiler::check(wrong_key).err().expect("array composition cannot change Map key contracts");
    assert_eq!(error.code, "type", "{}", error.message);
}

#[test]
fn separate_record_map_elements_join_callback_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7, 11];
            const first = new Map<string, Holder>();
            first.set("value", { run: (): int => values[0] });
            const second = new Map<string, Holder>();
            second.set("value", { run: (): int => values[1] });
            const boxes: Box[] = [{ handlers: first }, { handlers: second }];
            boxes[0].handlers.set("later", { run: (): int => values[2] });
            const left = boxes[0].handlers.get("value");
            const right = boxes[1].handlers.get("value");
            const later = boxes[0].handlers.get("later");
            if (left !== undefined) strictEqual(left.run(), 3);
            if (right !== undefined) strictEqual(right.run(), 7);
            if (later !== undefined) strictEqual(later.run(), 11);
        }
    "#);
}

#[test]
fn indexed_map_replacement_releases_capture_loans_at_container_scope_exit() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3];
            {
                const first = new Map<string, Holder>();
                first.set("first", { run: (): int => values[0] });
                const boxes: Box[] = [{ handlers: first }];
                const replacement = new Map<string, Holder>();
                replacement.set("next", { run: (): int => values[0] + 1 });
                boxes[0].handlers = replacement;
                const next = boxes[0].handlers.get("next");
                if (next !== undefined) strictEqual(next.run(), 4);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn indexed_record_map_fields_keep_callback_environments() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            const boxes: Box[] = [{ handlers: handlers }];
            boxes[0].handlers.set("second", { run: (): int => values[1] });
            const first = boxes[0].handlers.get("first");
            const second = boxes[0].handlers.get("second");
            if (first !== undefined) strictEqual(first.run(), 3);
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#;
    project(source);
    let nested = source.replace("const boxes: Box[] = [{ handlers: handlers }];", "const boxes: Box[][] = [[{ handlers: handlers }]];")
        .replace("boxes[0].handlers", "boxes[0][0].handlers");
    project(&nested);
}

#[test]
fn indexed_map_field_insertion_preserves_new_capture_loans() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const first=[3];const second=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});const boxes:Box[]=[{handlers:handlers}];boxes[0].handlers.set('second',{run:():int=>second[0]});second[0]=9;console.log(boxes[0].handlers.size);}";
    let error = tsrust_compiler::check(source).err().expect("indexed field insertion keeps the new capture owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn scoped_map_for_each_propagates_callback_failures_and_rejects_mutation() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            handlers.set("second", { run: (): int => { throw values[1]; } });
            let sum = 0;
            let caught = 0;
            try {
                handlers.forEach((holder: ReadOnly<Holder>): void => { sum += holder.run(); });
            } catch (error) {
                caught = 1;
            }
            strictEqual(sum, 3);
            strictEqual(caught, 1);
            strictEqual(handlers.size, 2);
        }
    "#);
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});handlers.forEach((holder:ReadOnly<Holder>):void=>{handlers.clear();console.log(holder.run());});}";
    let error = tsrust_compiler::check(source).err().expect("Map iteration holds a receiver read loan");
    assert!(error.code.contains("conflict") || error.message.to_lowercase().contains("borrow"), "{}: {}", error.code, error.message);
}

#[test]
fn map_for_each_invokes_borrowing_record_callbacks_from_distinct_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            let initial = 0;
            handlers.forEach((holder: ReadOnly<Holder>): void => { initial += holder.run(); });
            strictEqual(initial, 3);
            handlers.set("second", { run: (): int => values[1] });
            let sum = 0;
            handlers.forEach((holder: ReadOnly<Holder>, key: string): void => {
                sum += holder.run();
            });
            strictEqual(sum, 10);
        }
    "#);
}

#[test]
fn map_field_replacement_retains_capture_owners_and_rejects_live_views() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const firstValues=[3];const secondValues=[7];const first=new Map<string,Holder>();first.set('value',{run:():int=>firstValues[0]});let box:Box={handlers:first};const second=new Map<string,Holder>();second.set('value',{run:():int=>secondValues[0]});";
    let conflict = format!("{prefix}box.handlers=second;secondValues[0]=9;console.log(box.handlers.size);}}");
    let error = tsrust_compiler::check(&conflict).err().expect("replacement retains its callback owners");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let view = format!("{prefix}const old=box.handlers.get('value');box.handlers=second;if(old!==undefined)console.log(old.run());}}");
    let error = tsrust_compiler::check(&view).err().expect("replacement cannot invalidate an entry view");
    assert_eq!(error.code, "view-conflict", "{}", error.message);
}

#[test]
fn record_map_field_replacement_preserves_distinct_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const first = new Map<string, Holder>();
            first.set("value", { run: (): int => values[0] });
            let box: Box = { handlers: first };
            {
                const old = box.handlers.get("value");
                if (old !== undefined) strictEqual(old.run(), 3);
            }
            const second = new Map<string, Holder>();
            second.set("value", { run: (): int => values[1] });
            box.handlers = second;
            const replacement = box.handlers.get("value");
            if (replacement !== undefined) strictEqual(replacement.run(), 7);
        }
    "#);
}

#[test]
fn scoped_aggregate_callback_environments_drop_once_on_each_storage_boundary() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3];
            {
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            console.log("replace");
            handlers.set("first", { run: (): int => values[0] + 1 });
            console.log("delete");
            handlers.delete("first");
            handlers.set("clear", { run: (): int => values[0] + 2 });
            console.log("clear");
            handlers.clear();
            handlers.set("scope", { run: (): int => values[0] + 3 });
            console.log("scope");
            }
            console.log("array");
            {
                const items: Holder[] = [
                    { run: (): int => values[0] + 4 },
                    { run: (): int => values[0] + 5 }
                ];
                {
                    const removed = items.splice(0, 1);
                    console.log("removed");
                }
                const last = items.pop();
                console.log("pop");
            }
            console.log("record");
            {
                const handlers = new Map<string, Holder>();
                handlers.set("record", { run: (): int => values[0] + 6 });
                const box: Box = { handlers: handlers };
                const moved = box;
                console.log("record scope");
            }
        }
    "#);
    verify_disposal(&source, 7, "replace\ndrop 0\ndelete\ndrop 1\nclear\ndrop 2\nscope\ndrop 3\narray\nremoved\ndrop 4\npop\ndrop 5\nrecord\nrecord scope\ndrop 6\n");
}

#[test]
fn scoped_aggregate_failure_disposes_environments_before_catch_and_finally() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            try {
                const items: Holder[] = [{ run: (): int => { throw values[0]; } }];
                items[0].run();
            } catch (error) {
                console.log("caught array");
            } finally {
                console.log("finally array");
            }
            try {
                const handlers = new Map<string, Holder>();
                handlers.set("fail", { run: (): int => { throw values[0]; } });
                const handler = handlers.get("fail");
                if (handler !== undefined) handler.run();
            } catch (error) {
                console.log("caught map");
            } finally {
                console.log("finally map");
            }
        }
    "#);
    verify_disposal(&source, 2, "drop 0\ncaught array\nfinally array\ndrop 1\ncaught map\nfinally map\n");
}

fn verify_disposal(source: &str, environments: usize, expected: &str) {
    let analysis = tsrust_compiler::check(&source).unwrap();
    let mut rust = analysis.emit_rust();
    // Attach an observer to generated borrowed environments. The checked source
    // controls disposal boundaries; each observer runs only when Rust drops one.
    let closures = analysis.closures();
    assert_eq!(closures.len(), environments);
    for closure in closures {
        assert!(!closure.captures.is_empty());
        rust.push_str(&format!("\nimpl<'a> Drop for Closure{}<'a> {{ fn drop(&mut self) {{ println!(\"drop {}\"); }} }}\n", closure.id, closure.id));
    }
    let directory = support::TempDirectory::new();
    let path = directory.0.join("disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("disposal-{level}"));
        support::success(&support::run(support::rustc().arg(&path).arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg("-o").arg(&binary), 60));
        let output = support::run(std::process::Command::new(&binary).args(std::iter::empty::<&str>()), 60);
        support::success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn record_map_fields_preserve_new_capture_conflicts_and_reject_escape() {
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const first=[3];const second=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});const box:Box={handlers:handlers};box.handlers.set('second',{run:():int=>second[0]});second[0]=9;console.log(box.handlers.size);}";
    let error = tsrust_compiler::check(conflict).err().expect("field insertion retains the new callback owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let escape = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function bad():Box{const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const box:Box={handlers:handlers};return box;}function main(){}";
    let error = tsrust_compiler::check(escape).err().expect("record Map cannot conceal a local borrowed capture");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn record_fields_hold_maps_with_borrowing_callback_values() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            const box: Box = { handlers: handlers };
            box.handlers.set("second", { run: (): int => values[1] });
            const first = box.handlers.get("first");
            const second = box.handlers.get("second");
            if (first !== undefined) strictEqual(first.run(), 3);
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#);
}

#[test]
fn moved_map_can_widen_to_another_callback_origin() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            const moved = handlers;
            moved.set("second", { run: (): int => values[1] });
            const finalMap = moved;
            const first = finalMap.get("first");
            const second = finalMap.get("second");
            if (first !== undefined) strictEqual(first.run(), 3);
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#);
}

#[test]
fn moved_map_keeps_capture_loans_and_transfer_diagnostics() {
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const moved=handlers;values[0]=9;console.log(moved.size);}";
    let error = tsrust_compiler::check(conflict).err().expect("moving a Map retains its capture owners");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let transferred = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const moved=handlers;console.log(handlers.size,moved.size);}";
    let error = tsrust_compiler::check(transferred).err().expect("Map move consumes the source binding");
    assert_eq!(error.code, "transferred", "{}", error.message);
}

#[test]
fn map_get_before_later_insertion_keeps_callback_environment() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            {
                const first = handlers.get("first");
                if (first !== undefined) strictEqual(first.run(), 3);
            }
            handlers.set("second", { run: (): int => values[1] });
            const second = handlers.get("second");
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#);
}

#[test]
fn scoped_map_values_keep_capture_conflicts_and_reject_local_escape() {
    let live_view = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3,7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const first=handlers.get('first');handlers.set('second',{run:():int=>values[1]});if(first!==undefined)console.log(first.run());}";
    let error = tsrust_compiler::check(live_view).err().expect("Map mutation cannot invalidate a live entry view");
    assert_eq!(error.code, "view-conflict", "{}", error.message);
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});values[0]=9;console.log(handlers.size);}";
    let error = tsrust_compiler::check(conflict).err().expect("Map payload retains its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let escape = "type Holder={run:ReadOnly<Function<int>>;};function bad():Map<string,Holder>{const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});return handlers;}function main(){}";
    let error = tsrust_compiler::check(escape).err().expect("Map cannot conceal local borrowed payload");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn map_values_keep_borrowing_record_callback_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            handlers.set("second", { run: (): int => values[1] });
            const handler = handlers.get("first");
            if (handler !== undefined) strictEqual(handler.run(), 3);
            const other = handlers.get("second");
            if (other !== undefined) strictEqual(other.run(), 7);
        }
    "#);
}

#[test]
fn alternating_path_removal_keeps_inserted_capture_after_scope_exit() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={items:Holder[];};function main(){const first=[3];const second=[7];let removed:Holder|undefined;{const boxes:Box[]=[{items:[{run:():int=>first[0]}]}];boxes[0].items.push({run:():int=>second[0]});removed=boxes[0].items.pop();}second[0]=9;if(removed!==undefined)console.log(removed.run());}";
    let error = tsrust_compiler::check(source).err().expect("removed callback must retain the inserted owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn indexed_records_keep_array_field_callback_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { items: Holder[]; };
        function main() {
            const values = [3, 7, 11];
            const boxes: Box[] = [{ items: [{ run: (): int => values[0] }] }];
            boxes[0].items.push({ run: (): int => values[1] });
            const removed = boxes[0].items.pop();
            boxes[0].items[0] = { run: (): int => values[2] };
            if (removed !== undefined) strictEqual(removed.run(), 7);
            strictEqual(boxes[0].items[0].run(), 11);
        }
    "#);
}

#[test]
fn mixed_path_insertions_keep_new_capture_conflicts() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={groups:Holder[][];};function main(){const first=[3];const second=[7];const box:Box={groups:[[{run:():int=>first[0]}]]};box.groups[0].push({run:():int=>second[0]});second[0]=9;console.log(box.groups[0][1].run());}";
    let error = tsrust_compiler::check(source).err().expect("mixed path insertion retains its owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn mixed_field_index_paths_preserve_callback_record_arrays() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { groups: Holder[][]; };
        function main() {
            const values = [3, 7, 11];
            const box: Box = { groups: [[{ run: (): int => values[0] }]] };
            box.groups[0].push({ run: (): int => values[1] });
            const removed = box.groups[0].pop();
            box.groups[0][0] = { run: (): int => values[2] };
            if (removed !== undefined) strictEqual(removed.run(), 7);
            strictEqual(box.groups[0][0].run(), 11);
        }
    "#);
}

#[test]
fn record_field_removals_keep_capture_conflicts_after_record_scope() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={items:Holder[];};function main(){const first=[3];const second=[7];let removed:Holder|undefined;{const box:Box={items:[{run:():int=>first[0]}]};box.items.push({run:():int=>second[0]});removed=box.items.pop();}second[0]=9;if(removed!==undefined)console.log(removed.run());}";
    let error = tsrust_compiler::check(source).err().expect("removed field payload retains its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn record_fields_hold_and_mutate_callback_record_arrays() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { items: Holder[]; };
        function main() {
            const values = [3, 7];
            const box: Box = { items: [{ run: (): int => values[0] }] };
            box.items.push({ run: (): int => values[1] });
            strictEqual(box.items[0].run(), 3);
            strictEqual(box.items[1].run(), 7);
            const last = box.items.pop();
            if (last !== undefined) strictEqual(last.run(), 7);
        }
    "#);
}

#[test]
fn indexed_nested_removal_retains_capture_after_container_scope() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];let removed:Holder|undefined;{const groups:Holder[][]=[[{run:():int=>values[0]}]];removed=groups[0].pop();}values[0]=9;if(removed!==undefined)console.log(removed.run());}";
    let error = tsrust_compiler::check(source).err().expect("indexed removal retains borrowed owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn indexed_nested_removals_keep_dispatch_after_later_widening() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            const groups: Holder[][] = [[{ run: (): int => values[0] }]];
            const last = groups[0].pop();
            groups[0].push({ run: (): int => values[1] });
            const removed = groups[0].splice(0, 1);
            groups[0].push({ run: (): int => values[2] });
            if (last !== undefined) strictEqual(last.run(), 3);
            strictEqual(removed[0].run(), 7);
            strictEqual(groups[0][0].run(), 11);
        }
    "#);
}

#[test]
fn nested_array_valued_writes_join_record_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            const groups: Holder[][] = [[{ run: (): int => values[0] }]];
            groups.push([{ run: (): int => values[1] }]);
            strictEqual(groups[0][0].run(), 3);
            strictEqual(groups[1][0].run(), 7);
            groups[0] = [{ run: (): int => values[2] }];
            strictEqual(groups[0][0].run(), 11);
            strictEqual(groups[1][0].run(), 7);
        }
    "#);
}

#[test]
fn nested_insertions_protect_new_capture_owners() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const first=[3];const second=[7];const groups:Holder[][]=[[{run:():int=>first[0]}]];groups[0].push({run:():int=>second[0]});second[0]=9;console.log(groups[0][1].run());}";
    let error = tsrust_compiler::check(source).err().expect("inserted callback retains its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn nested_indexed_arrays_accept_new_record_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            const groups: Holder[][] = [[{ run: (): int => values[0] }]];
            groups[0].push({ run: (): int => values[1] });
            strictEqual(groups[0][0].run(), 3);
            strictEqual(groups[0][1].run(), 7);
            groups[0][0] = { run: (): int => values[2] };
            strictEqual(groups[0][0].run(), 11);
            strictEqual(groups[0][1].run(), 7);
        }
    "#);
}

#[test]
fn nested_array_conversion_consumes_its_sources() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3,7];const first:Holder[]=[{run:():int=>values[0]}];const second:Holder[]=[{run:():int=>values[1]}];const groups:Holder[][]=[first,second];console.log(first.length,groups[0][0].run());}";
    let error = tsrust_compiler::check(source).err().expect("conversion moves existing arrays");
    assert_eq!(error.code, "transferred", "{}", error.message);
}

#[test]
fn nested_storage_moves_existing_arrays_with_distinct_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const first: Holder[] = [{ run: (): int => values[0] }];
            const second: Holder[] = [{ run: (): int => values[1] }];
            const groups: Holder[][] = [first, second];
            strictEqual(groups[0][0].run(), 3);
            strictEqual(groups[1][0].run(), 7);
        }
    "#);
}

#[test]
fn nested_record_arrays_keep_capture_conflicts_and_reject_escape() {
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const groups:Holder[][]=[[{run:():int=>values[0]}]];values[0]=9;console.log(groups[0][0].run());}";
    let error = tsrust_compiler::check(conflict).err().expect("nested storage retains captures");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let escape = "type Holder={run:ReadOnly<Function<int>>;};function bad():Holder[][]{const values=[3];return [[{run:():int=>values[0]}]];}function main(){}";
    let error = tsrust_compiler::check(escape).err().expect("nested storage cannot hide local captures");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn nested_arrays_preserve_distinct_record_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const groups: Holder[][] = [
                    [{ run: (): int => values[0] }],
                    [{ run: (): int => values[1] }]
                ];
                strictEqual(groups[0][0].run(), 3);
                strictEqual(groups[1][0].run(), 7);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn reassigned_nullable_removal_retains_capture_conflicts() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];let outer:Holder|undefined;{const holders:Holder[]=[{run:():int=>values[0]}];outer=holders.pop();}values[0]=9;if(outer!==undefined)console.log(outer.run());}";
    let error = tsrust_compiler::check(source).err().expect("nullable storage must retain its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn nullable_removal_moves_and_reassignment_preserve_environment_layout() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            const removed = holders.pop();
            const saved = removed;
            holders.push({ run: (): int => values[1] });
            if (saved !== undefined) strictEqual(saved.run(), 3);
            let outer: Holder | undefined;
            {
                const queue: Holder[] = [{ run: (): int => values[0] }];
                outer = queue.shift();
            }
            if (outer !== undefined) strictEqual(outer.run(), 3);
        }
    "#);
}

#[test]
fn loop_insertions_keep_initial_and_repeated_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            for (let i = 0; i < 2; i += 1) {
                holders.push({ run: (): int => values[1] });
            }
            strictEqual(holders[0].run(), 3);
            strictEqual(holders[1].run(), 7);
            strictEqual(holders[2].run(), 7);
        }
    "#);
}

#[test]
fn nullable_removed_record_keeps_dispatch_after_array_widens() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            const removed = holders.pop();
            holders.push({ run: (): int => values[1] });
            if (removed !== undefined) strictEqual(removed.run(), 3);
            strictEqual(holders[0].run(), 7);
            const queue: Holder[] = [{ run: (): int => values[0] }];
            const first = queue.shift();
            queue.push({ run: (): int => values[1] });
            if (first !== undefined) strictEqual(first.run(), 3);
            strictEqual(queue[0].run(), 7);
        }
    "#);
}

#[test]
fn removed_array_preserves_dispatch_after_source_widens() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            const removed = holders.splice(0, 1);
            holders.push({ run: (): int => values[1] });
            strictEqual(removed[0].run(), 3);
            strictEqual(holders[0].run(), 7);
        }
    "#);
}

#[test]
fn branching_insertions_share_one_array_environment_layout() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function exercise(flag: boolean) {
            const values = [3, 7, 11];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            if (flag) {
                holders.push({ run: (): int => values[1] });
            } else {
                holders.push({ run: (): int => values[2] });
            }
            strictEqual(holders[0].run(), 3);
            strictEqual(holders[1].run(), flag ? 7 : 11);
        }
        function main() { exercise(true); exercise(false); }
    "#);
}

#[test]
fn moved_array_can_widen_to_another_callback_origin() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const initial: Holder[] = [{ run: (): int => values[0] }];
            const moved = initial;
            moved.push({ run: (): int => values[1] });
            strictEqual(moved[0].run(), 3);
            strictEqual(moved[1].run(), 7);
        }
    "#);
}

#[test]
fn removed_array_moves_into_outer_storage_with_its_environment() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            let removed: Holder[];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                removed = holders.splice(0, 1);
            }
            strictEqual(removed[0].run(), 3);
        }
    "#);
}

#[test]
fn removed_array_retains_capture_conflict_after_source_container_ends() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];let removed:Holder[];{const holders:Holder[]=[{run:():int=>values[0]}];removed=holders.splice(0,1);}values[0]=9;console.log(removed[0].run());}";
    let error = tsrust_compiler::check(source).err().expect("removed payload must retain its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn splice_keeps_removed_and_inserted_record_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                const removed = holders.splice(0, 1, { run: (): int => values[1] });
                strictEqual(removed[0].run(), 3);
                strictEqual(holders[0].run(), 7);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn unshift_and_indexed_replacement_keep_distinct_record_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                holders.unshift({ run: (): int => values[1] });
                strictEqual(holders[0].run(), 7);
                strictEqual(holders[1].run(), 3);
                holders[1] = { run: (): int => values[2] };
                strictEqual(holders[0].run(), 7);
                strictEqual(holders[1].run(), 11);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

fn project(source: &str) {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(source);
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn returned_array_cannot_hide_local_capture_borrows() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function bad():Holder[]{const values=[3];return [{run:():int=>values[0]}];}function main(){}";
    let error = tsrust_compiler::check(source).err().expect("aggregate escape must be rejected");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn mutable_record_elements_update_original_capture_and_release_at_scope_exit() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Counter = { run: Mutable<Function<int>>; };
        function main() {
            let count = 0;
            {
                const counters: Counter[] = [{ run: (): int => { count += 1; return count; } }];
                strictEqual(counters[0].run(), 1);
                strictEqual(counters[0].run(), 2);
            }
            count += 3;
            strictEqual(count, 5);
        }
    "#);
}

#[test]
fn inserting_another_record_origin_preserves_both_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                strictEqual(holders[0].run(), 3);
                holders.push({ run: (): int => values[1] });
                strictEqual(holders[0].run(), 3);
                strictEqual(holders[1].run(), 7);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn extracted_record_keeps_capture_conflicts_after_container_scope_ends() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            let escaped: Holder;
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                escaped = holders.pop()!;
            }
            values[0] = 9;
            console.log(escaped.run());
        }
    "#;
    let error = tsrust_compiler::check(&support::format_round_trip(source)).err().expect("capture conflict must be rejected");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn dense_arrays_of_borrowing_records_keep_capture_owners_live() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [
                    { run: (): int => values[0] },
                    { run: (): int => values[1] }
                ];
                strictEqual(holders[0].run(), 3);
                strictEqual(holders[1].run(), 7);
                for (const holder of holders) strictEqual(holder.run() > 0, true);
                const last = holders.pop();
                if (last !== undefined) strictEqual(last.run(), 7);
                const first = holders.shift();
                if (first !== undefined) strictEqual(first.run(), 3);
                strictEqual(holders.length, 0);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
