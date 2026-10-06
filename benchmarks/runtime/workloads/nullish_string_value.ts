type int = number;
type Trace = { calls: int; };
function workload(n: int, seed: int, input: string): string[][] {
    const output: string[][] = []; const trace: Trace = { calls: 0 }; let i = 0;
    while (i < n) {
        const value = i + seed; const left: string | null | undefined = makeLeft(value, input);
        const before = trace.calls;
        const selected = left ?? fallback(trace, value, input);
        const delta = trace.calls - before;
        output.push([selected, String(delta)]);
        i = i + 1;
    }
    output.push([String(trace.calls)]);
    return output;
}
function makeLeft(value: int, input: string): string | null | undefined {
    const tag = value % 5; if (tag === 0) { return null; } if (tag === 1) { return undefined; }
    return value % 3 === 0 ? "" : input.concat(":", String(value));
}
function fallback(trace: Trace, value: int, input: string): string {
    trace.calls = trace.calls + 1;
    return input.concat("|fallback:", String(value));
}
function main() { const result = workload(2, 7, "dynamic text"); }
