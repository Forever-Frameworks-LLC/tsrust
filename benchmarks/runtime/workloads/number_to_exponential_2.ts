type int = number;
function workload(n: int, seed: int): string[] {
    const values: number[] = [0.0, -0.0, 1.0, -1.0, 1.234, -1.234, 9.999, -9.999, 12345.678, -12345.678, 1e-7, 1e-6, 9.999e-7, 9.999e-6, 1e20, 1e21, 1.0000123e21, -1e21, 1.0 / 0.0, -1.0 / 0.0, 0.0 / 0.0, -1e-7, 5e-324, 1.7976931348623157e308];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 24;
        const selected: string = values[at].toExponential(2);
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
