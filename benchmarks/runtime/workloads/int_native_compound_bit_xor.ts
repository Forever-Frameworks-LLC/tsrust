function workload(n: number, seed: number): number[] {
    const operands: number[] = [0, 1, 3, 7, 15, 63, 255, 65535];
    const rhs: number[] = [0, 2, 5, 8, 17, 42, 170, 43690];
    const output: number[] = []; let i = 0;
    while (i < n) {
        let value: number = operands[(i * 13 + seed) % 8];
        const other: number = rhs[(i * 7 + seed) % 8];
        value ^= other;
        output.push(value);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
