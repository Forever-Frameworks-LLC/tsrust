function workload(n: number, seed: number): number[] {
    const operands: number[] = [2, 3, 8, 16, 64, 129, 256, 1024];
    const output: number[] = []; let i = 0;
    while (i < n) {
        let value: number = operands[(i * 13 + seed) % 8];
        const observed: number = value--;
        output.push(observed);
        output.push(value);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
