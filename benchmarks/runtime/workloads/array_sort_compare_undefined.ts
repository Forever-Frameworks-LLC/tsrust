type int = number;
function workload(n: int, seed: int): string[][] {
    const output: string[][] = [];
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0;
    while (i < n) {
        const count = sizes[(i + seed) % 5];
        const values: undefined[] = []; let j = 0;
        while (j < count) {
            values.push(undefined);
            j = j + 1;
        }
        values.sort(compare);
        output.push(describe(values));
        i = i + 1;
    }
    return output;
}
function compare(left: undefined, right: undefined): int { return 0; }
function describe(values: undefined[]): string[] { const result: string[] = []; let at = 0; while (at < values.length) { result.push(String(values[at])); at = at + 1; } return result; }
function main() { const result = workload(2, 7); }
