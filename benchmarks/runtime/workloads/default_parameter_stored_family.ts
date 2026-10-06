type int = number;
function clone<T>(value: T): T { return value; }
type Trace = { calls: int; defaults: int; };
type Holder = { run: (trace: Trace, input?: int) => int; };
function first(trace: Trace, input: int = defaultValue(trace, 10)): int { trace.calls = trace.calls + 1; return input + 1; }
function second(trace: Trace, input: int = defaultValue(trace, 100)): int { trace.calls = trace.calls + 1; return input + 2; }
function defaultValue(trace: Trace, value: int): int { trace.defaults = trace.defaults + 1; return value; }
function forward(trace: Trace, holder: Holder, value?: int): int { return holder.run(trace, value); }
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = []; const trace: Trace = { calls: 0, defaults: 0 };
    let i = 0; while (i < n) {
        const mode = (i + seed) % 4; const value = (i * 13 + seed) % 97;
        const beforeCalls = trace.calls; const beforeDefaults = trace.defaults;
        const holder: Holder = { run: (i + seed) % 2 === 0 ? first : second };
        const observed = mode === 0 ? forward(trace, holder) : mode === 1 ? forward(trace, holder, undefined) : forward(trace, holder, value);
        output.push([observed]); output.push([trace.calls - beforeCalls, trace.defaults - beforeDefaults]);
        i = i + 1;
    } return output;
}
function main() { const result = workload(3, 0, "abc"); }
