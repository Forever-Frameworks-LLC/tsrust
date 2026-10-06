function workload(n: number, seed: number): number[] {
    const operands: number[] = [0, 1, 2, 3, 4, 5, 6, 7];
    const rhs: number[] = [0, 1, 2, 3, 4, 5, 6, 7];
    const output: number[] = []; let i = 0;
    while (i < n) {
        let value: number = operands[(i * 13 + seed) % 8];
        const other: number = rhs[(i * 7 + seed) % 8];
        value **= other;
        output.push(value);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
