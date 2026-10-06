type int = number;
type Trace = { bases: int; indices: int; args: int; };
type Child = { text: string; };
type Carrier = { child: Child | null | undefined; };
function workload(n: int, seed: int, input: string): string[][] {
    const output: string[][] = []; const trace: Trace = { bases: 0, indices: 0, args: 0 }; let i = 0;
    while (i < n) {
        const value = i + seed; const bb = trace.bases; const bi = trace.indices; const ba = trace.args;
        const carrier: Carrier | null | undefined = makeCarrier(trace, value, input);
        const selected = carrier?.child?.text.concat(nextArgument(trace, value, input));
        const db = trace.bases - bb; const di = trace.indices - bi; const da = trace.args - ba;
        if (selected === undefined) { output.push([]); } else { output.push([selected]); } output.push([String(db), String(di), String(da)]);
        i = i + 1;
    }
    return output;
}
function makeCarrier(trace: Trace, value: int, input: string): Carrier | null | undefined {
    trace.bases = trace.bases + 1; const tag = value % 5; if (tag === 0) { return null; } if (tag === 1) { return undefined; }
    const childTag = value % 15; if (childTag < 5) { return { child: null }; } if (childTag < 10) { return { child: undefined }; } return { child: { text: input.concat(":", String(value)) } };
}
function nextArgument(trace: Trace, value: int, input: string): string { trace.args = trace.args + 1; return value % 2 === 0 ? "|" : input.concat(":", String(value)); }
function main() { const result = workload(2, 7, "dynamic text"); }
