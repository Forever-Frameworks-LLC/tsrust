type int = number;
function workload(n: int, seed: int): string[] {
    const output: string[] = []; let i = 0;
    while (i < n) {
        const count = (i + seed) % 4;
        const values: int[][] = [];
        if (count > 0) { values.push([seed, 1]); }
        if (count > 1) { values.push([]); }
        if (count > 2) { values.push([seed + 2, 3]); }
        const index = (i * 5 + seed) % 9 - 4;
        const selected = values.at(index);
        output.push(typeof selected);
        if (selected === undefined) { output.push("undefined"); } else { output.push(describe(selected)); }
        output.push(String(values.length));
        if (values.length > 0) { output.push(describe(values[0])); } else { output.push("<absent>"); }
        if (values.length > 1) { output.push(describe(values[1])); } else { output.push("<absent>"); }
        if (values.length > 2) { output.push(describe(values[2])); } else { output.push("<absent>"); }
        i = i + 1;
    }
    return output;
}
function describe(value: int[]): string { return String(value.length).concat(":", value.join(",")); }
function main() { const result = workload(2, 7); }
