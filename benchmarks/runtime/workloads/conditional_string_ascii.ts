type int = number;
type Trace = { predicates: int; left: int; right: int; };
function workload(n: int, seed: int, input: string): string[] {
    const output: string[] = []; const trace: Trace = { predicates: 0, left: 0, right: 0 }; let i = 0;
    while (i < n) {
        const selected: string = predicate(trace, (i + seed) % 2 === 0) ? leftValue(trace, input) : rightValue(trace, input);
        output.push(selected);
        i = i + 1;
    }
    output.push("P".repeat(trace.predicates)); output.push("L".repeat(trace.left)); output.push("R".repeat(trace.right));
    return output;
}
function predicate(trace: Trace, flag: boolean): boolean { trace.predicates = trace.predicates + 1; return flag; }
function leftValue(trace: Trace, input: string): string { trace.left = trace.left + 1; return "L" + input.repeat(1); }
function rightValue(trace: Trace, input: string): string { trace.right = trace.right + 1; return "R" + input.repeat(2); }
