type int = number;
function workload(n: int, seed: int, input: string): string[][] {
    const values: string[] = [clone(input), "  Pat  ", clone(input), "", "é", "😀", "e\u0301", "a\u0000b"];
    const output: string[][] = []; let trace = seed; let visits = 0; let i = 0;
    while (i < n) {
        const selected: string[] = values.filter((value: string, index: int): boolean => {
            trace = (trace * 131 + visits + i + seed) % 1000000007; visits = visits + 1;
            return value !== "" && (index + i) % 3 !== 0;
        });
        output.push(selected); i = i + 1;
    }
    output.push(trace_row(trace, visits));
    return output;
}
function main() { const result = workload(2, 7, "  élán 😀 e\u0301  "); console.log(result.length); }
function trace_row(trace: int, visits: int): string[] {
    const row: string[] = ["trace"]; let word = trace; let i = 0;
    while (i < 31) {
        const bit = word % 2;
        if (bit === 0) { row.push("0"); } else { row.push("1"); }
        word = (word - bit) / 2; i = i + 1;
    }
    row.push("visits"); word = visits; i = 0;
    while (i < 16) {
        const bit = word % 2;
        if (bit === 0) { row.push("0"); } else { row.push("1"); }
        word = (word - bit) / 2; i = i + 1;
    }
    return row;
}
// JS strings are immutable; native clone may own an independent UTF8 copy.
function clone(value: string): string { return value; }
