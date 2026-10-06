type int = number;
function workload(n: int, seed: int, input: string): string[] {
    const values: string[] = [clone(input), "", "ABC", "a\u0000b", " x ", "quote\"", "slash\\", input.repeat(2)];
    const fractions: number[] = [0.5, 1.25, 2.5, 12.75];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 8;
        const count: int = (i * 7 + seed) % 97;
        const fraction: number = fractions[(i + seed) % 4];
        const flag: boolean = (i + seed) % 3 === 0;
        const selected: string = `head:${values[at]}|${count}|${fraction}|${flag}:tail`;
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "dynamic ASCII"); }
function clone<T>(value: T): T { return value; }
