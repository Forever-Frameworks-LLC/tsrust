type int = number;
function workload(n: int, seed: int): int[][] {
    const output: int[][] = [];
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0;
    while (i < n) {
        const count = sizes[(i + seed) % 5];
        const source: int[] = [];
        let j = 0;
        while (j < count) {
            const ordinal = j; source.push(seed + ordinal);
            j = j + 1;
        }
        const flattened = source.flatMap(mapPiece);
        output.push(flattened);
        i = i + 1;
    }
    return output;
}
function mapPiece(value: int, index: int, source: int[]): int[] {
    if (index % 3 === 0) { return []; }
    return [value, value + index + source.length];
}
function main() { const result = workload(2, 7); }
