type int = number;
function workload(n: int, seed: int): number[] {
    const output: number[] = []; let i = 0;
    while (i < n) {
        const values: number[] = [Number(seed) + Number(i) * 0.25, -0.0, 0.0 / 0.0, 1.0 / 0.0, -1.0 / 0.0, 0.5, 5e-324, 1.7976931348623157e308];
        const index = (i * 3 + seed) % 8; values[index] = Number(i) + 0.75;
        const saved = clone(values); values.reverse();
        for (const value of saved) { output.push(value); }
        for (const value of values) { output.push(value); }
        output.push(Number(values.includes(0.5))); output.push(Number(values.indexOf(-0.0)));
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); console.log(result.length); }
// Standard JavaScript clone counterpart performs an independent shallow scalar array copy.
function clone(values: number[]): number[] { return values.slice(); }
