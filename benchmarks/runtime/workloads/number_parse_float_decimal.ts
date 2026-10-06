type int = number;
function workload(n: int, seed: int, input: string): number[] {
    const values: string[] = [clone(input), "0", "-0", "+17", "0012", "0.5", ".5", "5.", "123.456", "9007199254740991", "-9007199254740991", "1.0000000000000002", "-0.000", "3.141592653589793", "12345678901234567890", "000000.125"];
    const output: number[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 16;
        const selected: number = Number.parseFloat(values[at]);
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "-123.625"); }
function clone<T>(value: T): T { return value; }
