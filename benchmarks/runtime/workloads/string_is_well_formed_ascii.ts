/// <reference lib="es2024.string" />
type int = number;
function workload(n: int, seed: int, input: string): boolean[] {
    const values: string[] = [clone(input), "", "ABC", "a\u0000b", " e ", "Z", input.repeat(2), "tail"];
    const output: boolean[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 8;
        const selected: boolean = values[at].isWellFormed();
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, " A alpha Z "); }
function clone<T>(value: T): T { return value; }
