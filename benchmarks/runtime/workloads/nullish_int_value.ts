type int = number;
type Trace = { calls: int; };
function workload(n: int, seed: int): int[][] {
    const output: int[][] = []; const trace: Trace = { calls: 0 }; let i = 0;
    while (i < n) {
        const value = i + seed; const left: int | null | undefined = makeLeft(value);
        const before = trace.calls;
        const selected = left ?? fallback(trace, value);
        const delta = trace.calls - before;
        output.push([selected, delta]);
        i = i + 1;
    }
    output.push([trace.calls]);
    return output;
}
function makeLeft(value: int): int | null | undefined {
    const tag = value % 5; if (tag === 0) { return null; } if (tag === 1) { return undefined; }
    return value % 4;
}
function fallback(trace: Trace, value: int): int {
    trace.calls = trace.calls + 1;
    return value + 100;
}
function main() { const result = workload(2, 7); }
