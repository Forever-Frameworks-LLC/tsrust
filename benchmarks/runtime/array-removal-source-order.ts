type int = number;
function fresh(trace: int[]): int[] { trace.push(1); return [4, 5]; }
function main() {
    const trace: int[] = [];
    const fromFresh = fresh(trace).shift();
    console.log(String(fromFresh)); console.log(trace[0]);
    const values: string[] = ["é".repeat(1), "x".repeat(1)];
    const first = values.shift();
    if (first !== undefined) { console.log(first.concat("!")); }
    console.log(values.join("|"));
    const last = values.pop();
    if (last !== undefined) { console.log(last); }
    console.log(typeof values.pop()); console.log(values.length);
}
