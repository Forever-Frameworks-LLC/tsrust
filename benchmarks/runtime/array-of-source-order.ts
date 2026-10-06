type int = number;
function item(trace: int[], value: int): int { trace.push(value); return value; }
function main() {
    const trace: int[] = [];
    const values = Array.of(item(trace, 1), item(trace, 2), item(trace, 3), item(trace, 4), item(trace, 5));
    console.log(trace.join(",")); console.log(values.join(","));
    const empty: int[] = Array.of(); console.log(empty.length);
    const one = Array.of(item(trace, 9)); console.log(one.join(",")); console.log(trace.join(","));
    const owned = Array.of("é".repeat(2), "😀".repeat(1)); console.log(owned.join("|"));
}
