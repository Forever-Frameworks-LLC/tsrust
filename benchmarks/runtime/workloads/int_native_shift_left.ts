function workload(n: number, seed: number): number[] {
    const operands: number[] = [0, 1, 2, 3, 7, 15, 31, 63];
    const rhs: number[] = [0, 1, 2, 3, 4, 5, 10, 20];
    const output: number[] = []; let i = 0;
    while (i < n) {
        const value: number = operands[(i * 13 + seed) % 8];
        const other: number = rhs[(i * 7 + seed) % 8];
        const observed: number = value << other;
        output.push(observed);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
