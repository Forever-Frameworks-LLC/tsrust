type int = number;
type Trace = { calls: int; };
function workload(n: int, seed: int): boolean[][] {
    const output: boolean[][] = []; const trace: Trace = { calls: 0 }; let i = 0;
    while (i < n) {
        const value = i + seed; let target: boolean | null | undefined = makeLeft(value);
        const before = trace.calls;
        const selected = (target ??= fallback(trace, value));
        const delta = trace.calls - before;
        output.push([selected, target, delta === 0, delta === 1]);
        i = i + 1;
    }
    return output;
}
function makeLeft(value: int): boolean | null | undefined {
    const tag = value % 5; if (tag === 0) { return null; } if (tag === 1) { return undefined; }
    return value % 3 !== 0;
}
function fallback(trace: Trace, value: int): boolean {
    trace.calls = trace.calls + 1;
    return value % 2 === 0;
}
function main() { const result = workload(2, 7); }
