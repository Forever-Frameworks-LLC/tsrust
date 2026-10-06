type int = number;
type SortRecord = { id: int; label: string; };
function workload(n: int, seed: int, input: string): string[][] {
    const output: string[][] = [];
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0;
    while (i < n) {
        const count = sizes[(i + seed) % 5];
        const values: SortRecord[] = []; let j = 0;
        while (j < count) {
            values.push({ id: (seed + j) % 3, label: input.concat(":", String(j)) });
            j = j + 1;
        }
        values.sort(compare);
        output.push(describe(values));
        i = i + 1;
    }
    return output;
}
function compare(left: SortRecord, right: SortRecord): int { return left.id - right.id; }
function describe(values: SortRecord[]): string[] { const result: string[] = []; let at = 0; while (at < values.length) { result.push(String(values[at].id).concat(":", values[at].label)); at = at + 1; } return result; }
function main() { const result = workload(2, 7, "dynamic text"); }
