mod support;

#[test]
fn nullable_scoped_record_starts_absent_and_releases_replacements() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];let saved:Holder|undefined=undefined;console.log(String(saved));saved={run:():int=>first[0]};if(saved!==undefined)console.log(saved.run());saved={run:():int=>second[0]};first[0]=7;if(saved!==undefined)console.log(saved.run(),first[0]);saved=undefined;second[0]=9;console.log(String(saved),second[0]);}");
    support::execute(&source,"undefined\n3\n5 7\nundefined 9\n",None);
    verify_disposal(&source,2,"undefined\n3\ndrop 0\n5 7\ndrop 1\nundefined 9\n");
    let absent=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const saved:Holder|undefined=undefined;console.log(saved===undefined);}");
    support::execute(&absent,"true\n",None);

}

#[test]
fn nullable_scoped_record_null_storage_moves_and_protects_captures() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const values=[3];let saved:Holder|null=null;console.log(saved===null);saved={run:():int=>values[0]};if(saved!==null)console.log(saved.run());saved=null;values[0]=7;console.log(saved===null,values[0]);}");
    support::execute(&source,"true\n3\ntrue 7\n",None);
    verify_disposal(&source,1,"true\n3\ndrop 0\ntrue 7\n");
    let both=source.replace("Holder | null", "Holder | null | undefined").replace("saved = null;", "saved=undefined;").replace("if (saved !== null)", "if(saved!==null && saved!==undefined)");
    support::execute(&support::format_round_trip(&both),"true\n3\nfalse 7\n",None);

    let moved=source.replace("saved = null;", "const moved=saved;saved=null;if(moved!==null)console.log(moved.run());");
    assert_eq!(tsrust_compiler::check(&moved).err().expect("moved result keeps captured values protected").code,"capture-conflict");
    let conflict=source.replace("if (saved !== null)","values[0]=9;if(saved!==null)");
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("stored callback protects its capture").code,"capture-conflict");
}

#[test]
fn comma_reassignment_releases_prefix_and_previous_result_in_order() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const third=[7];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]},{run:():int=>third[0]}];let removed=handlers.pop();removed=(handlers.pop(),second[0]=9,handlers.shift());third[0]=11;if(removed!==undefined)console.log(removed.run(),second[0],third[0]);removed=undefined;first[0]=13;console.log(first[0]);}");
    support::execute(&source,"3 9 11\n13\n",None);
    verify_disposal(&source,3,"drop 1\ndrop 2\n3 9 11\ndrop 0\n13\n");
    for (from,to) in [("second[0] = 9", "third[0] = 9"),("third[0] = 11", "first[0] = 11")] {
        let conflict=source.replace(from,to);
        assert_ne!(source,conflict);
        assert_eq!(tsrust_compiler::check(&conflict).err().expect("current result owner stays live").code,"capture-conflict");
    }
    let empty=source.replace("removed = undefined;", "removed=(handlers.pop(),handlers.shift());");
    support::execute(&support::format_round_trip(&empty),"3 9 11\n13\n",None);
    let sentinel=source.replace("let removed = handlers.pop();", "handlers.pop();let removed:Holder|undefined=undefined;");
    support::execute(&support::format_round_trip(&sentinel),"3 9 11\n13\n",None);

}

#[test]
fn retained_comma_removal_releases_only_discarded_prefix_captures() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];{const removed=(handlers.pop(),second[0]=9,handlers.shift());if(removed!==undefined)console.log(removed.run(),second[0]);}first[0]=7;console.log(first[0],second[0],handlers.length);}");
    support::execute(&source,"3 9\n7 9 0\n",None);
    verify_disposal(&source,2,"drop 1\n3 9\ndrop 0\n7 9 0\n");
}

#[test]
fn retained_comma_removal_preserves_final_owner_and_empty_results() {
    let prefix="type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];";
    let source=support::format_round_trip(&format!("{prefix}const note=\"kept\";let removed=(note,(handlers.shift(),first[0]=7),handlers.pop());if(removed!==undefined)console.log(note,removed.run(),first[0]);removed=undefined;second[0]=9;console.log(second[0]);}}"));
    support::execute(&source,"kept 5 7\n9\n",None);
    verify_disposal(&source,2,"drop 0\nkept 5 7\ndrop 1\n9\n");
    let source=support::format_round_trip(&format!("{prefix}const removed=(handlers.pop(),second[0]=9,handlers.shift(),first[0]=7,handlers.pop());console.log(String(removed),first[0],second[0]);}}"));
    support::execute(&source,"undefined 7 9\n",None);
    let conflict=format!("{prefix}const removed=(handlers.pop(),second[0]=9,handlers.shift());first[0]=7;if(removed!==undefined)console.log(removed.run());}}");
    assert_eq!(tsrust_compiler::check(&support::format_round_trip(&conflict)).err().expect("retained final capture remains live").code,"capture-conflict");
}

#[test]
fn discarded_comma_removal_releases_before_the_next_operand() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];(handlers.pop(),second[0]=9);console.log(handlers[0].run(),second[0]);}");
    support::execute(&source, "3 9\n", None);
    verify_disposal(&source, 2, "drop 1\n3 9\ndrop 0\n");
}

#[test]
fn discarded_comma_removals_preserve_order_and_surviving_owners() {
    let prefix="type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];";
    let source=support::format_round_trip(&format!("{prefix}(handlers.shift(),first[0]=9);console.log(handlers[0].run(),first[0]);}}"));
    support::execute(&source,"5 9\n",None);
    verify_disposal(&source,2,"drop 0\n5 9\ndrop 1\n");
    let source=support::format_round_trip(&format!("{prefix}((handlers.pop(),second[0]=9),(handlers.pop(),first[0]=7),handlers.pop());console.log(first[0],second[0],handlers.length);}}"));
    support::execute(&source,"7 9 0\n",None);
    verify_disposal(&source,2,"drop 1\ndrop 0\n7 9 0\n");
    let read_prefix=support::format_round_trip(&format!("{prefix}const note=\"kept\";(note,handlers.pop(),second[0]=9);console.log(note,handlers[0].run(),second[0]);}}"));
    support::execute(&read_prefix,"kept 3 9\n",None);
    let conflict=format!("{prefix}(handlers.pop(),first[0]=9);console.log(handlers[0].run());}}");
    assert_eq!(tsrust_compiler::check(&support::format_round_trip(&conflict)).err().expect("retained capture remains borrowed").code,"capture-conflict");
    let retained=format!("{prefix}const saved=handlers[0];(handlers.pop(),handlers.shift(),first[0]=9);console.log(saved.run());}}");
    assert_eq!(tsrust_compiler::check(&support::format_round_trip(&retained)).err().expect("indexed view prevents removal").code,"view-conflict");
}

#[test]
fn reassignable_array_result_tracks_new_origins_and_normal_blocks() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const third=[7];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];let removed=handlers.pop();";
    for tail in [
        "removed={run:():int=>third[0]};second[0]=9;if(removed!==undefined)console.log(removed.run());console.log(handlers[0].run(),second[0]);}",
        "{removed={run:():int=>third[0]};}second[0]=9;if(removed!==undefined)console.log(removed.run());console.log(handlers[0].run(),second[0]);}",
    ] {
        support::execute(&support::format_round_trip(&format!("{prefix}{tail}")), "7\n3 9\n", None);
    }
    let repeated = format!("{prefix}removed=handlers.pop();second[0]=9;if(removed!==undefined)console.log(removed.run());removed=undefined;first[0]=9;console.log(first[0],second[0]);}}");
    support::execute(&support::format_round_trip(&repeated), "3\n9 9\n", None);
    let empty = format!("{prefix}removed=handlers.pop();removed=handlers.pop();first[0]=9;second[0]=9;console.log(String(removed),handlers.length,first[0],second[0]);}}");
    support::execute(&support::format_round_trip(&empty), "undefined 0 9 9\n", None);
    let branch = format!("{prefix}if(first[0]===3)removed={{run:():int=>third[0]}};if(removed!==undefined)console.log(removed.run());}}");
    assert_eq!(tsrust_compiler::check(&support::format_round_trip(&branch)).err().expect("unsupported joins have source diagnostics").code, "unsupported");
    let conflict = format!("{prefix}removed={{run:():int=>third[0]}};third[0]=9;if(removed!==undefined)console.log(removed.run());}}");
    assert_eq!(tsrust_compiler::check(&support::format_round_trip(&conflict)).err().expect("replacement keeps new capture live").code, "capture-conflict");
}

#[test]
fn reassignable_array_result_releases_replaced_captures() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];let removed=handlers.pop();removed=undefined;second[0]=9;console.log(handlers[0].run(),second[0]);}");
    support::execute(&source, "3 9\n", None);
    verify_disposal(&source, 2, "drop 1\n3 9\ndrop 0\n");
    let front = source.replace("handlers.pop()", "handlers.shift()")
        .replace("second[0] = 9", "first[0] = 9")
        .replace("handlers[0].run(), second[0]", "handlers[0].run(), first[0]");
    support::execute(&front, "5 9\n", None);
    verify_disposal(&front, 2, "drop 0\n5 9\ndrop 1\n");
}

#[test]
fn reassignable_array_result_keeps_only_its_selected_capture() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];let removed=handlers.pop();handlers[0]={run:():int=>second[0]};first[0]=9;if(removed!==undefined)console.log(removed.run());console.log(handlers[0].run(),first[0]);}");
    support::execute(&source, "5\n5 9\n", None);
    let self_assignment = source.replace("let removed = handlers.pop();", "let removed=handlers.pop();removed=removed;");
    support::execute(&self_assignment, "5\n5 9\n", None);
    let moved_owner = source.replace("handlers[0] =", "const saved=removed;removed=undefined;second[0]=9;handlers[0] =");
    assert_eq!(tsrust_compiler::check(&moved_owner).err().expect("a moved result owner retains its capture").code, "capture-conflict");
}

#[test]
fn retained_array_result_preserves_thrown_callback_errors() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function example(value:int):int{const first=[3];const second=[value];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>{if(second[0]===5)throw 7;return second[0];}}];const removed=handlers.pop();handlers[0]={run:():int=>second[0]};first[0]=9;try{if(removed!==undefined)return removed.run();}catch(error){if(typeof error===\"number\")return error;throw error;}return 0;}function main():void{console.log(example(5));}");
    support::execute(&source, "7\n", None);
}

#[test]
fn retained_array_result_does_not_keep_other_slots_borrowed() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];const removed=handlers.pop();handlers[0]={run:():int=>second[0]};first[0]=9;if(removed!==undefined)console.log(removed.run());console.log(handlers[0].run(),first[0]);}");
    support::execute(&source, "5\n5 9\n", None);
    let fallible = source.replace("function main(): void", "function example(value:int): void")
        .replace("const first = [3]", "const first = [value]")
        .replace("const second = [5]", "const second = [value+2]");
    support::execute(&(fallible + "function main():void{example(3);}"), "5\n5 9\n", None);
}

#[test]
fn retained_array_removal_source_ownership_follows_the_result() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];";
    for tail in [
        "{const removed=handlers.pop();if(removed!==undefined)console.log(removed.run());}second[0]=9;console.log(handlers[0].run());}",
        "{const removed=handlers.shift();if(removed!==undefined)console.log(removed.run());}first[0]=9;console.log(handlers[0].run());}",
        "{const removed=handlers.pop();const moved=removed;if(moved!==undefined)console.log(moved.run());}second[0]=9;console.log(handlers[0].run());}",
    ] {
        let source = support::format_round_trip(&format!("{prefix}{tail}"));
        tsrust_compiler::check(&source).expect("removed result releases only its own capture at scope exit");
    }
    for tail in [
        "const removed=handlers.pop();second[0]=9;if(removed!==undefined)console.log(removed.run());}",
        "{const removed=handlers.pop();}first[0]=9;console.log(handlers[0].run());}",
        "const removed=handlers.shift();first[0]=9;if(removed!==undefined)console.log(removed.run());}",
        "{const removed=handlers.shift();}second[0]=9;console.log(handlers[0].run());}",
    ] {
        let source = support::format_round_trip(&format!("{prefix}{tail}"));
        assert_eq!(tsrust_compiler::check(&source).err().expect("live result or surviving element protects capture").code, "capture-conflict");
    }
}

#[test]
fn scoped_removed_array_result_releases_its_capture_after_disposal() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];{const removed=handlers.pop();if(removed!==undefined)console.log(removed.run());}second[0]=9;console.log(handlers[0].run(),second[0]);}");
    support::execute(&source, "5\n3 9\n", None);
    verify_disposal(&source, 2, "5\ndrop 1\n3 9\ndrop 0\n");
    let front = source.replace("handlers.pop()", "handlers.shift()")
        .replace("second[0] = 9", "first[0] = 9")
        .replace("handlers[0].run(), second[0]", "handlers[0].run(), first[0]");
    assert_ne!(front, source);
    support::execute(&front, "3\n5 9\n", None);
    let moved = source.replace("if (removed !== undefined)", "const moved=removed;if (moved !== undefined)")
        .replace("console.log(removed.run())", "console.log(moved.run())");
    assert_ne!(moved, source);
    support::execute(&moved, "5\n3 9\n", None);
    let nested = source.replace("const removed = handlers.pop();", "{const removed = handlers.pop();")
        .replace("second[0] = 9;", "}second[0] = 9;");
    support::execute(&nested, "5\n3 9\n", None);
    let field = source.replace("function main()", "type Owner={handlers:Holder[];};function main()")
        .replace("const handlers: Holder[] =", "const owner:Owner={handlers:")
        .replace("}];", "}]};")
        .replace("handlers.pop()", "owner.handlers.pop()")
        .replace("handlers[0]", "owner.handlers[0]");
    support::execute(&field, "5\n3 9\n", None);
    let combined = source.replace("const first =", "const extra=[7];const map=new Map<string,Holder>();map.set(\"x\",{run:():int=>extra[0]});const first =")
        .replace("const removed = handlers.pop();", "map.clear();const removed = handlers.pop();")
        .replace("second[0] = 9;", "second[0]=9;extra[0]=9;");
    support::execute(&combined, "5\n3 9\n", None);
}

#[test]
fn discarded_array_pop_releases_only_the_removed_capture() {
    let raw = "type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];handlers.pop();second[0]=9;console.log(handlers[0].run(),second[0]);}";
    let source = support::format_round_trip(raw);
    support::execute(&source, "3 9\n", None);
    verify_disposal(&source, 2, "drop 1\n3 9\ndrop 0\n");
    let front = raw.replace("handlers.pop();second[0]=9;", "handlers.shift();first[0]=9;")
        .replace("handlers[0].run(),second[0]", "handlers[0].run(),first[0]");
    support::execute(&support::format_round_trip(&front), "5 9\n", None);
    let empty = raw.replace("handlers.pop();second[0]=9;", "handlers.pop();handlers.pop();handlers.pop();first[0]=9;second[0]=9;")
        .replace("handlers[0].run(),second[0]", "handlers.length,first[0],second[0]");
    support::execute(&support::format_round_trip(&empty), "0 9 9\n", None);
    let field = raw.replace("function main()", "type Owner={handlers:Holder[];};function main()")
        .replace("const handlers:Holder[]=", "const owner:Owner={handlers:")
        .replace("}];handlers.pop();", "}]};owner.handlers.pop();")
        .replace("console.log(handlers[0]", "console.log(owner.handlers[0]");
    support::execute(&support::format_round_trip(&field), "3 9\n", None);
    let moved = raw.replace("handlers.pop();", "const moved=handlers;moved.pop();")
        .replace("console.log(handlers[0]", "console.log(moved[0]");
    support::execute(&support::format_round_trip(&moved), "3 9\n", None);
}

#[test]
fn retained_array_pop_result_keeps_its_capture_live() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];const removed=handlers.pop();second[0]=9;if(removed!==undefined)console.log(removed.run());}");
    assert_eq!(tsrust_compiler::check(&source).err().expect("a retained removal result protects its capture").code, "capture-conflict");
    let temporary = format!("{}observe(handlers.pop(),second[0]=9);}}function observe(entry:Holder|undefined,value:int):void{{if(entry!==undefined)console.log(entry.run(),value);}}", source.split_once("const removed =").expect("formatted binding").0);
    let error = tsrust_compiler::check(&temporary).err().expect("a removal temporary remains live during later arguments");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn replacing_record_field_array_element_releases_only_its_capture() {
    let raw = "type Holder={run:ReadOnly<Function<int>>;};type Owner={handlers:Holder[];marker:int;};function main():void{const first=[3];const second=[5];const owner:Owner={handlers:[{run:():int=>first[0]},{run:():int=>second[0]}],marker:7};owner.handlers[1]={run:():int=>first[0]};second[0]=9;console.log(owner.handlers[0].run(),owner.handlers[1].run(),second[0],owner.marker);}";
    let source = support::format_round_trip(raw);
    support::execute(&source, "3 3 9 7\n", None);
    verify_disposal(&source, 3, "drop 1\n3 3 9 7\ndrop 0\ndrop 2\n");
    let nested = raw.replace("type Owner={", "type Outer={owner:Owner;};type Owner={")
        .replace("const owner:Owner={handlers:", "const outer:Outer={owner:{handlers:")
        .replace("],marker:7};", "],marker:7}};")
        .replace("owner.handlers", "outer.owner.handlers")
        .replace("owner.marker", "outer.owner.marker");
    assert_ne!(nested, raw);
    support::execute(&support::format_round_trip(&nested), "3 3 9 7\n", None);
    let moved = raw.replace("const owner:Owner=", "const original:Owner=")
        .replace("owner.handlers[1]=", "const owner=original;owner.handlers[1]=");
    support::execute(&support::format_round_trip(&moved), "3 3 9 7\n", None);
    let moved_after = raw.replace("const owner:Owner=", "const original:Owner=")
        .replace("owner.handlers[1]=", "original.handlers[1]=")
        .replace("second[0]=9;", "second[0]=9;const owner=original;");
    support::execute(&support::format_round_trip(&moved_after), "3 3 9 7\n", None);
    let moved_field = raw.replace("owner.handlers[1]=", "const handlers=owner.handlers;handlers[1]=")
        .replace("owner.handlers[0].run()", "handlers[0].run()")
        .replace("owner.handlers[1].run()", "handlers[1].run()");
    support::execute(&support::format_round_trip(&moved_field), "3 3 9 7\n", None);
    let sibling = raw.replace("second[0]=9;", "second[0]=9;owner.marker+=1;");
    support::execute(&support::format_round_trip(&sibling), "3 3 9 8\n", None);
    let independent = raw.replace("handlers:Holder[];marker:int;", "handlers:Holder[];other:Holder[];marker:int;")
        .replace("const first=[3];", "const third=[7];const first=[3];")
        .replace("],marker:7};", "],other:[{run:():int=>third[0]}],marker:7};")
        .replace("second[0],owner.marker", "second[0],owner.other[0].run()");
    support::execute(&support::format_round_trip(&independent), "3 3 9 7\n", None);
    let conflict = independent.replace("second[0]=9;", "third[0]=9;");
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("the sibling array retains its capture").code, "capture-conflict");
}

#[test]
fn moved_array_preserves_known_slot_capture_release() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const original:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];const handlers=original;handlers[1]={run:():int=>first[0]};second[0]=9;console.log(handlers[0].run(),handlers[1].run(),second[0]);}");
    support::execute(&source, "3 3 9\n", None);
    let after = source.replace("const handlers = original;", "")
        .replace("handlers[1] =", "original[1] =")
        .replace("second[0] = 9;", "second[0]=9;const handlers=original;");
    assert_ne!(after, source);
    support::execute(&support::format_round_trip(&after), "3 3 9\n", None);
}

#[test]
fn array_element_replacement_keeps_surviving_and_aliased_captures_live() {
    for (tail, expected) in [
        ("first[0]=9;", "capture-conflict"),
        ("const saved=handlers[1];handlers[1]={run:():int=>first[0]};second[0]=9;console.log(saved.run());", "view-conflict"),
        ("let index=0;handlers[index]={run:():int=>first[0]};second[0]=9;", "capture-conflict"),
        ("handlers.reverse();handlers[1]={run:():int=>second[0]};first[0]=9;", "capture-conflict"),
        ("handlers.shift();handlers[0]={run:():int=>second[0]};second[0]=9;", "capture-conflict"),
        ("handlers.unshift({run:():int=>first[0]});handlers[1]={run:():int=>first[0]};second[0]=9;", "capture-conflict"),
    ] {
        let source = support::format_round_trip(&format!(
            "type Holder={{run:ReadOnly<Function<int>>;}};function main():void{{const first=[3];const second=[5];const handlers:Holder[]=[{{run:():int=>first[0]}},{{run:():int=>second[0]}}];{tail}}}"
        ));
        let error = tsrust_compiler::check(&source)
            .err().expect("a surviving or aliased element must protect its capture");
        assert_eq!(error.code, expected, "{tail}: {}", error.message);
    }
}

#[test]
fn known_array_slot_replacement_source_ownership_is_precise() {
    let source = support::format_round_trip(
        "type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];handlers[1]={run:():int=>first[0]};second[0]=9;console.log(handlers[0].run(),handlers[1].run(),second[0]);}",
    );
    tsrust_compiler::check(&source).expect("replacing the only owner releases its capture");
}

#[test]
fn replacing_known_array_element_releases_only_its_capture() {
    let source = support::format_round_trip(
        "type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers:Holder[]=[{run:():int=>first[0]},{run:():int=>second[0]}];handlers[1]={run:():int=>first[0]};second[0]=9;console.log(handlers[0].run(),handlers[1].run(),second[0]);}",
    );
    support::execute(&source, "3 3 9\n", None);
    verify_disposal(&source, 3, "drop 1\n3 3 9\ndrop 0\ndrop 2\n");
    let alias = source.replace(
        "console.log(handlers[0].run(), handlers[1].run(), second[0]);",
        "const saved=handlers[0];console.log(saved.run(),handlers[1].run(),second[0]);",
    );
    assert_ne!(alias, source);
    support::execute(&support::format_round_trip(&alias), "3 3 9\n", None);
    let repeated = source.replace(
        "second[0] = 9;",
        "second[0]=9;handlers[0]={run:():int=>first[0]};",
    );
    assert_ne!(repeated, source);
    support::execute(&support::format_round_trip(&repeated), "3 3 9\n", None);
}

#[test]
fn map_cleanup_controls_override_pending_return_throw_and_break() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(mode:int):int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});let turn=0;outer:while(turn<2){inner:while(true){try{if(mode===0)return 42;if(mode===1)throw 7;break inner;}finally{handlers.delete('first');first[0]+=1;turn+=1;if(turn<2)continue outer;break outer;}}}console.log('done',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();return 99;}function main():void{console.log(scenario(0));console.log(scenario(1));console.log(scenario(2));}");
 support::execute(&source,"done 5\nsecond 5\n99\ndone 5\nsecond 5\n99\ndone 5\nsecond 5\n99\n",None);
 verify_disposal(&source,2,"drop 0\ndone 5\nsecond 5\ndrop 1\n99\ndrop 0\ndone 5\nsecond 5\ndrop 1\n99\ndrop 0\ndone 5\nsecond 5\ndrop 1\n99\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"done 5\nsecond 5\n99\ndone 5\nsecond 5\n99\ndone 5\nsecond 5\n99\n",None);
 let caught=source.replacen("try {","try{throw 17;}catch(error){",1);assert_ne!(caught,source);
 support::execute(&support::format_round_trip(&caught),"done 5\nsecond 5\n99\ndone 5\nsecond 5\n99\ndone 5\nsecond 5\n99\n",None);
 let owned=source.replace("function scenario(mode: int): int", "function scenario(mode:int):string[]").replace("return 42;","return ['saved'];").replace("return 99;","return ['done'];").replace("scenario(0))","scenario(0)[0])").replace("scenario(1))","scenario(1)[0])").replace("scenario(2))","scenario(2)[0])");assert_ne!(owned,source);
 support::execute(&support::format_round_trip(&owned),"done 5\nsecond 5\ndone\ndone 5\nsecond 5\ndone\ndone 5\nsecond 5\ndone\n",None);

}

#[test]
fn map_cleanup_controls_preserve_distinct_label_destinations() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(mode:int):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});let turn=0;outer:while(turn<2){inner:while(true){try{if(mode===0){handlers.delete('first');first[0]+=1;break outer;}if(mode===1){handlers.delete('second');second[0]+=1;break inner;}handlers.delete('first');first[0]+=1;turn+=1;continue outer;}finally{console.log(handlers.size);}}turn+=1;}handlers.clear();}function main():void{scenario(0);scenario(1);scenario(2);}");
 support::execute(&source,"1\n1\n1\n1\n1\n",None);
 verify_disposal(&source,2,"drop 0\n1\ndrop 1\ndrop 1\n1\n1\ndrop 0\ndrop 0\n1\n1\ndrop 1\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"1\n1\n1\n1\n1\n",None);
 let caught=source.replacen("try {", "try{throw 7;}catch(error){",1);assert_ne!(caught,source);
 support::execute(&support::format_round_trip(&caught),"1\n1\n1\n1\n1\n",None);
 verify_disposal(&support::format_round_trip(&caught),2,"drop 0\n1\ndrop 1\ndrop 1\n1\n1\ndrop 0\ndrop 0\n1\n1\ndrop 1\n");
 verify_disposal(&support::format_round_trip(&field),2,"drop 0\n1\ndrop 1\ndrop 1\n1\n1\ndrop 0\ndrop 0\n1\n1\ndrop 1\n");
 let nested=source.replacen("try {", "try{try{",1).replace("} finally {","}catch(error){throw error;}finally{console.log('inner');}}finally{");assert_ne!(nested,source);
 support::execute(&support::format_round_trip(&nested),"inner\n1\ninner\n1\ninner\n1\ninner\n1\ninner\n1\n",None);

}

#[test]
fn map_return_and_empty_break_cleanup_preserve_reachable_controls() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(mode:int):int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});outer:while(true){try{if(mode===0){handlers.delete('first');return 4;}handlers.clear();break outer;}finally{first[0]+=10;console.log('cleanup',first[0],handlers.size);}}return 99;}function main():void{console.log(scenario(0));console.log(scenario(1));}");
 support::execute(&source,"cleanup 13 1\n4\ncleanup 13 0\n99\n",None);
}

#[test]
fn map_catch_return_cleanup_overrides_owned_results_and_disposes_survivors() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(mode:int):string[]{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{throw 7;}catch(error){handlers.delete('first');first[0]+=1;return ['saved'];}finally{first[0]+=10;console.log('cleanup',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}if(mode===1)return ['override'];if(mode===2)throw 8;}}function main():void{console.log(scenario(0)[0]);console.log(scenario(1)[0]);try{console.log(scenario(2)[0]);}catch(error){console.log('caught');}}");
 support::execute(&source,"cleanup 14\nsecond 5\nsaved\ncleanup 14\nsecond 5\noverride\ncleanup 14\nsecond 5\ncaught\n",None);
 verify_disposal(&source,2,"drop 0\ncleanup 14\nsecond 5\ndrop 1\nsaved\ndrop 0\ncleanup 14\nsecond 5\ndrop 1\noverride\ndrop 0\ncleanup 14\nsecond 5\ndrop 1\ncaught\n");
 let both=source.replace("try {\n        throw 7;", "try {if(mode===0){handlers.delete('first');first[0]+=1;return ['saved'];}throw 7;");assert_ne!(both,source);
 support::execute(&support::format_round_trip(&both),"cleanup 14\nsecond 5\nsaved\ncleanup 14\nsecond 5\noverride\ncleanup 14\nsecond 5\ncaught\n",None);
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&both).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,both);
 support::execute(&support::format_round_trip(&field),"cleanup 14\nsecond 5\nsaved\ncleanup 14\nsecond 5\noverride\ncleanup 14\nsecond 5\ncaught\n",None);
 verify_disposal(&support::format_round_trip(&field),2,"drop 0\ncleanup 14\nsecond 5\ndrop 1\nsaved\ndrop 0\ncleanup 14\nsecond 5\ndrop 1\noverride\ndrop 0\ncleanup 14\nsecond 5\ndrop 1\ncaught\n");

}

#[test]
fn map_return_cleanup_overrides_owned_results_and_disposes_survivors() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(mode:int):string[]{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');first[0]+=1;return ['saved'];}finally{first[0]+=10;console.log('cleanup',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}if(mode===1)return ['override'];if(mode===2)throw 8;}}function main():void{console.log(scenario(0)[0]);console.log(scenario(1)[0]);try{console.log(scenario(2)[0]);}catch(error){console.log('caught');}}");
 support::execute(&source,"cleanup 14\nsecond 5\nsaved\ncleanup 14\nsecond 5\noverride\ncleanup 14\nsecond 5\ncaught\n",None);
 verify_disposal(&source,2,"drop 0\ncleanup 14\nsecond 5\ndrop 1\nsaved\ndrop 0\ncleanup 14\nsecond 5\ndrop 1\noverride\ndrop 0\ncleanup 14\nsecond 5\ndrop 1\ncaught\n");
}

#[test]
fn map_return_cleanup_mutates_released_capture_and_preserves_return_value() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario():int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');first[0]+=1;return first[0];}finally{first[0]+=10;console.log('cleanup',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}}}function main():void{console.log('result',scenario());}");
 support::execute(&source,"cleanup 14\nsecond 5\nresult 4\n",None);
 verify_disposal(&source,2,"drop 0\ncleanup 14\nsecond 5\ndrop 1\nresult 4\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"cleanup 14\nsecond 5\nresult 4\n",None);
 verify_disposal(&support::format_round_trip(&field),2,"drop 0\ncleanup 14\nsecond 5\ndrop 1\nresult 4\n");

 let nested=source.replace("try {\n        handlers.delete('first');","try {try {\n        handlers.delete('first');").replace("} finally {","}catch(error){throw error;}finally{console.log('inner');}} finally {");assert_ne!(nested,source);
 support::execute(&support::format_round_trip(&nested),"inner\ncleanup 14\nsecond 5\nresult 4\n",None);
 verify_disposal(&support::format_round_trip(&nested),2,"drop 0\ninner\ncleanup 14\nsecond 5\ndrop 1\nresult 4\n");
}

#[test]
fn map_catch_finally_completion_modes_preserve_survivors_and_disposal() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(mode:int):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{try{handlers.delete('first');if(mode!==0)throw 37;}catch(error){first[0]+=1;if(mode>=2)throw 38;}finally{first[0]+=1;console.log('cleanup',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}if(mode===3)throw 39;}}catch(error){first[0]+=1;console.log('outer',first[0]);}handlers.clear();console.log('done',first[0]);}function main():void{scenario(0);scenario(1);scenario(2);scenario(3);}");
 support::execute(&source,"cleanup 4\nsecond 5\ndone 4\ncleanup 5\nsecond 5\ndone 5\ncleanup 5\nsecond 5\nouter 6\ndone 6\ncleanup 5\nsecond 5\nouter 6\ndone 6\n",None);
 verify_disposal(&source,2,"drop 0\ncleanup 4\nsecond 5\ndrop 1\ndone 4\ndrop 0\ncleanup 5\nsecond 5\ndrop 1\ndone 5\ndrop 0\ncleanup 5\nsecond 5\nouter 6\ndrop 1\ndone 6\ndrop 0\ncleanup 5\nsecond 5\nouter 6\ndrop 1\ndone 6\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"cleanup 4\nsecond 5\ndone 4\ncleanup 5\nsecond 5\ndone 5\ncleanup 5\nsecond 5\nouter 6\ndone 6\ncleanup 5\nsecond 5\nouter 6\ndone 6\n",None);
 verify_disposal(&support::format_round_trip(&field),2,"drop 0\ncleanup 4\nsecond 5\ndrop 1\ndone 4\ndrop 0\ncleanup 5\nsecond 5\ndrop 1\ndone 5\ndrop 0\ncleanup 5\nsecond 5\nouter 6\ndrop 1\ndone 6\ndrop 0\ncleanup 5\nsecond 5\nouter 6\ndrop 1\ndone 6\n");
}

#[test]
fn map_handler_finally_errors_preserve_survivors_and_disposal() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{try{throw 7;}catch(error){handlers.delete('first');first[0]+=1;throw 8;}finally{first[0]+=1;console.log('cleanup',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}}}catch(error){first[0]+=1;console.log('outer',first[0]);}handlers.clear();console.log('done',first[0]);}");
 support::execute(&source,"cleanup 5\nsecond 5\nouter 6\ndone 6\n",None);
 verify_disposal(&source,2,"drop 0\ncleanup 5\nsecond 5\nouter 6\ndrop 1\ndone 6\n");
}

#[test]
fn map_finally_outgoing_error_preserves_survivors_and_disposal() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{try{throw 7;}finally{handlers.delete('first');first[0]+=0;}}catch(error){first[0]+=1;console.log('outer',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}}handlers.clear();console.log('done',first[0]);}");
 support::execute(&source,"outer 4\nsecond 5\ndone 4\n",None);
 verify_disposal(&source,2,"drop 0\nouter 4\nsecond 5\ndrop 1\ndone 4\n");
 let override_error=source.replace("first[0] += 0;","first[0]+=0;throw 8;");assert_ne!(override_error,source);
 support::execute(&support::format_round_trip(&override_error),"outer 4\nsecond 5\ndone 4\n",None);
 verify_disposal(&support::format_round_trip(&override_error),2,"drop 0\nouter 4\nsecond 5\ndrop 1\ndone 4\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&override_error).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,override_error);
 support::execute(&support::format_round_trip(&field),"outer 4\nsecond 5\ndone 4\n",None);
 verify_disposal(&support::format_round_trip(&field),2,"drop 0\nouter 4\nsecond 5\ndrop 1\ndone 4\n");

}

#[test]
fn map_handler_rethrow_preserves_survivors_and_disposal() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{try{throw 7;}catch(error){handlers.delete('first');first[0]+=1;console.log('inner',first[0]);throw 8;}}catch(error){first[0]+=1;console.log('outer',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}}handlers.clear();second[0]+=1;console.log('done',second[0]);}");
    support::execute(&source,"inner 4\nouter 5\nsecond 5\ndone 6\n",None);
    verify_disposal(&source,2,"drop 0\ninner 4\nouter 5\nsecond 5\ndrop 1\ndone 6\n");
    let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
    support::execute(&support::format_round_trip(&field),"inner 4\nouter 5\nsecond 5\ndone 6\n",None);
    verify_disposal(&support::format_round_trip(&field),2,"drop 0\ninner 4\nouter 5\nsecond 5\ndrop 1\ndone 6\n");

}

#[test]
fn map_finally_error_entry_preserves_survivors_and_disposal() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');failIf(flag);}finally{for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();}first[0]+=1;console.log('normal',first[0]);}function main():void{try{scenario(false);scenario(true);}catch(error){console.log('caught');}}");
 support::execute(&source,"second 5\nnormal 4\nsecond 5\ncaught\n",None);
 verify_disposal(&source,2,"drop 0\nsecond 5\ndrop 1\nnormal 4\ndrop 0\nsecond 5\ndrop 1\ncaught\n");
 let mutated=source.replace("} finally {\n        for", "} finally {\n        first[0]+=1;console.log('released',first[0]);\n        for");assert_ne!(mutated,source);
 support::execute(&support::format_round_trip(&mutated),"released 4\nsecond 5\nnormal 5\nreleased 4\nsecond 5\ncaught\n",None);
 verify_disposal(&support::format_round_trip(&mutated),2,"drop 0\nreleased 4\nsecond 5\ndrop 1\nnormal 5\ndrop 0\nreleased 4\nsecond 5\ndrop 1\ncaught\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&mutated).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,mutated);
 support::execute(&support::format_round_trip(&field),"released 4\nsecond 5\nnormal 5\nreleased 4\nsecond 5\ncaught\n",None);
 verify_disposal(&support::format_round_trip(&field),2,"drop 0\nreleased 4\nsecond 5\ndrop 1\nnormal 5\ndrop 0\nreleased 4\nsecond 5\ndrop 1\ncaught\n");
}

#[test]
fn loop_local_map_error_storage_retargets_borrowed_get() {
 let source="type Holder={run:ReadOnly<Function<int>>;};function main():void{let turn=0;while(turn<2){const first=[3];const second=[7];const handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>first[0]});try{handlers.clear();handlers.set('new',{run:():int=>second[0]});throw 5;}catch(error){console.log(handlers.size);}first[0]=9;{const current=handlers.get('new');if(current!==undefined)console.log(current.run());}handlers.clear();second[0]=11;turn+=1;}}";
 support::execute(&support::format_round_trip(source),"1\n7\n1\n7\n",None);
 let counted=source.replace("let turn=0;while(turn<2){","for(let turn=0;turn<2;turn+=1){").replace("turn+=1;}}","}}");
 support::execute(&support::format_round_trip(&counted),"1\n7\n1\n7\n",None);
 let post=source.replace("while(turn<2){","do{").replace("turn+=1;}}","turn+=1;}while(turn<2);}");
 support::execute(&support::format_round_trip(&post),"1\n7\n1\n7\n",None);
}


#[test]
fn map_error_phase_releases_deleted_capture_at_handler_entry() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');failIf(flag);}catch(error){first[0]+=1;for(const [key,entry] of handlers){console.log('caught',key,entry.run());}}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
 support::execute(&source,"done 3\ncaught second 5\ndone 4\n",None);
 verify_disposal(&source,2,"drop 0\ndrop 1\ndone 3\ndrop 0\ncaught second 5\ndrop 1\ndone 4\n");
 let finally=source.replace("    handlers.clear();\n", "    finally{console.log('cleanup',handlers.size); }\n    handlers.clear();\n");assert_ne!(finally,source);
 support::execute(&support::format_round_trip(&finally),"cleanup 1\ndone 3\ncaught second 5\ncleanup 1\ndone 4\n",None);
 verify_disposal(&support::format_round_trip(&finally),2,"drop 0\ncleanup 1\ndrop 1\ndone 3\ndrop 0\ncaught second 5\ncleanup 1\ndrop 1\ndone 4\n");
 let direct=source.replace("failIf(flag);","if(flag)throw 7;");assert_ne!(direct,source);
 support::execute(&support::format_round_trip(&direct),"done 3\ncaught second 5\ndone 4\n",None);
 let nested=source.replace("failIf(flag);","try{failIf(flag);}finally{console.log('cleanup');}");assert_ne!(nested,source);
 support::execute(&support::format_round_trip(&nested),"cleanup\ndone 3\ncleanup\ncaught second 5\ndone 4\n",None);
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"done 3\ncaught second 5\ndone 4\n",None);
}


#[test]
fn map_finally_normal_result_releases_deleted_capture_after_cleanup() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{failIf(flag);}catch(error){console.log('caught');}finally{handlers.delete('first');}first[0]+=1;for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
 support::execute(&source,"second 5\ndone 4\ncaught\nsecond 5\ndone 4\n",None);
 verify_disposal(&source,2,"drop 0\nsecond 5\ndrop 1\ndone 4\ncaught\ndrop 0\nsecond 5\ndrop 1\ndone 4\n");
 let no_catch=source.replace("} catch (error) {\n        console.log('caught');\n    } finally {","}finally{").replace("failIf(flag);","failIf(false);");assert_ne!(no_catch,source);
 support::execute(&support::format_round_trip(&no_catch),"second 5\ndone 4\nsecond 5\ndone 4\n",None);
 verify_disposal(&support::format_round_trip(&no_catch),2,"drop 0\nsecond 5\ndrop 1\ndone 4\ndrop 0\nsecond 5\ndrop 1\ndone 4\n");
 let returned=source.replace("handlers.delete('first');","handlers.delete('first');if(flag)return;");assert_ne!(returned,source);
 support::execute(&support::format_round_trip(&returned),"second 5\ndone 4\ncaught\n",None);
 verify_disposal(&support::format_round_trip(&returned),2,"drop 0\nsecond 5\ndrop 1\ndone 4\ncaught\ndrop 0\ndrop 1\n");
 let thrown=no_catch.replace("failIf(false);","failIf(flag);").replace("scenario(true);","try{scenario(true);}catch(error){console.log('caught');}");assert_ne!(thrown,no_catch);
 support::execute(&support::format_round_trip(&thrown),"second 5\ndone 4\ncaught\n",None);
 verify_disposal(&support::format_round_trip(&thrown),2,"drop 0\nsecond 5\ndrop 1\ndone 4\ndrop 0\ndrop 1\ncaught\n");
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();","const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"second 5\ndone 4\ncaught\nsecond 5\ndone 4\n",None);
}



#[test]
fn map_phase_join_releases_capture_after_body_and_handler_deletion() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(flag:boolean):void{if(flag)throw 7;}function scenario(flag:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{failIf(flag);handlers.delete('first');}catch(error){handlers.delete('first');}first[0]+=1;for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
 support::execute(&source,"second 5\ndone 4\nsecond 5\ndone 4\n",None);
 verify_disposal(&source,2,"drop 0\nsecond 5\ndrop 1\ndone 4\ndrop 0\nsecond 5\ndrop 1\ndone 4\n");
 let handled=source.replace("} catch (error) {\n        handlers.delete('first');", "}catch(error){handlers.delete('first');first[0]+=2;");
 assert_ne!(handled,source);
 support::execute(&support::format_round_trip(&handled),"second 5\ndone 4\nsecond 5\ndone 6\n",None);
 let field=("type Owner={handlers:Map<string,Holder>;};".to_owned()+&source).replace("const handlers = new Map<string, Holder>();", "const owner:Owner={handlers:new Map<string,Holder>()};").replace("handlers.","owner.handlers.").replace("of handlers)","of owner.handlers)");
 assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"second 5\ndone 4\nsecond 5\ndone 4\n",None);
 let throwing=source.replace("} catch (error) {\n        handlers.delete('first');", "}catch(error){throw error;").replace("scenario(true);", "try{scenario(true);}catch(error){console.log('caught');}");assert_ne!(throwing,source);
 support::execute(&support::format_round_trip(&throwing),"second 5\ndone 4\ncaught\n",None);
 verify_disposal(&support::format_round_trip(&throwing),2,"drop 0\nsecond 5\ndrop 1\ndone 4\ndrop 0\ndrop 1\ncaught\n");
}


#[test]
fn ordinary_map_loop_without_backedge_preserves_break_survivor() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});while(true){handlers.delete('first');first[0]+=1;break;}console.log('after',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}handlers.clear();first[0]+=10;console.log('done',first[0]);}");
    support::execute(&source,"after 1\nkept second 5\ndone 14\n",None);
    verify_disposal(&source,2,"drop 0\nafter 1\nkept second 5\ndrop 1\ndone 14\n");
    for replacement in ["for(let step=0;step<1;step+=1){","do{","for(const step of [0]){"] {
        let mut variant=source.replace("while (true) {",replacement);assert_ne!(variant,source);
        if replacement=="do{" {variant=variant.replace("console.log('after', handlers.size);","while(false);console.log('after',handlers.size);");}
        support::execute(&support::format_round_trip(&variant),"after 1\nkept second 5\ndone 14\n",None);
    }
}

#[test]
fn ordinary_map_loop_without_backedge_releases_capture_before_return() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario():int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});while(true){handlers.delete('first');first[0]+=1;return first[0];}return 0;}function main():void{console.log('result',scenario());}");
    support::execute(&source,"result 4\n",None);
    verify_disposal(&source,2,"drop 0\ndrop 1\nresult 4\n");
    let failed=source.replace("return first[0];","throw first[0];").replace("console.log('result', scenario());","try{console.log('result',scenario());}catch(error){if(typeof error==='number')console.log('error',error);else throw error;}");
    support::execute(&support::format_round_trip(&failed),"error 4\n",None);
    verify_disposal(&support::format_round_trip(&failed),2,"drop 0\ndrop 1\nerror 4\n");
}


#[test]
fn ordinary_map_loop_preserves_different_entry_back_and_break_origins() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int,stop:boolean):void{const first=[7];const second=[11];const handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>first[0]});let turn=0;while(turn<limit){handlers.delete('old');first[0]+=1;turn++;if(stop)break;handlers.set('new',{run:():int=>second[0]});if(turn<limit)continue;}console.log('after',turn,handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}handlers.clear();first[0]+=10;console.log('done',first[0]);}function main():void{scenario(0,false);scenario(2,false);scenario(2,true);}");
    support::execute(&source,"after 0 1\nkept old 7\ndone 17\nafter 2 1\nkept new 11\ndone 19\nafter 1 0\ndone 18\n",None);
    verify_disposal(&source,2,"after 0 1\nkept old 7\ndrop 0\ndone 17\ndrop 0\ndrop 1\nafter 2 1\nkept new 11\ndrop 1\ndone 19\ndrop 0\nafter 1 0\ndone 18\n");
    for replacement in ["for(let step=0;step<limit;step+=1){","for(const step of [0,1].slice(0,limit)){"] {
        let variant=source.replace("while (turn < limit) {",replacement);assert_ne!(variant,source);
        support::execute(&support::format_round_trip(&variant),"after 0 1\nkept old 7\ndone 17\nafter 2 1\nkept new 11\ndone 19\nafter 1 0\ndone 18\n",None);
    }
    let post=source.replace("while (turn < limit) {","do{").replace("console.log('after', turn, handlers.size);","while(turn<limit);console.log('after',turn,handlers.size);");
    support::execute(&support::format_round_trip(&post),"after 1 1\nkept new 11\ndone 18\nafter 2 1\nkept new 11\ndone 19\nafter 1 0\ndone 18\n",None);
    let narrowed=source.replace("handlers.set('old', {","const discarded=[13];handlers.set('discard',{run:():int=>discarded[0]});handlers.delete('discard');discarded[0]+=1;handlers.set('old',{");
    assert_ne!(narrowed,source);
    support::execute(&support::format_round_trip(&narrowed),"after 0 1\nkept old 7\ndone 17\nafter 2 1\nkept new 11\ndone 19\nafter 1 0\ndone 18\n",None);
    let early=source.replace("handlers.delete('old');","if(stop)break;handlers.delete('old');");
    support::execute(&support::format_round_trip(&early),"after 0 1\nkept old 7\ndone 17\nafter 2 1\nkept new 11\ndone 19\nafter 0 1\nkept old 7\ndone 17\n",None);
}


#[test]
fn labelled_map_block_exit_from_nested_loop_preserves_cleanup() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});exit:{try{while(true){handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;break exit;}}finally{console.log('cleanup',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}first[0]+=10;console.log('after',handlers.size,first[0]);handlers.clear();}");
    support::execute(&source,"cleanup 1\nkept second 5\nafter 1 14\n",None);
    verify_disposal(&source,2,"drop 1\ncleanup 1\nkept second 5\nafter 1 14\ndrop 0\n");
    for replacement in ["for(let step=0;step<1;step+=1){","do{","for(const step of [0]){"] {
        let mut variant=source.replace("while (true) {",replacement);assert_ne!(variant,source);
        if replacement=="do{" {variant=variant.replace("} finally {","while(false);}finally{");}
        support::execute(&support::format_round_trip(&variant),"cleanup 1\nkept second 5\nafter 1 14\n",None);
    }
}


#[test]
fn labelled_map_block_reaching_exit_retains_early_entries_and_refill() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const third=[7];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});exit:{handlers.set('first',{run:():int=>first[0]});if(stop)break exit;handlers.delete('first');first[0]+=1;handlers.set('third',{run:():int=>third[0]});}console.log('after',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}handlers.clear();first[0]+=10;third[0]+=10;console.log('done',first[0],third[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"after 2\nkept second 5\nkept third 7\ndone 14 17\nafter 2\nkept second 5\nkept first 3\ndone 13 17\n",None);
}


#[test]
fn labelled_map_block_direct_exit_preserves_survivors() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});exit:{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;if(stop)break exit;console.log('tail');}console.log('after',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"tail\nafter 1\nkept second 5\ndone 4\nafter 1\nkept second 5\ndone 4\n",None);
    verify_disposal(&source,2,"drop 1\ntail\nafter 1\nkept second 5\ndrop 0\ndone 4\ndrop 1\nafter 1\nkept second 5\ndrop 0\ndone 4\n");
    let nested=source.replace("exit: {","outer:{exit:{").replace("break exit;","break outer;").replace("console.log('after', handlers.size);","}console.log('after',handlers.size);");
    assert_ne!(nested,source);
    support::execute(&support::format_round_trip(&nested),"tail\nafter 1\nkept second 5\ndone 4\nafter 1\nkept second 5\ndone 4\n",None);
}


#[test]
fn labelled_map_block_exit_preserves_survivors_after_cleanup() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});exit:{try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;if(stop)break exit;console.log('tail');}finally{console.log('cleanup',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}console.log('after',handlers.size);handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"tail\ncleanup 1\nkept second 5\nafter 1\ndone 4\ncleanup 1\nkept second 5\nafter 1\ndone 4\n",None);
    verify_disposal(&source,2,"drop 1\ntail\ncleanup 1\nkept second 5\nafter 1\ndrop 0\ndone 4\ndrop 1\ncleanup 1\nkept second 5\nafter 1\ndrop 0\ndone 4\n");
    let overwritten=source.replace("console.log('cleanup', handlers.size);","console.log('cleanup',handlers.size);if(!stop)break exit;");
    support::execute(&support::format_round_trip(&overwritten),"tail\ncleanup 1\nafter 1\ndone 4\ncleanup 1\nkept second 5\nafter 1\ndone 4\n",None);
    let outside=source.replace("console.log('after', handlers.size);","first[0]+=10;console.log('after',handlers.size);");
    support::execute(&support::format_round_trip(&outside),"tail\ncleanup 1\nkept second 5\nafter 1\ndone 14\ncleanup 1\nkept second 5\nafter 1\ndone 14\n",None);

    let conflict=source.replace("first[0] += 1;","second[0]+=1;");
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("surviving block capture remains borrowed").code,"capture-conflict");
}


#[test]
fn nested_map_direct_outer_controls_preserve_survivors() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;outer:while(turn<3){inner:while(true){handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(stop)break outer;if(turn<3)continue outer;break inner;}console.log('tail',turn);}console.log('size',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"tail 3\nsize 1\nkept second 5\ndone 6\nsize 1\nkept second 5\ndone 4\n",None);
    verify_disposal(&source,2,"drop 1\ndrop 1\ndrop 1\ntail 3\nsize 1\nkept second 5\ndrop 0\ndone 6\ndrop 1\nsize 1\nkept second 5\ndrop 0\ndone 4\n");
    for replacement in ["inner:for(let step=0;step<1;step+=1){","inner:do{","inner:for(const step of [0]){"] {
        let mut variant=source.replace("inner: while (true) {",replacement);
        assert_ne!(variant,source);
        if replacement=="inner:do{" {variant=variant.replace("console.log('tail', turn);","while(false);console.log('tail',turn);");}
        support::execute(&support::format_round_trip(&variant),"tail 3\nsize 1\nkept second 5\ndone 6\nsize 1\nkept second 5\ndone 4\n",None);
    }
    let conflict=source.replace("first[0] += 1;","second[0]+=1;");
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("surviving capture remains borrowed").code,"capture-conflict");
}


#[test]
fn nested_map_cleanup_loops_run_inner_finally_before_outer_control() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;outer:while(turn<3){try{inner:while(true){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(stop)break outer;if(turn<3)continue outer;break inner;}finally{console.log('inner',turn);}}console.log('tail',turn);}finally{console.log('cleanup',turn,handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"inner 1\ncleanup 1 1\nkept second 5\ninner 2\ncleanup 2 1\nkept second 5\ninner 3\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ninner 1\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    verify_disposal(&source,2,"drop 1\ninner 1\ncleanup 1 1\nkept second 5\ndrop 1\ninner 2\ncleanup 2 1\nkept second 5\ndrop 1\ninner 3\ntail 3\ncleanup 3 1\nkept second 5\ndrop 0\ndone 6\ndrop 1\ninner 1\ncleanup 1 1\nkept second 5\ndrop 0\ndone 4\n");
    for replacement in ["inner:for(let step=0;step<1;step+=1){","inner:do{","inner:for(const step of [0]){"] {
        let mut variant=source.replace("inner: while (true) {",replacement);
        assert_ne!(variant,source);
        if replacement=="inner:do{" {variant=variant.replace("console.log('tail', turn);","while(false);console.log('tail',turn);");}
        support::execute(&support::format_round_trip(&variant),"inner 1\ncleanup 1 1\nkept second 5\ninner 2\ncleanup 2 1\nkept second 5\ninner 3\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ninner 1\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    }
    let overridden=source.replace("console.log('inner', turn);","console.log('inner',turn);if(turn===1)continue inner;");
    assert_ne!(overridden,source);
    support::execute(&support::format_round_trip(&overridden),"inner 1\ninner 2\ncleanup 2 1\nkept second 5\ninner 3\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ninner 1\ninner 2\ncleanup 2 1\nkept second 5\ndone 5\n",None);
    let mut caught=source.replace("if (stop) break outer;","if(stop)throw 7;");
    let boundary=caught.rfind("} finally {").expect("outer cleanup");
    caught.insert_str(boundary+1,"catch(error){console.log('caught',error);if(stop)break outer;}");
    support::execute(&support::format_round_trip(&caught),"inner 1\ncleanup 1 1\nkept second 5\ninner 2\ncleanup 2 1\nkept second 5\ninner 3\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ninner 1\ncaught 7\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    let conflict=source.replace("first[0] += 1;","second[0]+=1;");
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("nested survivor capture remains borrowed").code,"capture-conflict");
}

#[test]
fn labelled_map_cleanup_loop_preserves_survivors_and_control() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;outer:while(turn<3){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(stop)break outer;if(turn<3)continue outer;console.log('tail',turn);}finally{console.log('cleanup',turn,handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    verify_disposal(&source,2,"drop 1\ncleanup 1 1\nkept second 5\ndrop 1\ncleanup 2 1\nkept second 5\ndrop 1\ntail 3\ncleanup 3 1\nkept second 5\ndrop 0\ndone 6\ndrop 1\ncleanup 1 1\nkept second 5\ndrop 0\ndone 4\n");
    for replacement in ["outer:for(let step=0;step<3;step+=1){","outer:do{","outer:for(const step of [0,1,2]){"] {
        let mut variant=source.replace("outer: while (turn < 3) {",replacement);
        assert_ne!(variant,source);
        if replacement=="outer:do{" {variant=variant.replace("handlers.clear();","while(turn<3);handlers.clear();");}
        support::execute(&support::format_round_trip(&variant),"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    }
}

#[test]
fn map_cleanup_loop_without_backedge_releases_deleted_capture() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario():int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});while(true){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;return first[0];}finally{console.log('cleanup',handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}return 0;}function main():void{console.log('result',scenario());}");
    support::execute(&source,"cleanup 1\nkept second 5\nresult 4\n",None);
    verify_disposal(&source,2,"drop 1\ncleanup 1\nkept second 5\ndrop 0\nresult 4\n");
    for replacement in ["for(let step=0;step<3;step+=1){","do{","for(const step of [0,1,2]){"] {
        let mut variant=source.replace("while (true) {",replacement);
        assert_ne!(variant,source);
        if replacement=="do{" {variant=variant.replace("return 0;","while(false);");}
        support::execute(&support::format_round_trip(&variant),"cleanup 1\nkept second 5\nresult 4\n",None);
    }
    let unsafe_cleanup=source.replace("} finally {","}finally{first[0]+=10;");
    assert_eq!(tsrust_compiler::check(&unsafe_cleanup).err().expect("pre-deletion failure retains capture").code,"capture-conflict","a failure before deletion can retain the capture entering cleanup");
    let evaluated=source.replace("} finally {","}finally{handlers.delete('first');first[0]+=10;");
    assert_ne!(evaluated,source);
    support::execute(&support::format_round_trip(&evaluated),"cleanup 1\nkept second 5\nresult 4\n",None);
}

#[test]
fn map_cleanup_loop_preserves_pending_return_and_error() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario():int{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<3){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(turn===2)return first[0];}finally{console.log('cleanup',turn,handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}handlers.clear();return first[0];}function main():void{console.log('result',scenario());}");
    support::execute(&source,"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\nresult 5\n",None);
    verify_disposal(&source,2,"drop 1\ncleanup 1 1\nkept second 5\ndrop 1\ncleanup 2 1\nkept second 5\ndrop 0\nresult 5\n");
    for replacement in ["for(let step=0;step<3;step+=1){","do{","for(const step of [0,1,2]){"] {
        let mut variant=source.replace("while (turn < 3) {",replacement);
        assert_ne!(variant,source);
        if replacement=="do{" {variant=variant.replace("handlers.clear();","while(turn<3);handlers.clear();");}
        support::execute(&support::format_round_trip(&variant),"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\nresult 5\n",None);
    }
    let replaced=source.replace("} finally {","}finally{if(turn===2)return 99;");
    assert_ne!(replaced,source);
    let replaced=support::format_round_trip(&replaced);
    support::execute(&replaced,"cleanup 1 1\nkept second 5\nresult 99\n",None);
    verify_disposal(&replaced,2,"drop 1\ncleanup 1 1\nkept second 5\ndrop 1\ndrop 0\nresult 99\n");
    let resumed=source.replace("} finally {","}finally{if(turn===2)continue;");
    assert_ne!(resumed,source);
    support::execute(&support::format_round_trip(&resumed),"cleanup 1 1\nkept second 5\ncleanup 3 1\nkept second 5\nresult 6\n",None);
    let failed=source.replacen("return first[0];", "throw first[0];",1).replace("console.log('result', scenario());", "try{console.log('result',scenario());}catch(error){console.log('error',error);}");
    assert!(failed.contains("throw first[0]"),"formatted return witness must be replaced");
    support::execute(&support::format_round_trip(&failed),"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\nerror 5\n",None);
    let recovered=failed.replace("} finally {","}finally{if(turn===2)return 99;");
    assert_ne!(recovered,failed);
    support::execute(&support::format_round_trip(&recovered),"cleanup 1 1\nkept second 5\nresult 99\n",None);
    let replaced_error=source.replace("} finally {","}finally{if(turn===2)throw 99;").replace("console.log('result', scenario());", "try{console.log('result',scenario());}catch(error){console.log('error',error);}");
    support::execute(&support::format_round_trip(&replaced_error),"cleanup 1 1\nkept second 5\nerror 99\n",None);
}

#[test]
fn cleanup_loop_preserves_populated_entry_before_empty_backedge() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<2){try{console.log('entry',turn,handlers.size);handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;}finally{handlers.clear();if(turn===1)continue;}}handlers.clear();console.log('done',first[0]);}");
    support::execute(&source,"entry 0 1\nentry 1 0\ndone 5\n",None);
    verify_disposal(&source,2,"entry 0 1\ndrop 1\ndrop 0\nentry 1 0\ndrop 1\ndone 5\n");
}

#[test]
fn map_deletion_inside_try_carries_survivor_through_finally() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});try{handlers.delete('first');first[0]=9;console.log('body',first[0]);}finally{console.log('cleanup',handlers.size);for(const [key,entry] of handlers){console.log(key,entry.run());}}handlers.clear();console.log('done',first[0]);}");
    support::execute(&source,"body 9\ncleanup 1\nsecond 5\ndone 9\n",None);
    verify_disposal(&source,2,"drop 0\nbody 9\ncleanup 1\nsecond 5\ndrop 1\ndone 9\n");
    let failed=source.replace("console.log('body', first[0]);","for(const [key,entry] of handlers){console.log('value',entry.run());throw 7;}").replace("} finally {","}catch(error){console.log('caught',handlers.size);}finally{");
    assert_ne!(failed,source);
    let failed=support::format_round_trip(&failed);
    support::execute(&failed,"value 5\ncaught 1\ncleanup 1\nsecond 5\ndone 9\n",None);
    verify_disposal(&failed,2,"drop 0\nvalue 5\ncaught 1\ncleanup 1\nsecond 5\ndrop 1\ndone 9\n");
    let handled=source.replace("try {","try{throw 7;}catch(error){");
    assert_ne!(handled,source);
    let handled=support::format_round_trip(&handled);
    support::execute(&handled,"body 9\ncleanup 1\nsecond 5\ndone 9\n",None);
    verify_disposal(&handled,2,"drop 0\nbody 9\ncleanup 1\nsecond 5\ndrop 1\ndone 9\n");
    let cleaned=source.replace("handlers.delete('first');", "").replace("first[0] = 9;", "").replace("} finally {", "}finally{handlers.delete('first');first[0]=9;");
    assert_ne!(cleaned,source);
    let cleaned=support::format_round_trip(&cleaned);
    support::execute(&cleaned,"body 3\ncleanup 1\nsecond 5\ndone 9\n",None);
    verify_disposal(&cleaned,2,"body 3\ndrop 0\ncleanup 1\nsecond 5\ndrop 1\ndone 9\n");
}

#[test]
fn map_entry_iteration_control_runs_finally_before_leaving_body() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<3){try{handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(stop)break;if(turn<3)continue;console.log('tail',turn);}finally{console.log('cleanup',turn,handlers.size);for(const [key,entry] of handlers){console.log('kept',key,entry.run());}}}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    support::execute(&source,"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    verify_disposal(&source,2,"drop 1\ncleanup 1 1\nkept second 5\ndrop 1\ncleanup 2 1\nkept second 5\ndrop 1\ntail 3\ncleanup 3 1\nkept second 5\ndrop 0\ndone 6\ndrop 1\ncleanup 1 1\nkept second 5\ndrop 0\ndone 4\n");
    for replacement in ["for(let step=0;step<3;step+=1){","do{","for(const step of [0,1,2]){"] {
        let mut variant=source.replace("while (turn < 3) {",replacement);
        assert_ne!(variant,source);
        if replacement=="do{" {variant=variant.replace("handlers.clear();","while(turn<3);handlers.clear();");}
        let variant=support::format_round_trip(&variant);
        support::execute(&variant,"cleanup 1 1\nkept second 5\ncleanup 2 1\nkept second 5\ntail 3\ncleanup 3 1\nkept second 5\ndone 6\ncleanup 1 1\nkept second 5\ndone 4\n",None);
        verify_disposal(&variant,2,"drop 1\ncleanup 1 1\nkept second 5\ndrop 1\ncleanup 2 1\nkept second 5\ndrop 1\ntail 3\ncleanup 3 1\nkept second 5\ndrop 0\ndone 6\ndrop 1\ncleanup 1 1\nkept second 5\ndrop 0\ndone 4\n");
    }
    let override_control=source.replace("} finally {","}finally{if(turn===1){console.log('override',turn);continue;}if(turn===2){console.log('override',turn);break;}");
    assert_ne!(override_control,source);
    let override_control=support::format_round_trip(&override_control);
    support::execute(&override_control,"override 1\noverride 2\ndone 5\noverride 1\noverride 2\ndone 5\n",None);
    verify_disposal(&override_control,2,"drop 1\noverride 1\ndrop 1\noverride 2\ndrop 0\ndone 5\ndrop 1\noverride 1\ndrop 1\noverride 2\ndrop 0\ndone 5\n");
    let override_break=source.replace("} finally {","}finally{if(turn===1){console.log('override',turn);break;}");
    assert_ne!(override_break,source);
    let override_break=support::format_round_trip(&override_break);
    support::execute(&override_break,"override 1\ndone 4\noverride 1\ndone 4\n",None);
    verify_disposal(&override_break,2,"drop 1\noverride 1\ndrop 0\ndone 4\ndrop 1\noverride 1\ndrop 0\ndone 4\n");
    let nested=source.replace("try {","try{try{").replace("} finally {","}finally{console.log('inner',turn);}}finally{");
    assert_ne!(nested,source);
    let nested=support::format_round_trip(&nested);
    support::execute(&nested,"inner 1\ncleanup 1 1\nkept second 5\ninner 2\ncleanup 2 1\nkept second 5\ntail 3\ninner 3\ncleanup 3 1\nkept second 5\ndone 6\ninner 1\ncleanup 1 1\nkept second 5\ndone 4\n",None);
    verify_disposal(&nested,2,"drop 1\ninner 1\ncleanup 1 1\nkept second 5\ndrop 1\ninner 2\ncleanup 2 1\nkept second 5\ndrop 1\ntail 3\ninner 3\ncleanup 3 1\nkept second 5\ndrop 0\ndone 6\ndrop 1\ninner 1\ncleanup 1 1\nkept second 5\ndrop 0\ndone 4\n");
}

#[test]
fn counted_map_update_keeps_body_payload_until_update_deletes_it() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const third=[7];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;for(let step=0;step<3;step=(console.log('update',step,handlers.size),handlers.delete('temporary'),step+1)){handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;handlers.set('temporary',{run:():int=>third[0]});turn++;if(turn<3)continue;console.log('tail',turn);}third[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();console.log('done',first[0],third[0]);}");
    support::execute(&source,"update 0 2\nupdate 1 2\ntail 3\nupdate 2 2\nsecond 5\ndone 6 9\n",None);
    verify_disposal(&source,3,"drop 1\nupdate 0 2\ndrop 2\ndrop 1\nupdate 1 2\ndrop 2\ndrop 1\ntail 3\nupdate 2 2\ndrop 2\nsecond 5\ndrop 0\ndone 6 9\n");
    let stopped=source.replace("turn++;", "turn++;if(turn===2){handlers.delete('temporary');break;}");
    assert_ne!(stopped,source);
    support::execute(&support::format_round_trip(&stopped),"update 0 2\nsecond 5\ndone 5 9\n",None);
    verify_disposal(&support::format_round_trip(&stopped),3,"drop 1\nupdate 0 2\ndrop 2\ndrop 1\ndrop 2\nsecond 5\ndrop 0\ndone 5 9\n");
    let conflict=source.replace("handlers.delete('temporary'),", "third[0]=8,handlers.delete('temporary'),");
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("update cannot change a retained temporary capture").code,"capture-conflict");
}

#[test]
fn map_iteration_release_survives_continue_and_break() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(stop:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<3){handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;turn++;if(stop)break;if(turn<3)continue;console.log('tail',turn);}for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(false);scenario(true);}");
    let conflict = source.replace("first[0] += 1;", "second[0]+=1;");
    assert_ne!(conflict, source);
    assert_eq!(tsrust_compiler::check(&conflict).err().expect("retained entry protects its capture on abrupt edges").code, "capture-conflict");
    let early = source.replace("handlers.delete('first');", "if(stop)break;handlers.delete('first');")
        .replace("handlers.clear();", "first[0]=99;handlers.clear();");
    assert_eq!(tsrust_compiler::check(&early).err().expect("break before deletion keeps the first capture live").code,"capture-conflict");
    support::execute(&source, "tail 3\nsecond 5\ndone 6\nsecond 5\ndone 4\n", None);
    let counted = "function advance(value:int):int{console.log('update',value);return value+1;}".to_owned()
        + &source.replace("while (turn < 3)", "for(let step=0;step<3;step=advance(step))");
    support::execute(&support::format_round_trip(&counted), "update 0\nupdate 1\ntail 3\nupdate 2\nsecond 5\ndone 6\nsecond 5\ndone 4\n", None);
    let post = source.replace("while (turn < 3)", "do").replace("console.log('tail', turn);\n    }", "console.log('tail',turn);}while(turn<3);");
    assert_ne!(post,source);
    support::execute(&support::format_round_trip(&post), "tail 3\nsecond 5\ndone 6\nsecond 5\ndone 4\n", None);
    let for_of = source.replace("while (turn < 3)", "for(const step of [0,1,2])");
    support::execute(&support::format_round_trip(&for_of), "tail 3\nsecond 5\ndone 6\nsecond 5\ndone 4\n", None);
    verify_disposal(&source,2,"drop 1\ndrop 1\ndrop 1\ntail 3\nsecond 5\ndrop 0\ndone 6\ndrop 1\nsecond 5\ndrop 0\ndone 4\n");
}

#[test]
fn map_entry_release_inside_iteration_preserves_refill_and_survivor() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<limit){handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;console.log('turn',turn,first[0]);turn++;}for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(0);scenario(2);}");
 let conflict=source.replace("first[0] += 1;","second[0]+=1;");
 assert_ne!(conflict,source);
 assert_eq!(tsrust_compiler::check(&conflict).err().expect("surviving entry protects its capture").code,"capture-conflict");
 support::execute(&source,"second 5\ndone 3\nturn 0 4\nturn 1 5\nsecond 5\ndone 5\n",None);
 let counted=source.replace("while (turn < limit)","for(let step=0;step<limit;step+=1)");
 assert_ne!(counted,source);
 support::execute(&support::format_round_trip(&counted),"second 5\ndone 3\nturn 0 4\nturn 1 5\nsecond 5\ndone 5\n",None);
 let for_of=source.replace("let turn = 0;","let turn=0;const steps:int[]=[];for(let step=0;step<limit;step++){steps.push(step);}").replace("while (turn < limit)","for(const step of steps)");
 assert_ne!(for_of,source);
 support::execute(&support::format_round_trip(&for_of),"second 5\ndone 3\nturn 0 4\nturn 1 5\nsecond 5\ndone 5\n",None);
 let text=for_of.replace("for(const step of steps)","for(const step of (limit===0?'':'ab'))");
 assert_ne!(text,for_of);
 support::execute(&support::format_round_trip(&text),"second 5\ndone 3\nturn 0 4\nturn 1 5\nsecond 5\ndone 5\n",None);
 let field="type Box={handlers:Map<string,Holder>;marker:int;};".to_owned()+&source.replace("const handlers = new Map<string, Holder>();","const box:Box={handlers:new Map<string,Holder>(),marker:17};").replace("handlers.","box.handlers.").replace("of handlers)","of box.handlers)").replace("console.log('done', first[0]);","console.log('done',first[0],box.marker);");
 assert_ne!(field,source);
 support::execute(&support::format_round_trip(&field),"second 5\ndone 3 17\nturn 0 4\nturn 1 5\nsecond 5\ndone 5 17\n",None);
 let start=source.find("    handlers.set('second'").unwrap();let end=source.find("    let turn").unwrap();
 let empty=format!("{}{}",&source[..start],&source[end..]);
 support::execute(&support::format_round_trip(&empty),"done 3\nturn 0 4\nturn 1 5\ndone 5\n",None);
 verify_disposal(&source,2,"second 5\ndrop 0\ndone 3\ndrop 1\nturn 0 4\ndrop 1\nturn 1 5\nsecond 5\ndrop 0\ndone 5\n");
}

#[test]
fn post_test_map_iteration_release_keeps_first_entry_state_distinct() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('second',{run:():int=>second[0]});let turn=0;while(turn<limit){handlers.set('first',{run:():int=>first[0]});handlers.delete('first');first[0]+=1;console.log('turn',turn,first[0]);turn++;}for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();console.log('done',first[0]);}function main():void{scenario(0);scenario(2);}");
 let post=source.replace("handlers.set('second',", "handlers.set('first',{run:():int=>first[0]});handlers.set('second',").replace("while (turn < limit)","do").replace("turn++;\n    }","turn++;}while(turn<limit);");
 assert_ne!(post,source);
 support::execute(&support::format_round_trip(&post),"turn 0 4\nsecond 5\ndone 4\nturn 0 4\nturn 1 5\nsecond 5\ndone 5\n",None);
 verify_disposal(&support::format_round_trip(&post),3,"drop 0\ndrop 2\nturn 0 4\nsecond 5\ndrop 1\ndone 4\ndrop 0\ndrop 2\nturn 0 4\ndrop 2\nturn 1 5\nsecond 5\ndrop 1\ndone 5\n");
 let formatted=support::format_round_trip(&post);
 let start=formatted.find("    handlers.set('second'").unwrap();let end=formatted.find("    let turn").unwrap();
 let empty=format!("{}{}",&formatted[..start],&formatted[end..]);
 support::execute(&support::format_round_trip(&empty),"turn 0 4\ndone 4\nturn 0 4\nturn 1 5\ndone 5\n",None);
 let guarded="function repeat(turn:int,limit:int):boolean{console.log('check',turn);return turn<limit;}".to_owned()+&post.replace("while(turn<limit);","while(repeat(turn,limit));");
 assert_ne!(guarded,post);
 support::execute(&support::format_round_trip(&guarded),"turn 0 4\ncheck 1\nsecond 5\ndone 4\nturn 0 4\ncheck 1\nturn 1 5\ncheck 2\nsecond 5\ndone 5\n",None);

}

#[test]
fn narrowed_map_branch_refill_preserves_new_origins_at_join() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const third=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');first[0]=9;if(branch){handlers.set('third',{run:():int=>third[0]});}for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();third[0]=11;console.log('released',first[0],third[0]);}function main():void{scenario(true);scenario(false);}");
 support::execute(&source,"second 5\nthird 7\nreleased 9 11\nsecond 5\nreleased 9 11\n",None);
 verify_disposal(&source,3,"drop 0\nsecond 5\nthird 7\ndrop 1\ndrop 2\nreleased 9 11\ndrop 0\nsecond 5\ndrop 1\nreleased 9 11\n");
 let split=source.find("    if (branch)").unwrap();
 let moved=format!("{}const moved=handlers;{}",&source[..split],source[split..].replace("handlers","moved"));
 support::execute(&support::format_round_trip(&moved),"second 5\nthird 7\nreleased 9 11\nsecond 5\nreleased 9 11\n",None);
 let multiple=source.replace("const third = [7];","const third=[7];const fourth=[8];").replace("handlers.set('third',", "handlers.set('fourth',{run:():int=>fourth[0]});handlers.set('third',");
 assert_ne!(multiple,source);
 support::execute(&support::format_round_trip(&multiple),"second 5\nfourth 8\nthird 7\nreleased 9 11\nsecond 5\nreleased 9 11\n",None);

}

#[test]
fn deleting_last_map_entry_releases_its_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const values=[3];let handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});console.log('deleted',handlers.delete('first'));values[0]=9;console.log('after',handlers.size,values[0],handlers.has('first'));handlers.clear();}");
 verify_disposal(&source,1,"drop 0\ndeleted true\nafter 0 9 false\n");
 let conditional=source.replace("console.log('deleted', handlers.delete('first'));", "const deleted=handlers.delete('first')?1:0;console.log('deleted',deleted);");
 assert_ne!(conditional,source);
 verify_disposal(&support::format_round_trip(&conditional),1,"drop 0\ndeleted 1\nafter 0 9 false\n");
}

#[test]
fn deleting_from_a_moved_map_releases_only_its_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];let original=new Map<string,Holder>();original.set('first',{run:():int=>first[0]});original.set('second',{run:():int=>second[0]});const moved=original;console.log('deleted',moved.delete('first'));first[0]=9;{const saved=moved.get('second');if(saved!==undefined)console.log('saved',saved.run());}console.log('after',moved.size,first[0]);moved.clear();second[0]=11;console.log('cleared',second[0]);}");
 verify_disposal(&source,2,"drop 0\ndeleted true\nsaved 5\nafter 1 9\ndrop 1\ncleared 11\n");
 let narrowed=source.replace("const moved = original;\n    console.log('deleted', moved.delete('first'));", "console.log('deleted',original.delete('first'));const moved=original;");
 assert_ne!(narrowed,source);
 verify_disposal(&support::format_round_trip(&narrowed),2,"drop 0\ndeleted true\nsaved 5\nafter 1 9\ndrop 1\ncleared 11\n");
}

#[test]
fn deleting_one_map_entry_releases_only_its_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];let handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});console.log('deleted',handlers.delete('first'));first[0]=9;{const saved=handlers.get('second');if(saved!==undefined)console.log('saved',saved.run());}console.log('after',handlers.size,first[0]);handlers.clear();second[0]=11;console.log('cleared',second[0]);}");
 let conflict=source.replace("first[0] = 9;", "second[0]=9;");
 assert_ne!(conflict,source);
 assert_eq!(tsrust_compiler::check(&conflict).err().expect("remaining entry retains capture").code,"capture-conflict");
 verify_disposal(&source,2,"drop 0\ndeleted true\nsaved 5\nafter 1 9\ndrop 1\ncleared 11\n");
 let iterated=source.replace("console.log('after', handlers.size, first[0]);", "for(const [key,entry] of handlers){console.log('entry',key,entry.run());}console.log('after',handlers.size,first[0]);");
 assert_ne!(iterated,source);
 verify_disposal(&support::format_round_trip(&iterated),2,"drop 0\ndeleted true\nsaved 5\nentry second 5\nafter 1 9\ndrop 1\ncleared 11\n");
 let repeated=source.replace("first[0] = 9;", "console.log('again',handlers.delete('first'),handlers.delete('missing'));first[0]=9;");
 assert_ne!(repeated,source);
 verify_disposal(&support::format_round_trip(&repeated),2,"drop 0\ndeleted true\nagain false false\nsaved 5\nafter 1 9\ndrop 1\ncleared 11\n");
}

#[test]
fn replacing_one_map_entry_releases_its_previous_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];let handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});console.log('replace');handlers.set('first',{run:():int=>second[0]});first[0]=9;{const saved=handlers.get('first');if(saved!==undefined)console.log('saved',saved.run());}handlers.clear();second[0]=11;console.log('after',first[0],second[0]);}");
 verify_disposal(&source,2,"replace\ndrop 0\nsaved 5\ndrop 1\nafter 9 11\n");
}

#[test]
fn failed_map_replacement_keeps_the_previous_environment() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const first=[3];const second=[5];let handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});try{handlers.set('first',(failIf(true),{run:():int=>second[0]}));}catch(error){console.log('caught',error);}{const saved=handlers.get('first');if(saved!==undefined)console.log('saved',saved.run());}handlers.clear();console.log('done');}");
 support::execute(&source,"caught 7\nsaved 3\ndone\n",None);
 verify_disposal(&source,2,"caught 7\nsaved 3\ndrop 0\ndone\n");
 let successful=source.replace("failIf(true)","failIf(false)");
 assert_ne!(successful,source);
 verify_disposal(&support::format_round_trip(&successful),2,"drop 0\nsaved 5\ndrop 1\ndone\n");
 let conflict=source.replace("console.log('caught', error);","first[0]=9;console.log('caught',error);");
 assert_ne!(conflict,source);
 assert_eq!(tsrust_compiler::check(&conflict).err().expect("failed replacement retains previous capture").code,"capture-conflict");
}

#[test]
fn deleting_record_map_field_releases_only_its_entry_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;marker:int;};function main():void{const first=[3];const second=[5];const box:Box={handlers:new Map<string,Holder>(),marker:23};box.handlers.set('first',{run:():int=>first[0]});box.handlers.set('second',{run:():int=>second[0]});console.log('deleted',box.handlers.delete('first'));first[0]=9;{const saved=box.handlers.get('second');if(saved!==undefined)console.log('saved',saved.run());}console.log('marker',box.marker);box.handlers.clear();second[0]=11;console.log('after',first[0],second[0]);}");
 verify_disposal(&source,2,"drop 0\ndeleted true\nsaved 5\nmarker 23\ndrop 1\nafter 9 11\n");
 let refill=source.replace("console.log('marker', box.marker);", "box.handlers.set('next',{run:():int=>first[0]});{const next=box.handlers.get('next');if(next!==undefined)console.log('next',next.run());}console.log('marker',box.marker);");
 assert_ne!(refill,source);
 verify_disposal(&support::format_round_trip(&refill),3,"drop 0\ndeleted true\nsaved 5\nnext 9\nmarker 23\ndrop 1\ndrop 2\nafter 9 11\n");
 let iterated=refill.replace("console.log('marker',box.marker);","for(const [key,entry] of box.handlers){console.log('entry',key,entry.run());}console.log('marker',box.marker);");
 assert_ne!(iterated,refill);
 verify_disposal(&support::format_round_trip(&iterated),3,"drop 0\ndeleted true\nsaved 5\nnext 9\nentry second 5\nentry next 9\nmarker 23\ndrop 1\ndrop 2\nafter 9 11\n");
}

#[test]
fn nested_record_map_release_and_refill_preserve_siblings() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};type Inner={handlers:Map<string,Holder>;label:string;};type Outer={inner:Inner;marker:int;};function main():void{const first=[3];const second=[5];const outer:Outer={inner:{handlers:new Map<string,Holder>(),label:'kept'},marker:23};outer.inner.handlers.set('first',{run:():int=>first[0]});outer.inner.handlers.set('second',{run:():int=>second[0]});console.log('deleted',outer.inner.handlers.delete('first'));first[0]=9;outer.inner.handlers.set('next',{run:():int=>first[0]});for(const [key,entry] of outer.inner.handlers){console.log('entry',key,entry.run());}console.log('siblings',outer.inner.label,outer.marker);outer.inner.handlers.clear();first[0]=11;second[0]=13;console.log('after',first[0],second[0]);}");
 verify_disposal(&source,3,"drop 0\ndeleted true\nentry second 5\nentry next 9\nsiblings kept 23\ndrop 1\ndrop 2\nafter 11 13\n");
 let moved=source.replace("outer.inner.handlers.set('first',", "const moved=outer;outer.inner.handlers.set('first',").replace("outer.inner", "moved.inner").replace("outer.marker", "moved.marker");
 assert_ne!(moved,source);
 verify_disposal(&support::format_round_trip(&moved),3,"drop 0\ndeleted true\nentry second 5\nentry next 9\nsiblings kept 23\ndrop 1\ndrop 2\nafter 11 13\n");
 let conditioned=moved.replace("console.log('deleted', moved.inner.handlers.delete('first'));", "const deleted=moved.inner.handlers.delete('first')?1:0;console.log('deleted',deleted);");
 assert_ne!(conditioned,moved);
 verify_disposal(&support::format_round_trip(&conditioned),3,"drop 0\ndeleted 1\nentry second 5\nentry next 9\nsiblings kept 23\ndrop 1\ndrop 2\nafter 11 13\n");
 let called=conditioned.replace("function main(): void", "function scenario(): void")+"function main():void{scenario();}";
 assert_ne!(called,conditioned);
 verify_disposal(&support::format_round_trip(&called),3,"drop 0\ndeleted 1\nentry second 5\nentry next 9\nsiblings kept 23\ndrop 1\ndrop 2\nafter 11 13\n");
}

#[test]
fn narrowed_map_callback_errors_use_the_surviving_contract() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(value:int):int{const first=[value];const second=[value+1];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');first[0]=value+2;let result=0;{const saved=handlers.get('second');if(saved!==undefined)result=saved.run();}handlers.clear();return result+first[0];}function main():void{console.log(scenario(3));}");
 support::execute(&source,"9\n",None);
 let caught=source.replace("console.log(scenario(3));", "try{console.log(scenario(3));}catch(error){console.log('caught');}");
 assert_ne!(caught,source);
 support::execute(&support::format_round_trip(&caught),"9\n",None);
 let failing=caught.replace("second[0]","second[value]");
 assert_ne!(failing,caught);
 let failing=support::format_round_trip(&failing);
 verify_disposal(&failing,2,"drop 0\ndrop 1\ncaught\n");
}

#[test]
fn deleting_the_same_entry_on_both_branches_releases_its_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});if(branch){console.log('left',handlers.delete('first'));}else{console.log('right',handlers.delete('first'));}first[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0]);handlers.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,2,"drop 0\nleft true\nsecond 5\nreleased 9\ndrop 1\ndrop 0\nright true\nsecond 5\nreleased 9\ndrop 1\n");
 let conflict=source.replace("console.log('right', handlers.delete('first'));", "console.log('right');");
 assert_ne!(conflict,source);
 assert_eq!(tsrust_compiler::check(&conflict).err().expect("one branch retains capture").code,"capture-conflict");
 let reinserted=source.replace("console.log('right', handlers.delete('first'));", "handlers.delete('first');handlers.set('first',{run:():int=>first[0]});");
 assert_ne!(reinserted,source);
 assert_eq!(tsrust_compiler::check(&reinserted).err().expect("reinserted capture remains live").code,"capture-conflict");
 let exiting=source.replace("console.log('left', handlers.delete('first'));", "console.log('left');return;");
 assert_ne!(exiting,source);
 verify_disposal(&support::format_round_trip(&exiting),2,"left\ndrop 0\ndrop 1\ndrop 0\nright true\nsecond 5\nreleased 9\ndrop 1\n");
}

#[test]
fn conditional_deletion_releases_shared_capture_from_two_maps() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const third=[7];const left=new Map<string,Holder>();const right=new Map<string,Holder>();left.set('first',{run:():int=>first[0]});left.set('second',{run:():int=>second[0]});right.set('first',{run:():int=>first[0]});right.set('third',{run:():int=>third[0]});if(branch){left.delete('first');right.delete('first');}else{right.delete('first');left.delete('first');}first[0]=9;for(const [key,entry] of left){console.log(key,entry.run());}for(const [key,entry] of right){console.log(key,entry.run());}console.log('released',first[0]);left.clear();right.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,4,"drop 0\ndrop 2\nsecond 5\nthird 7\nreleased 9\ndrop 1\ndrop 3\ndrop 2\ndrop 0\nsecond 5\nthird 7\nreleased 9\ndrop 1\ndrop 3\n");
 let fields=source.replace("function scenario", "type Pair={left:Map<string,Holder>;right:Map<string,Holder>;marker:int;};function scenario").replace("const left = new Map<string, Holder>();\n    const right = new Map<string, Holder>();", "const box:Pair={left:new Map<string,Holder>(),right:new Map<string,Holder>(),marker:23};").replace("left.","box.left.").replace("right.","box.right.").replace("of left)","of box.left)").replace("of right)","of box.right)").replace("console.log('released', first[0]);", "console.log('released',first[0],box.marker);");
 assert_ne!(fields,source);
 verify_disposal(&support::format_round_trip(&fields),4,"drop 0\ndrop 2\nsecond 5\nthird 7\nreleased 9 23\ndrop 1\ndrop 3\ndrop 2\ndrop 0\nsecond 5\nthird 7\nreleased 9 23\ndrop 1\ndrop 3\n");

}

#[test]
fn conditional_clear_and_delete_release_the_common_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});if(branch){handlers.clear();}else{handlers.delete('first');}first[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0],handlers.size);handlers.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,2,"drop 0\ndrop 1\nreleased 9 0\ndrop 0\nsecond 5\nreleased 9 1\ndrop 1\n");
}

#[test]
fn replacing_the_same_entry_on_both_branches_releases_its_old_capture() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});if(branch){handlers.set('first',{run:():int=>second[0]});}else{handlers.set('first',{run:():int=>second[0]+1});}first[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0]);handlers.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,3,"drop 0\nfirst 5\nreleased 9\ndrop 1\ndrop 0\nfirst 6\nreleased 9\ndrop 2\n");
}

#[test]
fn branch_local_insertion_and_deletion_release_at_the_join() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();if(branch){handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');}else{handlers.set('second',{run:():int=>second[0]+1});}first[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0]);handlers.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,3,"drop 0\nsecond 5\nreleased 9\ndrop 1\nsecond 6\nreleased 9\ndrop 2\n");
}

#[test]
fn post_test_map_deletion_releases_capture_after_every_exit() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});let turn=0;do{handlers.delete('first');turn+=1;}while(turn<limit);first[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0],turn);handlers.clear();}function main():void{scenario(0);scenario(2);}");
 verify_disposal(&source,2,"drop 0\nsecond 5\nreleased 9 1\ndrop 1\ndrop 0\nsecond 5\nreleased 9 2\ndrop 1\n");
 let infinite=source.replace("do {\n        handlers.delete('first');\n        turn += 1;\n    }\n\n    while (turn < limit);", "for(;;){handlers.delete('first');turn+=1;break;}");
 assert_ne!(infinite,source);
 verify_disposal(&support::format_round_trip(&infinite),2,"drop 0\nsecond 5\nreleased 9 1\ndrop 1\ndrop 0\nsecond 5\nreleased 9 1\ndrop 1\n");
}

#[test]
fn pre_test_map_entry_release_preserves_zero_and_repeated_iterations() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();let turn=0;while(turn<limit){handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');turn+=1;}first[0]=9;for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0],turn);handlers.clear();}function main():void{scenario(0);scenario(2);}");
 let expected="released 9 0\ndrop 0\ndrop 1\ndrop 0\nsecond 5\nreleased 9 2\ndrop 1\n";
 verify_disposal(&source,2,expected);
 let counted=source.replace("while (turn < limit)", "for(;turn<limit;)");
 assert_ne!(counted,source);
 verify_disposal(&support::format_round_trip(&counted),2,expected);
 let iterated=source.replace("while (turn < limit)", "for(const step of steps)").replace("let turn = 0;", "const steps:int[]=[];for(let step=0;step<limit;step+=1){steps.push(step);}let turn=0;");
 assert_ne!(iterated,source);
 verify_disposal(&support::format_round_trip(&iterated),2,expected);
 let text=source.replace("while (turn < limit)", "for(const step of steps)").replace("let turn = 0;", "const steps=limit===0?'':'ab';let turn=0;");
 assert_ne!(text,source);
 verify_disposal(&support::format_round_trip(&text),2,expected);
}

#[test]
fn map_entry_release_inside_branch_precedes_capture_mutation() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});if(branch){handlers.delete('first');first[0]=9;console.log('released',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();return;}console.log('retained',handlers.size);handlers.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,2,"drop 0\nreleased 9\nsecond 5\ndrop 1\nretained 2\ndrop 0\ndrop 1\n");
}

#[test]
fn map_entry_release_inside_falling_branch_transports_remaining_storage() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});if(branch){handlers.delete('first');first[0]=9;console.log('released',first[0]);for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();}console.log('retained',handlers.size);handlers.clear();}function main():void{scenario(true);scenario(false);}");
 verify_disposal(&source,2,"drop 0\nreleased 9\nsecond 5\ndrop 1\nretained 0\nretained 2\ndrop 0\ndrop 1\n");
 let retained=source.replace("        handlers.clear();", "");
 assert_ne!(retained,source);
 verify_disposal(&support::format_round_trip(&retained),2,"drop 0\nreleased 9\nsecond 5\nretained 1\ndrop 1\nretained 2\ndrop 0\ndrop 1\n");

}

#[test]
fn narrowed_map_refill_in_loop_keeps_survivors_and_new_origins() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function scenario(limit:int):void{const first=[3];const second=[5];const third=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');first[0]=9;let turn=0;while(turn<limit){handlers.set('third',{run:():int=>third[0]});turn++;}for(const [key,entry] of handlers){console.log(key,entry.run());}console.log('released',first[0],turn);handlers.clear();third[0]=11;console.log('cleared',third[0]);}function main():void{scenario(0);scenario(2);}");
 support::execute(&source,"second 5\nreleased 9 0\ncleared 11\nsecond 5\nthird 7\nreleased 9 2\ncleared 11\n",None);
 verify_disposal(&source,3,"drop 0\nsecond 5\nreleased 9 0\ndrop 1\ncleared 11\ndrop 0\ndrop 2\nsecond 5\nthird 7\nreleased 9 2\ndrop 1\ndrop 2\ncleared 11\n");
 let counted=source.replace("while (turn < limit)","for (;turn<limit;)");
 assert_ne!(counted,source);
 support::execute(&support::format_round_trip(&counted),"second 5\nreleased 9 0\ncleared 11\nsecond 5\nthird 7\nreleased 9 2\ncleared 11\n",None);
 let for_of=source.replace("let turn = 0;", "let turn=0;const steps:int[]=[];for(let step=0;step<limit;step++){steps.push(step);}").replace("while (turn < limit)","for(const step of steps)");
 assert_ne!(for_of,source);
 support::execute(&support::format_round_trip(&for_of),"second 5\nreleased 9 0\ncleared 11\nsecond 5\nthird 7\nreleased 9 2\ncleared 11\n",None);
 let post=source.replace("while (turn < limit)","do").replace("turn++;\n    }", "turn++;}while(turn<limit);");
 assert_ne!(post,source);
 support::execute(&support::format_round_trip(&post),"second 5\nthird 7\nreleased 9 1\ncleared 11\nsecond 5\nthird 7\nreleased 9 2\ncleared 11\n",None);
 let multiple=source.replace("const third = [7];","const third=[7];const fourth=[8];").replace("turn++;","handlers.set('fourth',{run:():int=>fourth[0]});turn++;");
 assert_ne!(multiple,source);
 support::execute(&support::format_round_trip(&multiple),"second 5\nreleased 9 0\ncleared 11\nsecond 5\nthird 7\nfourth 8\nreleased 9 2\ncleared 11\n",None);

}

#[test]
fn ordinary_loop_deletion_does_not_scan_unused_released_captures() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();let turn=0;while(turn<2){handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});console.log('before',first[0]);handlers.delete('first');turn+=1;}for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();}");
 let analysis=tsrust_compiler::check(&source).unwrap();
 assert!(!analysis.emit_rust().contains("tsr_owned_items().map(|entry|"),"loop-internal capture reads do not require conversion after the loop");
 verify_disposal(&source,2,"before 3\ndrop 0\ndrop 1\nbefore 3\ndrop 0\nsecond 5\ndrop 1\n");
}

#[test]
fn zero_iteration_and_early_loop_exits_keep_map_capture_loans() {
 let prefix="type Holder={run:ReadOnly<Function<int>>;};function scenario(branch:boolean):void{const first=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});";
 for action in ["while(branch){handlers.delete('first');break;}","for(let turn=0;turn<2;turn+=1){handlers.delete('first');}","do{if(branch)break;handlers.delete('first');}while(false);","do{if(branch)continue;handlers.delete('first');}while(false);"] {
  let source=support::format_round_trip(&format!("{prefix}{action}first[0]=9;console.log(handlers.size);}}function main():void{{scenario(false);}}"));
  assert_eq!(tsrust_compiler::check(&source).err().expect("a reaching exit keeps the entry").code,"capture-conflict","{action}");
 }
}

#[test]
fn ordinary_map_deletion_does_not_scan_survivor_environments() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const first=[3];const second=[5];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});handlers.set('second',{run:():int=>second[0]});handlers.delete('first');for(const [key,entry] of handlers){console.log(key,entry.run());}handlers.clear();}");
 let analysis=tsrust_compiler::check(&source).unwrap();
 assert!(!analysis.emit_rust().contains("tsr_owned_items().map(|entry|"),"ordinary deletion must not rebuild surviving entries");
 verify_disposal(&source,2,"drop 0\nsecond 5\ndrop 1\n");
}

#[test]
fn keyed_map_release_keeps_uncertain_and_shared_capture_owners() {
 let prefix="type Holder={run:ReadOnly<Function<int>>;};function scenario(key:string,branch:boolean):void{const values=[3];let handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});";
 for actions in [
  "handlers.set('second',{run:():int=>values[0]});handlers.delete('first');",
  "handlers.set(key,{run:():int=>values[0]});handlers.delete('first');",
  "handlers.delete(key);",
  "handlers.delete('missing');",
  "if(branch)handlers.delete('first');",
  "handlers.set('first',{run:():int=>values[0]});",
 ] {
  let source=support::format_round_trip(&format!("{prefix}{actions}values[0]=9;console.log(handlers.size);}}function main():void{{scenario('second',false);}}"));
  let error=tsrust_compiler::check(&source).err().expect(actions);
  assert_eq!(error.code,"capture-conflict","{actions}: {}",error.message);
 }
}

#[test]
fn for_of_loops_release_cleared_map_captures() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const values=[3];const steps=[0,1,2];let handlers=new Map<string,Holder>();for(const step of steps){handlers.set('value',{run:():int=>values[0]});try{failIf(step!==1);}catch(error){if(step===0)continue;break;}finally{console.log('cleanup',step,handlers.size);handlers.clear();values[0]+=2;}}console.log('after',handlers.size,values[0]);}");
 verify_disposal(&source,1,"cleanup 0 1\ndrop 0\ncleanup 1 1\ndrop 0\ncleanup 2 1\ndrop 0\nafter 0 9\n");
 verify_disposal(&support::format_round_trip(&source.replace("const steps = [0, 1, 2];", "const steps:int[]=[];")),1,"after 0 3\n");
 let set=source.replace("const steps = [0, 1, 2];", "const steps=new Set<int>([0,1,2]);");
 assert_ne!(set,source);
 verify_disposal(&support::format_round_trip(&set),1,"cleanup 0 1\ndrop 0\ncleanup 1 1\ndrop 0\ncleanup 2 1\ndrop 0\nafter 0 9\n");
 let map=source.replace("const steps = [0, 1, 2];", "const steps=new Map<string,int>([['a',0],['b',1],['c',2]]);").replace("for (const step of steps)", "for(const [key,step] of steps)");
 assert_ne!(map,source);
 verify_disposal(&support::format_round_trip(&map),1,"cleanup 0 1\ndrop 0\ncleanup 1 1\ndrop 0\ncleanup 2 1\ndrop 0\nafter 0 9\n");
 for iterable in ["'abc'", "['a','b','c']"] {
  let text=source.replace("const steps = [0, 1, 2];",&format!("const steps={iterable};")).replace("step !== 1", "step!=='b'").replace("step === 0", "step==='a'");
  verify_disposal(&support::format_round_trip(&text),1,"cleanup a 1\ndrop 0\ncleanup b 1\ndrop 0\ncleanup c 1\ndrop 0\nafter 0 9\n");
 }
}

#[test]
fn counted_and_post_test_loops_release_cleared_map_captures() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const values=[3];let turn=0;let handlers=new Map<string,Holder>();for(;turn<3;turn+=1){handlers.set('value',{run:():int=>values[0]});try{failIf(turn!==1);}catch(error){if(turn===0)continue;break;}finally{console.log('cleanup',turn,handlers.size);handlers.clear();values[0]+=2;}}console.log('after',turn,handlers.size,values[0]);}");
 verify_disposal(&source,1,"cleanup 0 1\ndrop 0\ncleanup 1 1\ndrop 0\ncleanup 2 1\ndrop 0\nafter 2 0 9\n");
 let initialized=source.replace("for (; turn < 3; turn += 1)", "for(let step=0;step<3;step+=1)").replace("values[0] += 2;", "values[0]+=2;turn+=1;");
 assert_ne!(initialized,source);
 verify_disposal(&support::format_round_trip(&initialized),1,"cleanup 0 1\ndrop 0\ncleanup 1 1\ndrop 0\ncleanup 2 1\ndrop 0\nafter 3 0 9\n");
 verify_disposal(&support::format_round_trip(&source.replace("turn < 3", "turn < 0")),1,"after 0 0 3\n");
 let post=source.replace("for (; turn < 3; turn += 1)","do").replace("values[0] += 2;","values[0]+=2;turn+=1;").replace("console.log('after'", "while(turn<3);console.log('after'");
 assert_ne!(post,source);
 verify_disposal(&support::format_round_trip(&post),1,"cleanup 0 1\ndrop 0\ncleanup 1 1\ndrop 0\ncleanup 2 1\ndrop 0\nafter 3 0 9\n");
}

#[test]
fn empty_scoped_map_absent_calls_have_resolved_error_contracts() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function empty():int{const spare=new Map<string,Holder>();const found=spare.get('none');if(found!==undefined)return found.run();return 0;}function main():void{try{console.log(empty());throw 7;}catch(error){console.log(error);}}");
 support::execute(&source,"0\n7\n",None);
}

#[test]
fn empty_scoped_map_later_insertions_keep_concrete_environments() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function apply(action:ReadOnly<Function<int>>):int{return action();}function main():void{const spare=new Map<string,Holder>();{console.log('before',spare.size);}const values=[7];spare.set('value',{run:():int=>values[0]});{const found=spare.get('value');if(found!==undefined)console.log('value',found.run());}spare.clear();values[0]=9;console.log('parameter',apply(():int=>values[0]));}");
 support::execute(&source,"before 0\nvalue 7\nparameter 9\n",None);
}

#[test]
fn empty_scoped_record_map_observations_move_and_iteration() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const spare=new Map<string,Holder>();{const found=spare.get('none');if(found!==undefined)console.log(found.run());console.log('missing',found===undefined);}console.log('empty',spare.size,spare.has('none'),spare.delete('none'));for(const [key,holder] of spare){console.log(key,holder.run());}const moved=spare;console.log('moved',moved.size);moved.clear();}");
 support::execute(&source,"missing true\nempty 0 false false\nmoved 0\n",None);
}

#[test]
fn empty_scoped_record_map_nested_and_optional_callback_fields() {
 let source=support::format_round_trip("type Inner={run:ReadOnly<Function<int>>;};type Holder={inner:Inner;optional?:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;marker:int;};function main():void{const box:Box={handlers:new Map<string,Holder>(),marker:23};console.log(box.handlers.size,box.marker);box.handlers.clear();}");
 support::execute(&source,"0 23\n",None);
}

#[test]
fn empty_scoped_record_map_has_a_native_layout_without_insertions() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const spare=new Map<string,Holder>();console.log(spare.size);spare.clear();}");
 support::execute(&source,"0\n",None);
}

#[test]
fn cleanup_conditional_refill_retains_callback_until_explicit_clear() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let handlers=new Map<string,Holder>();while(turn<2){handlers.set('old',{run:():int=>old[0]});turn+=1;try{failIf(false);}catch(error){console.log('caught');}finally{handlers.clear();if(turn===2)handlers.set('next',{run:():int=>next[0]});}}{const saved=handlers.get('next');if(saved!==undefined)console.log('saved',saved.run());}console.log('size',handlers.size);handlers.clear();next[0]=9;console.log('after',next[0]);}");
 verify_disposal(&source,2,"drop 0\ndrop 0\nsaved 5\nsize 1\ndrop 1\nafter 9\n");
 let conflict=source.replace("console.log('size', handlers.size);", "next[0]=9;console.log('size',handlers.size);");
 assert_ne!(conflict,source);
 assert_eq!(tsrust_compiler::check(&conflict).err().expect("retained callback protects capture").code,"capture-conflict");
}

#[test]
fn nested_catch_pair_preserves_loop_control_and_cleanup() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};type Pair={left:Map<string,Holder>;right:Map<string,Holder>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let box:Pair={left:new Map<string,Holder>(),right:new Map<string,Holder>()};while(turn<3){box.left.set('old',{run:():int=>old[0]});box.right.set('next',{run:():int=>next[0]});turn+=1;try{try{failIf(turn!==2);}catch(error){if(turn===1)continue;break;}finally{console.log('inner',turn);}}catch(error){console.log('outer caught');}finally{console.log('cleanup',turn,box.left.size,box.right.size);box.left.clear();box.right.clear();old[0]=9;next[0]=11;}}console.log('after',box.left.size,box.right.size,old[0],next[0]);}");
 verify_disposal(&source,2,"inner 1\ncleanup 1 1 1\ndrop 0\ndrop 1\ninner 2\ncleanup 2 1 1\ndrop 0\ndrop 1\ninner 3\ncleanup 3 1 1\ndrop 0\ndrop 1\nafter 0 0 9 11\n");
 let overridden=source.replace("next[0] = 11;", "next[0]=11;if(turn===1)break;");
 assert_ne!(overridden,source);
 verify_disposal(&support::format_round_trip(&overridden),2,"inner 1\ncleanup 1 1 1\ndrop 0\ndrop 1\nafter 0 0 9 11\n");

 let rethrown=source.replace("if (turn === 1) continue;", "if(turn===1)throw 9;");
 assert_ne!(rethrown,source);
 verify_disposal(&support::format_round_trip(&rethrown),2,"inner 1\nouter caught\ncleanup 1 1 1\ndrop 0\ndrop 1\ninner 2\ncleanup 2 1 1\ndrop 0\ndrop 1\ninner 3\ncleanup 3 1 1\ndrop 0\ndrop 1\nafter 0 0 9 11\n");

}

#[test]
fn paired_record_map_loop_releases_both_cleanup_captures() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};type Pair={left:Map<string,Holder>;right:Map<string,Holder>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let box:Pair={left:new Map<string,Holder>(),right:new Map<string,Holder>()};while(turn<3){box.left.set('old',{run:():int=>old[0]});box.right.set('next',{run:():int=>next[0]});turn+=1;try{failIf(turn!==2);}catch(error){if(turn===1)continue;break;}finally{console.log('cleanup',turn,box.left.size,box.right.size);box.left.clear();box.right.clear();old[0]=9;next[0]=11;}}console.log('after',box.left.size,box.right.size,old[0],next[0]);}");
 verify_disposal(&source,2,"cleanup 1 1 1\ndrop 0\ndrop 1\ncleanup 2 1 1\ndrop 0\ndrop 1\ncleanup 3 1 1\ndrop 0\ndrop 1\nafter 0 0 9 11\n");
 let mut roots=source.clone();
 let start=roots.find("let box:").unwrap();let end=start+roots[start..].find(';').unwrap()+1;
 roots.replace_range(start..end,"let left=new Map<string,Holder>();let right=new Map<string,Holder>();");
 let roots=roots.replace("box.left","left").replace("box.right","right");
 assert_ne!(roots,source);
 verify_disposal(&support::format_round_trip(&roots),2,"cleanup 1 1 1\ndrop 0\ndrop 1\ncleanup 2 1 1\ndrop 0\ndrop 1\ncleanup 3 1 1\ndrop 0\ndrop 1\nafter 0 0 9 11\n");
 verify_disposal(&support::format_round_trip(&source.replace("turn < 3", "turn < 0")),2,"after 0 0 3 5\n");

 let spare=roots.replace("let left=new Map<string,Holder>();", "let spare=new Map<string,int>();let left=new Map<string,Holder>();");
 assert_ne!(spare,roots);
 verify_disposal(&support::format_round_trip(&spare),2,"cleanup 1 1 1\ndrop 0\ndrop 1\ncleanup 2 1 1\ndrop 0\ndrop 1\ncleanup 3 1 1\ndrop 0\ndrop 1\nafter 0 0 9 11\n");

}

#[test]
fn catch_cleanup_clear_releases_captures_before_cleanup_mutation() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let handlers=new Map<string,Holder>();while(turn<3){handlers.set('old',{run:():int=>old[0]});turn+=1;try{failIf(turn!==2);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('normal',turn);}catch(error){handlers.clear();handlers.set('next',{run:():int=>next[0]});if(turn===1)continue;break;}finally{console.log('cleanup',turn,handlers.size);handlers.clear();old[0]=9;next[0]=11;}console.log('fallthrough',turn);}old[0]=13;next[0]=17;console.log('after',handlers.size,old[0],next[0]);}");
 verify_disposal(&source,3,"drop 0\ncleanup 1 1\ndrop 2\ndrop 0\nnormal 2\ncleanup 2 1\ndrop 1\nfallthrough 2\ndrop 0\ncleanup 3 1\ndrop 2\nafter 0 13 17\n");
 let field=source.replace("function main()", "type Box={handlers:Map<string,Holder>;marker:int;};function main()")
  .replace("let handlers = new Map<string, Holder>();", "let box:Box={handlers:new Map<string,Holder>(),marker:23};")
  .replace("handlers.set", "box.handlers.set").replace("handlers.clear", "box.handlers.clear").replace("handlers.size", "box.handlers.size");
 assert_ne!(field,source);
 verify_disposal(&support::format_round_trip(&field),3,"drop 0\ncleanup 1 1\ndrop 2\ndrop 0\nnormal 2\ncleanup 2 1\ndrop 1\nfallthrough 2\ndrop 0\ncleanup 3 1\ndrop 2\nafter 0 13 17\n");

}

#[test]
fn outer_map_catch_loop_releases_iteration_captures() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let handlers=new Map<string,Holder>();while(turn<3){handlers.set('old',{run:():int=>old[0]});turn+=1;try{failIf(turn!==2);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('normal',turn);}catch(error){handlers.clear();handlers.set('next',{run:():int=>next[0]});if(turn===1)continue;break;}finally{console.log('cleanup',turn,handlers.size);handlers.clear();}old[0]=9;next[0]=11;console.log('fallthrough',turn);}old[0]=13;next[0]=17;console.log('after',handlers.size,old[0],next[0]);}");
 verify_disposal(&source,3,"drop 0\ncleanup 1 1\ndrop 2\ndrop 0\nnormal 2\ncleanup 2 1\ndrop 1\nfallthrough 2\ndrop 0\ncleanup 3 1\ndrop 2\nafter 0 13 17\n"); let zero=source.replace("turn < 3", "turn < 0");
 verify_disposal(&support::format_round_trip(&zero),3,"after 0 13 17\n");
 let field=source.replace("function failIf", "type Box={handlers:Map<string,Holder>;marker:int;};function failIf")
 .replace("let handlers = new Map<string, Holder>();", "const box:Box={handlers:new Map<string,Holder>(),marker:23};")
 .replace("handlers.set", "box.handlers.set").replace("handlers.clear", "box.handlers.clear").replace("handlers.size", "box.handlers.size");
 assert_ne!(field,source);
 verify_disposal(&support::format_round_trip(&field),3,"drop 0\ncleanup 1 1\ndrop 2\ndrop 0\nnormal 2\ncleanup 2 1\ndrop 1\nfallthrough 2\ndrop 0\ncleanup 3 1\ndrop 2\nafter 0 13 17\n");

}

#[test]
fn nested_handler_loop_exits_preserve_cleanup_and_capture_release() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;while(turn<3){let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});turn+=1;try{try{failIf(turn!==2);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('normal',turn);}catch(error){handlers.clear();handlers.set('next',{run:():int=>next[0]});if(turn===1)continue;break;}finally{console.log('inner',turn,handlers.size);handlers.clear();}}finally{handlers.clear();old[0]=9;next[0]=11;console.log('outer',turn,handlers.size);}console.log('fallthrough',turn);}old[0]=13;next[0]=17;console.log('after',old[0],next[0]);}");
 verify_disposal(&source,3,"drop 0\ninner 1 1\ndrop 2\nouter 1 0\ndrop 0\nnormal 2\ninner 2 1\ndrop 1\nouter 2 0\nfallthrough 2\ndrop 0\ninner 3 1\ndrop 2\nouter 3 0\nafter 13 17\n"); let overridden=source.replace("console.log('outer', turn, handlers.size);", "console.log('outer',turn,handlers.size);if(turn===1)break;");
 verify_disposal(&support::format_round_trip(&overridden),3,"drop 0\ninner 1 1\ndrop 2\nouter 1 0\nafter 13 17\n");
 let outer_owner=source.replace("while (turn < 3) {\n        let handlers = new Map<string, Holder>();", "let handlers=new Map<string,Holder>();while(turn<3){");
 assert_ne!(outer_owner,source,"outer owner witness must move initialization out of loop");
 verify_disposal(&support::format_round_trip(&outer_owner),3,"drop 0\ninner 1 1\ndrop 2\nouter 1 0\ndrop 0\nnormal 2\ninner 2 1\ndrop 1\nouter 2 0\nfallthrough 2\ndrop 0\ninner 3 1\ndrop 2\nouter 3 0\nafter 13 17\n");

}

#[test]
fn nested_handler_return_preserves_payload_and_cleanup_order() {
 let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function result(value:int):int{console.log('evaluated',value);return value;}function scenario(fail:boolean):int{const old=[3];const next=[5];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});try{try{failIf(fail);handlers.clear();handlers.set('next',{run:():int=>next[0]});return result(23);}catch(error){handlers.clear();handlers.set('next',{run:():int=>next[0]});return result(29);}finally{console.log('inner',handlers.size);handlers.clear();}}finally{console.log('outer',handlers.size);handlers.clear();}}function main():void{console.log('result',scenario(false));console.log('result',scenario(true));}");
 verify_disposal(&source,3,"drop 0\nevaluated 23\ninner 1\ndrop 1\nouter 0\nresult 23\ndrop 0\nevaluated 29\ninner 1\ndrop 2\nouter 0\nresult 29\n"); let unsafe_cleanup=source.replace("console.log('outer', handlers.size);", "old[0]=9;next[0]=11;console.log('outer',handlers.size);");
 assert_eq!(tsrust_compiler::check(&unsafe_cleanup).err().expect("fallible inner observation may precede clear").code,"capture-conflict");
 let released=source.replace("console.log('outer', handlers.size);", "handlers.clear();old[0]=9;next[0]=11;console.log('outer',handlers.size);");

 verify_disposal(&support::format_round_trip(&released),3,"drop 0\nevaluated 23\ninner 1\ndrop 1\nouter 0\nresult 23\ndrop 0\nevaluated 29\ninner 1\ndrop 2\nouter 0\nresult 29\n");

}

#[test]
fn nested_catch_preserves_map_owners_until_outer_cleanup() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function scenario(fail:boolean):void{const old=[3];const next=[5];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before');try{try{failIf(fail);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('normal');}catch(error){console.log('inner caught',error,handlers.size);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('handled');}}finally{console.log('outer',handlers.size);handlers.clear();console.log('outer cleared');}old[0]=9;next[0]=11;console.log('after');}function main():void{scenario(false);scenario(true);}");
    verify_disposal(&source,3,"before\ndrop 0\nnormal\nouter 1\ndrop 1\nouter cleared\nafter\nbefore\ninner caught 7 1\ndrop 0\nhandled\nouter 1\ndrop 2\nouter cleared\nafter\n");
    let rethrow = source.replace("console.log('handled');", "console.log('handled');throw 9;")
        .replace("scenario(true);", "try{scenario(true);}catch(error){console.log('caught',error);}");
    verify_disposal(&support::format_round_trip(&rethrow),3,"before\ndrop 0\nnormal\nouter 1\ndrop 1\nouter cleared\nafter\nbefore\ninner caught 7 1\ndrop 0\nhandled\nouter 1\ndrop 2\nouter cleared\ncaught 9\n");

    let cleanup = rethrow.replace("console.log('handled');throw 9;", "console.log('handled');throw 9;}finally{console.log('inner cleanup',handlers.size);handlers.clear();console.log('inner cleared');");
    verify_disposal(&support::format_round_trip(&cleanup),3,"before\ndrop 0\nnormal\ninner cleanup 1\ndrop 1\ninner cleared\nouter 0\nouter cleared\nafter\nbefore\ninner caught 7 1\ndrop 0\nhandled\ninner cleanup 1\ndrop 2\ninner cleared\nouter 0\nouter cleared\ncaught 9\n");
    let overriding = cleanup.replace("console.log('inner cleared');", "console.log('inner cleared');if(fail)throw 11;");
    verify_disposal(&support::format_round_trip(&overriding),3,"before\ndrop 0\nnormal\ninner cleanup 1\ndrop 1\ninner cleared\nouter 0\nouter cleared\nafter\nbefore\ninner caught 7 1\ndrop 0\nhandled\ninner cleanup 1\ndrop 2\ninner cleared\nouter 0\nouter cleared\ncaught 11\n");

}

#[test]
fn nested_finally_preserves_map_owners_and_cleanup_order() {
    let source=support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function scenario(fail:boolean):void{const old=[3];const next=[5];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before');try{try{failIf(fail);handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('normal');}finally{console.log('inner',handlers.size);handlers.clear();console.log('inner cleared');}}finally{console.log('outer',handlers.size);handlers.clear();console.log('outer cleared');}old[0]=9;next[0]=11;console.log('after');}function main():void{scenario(false);try{scenario(true);}catch(error){console.log('caught',error);}}");
    verify_disposal(&source,2,"before\ndrop 0\nnormal\ninner 1\ndrop 1\ninner cleared\nouter 0\nouter cleared\nafter\nbefore\ninner 1\ndrop 0\ninner cleared\nouter 0\nouter cleared\ncaught 7\n");
    let overriding=source.replace("console.log('inner cleared');", "console.log('inner cleared'); if(!fail)throw 9;")
        .replace("scenario(false);", "try{scenario(false);}catch(error){console.log('caught',error);}");
    verify_disposal(&support::format_round_trip(&overriding),2,"before\ndrop 0\nnormal\ninner 1\ndrop 1\ninner cleared\nouter 0\nouter cleared\ncaught 9\nbefore\ninner 1\ndrop 0\ninner cleared\nouter 0\nouter cleared\ncaught 7\n");
    let field=source.replace("function scenario", "type Box={handlers:Map<string,Holder>;marker:int;};function scenario")
        .replace("console.log('before');", "const box:Box={handlers:handlers,marker:23};console.log('before');")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("handlers.set('next'", "box.handlers.set('next'")
        .replace("handlers.size", "box.handlers.size")
        .replace("old[0] = 9;", "if(box.marker!==23)throw 99;old[0]=9;");
    verify_disposal(&support::format_round_trip(&field),2,"before\ndrop 0\nnormal\ninner 1\ndrop 1\ninner cleared\nouter 0\nouter cleared\nafter\nbefore\ninner 1\ndrop 0\ninner cleared\nouter 0\nouter cleared\ncaught 7\n");
    let returned=source.replace("scenario(fail: boolean): void", "scenario(fail: boolean): int")
        .replace("console.log('normal');", "console.log('normal');if(!fail)return 23;")
        .replace("console.log('after');", "console.log('after');return 29;")
        .replace("scenario(false);", "console.log('result',scenario(false));");
    verify_disposal(&support::format_round_trip(&returned),2,"before\ndrop 0\nnormal\ninner 1\ndrop 1\ninner cleared\nouter 0\nouter cleared\nresult 23\nbefore\ninner 1\ndrop 0\ninner cleared\nouter 0\nouter cleared\ncaught 7\n");

}

#[test]
fn nested_empty_map_initializer_adapts_record_callback_carriers() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;marker:int;};function main():void{const old=[3];const next=[5];const box:Box={handlers:new Map<string,Holder>(),marker:23};box.handlers.set('old',{run:():int=>old[0]});box.handlers.set('next',{run:():int=>next[0]});console.log(box.handlers.size,box.marker);box.handlers.clear();old[0]=13;next[0]=17;console.log(box.handlers.size,old[0],next[0]);}";
    support::execute(source,"2 23\n0 13 17\n",None);
    let ordered=source.replace("type Box={handlers:Map<string,Holder>;marker:int;};", "type Box={handlers:Map<string,Holder>;marker:int;label?:string;};function marker():int{console.log('initialized');return 23;}")
        .replace("handlers:new Map<string,Holder>(),marker:23", "marker:marker(),handlers:new Map<string,Holder>()")
        .replace("console.log(box.handlers.size,box.marker);", "console.log(box.handlers.size,box.marker,box.label===undefined);");
    support::execute(&ordered,"initialized\n2 23 true\n0 13 17\n",None);

}

#[test]
fn outer_record_map_cleanup_releases_captures_across_iterations() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;marker:int;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;const box:Box={handlers:new Map<string,Holder>(),marker:23};while(turn<3){box.handlers.set('old',{run:():int=>old[0]});console.log('before',turn);turn+=1;try{failIf(false);box.handlers.clear();box.handlers.set('next',{run:():int=>next[0]});if(turn===1)continue;if(turn===3)break;}finally{console.log('cleanup',box.handlers.size);box.handlers.clear();console.log('cleared');}old[0]=9;next[0]=11;console.log('inside',old[0],next[0],box.marker);}old[0]=13;next[0]=17;console.log('after',box.handlers.size,old[0],next[0],box.marker);}");
    verify_disposal(&source,2,"before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\ninside 9 11 23\nbefore 2\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 0 13 17 23\n");
    let zero=source.replace("turn < 3","turn < 0");
    verify_disposal(&support::format_round_trip(&zero),2,"after 0 13 17 23\n");
    let sibling=source.replace("const box: Box", "let box: Box")
        .replace("old[0] = 9;", "box.marker += 1; old[0]=9;")
        .replace("console.log('inside', old[0], next[0], box.marker);", "console.log('inside',old[0],next[0],box.marker,Object.keys(box)[0]);")
        .replace("console.log('after', box.handlers.size, old[0], next[0], box.marker);", "console.log('after',box.handlers.size,old[0],next[0],box.marker,Object.keys(box)[1]);");
    verify_disposal(&support::format_round_trip(&sibling),2,"before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\ninside 9 11 24 handlers\nbefore 2\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 0 13 17 24 marker\n");

}

#[test]
fn populated_outer_map_preserves_first_iteration_and_cleanup_disposal() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let handlers=new Map<string,Holder>();handlers.set('initial',{run:():int=>old[0]});while(turn<2){console.log('before',turn,handlers.size);turn+=1;try{failIf(false);handlers.clear();handlers.set('next',{run:():int=>next[0]});if(turn===1)continue;break;}finally{console.log('cleanup',handlers.size);handlers.clear();console.log('cleared');}}handlers.clear();old[0]=13;next[0]=17;console.log('after',handlers.size,old[0],next[0]);}");
    verify_disposal(&source,2,"before 0 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1 0\ncleanup 1\ndrop 1\ncleared\nafter 0 13 17\n");
}

#[test]
fn retained_outer_map_entries_survive_iterations_until_explicit_clear() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function main():void{const old=[3];const next=[5];let turn=0;let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});while(turn<2){handlers.set('next',{run:():int=>next[0]});console.log('iteration',turn,handlers.size);turn+=1;}console.log('retained',handlers.size);handlers.clear();old[0]=13;next[0]=17;console.log('after',handlers.size,old[0],next[0]);}");
    verify_disposal(&source,2,"iteration 0 2\ndrop 1\niteration 1 2\nretained 2\ndrop 0\ndrop 1\nafter 0 13 17\n");
}

#[test]
fn outer_map_survives_loop_exit_cleanup_and_capture_renewal() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;let handlers=new Map<string,Holder>();while(turn<3){handlers.set('old',{run:():int=>old[0]});console.log('before',turn);turn+=1;try{failIf(false);handlers.clear();handlers.set('next',{run:():int=>next[0]});if(turn===1)continue;if(turn===3)break;}finally{console.log('cleanup',handlers.size);handlers.clear();console.log('cleared');}old[0]=9;next[0]=11;console.log('inside',old[0],next[0]);}old[0]=13;next[0]=17;console.log('after',handlers.size,old[0],next[0]);}");
    verify_disposal(&source, 2, "before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\ninside 9 11\nbefore 2\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 0 13 17\n");
    let observed_cleanup=source.replace("console.log('cleared');", "console.log('cleared',handlers.size);");
    verify_disposal(&support::format_round_trip(&observed_cleanup),2,"before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared 0\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared 0\ninside 9 11\nbefore 2\ndrop 0\ncleanup 1\ndrop 1\ncleared 0\nafter 0 13 17\n");
    let zero = source.replace("turn < 3", "turn < 0");
    verify_disposal(&support::format_round_trip(&zero),2,"after 0 13 17\n");

}

#[test]
fn loop_exits_through_finally_release_renewed_map_captures() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function main():void{const old=[3];const next=[5];let turn=0;while(turn<2){let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before',turn);try{failIf(false);handlers.clear();handlers.set('next',{run:():int=>next[0]});if(turn===0){turn+=1;continue;}break;}finally{console.log('cleanup',handlers.size);handlers.clear();console.log('cleared');}}old[0]=9;next[0]=11;console.log('after',old[0],next[0]);}");
    verify_disposal(&source, 2, "before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 9 11\n");
    let fallthrough = source.replace("turn < 2", "turn < 3")
        .replace("console.log('before', turn);", "console.log('before', turn); turn += 1;")
        .replace("if (turn === 0) {", "if (turn === 1) {")
        .replace("turn += 1;\n                continue;", "continue;")
        .replace("            break;", "            if(turn === 3) break;")
        .replace("console.log('cleared');\n        }", "console.log('cleared');\n        } old[0]=9; next[0]=11; console.log('inside',old[0],next[0]);");
    verify_disposal(&support::format_round_trip(&fallthrough), 2, "before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\ninside 9 11\nbefore 2\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 9 11\n");

    let overriding = source.replace("console.log('cleared');", "console.log('cleared'); break;");
    verify_disposal(&support::format_round_trip(&overriding), 2, "before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 9 11\n");
    let field = fallthrough.replace("function main", "type Box={handlers:Map<string,Holder>;marker:int;};function main")
        .replace("console.log('before', turn);", "const box:Box={handlers:handlers,marker:23};console.log('before',turn);")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("handlers.set('next'", "box.handlers.set('next'")
        .replace("handlers.size", "box.handlers.size")
        .replace("console.log('inside'", "if(box.marker!==23)throw 99; console.log('inside'");
    verify_disposal(&support::format_round_trip(&field), 2, "before 0\ndrop 0\ncleanup 1\ndrop 1\ncleared\nbefore 1\ndrop 0\ncleanup 1\ndrop 1\ncleared\ninside 9 11\nbefore 2\ndrop 0\ncleanup 1\ndrop 1\ncleared\nafter 9 11\n");

}

#[test]
fn returning_through_finally_preserves_renewed_map_until_cleanup() {
    let source = support::format_round_trip("type Holder={run:ReadOnly<Function<int>>;};function failIf(fail:boolean):void{if(fail)throw 7;}function scenario(early:boolean):int{const old=[3];const next=[5];let handlers=new Map<string,Holder>();handlers.set('old',{run:():int=>old[0]});console.log('before');try{failIf(false);if(early)return 23;handlers.clear();handlers.set('next',{run:():int=>next[0]});console.log('return');return 29;}finally{console.log('cleanup',handlers.size);handlers.clear();console.log('cleared');}}function main():void{console.log('result',scenario(true));console.log('result',scenario(false));}");
    verify_disposal(&source, 2, "before\ncleanup 1\ndrop 0\ncleared\nresult 23\nbefore\ndrop 0\nreturn\ncleanup 1\ndrop 1\ncleared\nresult 29\n");
    let fallthrough = source.replace("return 29;", "").replace("\n}\n\nfunction main", "\n    old[0] = 9; next[0] = 11; return 29;\n}\n\nfunction main");
    assert_ne!(fallthrough, source);
    verify_disposal(&support::format_round_trip(&fallthrough), 2, "before\ncleanup 1\ndrop 0\ncleared\nresult 23\nbefore\ndrop 0\nreturn\ncleanup 1\ndrop 1\ncleared\nresult 29\n");
    let overriding = source.replace("console.log('cleared');", "console.log('cleared'); return 31;");
    verify_disposal(&support::format_round_trip(&overriding), 2, "before\ncleanup 1\ndrop 0\ncleared\nresult 31\nbefore\ndrop 0\nreturn\ncleanup 1\ndrop 1\ncleared\nresult 31\n");
    let field = fallthrough.replace("function scenario", "type Box={handlers:Map<string,Holder>;marker:int;};function scenario")
        .replace("console.log('before');", "const box:Box={handlers:handlers,marker:23};console.log('before');")
        .replace("handlers.clear()", "box.handlers.clear()")
        .replace("handlers.set('next'", "box.handlers.set('next'")
        .replace("handlers.size", "box.handlers.size")
        .replace("old[0] = 9;", "if(box.marker!==23)throw 99; old[0]=9;");
    verify_disposal(&support::format_round_trip(&field), 2, "before\ncleanup 1\ndrop 0\ncleared\nresult 23\nbefore\ndrop 0\nreturn\ncleanup 1\ndrop 1\ncleared\nresult 29\n");

}

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
