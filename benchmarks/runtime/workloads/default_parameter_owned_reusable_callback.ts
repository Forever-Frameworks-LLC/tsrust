type int = number;
function clone<T>(value: T): T { return value; }
type Trace = { calls: int; defaults: int; };
function fallback(trace: Trace, base: int): (input: int, trace: Trace) => int {
    trace.defaults = trace.defaults + 1; let state = base + 100;
    return (callValue: int, callTrace: Trace): int => { state = state + 1; callTrace.calls = callTrace.calls + 1; return callValue + state; };
}
function apply(trace: Trace, base: int, action: (input: int, trace: Trace) => int = fallback(trace, base)): int[] { const first = action(base, trace); const second = action(base + 1, trace); return [first, second]; }
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = []; const trace: Trace = { calls: 0, defaults: 0 };
    let i = 0; while (i < n) {
        const mode = (i + seed) % 4; const value = (i * 13 + seed) % 97;
        const beforeCalls = trace.calls; const beforeDefaults = trace.defaults;
        let state = value;
        const supplied: (input: int, trace: Trace) => int = (callValue: int, callTrace: Trace): int => { state = state + 1; callTrace.calls = callTrace.calls + 1; return callValue + state; };
        const observed = mode === 0 ? apply(trace, value) : mode === 1 ? apply(trace, value, undefined) : apply(trace, value, supplied);
        output.push(observed); output.push([trace.calls - beforeCalls, trace.defaults - beforeDefaults]);
        i = i + 1;
    } return output;
}
function main() { const result = workload(3, 0, "abc"); }
