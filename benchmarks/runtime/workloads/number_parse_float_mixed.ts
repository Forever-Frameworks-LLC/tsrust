type int = number;
function workload(n: int, seed: int, input: string): number[] {
    const values: string[] = [clone(input), "-0", "0x1", "1e+tail", "5e-324", "1e309", "NaN", "Infinitytail", "\ufeff.5", "\u008512", "-1e-9999", "9007199254740991", "1.7976931348623157e308", "-1.7976931348623157e308", "1.0000000000000002", "12\u0000tail"];
    const output: number[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 16;
        const selected: number = Number.parseFloat(values[at]);
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "\u00a0 -45.5e-2tail"); }
function clone<T>(value: T): T { return value; }
