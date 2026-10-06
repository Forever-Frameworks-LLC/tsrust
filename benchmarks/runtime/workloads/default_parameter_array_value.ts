type int = number;
function clone<T>(value: T): T { return value; }
type Trace = { calls: int; defaults: int; };
function fallback(trace: Trace, base: int): int[] { trace.defaults = trace.defaults + 1; return [base, base + 1]; }
function apply(trace: Trace, base: int, values: int[] = fallback(trace, base)): int[] { trace.calls = trace.calls + 1; return values; }
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = []; const trace: Trace = { calls: 0, defaults: 0 };
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0; while (i < n) {
        const mode = (i + seed) % 4; const value = (i * 13 + seed) % 97;
        const beforeCalls = trace.calls; const beforeDefaults = trace.defaults;
        const size = sizes[(i * 3 + seed) % 5]; const supplied: int[] = []; let at = 0;
        while (at < size) { supplied.push(value + at); at = at + 1; }
        const observed = mode === 0 ? apply(trace, value) : mode === 1 ? apply(trace, value, undefined) : apply(trace, value, supplied);
        output.push(observed); output.push([trace.calls - beforeCalls, trace.defaults - beforeDefaults]);
        i = i + 1;
    } return output;
}
function main() { const result = workload(3, 0, "abc"); }
