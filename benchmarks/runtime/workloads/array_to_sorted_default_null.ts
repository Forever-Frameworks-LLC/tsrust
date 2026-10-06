type int = number;
function workload(n: int, seed: int): string[][] {
    const output: string[][] = [];
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0;
    while (i < n) {
        const count = sizes[(i + seed) % 5];
        const values: null[] = []; let j = 0;
        while (j < count) {
            values.push(null);
            j = j + 1;
        }
        const sorted = values.toSorted();
        output.push(describe(values));
        output.push(describe(sorted));
        i = i + 1;
    }
    return output;
}
function describe(values: null[]): string[] { const result: string[] = []; let at = 0; while (at < values.length) { result.push(String(values[at])); at = at + 1; } return result; }
function main() { const result = workload(2, 7); }
