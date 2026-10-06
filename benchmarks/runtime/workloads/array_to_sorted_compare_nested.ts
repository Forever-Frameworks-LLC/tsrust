type int = number;
function workload(n: int, seed: int): int[][][] {
    const output: int[][][] = [];
    const sizes: int[] = [0, 1, 4, 5, 17];
    let i = 0;
    while (i < n) {
        const count = sizes[(i + seed) % 5];
        const values: int[][] = []; let j = 0;
        while (j < count) {
            if (j % 4 === 0) { values.push([seed, 1]); }
            else if (j % 4 === 1) { values.push([]); }
            else if (j % 4 === 2) { values.push([seed + 1, 2]); }
            else { values.push([seed + 2]); }
            j = j + 1;
        }
        const sorted = values.toSorted(compare);
        output.push(values);
        output.push(sorted);
        i = i + 1;
    }
    return output;
}
function compare(left: int[], right: int[]): int { return left.length - right.length; }
function main() { const result = workload(2, 7); }
