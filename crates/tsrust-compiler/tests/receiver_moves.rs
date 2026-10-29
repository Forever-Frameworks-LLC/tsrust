mod support;

fn emitted(source: &str) -> String {
    tsrust_compiler::check(&support::format_round_trip(source)).unwrap().emit_rust()
}

#[test]
fn dead_const_receivers_move_instead_of_copying() {
    let source = "type Row={code:int;text:string;};function pick(rows:Row[]):Row[]{const head=rows.slice(0,2);return head.concat();}function flip(values:int[]):int[]{const copy=values.slice(1);return copy.toReversed();}function main(){const rows:Row[]=[{code:1,text:\"é\"},{code:2,text:\"b\"},{code:3,text:\"c\"}];const picked=pick(rows);console.log(picked.length,picked[1].text,rows.length,flip([1,2,3,4]).join(\",\"));}";
    let rust = emitted(source);
    assert!(!rust.contains("concat_array"), "a dead receiver of concat() is moved");
    assert!(rust.contains("reversed_array.reverse()"), "a dead receiver of toReversed() is reversed in place");
    support::execute(&support::format_round_trip(source), "2 b 3 4,3,2\n", None);
}

#[test]
fn live_captured_rebindable_or_loop_carried_receivers_keep_copying() {
    for source in [
        // Read again after the copy.
        "function f(values:int[]):int{const copy=values.slice(0);const other=copy.concat();return copy.length+other.length;}function main(){console.log(f([1,2]));}",
        // Declared outside the loop that copies it.
        "function f(values:int[]):int{const copy=values.slice(0);let total=0;for(let i=0;i<2;i++){total+=copy.concat().length;}return total;}function main(){console.log(f([1,2]));}",
        // A closure in the function could capture it.
        "function f(values:int[]):int{const copy=values.slice(0);const count=():int=>values.length;return copy.concat().length+count();}function main(){console.log(f([1,2]));}",
        // A let binding may be reassigned.
        "function f(values:int[]):int{let copy=values.slice(0);return copy.concat().length;}function main(){console.log(f([1,2]));}",
    ] {
        let rust = emitted(source);
        assert!(rust.contains("concat_array"), "receiver must still be copied: {source}");
        support::execute(&support::format_round_trip(source), if source.contains("let copy") { "2\n" } else { "4\n" }, None);
    }
}
