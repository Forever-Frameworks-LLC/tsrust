function workload(n: number, seed: number): number[] {
    const output: number[] = []; let i = 0;
    while (i < n) {
        const base: number = (i * 13 + seed) % 64; const rhs: number = (i * 7 + seed) % 4 + 1;
        let value: number = base * rhs + base % rhs;
        value %= rhs;
        output.push(value);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
