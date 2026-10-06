type int = number;
type FlatRecord = { id: int; label: string; };
function workload(n: int, seed: int, input: string): string[][] {
    const output: string[][] = [];
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0;
    while (i < n) {
        const count = sizes[(i + seed) % 5];
        const source: FlatRecord[][] = [];
        let j = 0;
        while (j < count) {
            const part: FlatRecord[] = [];
            const width = (j + seed) % 3; let k = 0;
            while (k < width) { const ordinal = j * 3 + k; part.push({ id: seed + ordinal, label: input.concat(":", String(ordinal)) }); k = k + 1; }
            source.push(part);
            j = j + 1;
        }
        const flattened = source.flat();
        output.push(describe(flattened));
        i = i + 1;
    }
    return output;
}
function describe(values: FlatRecord[]): string[] { const result: string[] = []; let at = 0; while (at < values.length) { result.push(String(values[at].id).concat(":", values[at].label)); at = at + 1; } return result; }
function main() { const result = workload(2, 7, "dynamic text"); }
