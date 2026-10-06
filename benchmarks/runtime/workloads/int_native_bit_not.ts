function workload(n: number, seed: number): number[] {
    const operands: number[] = [-1, -2, -3, -8, -16, -256, -1024, -65536];
    const output: number[] = []; let i = 0;
    while (i < n) {
        const value: number = operands[(i * 13 + seed) % 8];
        const observed: number = ~value;
        output.push(observed);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
