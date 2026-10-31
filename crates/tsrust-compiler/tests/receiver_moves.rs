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
    // A closure that captures something else does not prevent the move.
    let other = emitted("function f(values:int[]):int{const copy=values.slice(0);const count=():int=>values.length;return copy.concat().length+count();}function main(){console.log(f([1,2]));}");
    assert!(!other.contains("concat_array"), "an unrelated capture keeps the move");
    support::execute(&support::format_round_trip("function f(values:int[]):int{const copy=values.slice(0);const count=():int=>values.length;return copy.concat().length+count();}function main(){console.log(f([1,2]));}"), "4\n", None);
    support::execute(&support::format_round_trip(source), "2 b 3 4,3,2\n", None);
}

#[test]
fn live_captured_rebindable_or_loop_carried_receivers_keep_copying() {
    for source in [
        // Read again after the copy.
        "function f(values:int[]):int{const copy=values.slice(0);const other=copy.concat();return copy.length+other.length;}function main(){console.log(f([1,2]));}",
        // Declared outside the loop that copies it.
        "function f(values:int[]):int{const copy=values.slice(0);let total=0;for(let i=0;i<2;i++){total+=copy.concat().length;}return total;}function main(){console.log(f([1,2]));}",
        // A closure captures the local itself.
        "function f(values:int[]):int{const copy=values.slice(0);const count=():int=>copy.length;return copy.concat().length+count();}function main(){console.log(f([1,2]));}",
        // A let binding may be reassigned.
        "function f(values:int[]):int{let copy=values.slice(0);return copy.concat().length;}function main(){console.log(f([1,2]));}",
    ] {
        let rust = emitted(source);
        assert!(rust.contains("concat_array"), "receiver must still be copied: {source}");
        support::execute(&support::format_round_trip(source), if source.contains("let copy") { "2\n" } else { "4\n" }, None);
    }
}

#[test]
fn dead_receivers_of_slice_and_to_sorted_keep_exact_results() {
    let source = "function byLength(a:ReadOnly<string[]>,b:ReadOnly<string[]>):int{return b.length-a.length;}function tail(rows:string[][],from:int,to:int):string[][]{const copy=rows.slice(0);return copy.slice(from,to);}function sorted(rows:string[][]):string[][]{const copy=rows.slice(0);return copy.toSorted(byLength);}function show(rows:string[][]):string{return rows.map((row:ReadOnly<string[]>):string=>row.join(\"+\")).join(\",\");}function main(){const rows:string[][]=[[\"a\"],[\"é\",\"b\",\"c\"],[\"d\",\"e\"]];console.log(show(tail(rows,1,3)),show(tail(rows,-2,10)),show(tail(rows,5,1)).length,show(sorted(rows)),show(rows));}";
    let rust = emitted(source);
    assert!(rust.contains("slice_array.drain(..slice_start)"), "slice() of a dead local trims its storage");
    assert!(rust.contains("let mut sorting_copy = value"), "toSorted() of a dead local sorts it in place");
    support::execute(&support::format_round_trip(source), "é+b+c,d+e é+b+c,d+e 0 é+b+c,d+e,a a,é+b+c,d+e\n", None);
}

#[test]
fn string_conversion_of_a_dead_string_local_moves_it() {
    let source = "function main(){const rows:string[]=[];let i=0;while(i<3){const pair:[int,string]=[i,`n${i}`];const [doubled,label]=pair;rows.push(String(label));const kept=`k${doubled}`;rows.push(String(kept));rows.push(kept);i+=1;}console.log(rows.join(\",\"));}";
    let rust = emitted(source);
    // `label` is moved; `kept` is read again, so its conversion still copies.
    assert_eq!(rust.matches("(*builtin_argument_0).clone()").count(), 1, "{rust}");
    support::execute(&support::format_round_trip(source), "n0,k0,k0,n1,k1,k1,n2,k2,k2\n", None);
}

#[test]
fn string_conversion_of_a_dead_record_field_moves_the_field() {
    let source = "type Pair={name:string;tag:string;n:int;};function main(){const rows:string[]=[];let i=0;while(i<3){const moved:Pair={name:`m${i}`,tag:\"t\",n:i};rows.push(String(moved.name));const kept:Pair={name:`k${i}`,tag:\"u\",n:i};rows.push(String(kept.name));rows.push(kept.tag);i+=1;}console.log(rows.join(\",\"));}";
    let rust = emitted(source);
    // `moved` is read once, so its field moves; `kept` is read again, so its field is copied.
    assert_eq!(rust.matches("(*builtin_argument_0).clone()").count(), 1, "{rust}");
    support::execute(&support::format_round_trip(source), "m0,k0,u,m1,k1,u,m2,k2,u\n", None);
}
